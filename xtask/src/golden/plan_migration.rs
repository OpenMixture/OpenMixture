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
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AllocationMigration {
    schema_version: u32,
    kind: String,
    migrations: Vec<AllocationEntry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AllocationEntry {
    material: String,
    case: String,
    v2: String,
    v3: String,
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
    if record.migrations.iter().any(|entry| {
        entry.material == material
            && entry.case == case
            && entry.v1 == baseline
            && entry.v2 == current
    }) {
        return Ok(true);
    }
    let allocation: AllocationMigration =
        serde_json::from_str(include_str!("../../../docs/plan-v3-migration.json"))?;
    if allocation.schema_version != 1
        || allocation.kind != "exact-resource-free-plan-v2-to-v3-allocation"
    {
        return Err("unsupported explicit allocation migration record".into());
    }
    Ok(allocation.migrations.iter().any(|entry| {
        entry.material == material
            && entry.case == case
            && entry.v3 == current
            && (entry.v2 == baseline
                || record.migrations.iter().any(|old| {
                    old.material == material
                        && old.case == case
                        && old.v1 == baseline
                        && old.v2 == entry.v2
                }))
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
    #[test]
    fn allocation_migration_requires_exact_chain_and_rejects_reverse_or_changed_hashes() {
        let old: Migration =
            serde_json::from_str(include_str!("../../../docs/plan-v2-migration.json")).unwrap();
        let new: AllocationMigration =
            serde_json::from_str(include_str!("../../../docs/plan-v3-migration.json")).unwrap();
        assert_eq!(new.migrations.len(), 11);
        for entry in new.migrations {
            let prior = old
                .migrations
                .iter()
                .find(|e| e.material == entry.material && e.case == entry.case)
                .unwrap();
            assert_eq!(prior.v2, entry.v2);
            assert!(matches(&entry.material, &entry.case, &prior.v1, &entry.v3).unwrap());
            assert!(matches(&entry.material, &entry.case, &entry.v2, &entry.v3).unwrap());
            for (material, case, baseline, current) in [
                (
                    "other",
                    entry.case.as_str(),
                    prior.v1.as_str(),
                    entry.v3.as_str(),
                ),
                (
                    entry.material.as_str(),
                    "other",
                    prior.v1.as_str(),
                    entry.v3.as_str(),
                ),
                (
                    entry.material.as_str(),
                    entry.case.as_str(),
                    "changed",
                    entry.v3.as_str(),
                ),
                (
                    entry.material.as_str(),
                    entry.case.as_str(),
                    prior.v1.as_str(),
                    "changed",
                ),
                (
                    entry.material.as_str(),
                    entry.case.as_str(),
                    entry.v3.as_str(),
                    entry.v2.as_str(),
                ),
            ] {
                assert!(!matches(material, case, baseline, current).unwrap());
            }
        }
    }
}
