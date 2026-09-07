//! Versioned repository fixture contracts; no graph or pixel execution semantics.

use crate::TaskResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub(super) const CHANNELS: [&str; 4] = ["baseColor", "normal", "roughness", "height"];
pub(super) const REVISION: &str = "694585a05946e1ed49b6bd577ca6537cbb57f025";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Acceptance {
    pub schema_version: u32,
    pub material: String,
    pub size: u32,
    pub software_revision: String,
    pub hardware_tolerance: Tolerance,
    pub channels: BTreeMap<String, Channel>,
    pub cases: Vec<Case>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Channel {
    pub encoding: String,
    pub source: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Tolerance {
    pub max_absolute: u8,
    pub mean_absolute: f64,
    pub pixel_threshold: u8,
    pub max_changed_pixel_ratio: f64,
}
impl Tolerance {
    pub const EXACT: Self = Self {
        max_absolute: 0,
        mean_absolute: 0.0,
        pixel_threshold: 0,
        max_changed_pixel_ratio: 0.0,
    };
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Case {
    pub id: String,
    /// null for the default; other cases load `variants/<id>.json`.
    pub variant: Option<String>,
    pub checks: BTreeMap<String, Structure>,
    /// Every variant states both its affected and its unchanged channels.
    pub changes: BTreeMap<String, Change>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Structure {
    Uniform {
        rgba: [u8; 4],
        tolerance: u8,
    },
    #[serde(rename_all = "camelCase")]
    Alternating {
        cells: [u32; 2],
        min_contrast: u8,
        max_balance_error: f64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum Change {
    Unchanged,
    #[serde(rename_all = "camelCase")]
    Changed {
        min_pixel_ratio: f64,
    },
    #[serde(rename_all = "camelCase")]
    MeanIncreases {
        min_delta: f64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Variant {
    pub overrides: BTreeMap<String, Value>,
}

pub(super) fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.starts_with(|c: char| c.is_ascii_lowercase())
        && id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

impl Acceptance {
    pub fn validate(&self, id: &str) -> TaskResult {
        let channels = CHANNELS.into_iter().collect::<BTreeSet<_>>();
        if self.schema_version != 1
            || self.material != id
            || !valid_id(id)
            || self.size != 1024
            || self.software_revision != REVISION
            || self
                .channels
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
                != channels
            || !(3..=16).contains(&self.cases.len())
        {
            return Err("invalid material acceptance contract (v1 requires 1K, four channels, default and at least two variants, and the pinned driver)".into());
        }
        let tolerance = self.hardware_tolerance;
        if !tolerance.mean_absolute.is_finite()
            || !(0.0..=255.0).contains(&tolerance.mean_absolute)
            || !unit(tolerance.max_changed_pixel_ratio)
        {
            return Err("invalid hardware tolerance".into());
        }
        for (id, channel) in &self.channels {
            let encoding = if id == "baseColor" {
                "rgba8-srgb"
            } else {
                "rgba8-linear"
            };
            if channel.encoding != encoding
                || !matches!(channel.source.as_str(), "connected" | "default")
            {
                return Err(format!("invalid encoding/provenance contract for {id}").into());
            }
        }
        let mut ids = BTreeSet::new();
        for (index, case) in self.cases.iter().enumerate() {
            if !valid_id(&case.id)
                || !ids.insert(&case.id)
                || case
                    .checks
                    .keys()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>()
                    != channels
                || (index == 0
                    && (case.id != "default" || case.variant.is_some() || !case.changes.is_empty()))
                || (index != 0
                    && (case.variant.as_deref() != Some(case.id.as_str())
                        || case
                            .changes
                            .keys()
                            .map(String::as_str)
                            .collect::<BTreeSet<_>>()
                            != channels
                        || case
                            .changes
                            .values()
                            .all(|c| matches!(c, Change::Unchanged))))
            {
                return Err(format!(
                    "invalid default/variant/check coverage for case {}",
                    case.id
                )
                .into());
            }
            for check in case.checks.values() {
                if let Structure::Alternating {
                    cells,
                    min_contrast,
                    max_balance_error,
                } = check
                    && (*min_contrast == 0
                        || !unit(*max_balance_error)
                        || cells.iter().any(|c| {
                            *c < 4
                                || *c > self.size / 2
                                || c % 2 != 0
                                || !self.size.is_multiple_of(*c)
                        }))
                {
                    return Err("alternating structure requires resolved even cells, contrast, and a valid balance tolerance".into());
                }
            }
            for change in case.changes.values() {
                let threshold = match change {
                    Change::Unchanged => continue,
                    Change::Changed { min_pixel_ratio } => *min_pixel_ratio,
                    Change::MeanIncreases { min_delta } => *min_delta,
                };
                if !unit(threshold) || threshold == 0.0 {
                    return Err("causality thresholds must be positive and at most one".into());
                }
            }
        }
        Ok(())
    }
}

fn unit(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}
