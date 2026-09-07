//! Streaming collection ceilings and strict JSON parameter objects.

use super::*;
use crate::{LimitExceeded, LimitKind};
use serde::de::{self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor};
use std::marker::PhantomData;

pub(super) fn document(
    bytes: &[u8],
    limits: &SafetyLimits,
) -> Result<MaterialDocument, DocumentError> {
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
    let result = DocumentSeed {
        limits,
        exceeded: &mut exceeded,
    }
    .deserialize(&mut parser)
    .and_then(|document| {
        parser.end()?;
        Ok(document)
    });
    if let Some(error) = exceeded {
        return Err(error.diagnostic(Stage::Parse).into());
    }
    let document = result.map_err(|source| {
        let code = if source.is_data() { DiagnosticCode::FormatInvalidDocument } else { DiagnosticCode::ParseInvalidJson };
        DocumentError::from(Diagnostic::error(code, Stage::Parse, "Material input does not conform to .mix v1 JSON.")
            .with_evidence("line", source.line() as u64).with_evidence("column", source.column() as u64)
            .with_evidence("sourceMessage", source.to_string())
            .with_suggestion("Use the documented .mix schema; remove duplicate or unknown fields and correct JSON types.")
            .with_source(source))
    })?;
    if document.version != FORMAT_VERSION {
        return Err(super::super::validation::unsupported_version(document.version).into());
    }
    Ok(document)
}

struct DocumentSeed<'a> {
    limits: &'a SafetyLimits,
    exceeded: &'a mut Option<LimitExceeded>,
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
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "version" if version.is_none() => version = Some(map.next_value()?),
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
        Ok(MaterialDocument {
            version: version.ok_or_else(|| de::Error::missing_field("version"))?,
            nodes: nodes.ok_or_else(|| de::Error::missing_field("nodes"))?,
            edges: edges.ok_or_else(|| de::Error::missing_field("edges"))?,
            exposed_parameters: exposed.unwrap_or_default(),
        })
    }
}
struct Bounded<'a, T> {
    limits: &'a SafetyLimits,
    exceeded: &'a mut Option<LimitExceeded>,
    kind: LimitKind,
    marker: PhantomData<T>,
}
impl<'a, T> Bounded<'a, T> {
    fn new(
        limits: &'a SafetyLimits,
        exceeded: &'a mut Option<LimitExceeded>,
        kind: LimitKind,
    ) -> Self {
        Self {
            limits,
            exceeded,
            kind,
            marker: PhantomData,
        }
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
            if items.len() as u64 == self.limits.maximum(self.kind) {
                if seq.next_element::<IgnoredAny>()?.is_some() {
                    let error = LimitExceeded {
                        kind: self.kind,
                        configured: self.limits.maximum(self.kind),
                        observed: (items.len() as u64).saturating_add(1),
                    };
                    *self.exceeded = Some(error);
                    return Err(de::Error::custom(error));
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

pub(super) fn parameters<'de, D: serde::Deserializer<'de>>(
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
