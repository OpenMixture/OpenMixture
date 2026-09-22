use crate::AssetLimits;
use crate::error::{AssetError, invalid, limit};
use serde::{
    Deserialize, Serialize,
    de::{SeqAccess, Visitor},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Document {
    pub path: String,
    pub byte_length: u64,
    pub sha256: String,
}
/// Validated image entry metadata; paths are fixed ordinal archive names, never OS paths.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resource {
    /// Case-sensitive logical resource ID.
    pub id: String,
    /// Canonical archive entry name.
    pub path: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Exactly rgba8-linear.
    pub format: String,
    /// Packed row stride.
    pub bytes_per_row: u64,
    /// Packed byte length.
    pub byte_length: u64,
    /// Core-domain SHA-256 identity.
    pub content_digest: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    pub format: String,
    pub version: u32,
    pub document: Document,
    #[serde(deserialize_with = "bounded_resources")]
    pub resources: Vec<Resource>,
}
fn bounded_resources<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Resource>, D::Error> {
    struct Bounded;
    impl<'de> Visitor<'de> for Bounded {
        type Value = Vec<Resource>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("at most eight resources")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut rows = Vec::new();
            while let Some(row) = seq.next_element::<Resource>()? {
                if rows.len() == 8 {
                    return Err(serde::de::Error::custom("package resource count exceeded"));
                }
                rows.push(row);
            }
            Ok(rows)
        }
    }
    d.deserialize_seq(Bounded)
}
pub(crate) fn decode(bytes: &[u8]) -> Result<Manifest, AssetError> {
    #[derive(Deserialize)]
    struct Version {
        version: u32,
    }
    let version: Version =
        serde_json::from_slice(bytes).map_err(|e| invalid(format!("Invalid manifest: {e}")))?;
    if version.version != 1 {
        return Err(AssetError::new(
            "MIX_PACKAGE_UNSUPPORTED_VERSION",
            "Unsupported asset version.",
        )
        .evidence("observed", version.version));
    }
    serde_json::from_slice(bytes).map_err(|e| {
        if e.to_string().starts_with("package resource count exceeded") {
            AssetError::new(
                "MIX_PACKAGE_LIMIT_EXCEEDED",
                "Resource count exceeds eight.",
            )
            .evidence("limit", "resourceCount")
            .evidence("configured", 8)
            .evidence("observed", 9)
        } else {
            invalid(format!("Invalid manifest: {e}"))
        }
    })
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub(crate) fn validate(m: &Manifest, l: &AssetLimits) -> Result<u64, AssetError> {
    if m.format != "openmixture-asset"
        || m.document.path != "material.mix"
        || !digest(&m.document.sha256)
    {
        return Err(invalid("Invalid document metadata."));
    }
    limit(
        "sourceBytes",
        m.document.byte_length,
        l.safety.decoded_bytes,
    )?;
    limit(
        "resourceCount",
        m.resources.len() as u64,
        l.resources.resource_count,
    )?;
    let mut previous: Option<&str> = None;
    let mut dimensions = None;
    let mut pixels = 0u64;
    let mut total = 0u64;
    for (i, r) in m.resources.iter().enumerate() {
        if previous.is_some_and(|p| p >= r.id.as_str())
            || !mixture_core::resources::valid_image_id(&r.id)
            || r.path != format!("images/{i:04}.rgba")
            || !digest(&r.content_digest)
            || r.format != "rgba8-linear"
        {
            return Err(
                invalid("Invalid resource identity or order.").evidence("resourceId", r.id.clone())
            );
        }
        previous = Some(&r.id);
        limit(
            "outputDimension",
            u64::from(r.width.max(r.height)),
            l.safety.output_dimension,
        )?;
        if r.width == 0 || r.height == 0 || dimensions.is_some_and(|d| d != [r.width, r.height]) {
            return Err(invalid("Resources must share positive dimensions."));
        }
        dimensions = Some([r.width, r.height]);
        let row = u64::from(r.width) * 4;
        let count = u64::from(r.width) * u64::from(r.height);
        let len = count
            .checked_mul(4)
            .ok_or_else(|| invalid("Resource size overflow."))?;
        if row != r.bytes_per_row || len != r.byte_length {
            return Err(invalid("Invalid packed resource length or stride."));
        }
        pixels = pixels
            .checked_add(count)
            .ok_or_else(|| invalid("Pixel sum overflow."))?;
        total = total
            .checked_add(len)
            .ok_or_else(|| invalid("Byte sum overflow."))?;
    }
    limit("resourcePixels", pixels, l.resources.resource_pixels)?;
    limit("resourceBytes", total, l.resources.resource_bytes)?;
    Ok(total)
}
