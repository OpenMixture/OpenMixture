//! Bounded caller pixels, immutable capture, and content-bound Core preparation.
use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    CompileError, CompileRequest, Diagnostic, DiagnosticCode as Code, LimitKind,
    NormalizedDocument, RenderPlan, Stage, ValidatedDocument, compiler,
};

/// Separate resource policy; existing SafetyLimits retain their wire contract.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceLimits {
    /// All supplied entries, including unused bindings; default 8.
    pub resource_count: u64,
    /// Sum of all supplied image pixels; default 16,777,216.
    pub resource_pixels: u64,
    /// Sum of packed input byte lengths; default 64 MiB.
    pub resource_bytes: u64,
}
impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            resource_count: 8,
            resource_pixels: 16_777_216,
            resource_bytes: 67_108_864,
        }
    }
}

/// Borrowed image input. Use an entry slice: duplicate IDs are errors, not overwrites.
#[derive(Clone, Copy)]
pub struct ImageBinding<'a> {
    /// Case-sensitive logical name, not a file path or URL.
    pub id: &'a str,
    /// Positive width, exactly the requested output width.
    pub width: u32,
    /// Positive height, exactly the requested output height.
    pub height: u32,
    /// Exactly `rgba8-linear`; unknown values return a structured error.
    pub format: &'a str,
    /// Exactly four times width; no padding is accepted.
    pub bytes_per_row: u64,
    /// Tightly packed top-left RGBA bytes; captured before prepare returns.
    pub data: &'a [u8],
}

/// Adapter image descriptor with a synchronous pixel source; semantic checks remain in Core.
#[derive(Clone, Copy)]
pub struct AdapterImageBinding<'a, D> {
    /// Case-sensitive logical name, not a file path or URL.
    pub id: &'a str,
    /// Positive width, exactly the requested output width.
    pub width: u32,
    /// Positive height, exactly the requested output height.
    pub height: u32,
    /// Exactly `rgba8-linear`; unknown values return a structured error.
    pub format: &'a str,
    /// Exactly four times width; no padding is accepted.
    pub bytes_per_row: u64,
    /// Tightly packed top-left RGBA bytes; captured before prepare returns.
    pub data: D,
}

/// Synchronous adapter byte source. Core validates lengths/budgets first, then
/// allocates the destination and hashes the copied bytes itself. Implementations
/// must keep their declared length stable and copy it without yielding or retaining
/// the target. Core hashes the destination, never a caller-provided identity.
pub trait ImageData {
    /// Exact packed input length, without copying pixels.
    fn byte_len(&self) -> usize;
    /// Copy into Core-owned storage; failures retain their structured diagnostic.
    fn copy_to(&self, target: &mut [u8]) -> Result<(), CompileError>;
}
impl ImageData for &[u8] {
    fn byte_len(&self) -> usize {
        self.len()
    }
    fn copy_to(&self, target: &mut [u8]) -> Result<(), CompileError> {
        if target.len() != self.len() {
            return Err(compiler::invariant(
                "Resource source length changed during capture.",
            ));
        }
        target.copy_from_slice(self);
        Ok(())
    }
}

/// Serialized resource identity. Construction of this value cannot forge a plan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResource {
    /// Logical request identity.
    pub id: String,
    /// Fixed linear RGBA8 format identifier.
    pub format: String,
    /// Image width.
    pub width: u32,
    /// Image height.
    pub height: u32,
    /// Packed row length.
    pub bytes_per_row: u64,
    /// Lowercase SHA-256 of the domain, dimensions and captured bytes.
    pub content_digest: String,
}

/// Immutable owned pixels; Debug intentionally omits their contents.
pub struct ResourceSnapshot {
    image: ImageResource,
    data: Vec<u8>,
}
impl std::fmt::Debug for ResourceSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceSnapshot")
            .field("image", &self.image)
            .field("byte_length", &self.data.len())
            .finish()
    }
}
impl ResourceSnapshot {
    /// Validated identity computed from the immutable captured data.
    pub fn image(&self) -> &ImageResource {
        &self.image
    }
    /// Read-only packed bytes. The caller cannot mutate or replace this snapshot.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Opaque plan/snapshot pairing. No deserializer or public constructor allows substitution.
#[derive(Debug)]
pub struct PreparedRender {
    plan: RenderPlan,
    resources: Vec<ResourceSnapshot>,
}
impl PreparedRender {
    /// Immutable executable semantics and content identities.
    pub fn plan(&self) -> &RenderPlan {
        &self.plan
    }
    /// Selected immutable snapshots in lexical ID order.
    pub fn resources(&self) -> &[ResourceSnapshot] {
        &self.resources
    }
    pub(crate) fn into_plan(self) -> RenderPlan {
        self.plan
    }
}
pub(crate) fn prepared(plan: RenderPlan, resources: Vec<ResourceSnapshot>) -> PreparedRender {
    PreparedRender { plan, resources }
}

/// Validate, capture and compile without GPU, filesystem, or browser dependencies.
///
/// All provided bindings are validated against the normalized full graph; only
/// the requested slice requires pixels and retains them. Neither the document
/// nor borrowed bytes can affect the returned request after this call completes.
pub fn prepare(
    document: &ValidatedDocument,
    request: &CompileRequest,
    bindings: &[ImageBinding<'_>],
    limits: &ResourceLimits,
) -> Result<PreparedRender, CompileError> {
    let inputs: Vec<_> = bindings
        .iter()
        .map(|b| AdapterImageBinding {
            id: b.id,
            width: b.width,
            height: b.height,
            format: b.format,
            bytes_per_row: b.bytes_per_row,
            data: b.data,
        })
        .collect();
    prepare_from(document, request, &inputs, limits)
}

/// Prepare from a synchronous adapter source without an intermediate pixel copy.
/// Core still owns every metadata check, allocation, content hash and snapshot.
pub fn prepare_from<D: ImageData>(
    document: &ValidatedDocument,
    request: &CompileRequest,
    bindings: &[AdapterImageBinding<'_, D>],
    limits: &ResourceLimits,
) -> Result<PreparedRender, CompileError> {
    let normalized = compiler::normalize(document, request)?;
    compiler::lower::compile(&normalized, request, bindings, limits)
}

pub(crate) fn valid_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-')
}
/// Whether a logical image ID satisfies the shared bounded ASCII contract.
pub fn valid_image_id(id: &str) -> bool {
    valid_id(id)
}
fn diagnostic(code: Code, id: &str, message: &str) -> Diagnostic {
    Diagnostic::error(code, Stage::Compile, message)
        .with_evidence("resourceId", id.chars().take(64).collect::<String>())
        .with_suggestion("Supply unique referenced resource IDs and same-size tightly packed rgba8-linear bytes within the explicit resource policy.")
}
fn budget(code: Code, name: &str, configured: u64, observed: u64) -> Option<Diagnostic> {
    (observed > configured).then(|| Diagnostic::error(code, Stage::Compile, "Resource budget exceeded.")
        .with_evidence("limit", name).with_evidence("configured", configured)
        .with_evidence("observed", observed)
        .with_suggestion("Reduce resource input or explicitly select a larger policy; limits are never raised automatically."))
}
fn arithmetic(id: &str, operation: &str, a: u64, b: u64) -> CompileError {
    diagnostic(
        Code::ResourceInvalidBinding,
        id,
        "Resource arithmetic exceeds the supported representation.",
    )
    .with_evidence("operation", operation)
    .with_evidence("left", a)
    .with_evidence("right", b)
    .into()
}
fn add(id: &str, a: u64, b: u64) -> Result<u64, CompileError> {
    a.checked_add(b).ok_or_else(|| arithmetic(id, "add", a, b))
}
fn mul(id: &str, a: u64, b: u64) -> Result<u64, CompileError> {
    a.checked_mul(b)
        .ok_or_else(|| arithmetic(id, "multiply", a, b))
}

pub(crate) fn capture<D: ImageData>(
    normalized: &NormalizedDocument,
    request: &CompileRequest,
    bindings: &[AdapterImageBinding<'_, D>],
    limits: &ResourceLimits,
    selected: &BTreeSet<String>,
) -> Result<Vec<ResourceSnapshot>, CompileError> {
    if let Some(error) = budget(
        Code::LimitResourceCountExceeded,
        "resourceCount",
        limits.resource_count,
        bindings.len() as u64,
    ) {
        return Err(error.into());
    }
    let mut referenced = BTreeSet::new();
    let mut required = BTreeMap::<&str, Vec<&str>>::new();
    for node in &normalized.document().nodes {
        if node.type_id == "image-input" {
            let id = node
                .parameters
                .get("resourceId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| compiler::invariant("Validated resource ID is missing."))?;
            referenced.insert(id);
            if selected.contains(&node.id) {
                required.entry(id).or_default().push(&node.id);
            }
        }
    }
    let mut entries = BTreeMap::new();
    let mut errors = Vec::new();
    let mut pixels = 0;
    let mut bytes = 0;
    // Metadata only: no hash or pixel allocation until every binding passes.
    // Sort borrowed entries so aggregate-overflow and diagnostics are order-independent.
    let mut ordered: Vec<_> = bindings.iter().collect();
    ordered.sort_by_key(|b| {
        (
            b.id,
            b.width,
            b.height,
            b.format,
            b.bytes_per_row,
            b.data.byte_len(),
        )
    });
    for binding in ordered {
        let b = binding;
        if !valid_id(b.id) {
            errors.push(diagnostic(
                Code::ResourceInvalidBinding,
                b.id,
                "Invalid resource identifier.",
            ));
        }
        match entries.entry(b.id) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(b);
            }
            std::collections::btree_map::Entry::Occupied(_) => errors.push(diagnostic(
                Code::ResourceDuplicateId,
                b.id,
                "Duplicate resource binding.",
            )),
        }
        if !referenced.contains(b.id) {
            errors.push(diagnostic(
                Code::ResourceUnknownId,
                b.id,
                "No normalized node references this resource.",
            ));
        }
        let (count, bounded_length) =
            check_metadata(b, request.size, &request.limits, &mut errors)?;
        pixels = add(b.id, pixels, count)?;
        // Both declared and actual lengths are bounded, including malformed input.
        bytes = add(b.id, bytes, bounded_length)?;
    }
    errors.extend(budget(
        Code::LimitResourcePixelsExceeded,
        "resourcePixels",
        limits.resource_pixels,
        pixels,
    ));
    errors.extend(budget(
        Code::LimitResourceBytesExceeded,
        "resourceBytes",
        limits.resource_bytes,
        bytes,
    ));
    for (id, nodes) in &required {
        if !entries.contains_key(id) {
            for node in nodes {
                let mut error = diagnostic(
                    Code::ResourceMissing,
                    id,
                    "Selected image node has no resource binding.",
                );
                error.node_id = Some((*node).into());
                error.parameter_id = Some("resourceId".into());
                errors.push(error);
            }
        }
    }
    if !errors.is_empty() {
        return Err(CompileError::new(errors));
    }
    let mut snapshots = Vec::new();
    for (id, binding) in entries {
        if !required.contains_key(id) {
            continue;
        }
        // Allocate from the already validated descriptor, not a second potentially
        // stateful adapter length query after enforcing budgets.
        let length = usize::try_from(mul(id, binding.bytes_per_row, u64::from(binding.height))?)
            .map_err(|_| {
                arithmetic(
                    id,
                    "hostLength",
                    binding.bytes_per_row,
                    u64::from(binding.height),
                )
            })?;
        let mut data = Vec::new();
        data.try_reserve_exact(length).map_err(|source| {
            CompileError::from(
                diagnostic(
                    Code::ResourceInvalidBinding,
                    id,
                    "Cannot allocate an owned resource snapshot.",
                )
                .with_source(source),
            )
        })?;
        data.resize(length, 0);
        binding.data.copy_to(&mut data)?;
        let content_digest = image_digest(binding.width, binding.height, &data);
        snapshots.push(ResourceSnapshot {
            image: ImageResource {
                id: id.into(),
                format: binding.format.into(),
                width: binding.width,
                height: binding.height,
                bytes_per_row: binding.bytes_per_row,
                content_digest,
            },
            data,
        });
    }
    Ok(snapshots)
}

fn check_metadata<D: ImageData>(
    b: &AdapterImageBinding<'_, D>,
    size: [u32; 2],
    safety: &crate::SafetyLimits,
    errors: &mut Vec<Diagnostic>,
) -> Result<(u64, u64), CompileError> {
    if b.format != "rgba8-linear" {
        errors.push(
            diagnostic(
                Code::ResourceFormatUnsupported,
                b.id,
                "Unsupported resource format.",
            )
            .with_evidence("expected", "rgba8-linear")
            .with_evidence("observed", b.format.chars().take(64).collect::<String>())
            .with_evidence("observedFormatBytes", b.format.len() as u64),
        );
    }
    if b.width == 0 || b.height == 0 || [b.width, b.height] != size {
        errors.push(
            diagnostic(
                Code::ResourceSizeMismatch,
                b.id,
                "Resource dimensions must equal the positive output dimensions.",
            )
            .with_evidence("expectedWidth", u64::from(size[0]))
            .with_evidence("expectedHeight", u64::from(size[1]))
            .with_evidence("observedWidth", u64::from(b.width))
            .with_evidence("observedHeight", u64::from(b.height)),
        );
    }
    for (axis, value) in [("width", b.width), ("height", b.height)] {
        if let Err(error) = safety.check(LimitKind::OutputDimension, u64::from(value)) {
            errors.push(
                error
                    .diagnostic(Stage::Compile)
                    .with_evidence("resourceId", b.id)
                    .with_evidence("axis", axis),
            );
        }
    }
    let count = mul(b.id, u64::from(b.width), u64::from(b.height))?;
    let row = mul(b.id, u64::from(b.width), 4)?;
    let length = mul(b.id, row, u64::from(b.height))?;
    let supplied_length = b.data.byte_len() as u64;
    if b.bytes_per_row != row || supplied_length != length {
        errors.push(
            diagnostic(
                Code::ResourceLengthMismatch,
                b.id,
                "Resource stride and byte length must be exactly packed RGBA8.",
            )
            .with_evidence("expectedBytesPerRow", row)
            .with_evidence("observedBytesPerRow", b.bytes_per_row)
            .with_evidence("expectedLength", length)
            .with_evidence("observedLength", supplied_length),
        );
    }
    usize::try_from(length)
        .map_err(|_| arithmetic(b.id, "hostLength", length, usize::MAX as u64))?;
    Ok((count, length.max(supplied_length)))
}

/// Validate one borrowed image and compute its identity without capturing pixels.
/// Caller policies and the ordinary executor use the same metadata and hash logic.
pub fn image_identity(
    binding: &ImageBinding<'_>,
    safety: &crate::SafetyLimits,
    limits: &ResourceLimits,
) -> Result<ImageResource, CompileError> {
    let b = AdapterImageBinding {
        id: binding.id,
        width: binding.width,
        height: binding.height,
        format: binding.format,
        bytes_per_row: binding.bytes_per_row,
        data: binding.data,
    };
    let mut errors = Vec::new();
    if !valid_id(b.id) {
        errors.push(diagnostic(
            Code::ResourceInvalidBinding,
            b.id,
            "Invalid resource identifier.",
        ));
    }
    let (pixels, bytes) = check_metadata(&b, [b.width, b.height], safety, &mut errors)?;
    errors.extend(budget(
        Code::LimitResourceCountExceeded,
        "resourceCount",
        limits.resource_count,
        1,
    ));
    errors.extend(budget(
        Code::LimitResourcePixelsExceeded,
        "resourcePixels",
        limits.resource_pixels,
        pixels,
    ));
    errors.extend(budget(
        Code::LimitResourceBytesExceeded,
        "resourceBytes",
        limits.resource_bytes,
        bytes,
    ));
    if !errors.is_empty() {
        return Err(CompileError::new(errors));
    }
    Ok(ImageResource {
        id: b.id.into(),
        width: b.width,
        height: b.height,
        format: b.format.into(),
        bytes_per_row: b.bytes_per_row,
        content_digest: image_digest(b.width, b.height, binding.data),
    })
}

fn image_digest(width: u32, height: u32, data: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(b"mixture-image-rgba8-linear-v1\0");
    hash.update(width.to_le_bytes());
    hash.update(height.to_le_bytes());
    hash.update(data);
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Full default resource closure in lexical order, including disconnected branches.
pub fn document_image_ids(document: &ValidatedDocument) -> Result<BTreeSet<String>, CompileError> {
    image_ids(document.document(), None)
}
fn image_ids(
    document: &crate::MaterialDocument,
    selected: Option<&BTreeSet<String>>,
) -> Result<BTreeSet<String>, CompileError> {
    let mut ids = BTreeSet::new();
    for node in &document.nodes {
        if node.type_id == "image-input" && selected.is_none_or(|s| s.contains(&node.id)) {
            let id = node
                .parameters
                .get("resourceId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| compiler::invariant("Validated resource ID is missing."))?;
            ids.insert(id.to_owned());
        }
    }
    Ok(ids)
}
/// Core-owned resource request inventory; no transport logic or pixel capture.
#[derive(Debug)]
pub struct ImageReferences {
    /// Selected resource IDs after validation and dependency slicing.
    pub selected: BTreeSet<String>,
    /// Supplied public override IDs targeting resource-reference parameters.
    pub overridden: BTreeSet<String>,
}
/// Validate every request/override before reporting resource selection and override kinds.
pub fn image_references(
    document: &ValidatedDocument,
    request: &CompileRequest,
) -> Result<ImageReferences, CompileError> {
    let normalized = compiler::normalize(document, request)?;
    let selected = compiler::lower::selected_nodes(&normalized, request)?;
    let mut overridden = BTreeSet::new();
    for exposed in &document.document().exposed_parameters {
        if request.overrides.contains_key(&exposed.id) {
            let kind = document
                .document()
                .nodes
                .iter()
                .find(|n| n.id == exposed.node_id)
                .and_then(|n| crate::registry::node_contract(&n.type_id))
                .and_then(|c| c.parameter(&exposed.parameter_id))
                .map(|p| p.kind);
            if matches!(kind, Some(crate::registry::ParameterKind::ResourceRef)) {
                overridden.insert(exposed.id.clone());
            }
        }
    }
    Ok(ImageReferences {
        selected: image_ids(normalized.document(), Some(&selected))?,
        overridden,
    })
}
