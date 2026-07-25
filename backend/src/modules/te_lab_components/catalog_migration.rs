use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::modules::te_lab_components::{
    catalog_model::{
        normalized_lookup, normalized_part_number, validate_part, validate_stock_placement,
        validate_storage_area, validate_storage_container, CatalogMigrationStatus,
        LegacyPartFields, Part, StockPlacement, StorageArea, StorageContainer,
        CATALOG_SCHEMA_VERSION,
    },
    model::{now_timestamp, CommandResult, InventoryEntry},
    store::InventoryDb,
};

const MIGRATION_MAPPING_VERSION: &str = "te-lab-components-catalog-v2";
const LEGACY_AREA_UUID: &str = "0000000000004000800000000000a014";
const LEGACY_CONTAINER_UUID: &str = "0000000000004000800000000000b014";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MigrationDuplicateGroup {
    pub key: String,
    pub entry_uuids: Vec<String>,
    pub entry_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogMigrationPreview {
    pub mapping_version: String,
    pub source_fingerprint: String,
    pub source_schema_version: Option<u32>,
    pub target_schema_version: u32,
    pub legacy_rows: usize,
    pub proposed_parts: usize,
    pub proposed_placements: usize,
    pub archived_parts: usize,
    pub blank_positive_quantity_locations: usize,
    pub generated_part_uuids: usize,
    pub duplicate_internal_part_numbers: Vec<MigrationDuplicateGroup>,
    pub likely_mpn_duplicates: Vec<MigrationDuplicateGroup>,
    pub invalid_rows: Vec<String>,
    pub warnings: Vec<String>,
    pub blocking: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct CatalogMigrationCommitInput {
    pub source_fingerprint: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogMigrationCommitResult {
    pub source_fingerprint: String,
    pub parts_created: usize,
    pub placements_created: usize,
    pub areas_created: usize,
    pub containers_created: usize,
    pub noop: bool,
    pub message: String,
}

struct MigrationEntities {
    parts: Vec<Part>,
    areas: Vec<StorageArea>,
    containers: Vec<StorageContainer>,
    placements: Vec<StockPlacement>,
    generated_part_uuids: usize,
    blank_positive_quantity_locations: usize,
    invalid_rows: Vec<String>,
}

pub(crate) fn catalog_migration_status(db: &InventoryDb) -> CommandResult<CatalogMigrationStatus> {
    let schema_version = db.schema_version()?;
    let legacy_entry_count = db.load_entries()?.len();
    let has_catalog_entities = db.has_catalog_entities()?;
    let catalog_initialized = schema_version == Some(CATALOG_SCHEMA_VERSION);
    let required = !catalog_initialized && (legacy_entry_count > 0 || has_catalog_entities);
    let message = if catalog_initialized {
        "Lab Components catalog schema v2 is ready.".to_string()
    } else if required {
        "Legacy Lab Components data requires a reviewed catalog migration before editing or shared sync."
            .to_string()
    } else {
        "No legacy Lab rows were found; catalog schema v2 will initialize on first use.".to_string()
    };

    Ok(CatalogMigrationStatus {
        schema_version,
        required,
        legacy_entry_count,
        catalog_initialized,
        message,
    })
}

pub(crate) fn ensure_catalog_initialized(db: &InventoryDb) -> CommandResult<()> {
    let status = catalog_migration_status(db)?;
    if status.catalog_initialized {
        return Ok(());
    }
    if status.required {
        return Err(status.message);
    }

    let sync_backup = db.backup_sync_state()?;
    if let Err(error) = db.reset_sync_state_for_catalog_v2() {
        let _ = db.restore_sync_state(sync_backup);
        return Err(error);
    }
    db.set_schema_version(CATALOG_SCHEMA_VERSION)?;
    db.flush();
    Ok(())
}

pub(crate) fn preview_catalog_migration(
    db: &InventoryDb,
) -> CommandResult<CatalogMigrationPreview> {
    let mut entries = db.load_entries()?;
    entries.sort_by(|left, right| {
        left.entry_uuid
            .cmp(&right.entry_uuid)
            .then_with(|| left.id.cmp(&right.id))
    });
    let source_schema_version = db.schema_version()?;
    let source_fingerprint = migration_fingerprint(source_schema_version, &entries)?;
    let entities = build_migration_entities(&entries);
    let duplicate_internal_part_numbers = duplicate_groups(
        &entries,
        |entry| normalized_part_number(&entry.asset_number),
        false,
    );
    let likely_mpn_duplicates = duplicate_groups(
        &entries,
        |entry| {
            let manufacturer = normalized_lookup(&entry.manufacturer);
            let mpn = normalized_lookup(&entry.model);
            if manufacturer.is_empty() || mpn.is_empty() {
                String::new()
            } else {
                format!("{manufacturer} | {mpn}")
            }
        },
        true,
    );
    let archived_parts = entries.iter().filter(|entry| entry.archived).count();
    let mut warnings = vec![
        "Every migrated quantity uses unit 'unknown' until a person reviews it.".to_string(),
        "Legacy free-text locations remain unstructured and are not parsed into grid coordinates."
            .to_string(),
        "Legacy verification becomes count state 'legacy_verified'; no count timestamp is invented."
            .to_string(),
    ];
    if !likely_mpn_duplicates.is_empty() {
        warnings.push(format!(
            "{} likely manufacturer/MPN duplicate group(s) require review; no records will be merged automatically.",
            likely_mpn_duplicates.len()
        ));
    }
    if entities.generated_part_uuids > 0 {
        warnings.push(format!(
            "{} row(s) had no entry UUID and receive deterministic migration identities.",
            entities.generated_part_uuids
        ));
    }
    let has_partial_catalog =
        source_schema_version != Some(CATALOG_SCHEMA_VERSION) && db.has_catalog_entities()?;
    if has_partial_catalog {
        warnings.push(
            "Partial catalog-v2 keys already exist and will be replaced only during confirmed migration."
                .to_string(),
        );
    }
    let blocking = !duplicate_internal_part_numbers.is_empty()
        || !entities.invalid_rows.is_empty()
        || (entries.is_empty() && has_partial_catalog);

    Ok(CatalogMigrationPreview {
        mapping_version: MIGRATION_MAPPING_VERSION.to_string(),
        source_fingerprint,
        source_schema_version,
        target_schema_version: CATALOG_SCHEMA_VERSION,
        legacy_rows: entries.len(),
        proposed_parts: entities.parts.len(),
        proposed_placements: entities.placements.len(),
        archived_parts,
        blank_positive_quantity_locations: entities.blank_positive_quantity_locations,
        generated_part_uuids: entities.generated_part_uuids,
        duplicate_internal_part_numbers,
        likely_mpn_duplicates,
        invalid_rows: entities.invalid_rows,
        warnings,
        blocking,
    })
}

pub(crate) fn commit_catalog_migration(
    input: CatalogMigrationCommitInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMigrationCommitResult> {
    if !input.confirmed {
        return Err("Confirm the reviewed Lab catalog migration before committing.".to_string());
    }
    if db.schema_version()? == Some(CATALOG_SCHEMA_VERSION) {
        return Ok(CatalogMigrationCommitResult {
            source_fingerprint: input.source_fingerprint,
            parts_created: db.load_parts()?.len(),
            placements_created: db.load_stock_placements()?.len(),
            areas_created: db.load_storage_areas()?.len(),
            containers_created: db.load_storage_containers()?.len(),
            noop: true,
            message: "Lab Components catalog migration was already committed.".to_string(),
        });
    }

    let preview = preview_catalog_migration(db)?;
    if preview.source_fingerprint != input.source_fingerprint {
        return Err(
            "Legacy Lab data changed after the migration preview. Run the dry-run again before committing."
                .to_string(),
        );
    }
    if preview.blocking {
        return Err(
            "Migration is blocked by duplicate internal part numbers, invalid rows, or partial data. Review the dry-run report."
                .to_string(),
        );
    }

    let entries = db.load_entries()?;
    let entities = build_migration_entities(&entries);
    let sync_backup = db.backup_sync_state()?;
    let write_result = (|| -> CommandResult<()> {
        db.clear_catalog_entities()?;
        for area in &entities.areas {
            validate_storage_area(area)?;
            db.put_storage_area(area)?;
        }
        for container in &entities.containers {
            validate_storage_container(container)?;
            db.put_storage_container(container)?;
        }
        for part in &entities.parts {
            validate_part(part)?;
            db.put_part(part)?;
        }
        for placement in &entities.placements {
            let container = entities
                .containers
                .iter()
                .find(|container| container.container_uuid == placement.container_uuid)
                .ok_or_else(|| {
                    "Migrated placement is missing its storage container.".to_string()
                })?;
            validate_stock_placement(placement, container)?;
            db.put_stock_placement(placement)?;
        }
        db.reset_sync_state_for_catalog_v2()?;
        db.set_schema_version(CATALOG_SCHEMA_VERSION)?;
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = db.clear_catalog_entities();
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(format!("Lab catalog migration rolled back: {error}"));
    }

    db.flush();
    Ok(CatalogMigrationCommitResult {
        source_fingerprint: preview.source_fingerprint,
        parts_created: entities.parts.len(),
        placements_created: entities.placements.len(),
        areas_created: entities.areas.len(),
        containers_created: entities.containers.len(),
        noop: false,
        message: format!(
            "Migrated {} Lab component part(s) and {} stock placement(s) to catalog schema v2.",
            entities.parts.len(),
            entities.placements.len()
        ),
    })
}

fn build_migration_entities(entries: &[InventoryEntry]) -> MigrationEntities {
    let timestamp = migration_timestamp(entries);
    let mut parts = Vec::with_capacity(entries.len());
    let mut placements = Vec::new();
    let mut generated_part_uuids = 0usize;
    let mut blank_positive_quantity_locations = 0usize;
    let mut invalid_rows = Vec::new();

    for entry in entries {
        let entry_uuid = if entry.entry_uuid.trim().is_empty() {
            generated_part_uuids += 1;
            deterministic_uuid("part", &format!("{}|{}", entry.id, entry.asset_number))
        } else {
            entry.entry_uuid.trim().to_string()
        };
        let part = Part {
            id: entry.id.clone(),
            database_id: entry.database_id.or_else(|| entry.id.parse::<i64>().ok()),
            entry_uuid: entry_uuid.clone(),
            internal_part_number: entry.asset_number.trim().to_string(),
            category: String::new(),
            subcategory: String::new(),
            manufacturer: entry.manufacturer.trim().to_string(),
            manufacturer_part_number: entry.model.trim().to_string(),
            display_value: String::new(),
            mounting_type: String::new(),
            package_type: String::new(),
            description: entry.description.trim().to_string(),
            attributes: BTreeMap::new(),
            supplier: String::new(),
            supplier_sku: String::new(),
            supplier_packaging: String::new(),
            product_url: entry.links.trim().to_string(),
            datasheet_url: String::new(),
            default_unit_of_measure: "unknown".to_string(),
            reorder_point: None,
            target_quantity: None,
            part_status: "unknown".to_string(),
            picture_path: entry.picture_path.trim().to_string(),
            notes: entry.notes.trim().to_string(),
            archived: entry.archived,
            legacy: LegacyPartFields {
                serial_number: entry.serial_number.clone(),
                project_name: entry.project_name.clone(),
                assigned_to: entry.assigned_to.clone(),
                lifecycle_status: entry.lifecycle_status.clone(),
                working_status: entry.working_status.clone(),
                condition: entry.condition.clone(),
                verified_in_survey: entry.verified_in_survey,
                manual_entry: entry.manual_entry,
            },
            created_at: entry.created_at.clone(),
            updated_at: entry.updated_at.clone(),
        };
        if let Err(error) = validate_part(&part) {
            invalid_rows.push(format!("Entry {}: {error}", entry.id));
        }

        if let Some(quantity) = entry.qty {
            let raw_location = entry.location.trim();
            let freeform_position = if raw_location.is_empty() && quantity > 0.0 {
                blank_positive_quantity_locations += 1;
                "Unassigned".to_string()
            } else {
                raw_location.to_string()
            };
            placements.push(StockPlacement {
                placement_uuid: deterministic_uuid("placement", &entry_uuid),
                part_uuid: entry_uuid.clone(),
                container_uuid: LEGACY_CONTAINER_UUID.to_string(),
                column_index: None,
                row_index: None,
                freeform_position,
                quantity,
                unit_of_measure: "unknown".to_string(),
                packaging: String::new(),
                lot_code: String::new(),
                date_code: String::new(),
                condition: "unknown".to_string(),
                count_state: if entry.verified_in_survey {
                    "legacy_verified".to_string()
                } else {
                    "uncounted".to_string()
                },
                last_counted_at: None,
                last_counted_by: String::new(),
                notes: String::new(),
                archived: false,
                created_at: entry.created_at.clone(),
                updated_at: entry.updated_at.clone(),
            });
        }
        parts.push(part);
    }

    let (areas, containers) = if placements.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        (
            vec![StorageArea {
                area_uuid: LEGACY_AREA_UUID.to_string(),
                name: "Legacy / Unstructured".to_string(),
                area_type: "other".to_string(),
                owner: String::new(),
                description: "Migration-owned area preserving original free-text Lab locations."
                    .to_string(),
                archived: false,
                created_at: timestamp.clone(),
                updated_at: timestamp.clone(),
            }],
            vec![StorageContainer {
                container_uuid: LEGACY_CONTAINER_UUID.to_string(),
                area_uuid: LEGACY_AREA_UUID.to_string(),
                name: "Imported Locations".to_string(),
                container_type: "other".to_string(),
                grid_enabled: false,
                row_count: None,
                column_count: None,
                row_start: 1,
                origin: "top_left".to_string(),
                description:
                    "Original Lab location text retained without automatic coordinate parsing."
                        .to_string(),
                archived: false,
                created_at: timestamp.clone(),
                updated_at: timestamp,
            }],
        )
    };

    MigrationEntities {
        parts,
        areas,
        containers,
        placements,
        generated_part_uuids,
        blank_positive_quantity_locations,
        invalid_rows,
    }
}

fn duplicate_groups(
    entries: &[InventoryEntry],
    key_for: impl Fn(&InventoryEntry) -> String,
    ignore_blank: bool,
) -> Vec<MigrationDuplicateGroup> {
    let mut groups = BTreeMap::<String, Vec<&InventoryEntry>>::new();
    for entry in entries {
        let key = key_for(entry);
        if key.is_empty() && ignore_blank {
            continue;
        }
        if key.is_empty() {
            continue;
        }
        groups.entry(key).or_default().push(entry);
    }
    groups
        .into_iter()
        .filter(|(_, entries)| entries.len() > 1)
        .map(|(key, entries)| MigrationDuplicateGroup {
            key,
            entry_uuids: entries
                .iter()
                .map(|entry| entry.entry_uuid.clone())
                .collect(),
            entry_ids: entries.iter().map(|entry| entry.id.clone()).collect(),
        })
        .collect()
}

fn migration_fingerprint(
    schema_version: Option<u32>,
    entries: &[InventoryEntry],
) -> CommandResult<String> {
    #[derive(Serialize)]
    struct FingerprintSource<'a> {
        mapping_version: &'a str,
        schema_version: Option<u32>,
        entries: &'a [InventoryEntry],
    }
    let bytes = serde_json::to_vec(&FingerprintSource {
        mapping_version: MIGRATION_MAPPING_VERSION,
        schema_version,
        entries,
    })
    .map_err(|error| error.to_string())?;
    let digest = Sha256::digest(bytes);
    Ok(format!("sha256:{digest:x}"))
}

fn deterministic_uuid(namespace: &str, value: &str) -> String {
    let digest = Sha256::digest(format!("{namespace}:{value}").as_bytes());
    format!("{digest:x}")[..32].to_string()
}

fn migration_timestamp(entries: &[InventoryEntry]) -> String {
    entries
        .iter()
        .filter_map(|entry| {
            let value = entry.created_at.trim();
            (!value.is_empty()).then(|| value.to_string())
        })
        .min()
        .unwrap_or_else(now_timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::te_lab_components::{model::InventoryEntry, store::SyncKeyspace};
    use std::{env, fs, path::PathBuf};
    use uuid::Uuid;

    #[test]
    fn dry_run_is_stable_and_does_not_mutate() {
        let db = test_db();
        db.put_entry(&legacy_entry(
            "1",
            "uuid-1",
            "LAB-1",
            Some(12.5),
            "Cabinet C7",
            true,
        ))
        .unwrap();
        db.flush();

        let first = preview_catalog_migration(&db).unwrap();
        let second = preview_catalog_migration(&db).unwrap();

        assert_eq!(first.source_fingerprint, second.source_fingerprint);
        assert_eq!(first.proposed_parts, 1);
        assert_eq!(first.proposed_placements, 1);
        assert!(!first.blocking);
        assert!(db.load_parts().unwrap().is_empty());
        assert_eq!(db.schema_version().unwrap(), None);
    }

    #[test]
    fn duplicate_internal_part_numbers_block_commit() {
        let db = test_db();
        db.put_entry(&legacy_entry(
            "1",
            "uuid-1",
            "LAB-1",
            Some(1.0),
            "A1",
            false,
        ))
        .unwrap();
        db.put_entry(&legacy_entry(
            "2",
            "uuid-2",
            " lab-1 ",
            Some(1.0),
            "A2",
            false,
        ))
        .unwrap();

        let preview = preview_catalog_migration(&db).unwrap();
        assert!(preview.blocking);
        assert_eq!(preview.duplicate_internal_part_numbers.len(), 1);
        let error = commit_catalog_migration(
            CatalogMigrationCommitInput {
                source_fingerprint: preview.source_fingerprint,
                confirmed: true,
            },
            &db,
        )
        .unwrap_err();
        assert!(error.contains("blocked"));
        assert!(db.load_parts().unwrap().is_empty());
    }

    #[test]
    fn commit_preserves_identity_quantity_location_and_legacy_fields() {
        let db = test_db();
        let mut entry = legacy_entry(
            "42",
            "uuid-42",
            "LAB-42",
            Some(3.5),
            " Cabinet A / C7 ",
            true,
        );
        entry.serial_number = "legacy-serial".to_string();
        entry.project_name = "Bench Controls".to_string();
        entry.assigned_to = "Alex".to_string();
        entry.lifecycle_status = "repair".to_string();
        entry.working_status = "limited".to_string();
        entry.condition = "Bent lead".to_string();
        entry.links = "https://example.com/product".to_string();
        db.put_entry(&entry).unwrap();
        db.put_sync_outbox_record(1, &serde_json::json!({"schemaVersion": 1, "legacy": true}))
            .unwrap();
        db.set_next_local_seq(2).unwrap();

        let preview = preview_catalog_migration(&db).unwrap();
        let result = commit_catalog_migration(
            CatalogMigrationCommitInput {
                source_fingerprint: preview.source_fingerprint.clone(),
                confirmed: true,
            },
            &db,
        )
        .unwrap();

        assert!(!result.noop);
        assert_eq!(db.schema_version().unwrap(), Some(CATALOG_SCHEMA_VERSION));
        assert_eq!(db.sync_schema_version().unwrap(), Some(2));
        let part = db.find_part("uuid-42").unwrap().unwrap();
        assert_eq!(part.id, "42");
        assert_eq!(part.internal_part_number, "LAB-42");
        assert_eq!(part.manufacturer_part_number, "MPN-42");
        assert_eq!(part.product_url, "https://example.com/product");
        assert_eq!(part.legacy.serial_number, "legacy-serial");
        assert_eq!(part.legacy.project_name, "Bench Controls");
        assert_eq!(part.legacy.condition, "Bent lead");
        let placement = db
            .load_stock_placements_for_part("uuid-42")
            .unwrap()
            .pop()
            .unwrap();
        assert_eq!(placement.quantity, 3.5);
        assert_eq!(placement.unit_of_measure, "unknown");
        assert_eq!(placement.freeform_position, "Cabinet A / C7");
        assert_eq!(placement.count_state, "legacy_verified");
        assert!(placement.last_counted_at.is_none());
        assert_eq!(outbox_count(&db), 0);
        assert_eq!(db.next_local_seq().unwrap(), 1);

        let second = commit_catalog_migration(
            CatalogMigrationCommitInput {
                source_fingerprint: preview.source_fingerprint,
                confirmed: true,
            },
            &db,
        )
        .unwrap();
        assert!(second.noop);
        assert_eq!(db.load_parts().unwrap().len(), 1);
    }

    #[test]
    fn commit_requires_current_fingerprint_and_confirmation() {
        let db = test_db();
        db.put_entry(&legacy_entry(
            "1",
            "uuid-1",
            "LAB-1",
            Some(1.0),
            "A1",
            false,
        ))
        .unwrap();
        let preview = preview_catalog_migration(&db).unwrap();

        assert!(commit_catalog_migration(
            CatalogMigrationCommitInput {
                source_fingerprint: preview.source_fingerprint.clone(),
                confirmed: false,
            },
            &db,
        )
        .unwrap_err()
        .contains("Confirm"));

        let mut changed = db.find_entry("uuid-1").unwrap().unwrap();
        changed.description = "Changed after preview".to_string();
        db.put_entry(&changed).unwrap();
        assert!(commit_catalog_migration(
            CatalogMigrationCommitInput {
                source_fingerprint: preview.source_fingerprint,
                confirmed: true,
            },
            &db,
        )
        .unwrap_err()
        .contains("changed"));
    }

    #[test]
    fn fixture_migration_matches_the_reviewed_acceptance_contract() {
        let db = test_db();
        let entries: Vec<InventoryEntry> = read_fixture("legacy_catalog_v1.json");
        for entry in entries {
            db.put_entry(&entry).unwrap();
        }
        let contract: PreviewContract = read_fixture("migration_preview_contract.json");
        let expected: ExpectedCatalog = read_fixture("catalog_v2_expected.json");

        let preview = preview_catalog_migration(&db).unwrap();
        assert_eq!(preview.mapping_version, contract.mapping_version);
        assert_eq!(
            preview.target_schema_version,
            contract.target_schema_version
        );
        assert_eq!(preview.legacy_rows, contract.legacy_rows);
        assert_eq!(preview.proposed_parts, contract.proposed_parts);
        assert_eq!(preview.proposed_placements, contract.proposed_placements);
        assert_eq!(preview.archived_parts, contract.archived_parts);
        assert_eq!(
            preview.blank_positive_quantity_locations,
            contract.blank_positive_quantity_locations
        );
        assert_eq!(preview.generated_part_uuids, contract.generated_part_uuids);
        assert_eq!(
            preview.duplicate_internal_part_numbers.len(),
            contract.duplicate_internal_part_number_groups
        );
        assert_eq!(
            preview.likely_mpn_duplicates.len(),
            contract.likely_mpn_duplicate_groups
        );
        assert_eq!(preview.invalid_rows.len(), contract.invalid_rows);
        assert_eq!(preview.blocking, contract.blocking);

        commit_catalog_migration(
            CatalogMigrationCommitInput {
                source_fingerprint: preview.source_fingerprint,
                confirmed: true,
            },
            &db,
        )
        .unwrap();

        let mut actual_parts = db
            .load_parts()
            .unwrap()
            .into_iter()
            .map(ExpectedPart::from)
            .collect::<Vec<_>>();
        actual_parts.sort_by(|left, right| left.entry_uuid.cmp(&right.entry_uuid));
        let mut expected_parts = expected.parts;
        expected_parts.sort_by(|left, right| left.entry_uuid.cmp(&right.entry_uuid));
        assert_eq!(actual_parts, expected_parts);

        let mut actual_placements = db
            .load_stock_placements()
            .unwrap()
            .into_iter()
            .map(ExpectedPlacement::from)
            .collect::<Vec<_>>();
        actual_placements.sort_by(|left, right| left.part_uuid.cmp(&right.part_uuid));
        let mut expected_placements = expected.placements;
        expected_placements.sort_by(|left, right| left.part_uuid.cmp(&right.part_uuid));
        assert_eq!(actual_placements, expected_placements);
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PreviewContract {
        mapping_version: String,
        target_schema_version: u32,
        legacy_rows: usize,
        proposed_parts: usize,
        proposed_placements: usize,
        archived_parts: usize,
        blank_positive_quantity_locations: usize,
        generated_part_uuids: usize,
        duplicate_internal_part_number_groups: usize,
        likely_mpn_duplicate_groups: usize,
        invalid_rows: usize,
        blocking: bool,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ExpectedCatalog {
        parts: Vec<ExpectedPart>,
        placements: Vec<ExpectedPlacement>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    struct ExpectedPart {
        entry_uuid: String,
        internal_part_number: String,
        manufacturer_part_number: String,
        product_url: String,
        archived: bool,
        legacy_serial_number: String,
        legacy_project_name: String,
        legacy_assigned_to: String,
        legacy_lifecycle_status: String,
        legacy_working_status: String,
        legacy_condition: String,
        legacy_verified_in_survey: bool,
        legacy_manual_entry: bool,
    }

    impl From<Part> for ExpectedPart {
        fn from(part: Part) -> Self {
            Self {
                entry_uuid: part.entry_uuid,
                internal_part_number: part.internal_part_number,
                manufacturer_part_number: part.manufacturer_part_number,
                product_url: part.product_url,
                archived: part.archived,
                legacy_serial_number: part.legacy.serial_number,
                legacy_project_name: part.legacy.project_name,
                legacy_assigned_to: part.legacy.assigned_to,
                legacy_lifecycle_status: part.legacy.lifecycle_status,
                legacy_working_status: part.legacy.working_status,
                legacy_condition: part.legacy.condition,
                legacy_verified_in_survey: part.legacy.verified_in_survey,
                legacy_manual_entry: part.legacy.manual_entry,
            }
        }
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "camelCase")]
    struct ExpectedPlacement {
        part_uuid: String,
        freeform_position: String,
        quantity: f64,
        unit_of_measure: String,
        count_state: String,
    }

    impl From<StockPlacement> for ExpectedPlacement {
        fn from(placement: StockPlacement) -> Self {
            Self {
                part_uuid: placement.part_uuid,
                freeform_position: placement.freeform_position,
                quantity: placement.quantity,
                unit_of_measure: placement.unit_of_measure,
                count_state: placement.count_state,
            }
        }
    }

    fn read_fixture<T: for<'de> Deserialize<'de>>(name: &str) -> T {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("te_lab_components")
            .join(name);
        let bytes = fs::read(path).unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    fn legacy_entry(
        id: &str,
        entry_uuid: &str,
        asset_number: &str,
        qty: Option<f64>,
        location: &str,
        verified: bool,
    ) -> InventoryEntry {
        InventoryEntry {
            id: id.to_string(),
            database_id: id.parse::<i64>().ok(),
            entry_uuid: entry_uuid.to_string(),
            asset_number: asset_number.to_string(),
            serial_number: String::new(),
            qty,
            manufacturer: "Stackpole".to_string(),
            model: format!("MPN-{id}"),
            description: format!("Legacy component {id}"),
            project_name: String::new(),
            location: location.to_string(),
            assigned_to: String::new(),
            links: String::new(),
            notes: String::new(),
            lifecycle_status: "active".to_string(),
            working_status: "unknown".to_string(),
            condition: String::new(),
            verified_in_survey: verified,
            archived: false,
            manual_entry: false,
            picture_path: String::new(),
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-02T00:00:00.000Z".to_string(),
        }
    }

    fn outbox_count(db: &InventoryDb) -> usize {
        let mut count = 0;
        db.scan_sync_range(SyncKeyspace::Outbox, usize::MAX, |_, _| {
            count += 1;
            Ok(true)
        })
        .unwrap();
        count
    }

    fn test_db() -> InventoryDb {
        let root = unique_test_dir("catalog-migration");
        fs::create_dir_all(&root).unwrap();
        InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap()
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
    }
}
