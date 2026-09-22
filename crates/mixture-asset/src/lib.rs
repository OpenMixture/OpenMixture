//! CPU-only canonical `.mixpack` transport. Core owns graph semantics and pixels.
//! Loading borrows archive payloads; preparation captures only selected resources.
#![doc = include_str!("../README.md")]
mod archive;
mod error;
mod limits;
mod manifest;
#[cfg(test)]
mod tests;
mod writer;

pub use error::{AssetError, PackageDiagnostic};
use error::{invalid, limit};
pub use limits::{AssetLimits, PackageLimits};
pub use manifest::Resource;
use mixture_core::{
    CompileRequest, ImageBinding, MaterialDocument, PreparedRender, ValidatedDocument,
};
use sha2::{Digest, Sha256};
use std::{borrow::Cow, collections::BTreeSet, ops::Range};
pub use writer::write;

#[derive(Clone, Debug)]
struct Parsed {
    manifest: manifest::Manifest,
    manifest_bytes: u64,
    source: Range<usize>,
    images: Vec<Range<usize>>,
    document: ValidatedDocument,
}
impl Parsed {
    fn scratch(&self) -> u64 {
        self.source.len() as u64 + self.manifest_bytes
    }
}

/// Validated immutable borrowed asset. No resource payload is copied by loading.
pub struct AssetView<'a> {
    bytes: &'a [u8],
    parsed: Cow<'a, Parsed>,
    limits: AssetLimits,
    owned_bytes: u64,
}
impl std::fmt::Debug for AssetView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetView")
            .field("package_bytes", &self.bytes.len())
            .field("resources", &self.resources())
            .finish()
    }
}
/// Stable inspection report, separate from RenderPlan and package format versions.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Inspection<'a> {
    /// Inspection contract version, currently 1.
    pub schema_version: u32,
    /// Transport format version, currently 1.
    pub package_version: u32,
    /// Raw archive identity, never substituted for a plan hash.
    pub package_sha256: String,
    /// Exact encoded archive length.
    pub package_bytes: u64,
    /// SHA-256 of exact source bytes.
    pub source_sha256: &'a str,
    /// Exact source length.
    pub source_bytes: u64,
    /// Validated resources in lexical ID order.
    pub resources: &'a [Resource],
}
impl<'a> AssetView<'a> {
    /// Validate canonical framing, all payloads, Core source and closed resource set.
    pub fn load(bytes: &'a [u8], limits: &AssetLimits) -> Result<Self, AssetError> {
        let limits = limits.checked()?;
        limit(
            "packageBytes",
            bytes.len() as u64,
            limits.package.package_bytes,
        )?;
        let mut cursor = 0;
        let range = archive::entry(
            bytes,
            &mut cursor,
            "manifest.json",
            limits.package.manifest_bytes,
        )?;
        limits.buffers(range.len() as u64)?;
        let manifest = manifest::decode(&bytes[range.clone()])?;
        manifest::validate(&manifest, &limits)?;
        limits.buffers(range.len() as u64 + manifest.document.byte_length)?;
        let source = archive::entry(
            bytes,
            &mut cursor,
            "material.mix",
            limits.safety.decoded_bytes,
        )?;
        exact_length(&source, manifest.document.byte_length)?;
        let mut images = Vec::with_capacity(manifest.resources.len());
        for r in &manifest.resources {
            let data =
                archive::entry(bytes, &mut cursor, &r.path, limits.resources.resource_bytes)?;
            exact_length(&data, r.byte_length)?;
            images.push(data);
        }
        archive::finish(bytes, cursor)?;
        if sha256(&bytes[source.clone()]) != manifest.document.sha256 {
            return Err(mismatch("material.mix"));
        }
        for (r, range) in manifest.resources.iter().zip(&images) {
            let binding = binding(r, &bytes[range.clone()]);
            let identity = mixture_core::resources::image_identity(
                &binding,
                &limits.safety,
                &limits.resources,
            )?;
            if identity.content_digest != r.content_digest {
                return Err(mismatch(&r.path));
            }
        }
        let document = MaterialDocument::decode(&bytes[source.clone()], &limits.safety)?
            .into_validated(&limits.safety)?;
        closed_set(&document, manifest.resources.iter().map(|r| r.id.as_str()))?;
        Ok(Self {
            bytes,
            parsed: Cow::Owned(Parsed {
                manifest,
                manifest_bytes: range.len() as u64,
                source,
                images,
                document,
            }),
            limits,
            owned_bytes: 0,
        })
    }
    /// Original source bytes, including whitespace and field ordering.
    pub fn source(&self) -> &[u8] {
        &self.bytes[self.parsed.source.clone()]
    }
    /// Validated metadata; mutation cannot change the asset.
    pub fn resources(&self) -> &[Resource] {
        &self.parsed.manifest.resources
    }
    /// Retained archive capacity plus conservative source/manifest scratch.
    pub fn loading_buffer_bytes(&self) -> u64 {
        self.owned_bytes + self.parsed.scratch()
    }
    /// Borrow the packed bytes of one logical resource.
    pub fn image(&self, id: &str) -> Option<&[u8]> {
        self.resources()
            .iter()
            .zip(&self.parsed.images)
            .find(|(r, _)| r.id == id)
            .map(|(_, range)| &self.bytes[range.clone()])
    }
    /// Borrow all validated bindings without allocating pixel buffers.
    pub fn bindings(&self) -> Vec<ImageBinding<'_>> {
        self.resources()
            .iter()
            .zip(&self.parsed.images)
            .map(|(r, range)| binding(r, &self.bytes[range.clone()]))
            .collect()
    }
    /// Package/source identity and validated resource table; no GPU access.
    pub fn inspect(&self) -> Inspection<'_> {
        Inspection {
            schema_version: 1,
            package_version: 1,
            package_sha256: sha256(self.bytes),
            package_bytes: self.bytes.len() as u64,
            source_sha256: &self.parsed.manifest.document.sha256,
            source_bytes: self.source().len() as u64,
            resources: self.resources(),
        }
    }
    fn request(&self, request: &CompileRequest) -> Result<(CompileRequest, u64), AssetError> {
        let mut request = request.clone();
        request.limits = limits::intersect(request.limits, self.limits.safety);
        limit(
            "sourceBytes",
            self.source().len() as u64,
            request.limits.decoded_bytes,
        )?;
        let references =
            mixture_core::resources::image_references(&self.parsed.document, &request)?;
        if !references.overridden.is_empty() {
            return Err(AssetError::new(
                "MIX_PACKAGE_RESOURCE_OVERRIDE",
                "Package v1 does not accept resourceRef overrides.",
            )
            .evidence("parameterIds", serde_json::json!(references.overridden)));
        }
        let selected = self
            .resources()
            .iter()
            .filter(|r| references.selected.contains(&r.id))
            .map(|r| r.byte_length)
            .sum::<u64>();
        let charged = self.owned_bytes + self.parsed.scratch() + selected;
        self.limits.buffers(charged)?;
        Ok((request, charged))
    }
    /// Charged byte-buffer bound before preparation: retained archive + D + M + S.
    /// Caller-owned borrowed input and typed metadata are not byte-buffer copies.
    pub fn preparation_buffer_bytes(&self, request: &CompileRequest) -> Result<u64, AssetError> {
        self.request(request).map(|(_, bytes)| bytes)
    }
    /// Delegate request validation, slicing, capture and plan construction to Core.
    /// Returned snapshots remain valid after the archive is dropped or changed.
    pub fn prepare(&self, request: &CompileRequest) -> Result<PreparedRender, AssetError> {
        let (request, _) = self.request(request)?;
        Ok(mixture_core::prepare(
            &self.parsed.document,
            &request,
            &self.bindings(),
            &self.limits.resources,
        )?)
    }
}

/// Asset owning exactly one archive allocation and immutable validated metadata.
pub struct OwnedAsset {
    bytes: Vec<u8>,
    parsed: Parsed,
    limits: AssetLimits,
}
impl std::fmt::Debug for OwnedAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.view().fmt(f)
    }
}
impl OwnedAsset {
    /// Move a caller allocation without copying it. Excess Vec capacity is charged.
    pub fn from_vec(bytes: Vec<u8>, limits: &AssetLimits) -> Result<Self, AssetError> {
        let limits = limits.checked()?;
        limit(
            "packageCapacity",
            bytes.capacity() as u64,
            limits.package.package_bytes,
        )?;
        limits.buffers(bytes.capacity() as u64)?;
        let view = AssetView::load(&bytes, &limits)?;
        limit(
            "packageCapacity",
            bytes.capacity() as u64,
            view.limits.package.package_bytes,
        )?;
        view.limits
            .buffers(bytes.capacity() as u64 + view.parsed.scratch())?;
        let limits = view.limits;
        let parsed = view.parsed.into_owned();
        Ok(Self {
            bytes,
            parsed,
            limits,
        })
    }
    /// Validate and budget before allocating a single archive copy.
    pub fn copy_from(bytes: &[u8], limits: &AssetLimits) -> Result<Self, AssetError> {
        let limits = limits.checked()?;
        limits.buffers(bytes.len() as u64)?;
        let view = AssetView::load(bytes, &limits)?;
        view.limits
            .buffers(bytes.len() as u64 + view.parsed.scratch())?;
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(bytes.len())
            .map_err(|_| error::allocation())?;
        view.limits
            .buffers(owned.capacity() as u64 + view.parsed.scratch())?;
        limit(
            "packageCapacity",
            owned.capacity() as u64,
            view.limits.package.package_bytes,
        )?;
        owned.extend_from_slice(bytes);
        Ok(Self {
            bytes: owned,
            limits: view.limits,
            parsed: view.parsed.into_owned(),
        })
    }
    /// Borrow already-validated metadata and payloads; no validation or copy repeats.
    pub fn view(&self) -> AssetView<'_> {
        AssetView {
            bytes: &self.bytes,
            parsed: Cow::Borrowed(&self.parsed),
            limits: self.limits,
            owned_bytes: self.bytes.capacity() as u64,
        }
    }
    /// Read-only complete archive bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Retained archive allocation charged to the package buffer policy.
    pub fn archive_capacity(&self) -> usize {
        self.bytes.capacity()
    }
    /// Prepare immutable Core-owned selected snapshots.
    pub fn prepare(&self, request: &CompileRequest) -> Result<PreparedRender, AssetError> {
        self.view().prepare(request)
    }
}
fn binding<'a>(r: &'a Resource, data: &'a [u8]) -> ImageBinding<'a> {
    ImageBinding {
        id: &r.id,
        width: r.width,
        height: r.height,
        format: &r.format,
        bytes_per_row: r.bytes_per_row,
        data,
    }
}
fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn mismatch(path: &str) -> AssetError {
    AssetError::new(
        "MIX_PACKAGE_CONTENT_MISMATCH",
        "Payload digest differs from the manifest.",
    )
    .evidence("entry", path)
}
fn exact_length(range: &Range<usize>, expected: u64) -> Result<(), AssetError> {
    if range.len() as u64 != expected {
        Err(invalid("Entry length differs from manifest."))
    } else {
        Ok(())
    }
}
fn closed_set<'a>(
    document: &ValidatedDocument,
    ids: impl Iterator<Item = &'a str>,
) -> Result<(), AssetError> {
    let actual: BTreeSet<_> = ids.map(str::to_owned).collect();
    if mixture_core::resources::document_image_ids(document)? != actual {
        return Err(invalid(
            "Resources must exactly match all default image references, including disconnected nodes.",
        ));
    }
    Ok(())
}
