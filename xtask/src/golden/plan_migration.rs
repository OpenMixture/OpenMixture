//! Exact, finite plan-schema migration; never relax pixel or semantic gates.
use crate::TaskResult;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Migration {
    schema_version: u32,
    kind: String,
    migrations: Vec<Entry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    material: String,
    case: String,
    v1: String,
    v2: String,
}

pub(super) fn matches(
    material: &str,
    case: &str,
    baseline: &str,
    current: &str,
) -> TaskResult<bool> {
    if baseline == current {
        return Ok(true);
    }
    let record: Migration =
        serde_json::from_str(include_str!("../../../docs/plan-v2-migration.json"))?;
    if record.schema_version != 1 || record.kind != "exact-resource-free-plan-v1-to-v2" {
        return Err("unsupported explicit plan migration record".into());
    }
    Ok(record.migrations.iter().any(|entry| {
        entry.material == material
            && entry.case == case
            && entry.v1 == baseline
            && entry.v2 == current
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migration_requires_exact_material_case_and_both_hashes() {
        let record: Migration =
            serde_json::from_str(include_str!("../../../docs/plan-v2-migration.json")).unwrap();
        assert_eq!(record.migrations.len(), 11);
        for entry in record.migrations {
            assert!(matches(&entry.material, &entry.case, &entry.v1, &entry.v2).unwrap());
            assert!(!matches("other", &entry.case, &entry.v1, &entry.v2).unwrap());
            assert!(!matches(&entry.material, "other", &entry.v1, &entry.v2).unwrap());
            assert!(!matches(&entry.material, &entry.case, "changed", &entry.v2).unwrap());
            assert!(!matches(&entry.material, &entry.case, &entry.v1, "changed").unwrap());
            assert!(!matches(&entry.material, &entry.case, &entry.v2, &entry.v1).unwrap());
        }
    }
}
