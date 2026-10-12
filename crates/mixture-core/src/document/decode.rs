//! Streaming collection ceilings and strict JSON parameter objects.

use super::*;
use crate::{LimitExceeded, LimitKind};
use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::marker::PhantomData;

pub(super) fn document(
    bytes: &[u8],
    limits: &SafetyLimits,
) -> Result<MaterialDocument, DocumentError> {
    let document = strict(
        bytes,
        limits,
        "Material input does not conform to .mix v1 JSON.",
        |parser, exceeded| DocumentSeed { limits, exceeded }.deserialize(parser),
    )?;
    if document.version != FORMAT_VERSION {
        return Err(super::super::validation::unsupported_version(document.version).into());
    }
    Ok(document)
}

/// Decode one bounded, strict UTF-8 JSON object with the shared parse diagnostics.
pub(crate) fn strict<T>(
    bytes: &[u8],
    limits: &SafetyLimits,
    mismatch: &str,
    read: impl FnOnce(
        &mut serde_json::Deserializer<serde_json::de::StrRead<'_>>,
        &mut Option<Diagnostic>,
    ) -> Result<T, serde_json::Error>,
) -> Result<T, DocumentError> {
    limits
        .check(LimitKind::DecodedBytes, bytes.len() as u64)
        .map_err(|error| DocumentError::from(error.diagnostic(Stage::Parse)))?;
    let text = std::str::from_utf8(bytes).map_err(|source| {
        DocumentError::from(
            Diagnostic::error(
                DiagnosticCode::ParseInvalidUtf8,
                Stage::Parse,
                "Material input must be UTF-8.",
            )
            .with_evidence("validUpTo", source.valid_up_to() as u64)
            .with_suggestion("Save the .mix file as UTF-8 JSON.")
            .with_source(source),
        )
    })?;
    let mut parser = serde_json::Deserializer::from_str(text);
    let mut exceeded = None;
    let result = read(&mut parser, &mut exceeded).and_then(|document| {
        parser.end()?;
        Ok(document)
    });
    if let Some(diagnostic) = exceeded {
        return Err(diagnostic.into());
    }
    result.map_err(|source| {
        let code = if source.is_data() { DiagnosticCode::FormatInvalidDocument } else { DiagnosticCode::ParseInvalidJson };
        DocumentError::from(Diagnostic::error(code, Stage::Parse, mismatch)
            .with_evidence("line", source.line() as u64).with_evidence("column", source.column() as u64)
            .with_evidence("sourceMessage", source.to_string())
            .with_suggestion("Use the documented .mix schema; remove duplicate or unknown fields and correct JSON types.")
            .with_source(source))
    })
}

struct DocumentSeed<'a> {
    limits: &'a SafetyLimits,
    exceeded: &'a mut Option<Diagnostic>,
}
impl<'de> DeserializeSeed<'de> for DocumentSeed<'_> {
    type Value = MaterialDocument;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for DocumentSeed<'_> {
    type Value = MaterialDocument;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a .mix document object")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let (mut version, mut nodes, mut edges, mut exposed) = (None, None, None, None);
        let mut instances = false;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "version" if version.is_none() => version = Some(map.next_value()?),
                // A v2 parent must fail on its version, not on this field; skip it
                // without allocation and reject it below when the version is 1.
                "instances" if !instances => {
                    instances = true;
                    map.next_value::<IgnoredAny>()?;
                }
                "nodes" if nodes.is_none() => {
                    nodes = Some(map.next_value_seed(Bounded::<Node>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::Nodes,
                    ))?)
                }
                "edges" if edges.is_none() => {
                    edges = Some(map.next_value_seed(Bounded::<Edge>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::Edges,
                    ))?)
                }
                "exposedParameters" if exposed.is_none() => {
                    exposed = Some(map.next_value_seed(Bounded::<ExposedParameter>::new(
                        self.limits,
                        self.exceeded,
                        LimitKind::ExposedParameters,
                    ))?)
                }
                "version" | "nodes" | "edges" | "exposedParameters" => {
                    return Err(de::Error::custom(format!("duplicate field {key:?}")));
                }
                _ => {
                    return Err(de::Error::unknown_field(
                        &key,
                        &["version", "nodes", "edges", "exposedParameters"],
                    ));
                }
            }
        }
        let version: u32 = version.ok_or_else(|| de::Error::missing_field("version"))?;
        if instances && version == FORMAT_VERSION {
            return Err(de::Error::unknown_field(
                "instances",
                &["version", "nodes", "edges", "exposedParameters"],
            ));
        }
        Ok(MaterialDocument {
            version,
            nodes: nodes.ok_or_else(|| de::Error::missing_field("nodes"))?,
            edges: edges.ok_or_else(|| de::Error::missing_field("edges"))?,
            exposed_parameters: exposed.unwrap_or_default(),
        })
    }
}
/// A collection ceiling and the diagnostic reported for its first excess item.
#[derive(Clone, Copy)]
pub(crate) enum Ceiling {
    /// A [`SafetyLimits`] category.
    Safety(LimitKind),
    /// The recipe instance ceiling of one parent document.
    Instances,
}
pub(crate) struct Bounded<'a, T> {
    maximum: u64,
    ceiling: Ceiling,
    exceeded: &'a mut Option<Diagnostic>,
    marker: PhantomData<T>,
}
impl<'a, T> Bounded<'a, T> {
    pub(crate) fn new(
        limits: &SafetyLimits,
        exceeded: &'a mut Option<Diagnostic>,
        kind: LimitKind,
    ) -> Self {
        Self::with(Ceiling::Safety(kind), limits.maximum(kind), exceeded)
    }
    pub(crate) fn with(
        ceiling: Ceiling,
        maximum: u64,
        exceeded: &'a mut Option<Diagnostic>,
    ) -> Self {
        Self {
            maximum,
            ceiling,
            exceeded,
            marker: PhantomData,
        }
    }
}
fn excess(ceiling: Ceiling, configured: u64, observed: u64) -> Diagnostic {
    match ceiling {
        Ceiling::Safety(kind) => LimitExceeded {
            kind,
            configured,
            observed,
        }
        .diagnostic(Stage::Parse),
        Ceiling::Instances => Diagnostic::error(
            DiagnosticCode::LimitInstanceCountExceeded,
            Stage::Parse,
            format!("instanceCount limit exceeded: configured {configured}, observed {observed}"),
        )
        .with_evidence("limit", "instanceCount")
        .with_evidence("configured", configured)
        .with_evidence("observed", observed)
        .with_suggestion("Reduce the request to the configured limit; raise limits only through an explicit caller policy."),
    }
}
impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Bounded<'_, T> {
    type Value = Vec<T>;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_seq(self)
    }
}
impl<'de, T: Deserialize<'de>> Visitor<'de> for Bounded<'_, T> {
    type Value = Vec<T>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a bounded array")
    }
    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        // Never reserve an untrusted size_hint or construct an over-budget item.
        let mut items = Vec::new();
        loop {
            if items.len() as u64 == self.maximum {
                if seq.next_element::<IgnoredAny>()?.is_some() {
                    let diagnostic = excess(
                        self.ceiling,
                        self.maximum,
                        (items.len() as u64).saturating_add(1),
                    );
                    let message = diagnostic.message.clone();
                    *self.exceeded = Some(diagnostic);
                    return Err(de::Error::custom(message));
                }
                return Ok(items);
            }
            match seq.next_element()? {
                Some(item) => items.push(item),
                None => return Ok(items),
            }
        }
    }
}

pub(crate) fn parameters<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Value>, D::Error> {
    struct Parameters;
    impl<'de> Visitor<'de> for Parameters {
        type Value = BTreeMap<String, Value>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("an object with unique parameter IDs")
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut values = BTreeMap::new();
            while let Some(key) = map.next_key::<String>()? {
                if values.contains_key(&key) {
                    return Err(de::Error::custom(format!("duplicate parameter {key:?}")));
                }
                values.insert(key, map.next_value::<StrictValue>()?.0);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(Parameters)
}

// serde_json::Value otherwise silently keeps the last duplicate object key.
// Retain invalid parameter shapes for semantic diagnostics, but never accept an
// ambiguous JSON object. serde_json's normal recursion limit remains enabled.
struct StrictValue(Value);
impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Strict;
        impl<'de> Visitor<'de> for Strict {
            type Value = StrictValue;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON value without duplicate keys")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(v)
                    .map(|n| StrictValue(Value::Number(n)))
                    .ok_or_else(|| E::custom("non-finite number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(StrictValue(v.into()))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Null))
            }
            fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element::<StrictValue>()? {
                    values.push(value.0);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!("duplicate object key {key:?}")));
                    }
                    values.insert(key, map.next_value::<StrictValue>()?.0);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        d.deserialize_any(Strict)
    }
}
