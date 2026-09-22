use crate::{AssetError, AssetLimits, Resource, archive, closed_set, error, manifest, sha256};
use mixture_core::{ImageBinding, MaterialDocument};

/// Emit deterministic canonical USTAR bytes, preserving source bytes exactly.
/// All resources are required, including those outside any selected output slice.
/// The returned archive is the only allocated payload buffer; caller pixels are borrowed.
pub fn write(
    source: &[u8],
    bindings: &[ImageBinding<'_>],
    limits: &AssetLimits,
) -> Result<Vec<u8>, AssetError> {
    let limits = limits.checked()?;
    error::limit(
        "sourceBytes",
        source.len() as u64,
        limits.safety.decoded_bytes,
    )?;
    error::limit(
        "resourceCount",
        bindings.len() as u64,
        limits.resources.resource_count,
    )?;
    limits.buffers(source.len() as u64)?;
    let mut sorted: Vec<_> = bindings.iter().collect();
    sorted.sort_by_key(|r| r.id);
    let mut resources = Vec::with_capacity(sorted.len());
    // Bound IDs and payload metadata before hashing or copying caller data.
    for (i, b) in sorted.iter().enumerate() {
        if !mixture_core::resources::valid_image_id(b.id) {
            return Err(error::invalid("Invalid resource ID."));
        }
        resources.push(Resource {
            id: b.id.into(),
            path: format!("images/{i:04}.rgba"),
            width: b.width,
            height: b.height,
            format: b.format.chars().take(32).collect(),
            bytes_per_row: b.bytes_per_row,
            byte_length: b.data.len() as u64,
            content_digest: "0".repeat(64),
        });
    }
    let mut manifest = manifest::Manifest {
        format: "openmixture-asset".into(),
        version: 1,
        document: manifest::Document {
            path: "material.mix".into(),
            byte_length: source.len() as u64,
            sha256: "0".repeat(64),
        },
        resources,
    };
    manifest::validate(&manifest, &limits)?;
    let document =
        MaterialDocument::decode(source, &limits.safety)?.into_validated(&limits.safety)?;
    closed_set(&document, sorted.iter().map(|b| b.id))?;
    manifest.document.sha256 = sha256(source);
    for (r, b) in manifest.resources.iter_mut().zip(sorted.iter()) {
        r.content_digest =
            mixture_core::resources::image_identity(b, &limits.safety, &limits.resources)?
                .content_digest;
    }
    // Fixed field set, bounded eight 64-byte IDs; serialization never grows with payloads.
    let json = serde_json::to_vec(&manifest)
        .map_err(|e| error::invalid(format!("Cannot encode manifest: {e}")))?;
    error::limit(
        "manifestBytes",
        json.len() as u64,
        limits.package.manifest_bytes,
    )?;
    let mut total = 1024usize;
    for len in std::iter::once(json.len())
        .chain(std::iter::once(source.len()))
        .chain(sorted.iter().map(|b| b.data.len()))
    {
        total = total
            .checked_add(512)
            .and_then(|n| n.checked_add(archive::padded(len).ok()?))
            .ok_or_else(|| error::invalid("Archive size overflow."))?;
    }
    error::limit("packageBytes", total as u64, limits.package.package_bytes)?;
    let scratch = source.len() as u64 + json.len() as u64;
    limits.buffers(total as u64 + scratch)?;
    let mut out = Vec::new();
    out.try_reserve_exact(total)
        .map_err(|_| error::allocation())?;
    limits.buffers(out.capacity() as u64 + scratch)?;
    archive::append(&mut out, "manifest.json", &json)?;
    archive::append(&mut out, "material.mix", source)?;
    for (r, b) in manifest.resources.iter().zip(sorted.iter()) {
        archive::append(&mut out, &r.path, b.data)?;
    }
    out.resize(total, 0);
    Ok(out)
}
