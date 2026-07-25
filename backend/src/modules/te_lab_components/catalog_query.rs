use crate::modules::te_lab_components::{
    catalog_migration::{catalog_migration_status, ensure_catalog_initialized},
    catalog_model::{catalog_counts, summarize_part_stock, CatalogSyncResult, PartStockSummary},
    model::{CommandResult, InventorySharedStatus},
    store::InventoryDb,
};

pub(crate) fn load_catalog_from_store(
    db: &InventoryDb,
    mut shared: InventorySharedStatus,
    entries_changed: Option<bool>,
) -> CommandResult<CatalogSyncResult> {
    let mut migration = catalog_migration_status(db)?;
    if !migration.required && !migration.catalog_initialized {
        ensure_catalog_initialized(db)?;
        migration = catalog_migration_status(db)?;
    }

    if migration.required {
        shared.can_modify = false;
        shared.mutation_mode = "migration_required".to_string();
        shared.message = migration.message.clone();
        return Ok(CatalogSyncResult {
            db_path: db.db_path_string(),
            parts: Vec::new(),
            storage_areas: Vec::new(),
            storage_containers: Vec::new(),
            stock_placements: Vec::new(),
            summaries: Vec::new(),
            counts: Default::default(),
            migration,
            entries_changed,
            shared,
        });
    }

    let parts = db.load_parts()?;
    let storage_areas = db.load_storage_areas()?;
    let storage_containers = db.load_storage_containers()?;
    let stock_placements = db.load_stock_placements()?;
    let summaries = parts
        .iter()
        .map(|part| summarize_part_stock(part, &stock_placements))
        .collect::<Vec<PartStockSummary>>();
    let counts = catalog_counts(&parts, &summaries);

    Ok(CatalogSyncResult {
        db_path: db.db_path_string(),
        parts,
        storage_areas,
        storage_containers,
        stock_placements,
        summaries,
        counts,
        migration,
        entries_changed,
        shared,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::te_lab_components::{
        catalog_model::{create_part, normalize_part_input, PartInput},
        model::InventorySharedStatus,
    };
    use std::{env, fs, path::PathBuf};
    use uuid::Uuid;

    #[test]
    fn empty_store_initializes_catalog_v2_and_returns_derived_counts() {
        let db = test_db();
        let loaded = load_catalog_from_store(&db, shared_status(), Some(true)).unwrap();

        assert!(loaded.migration.catalog_initialized);
        assert_eq!(loaded.migration.schema_version, Some(2));
        assert_eq!(loaded.counts.total_parts, 0);
        assert!(loaded.shared.can_modify);
    }

    #[test]
    fn load_derives_stock_summaries_instead_of_persisting_totals() {
        let db = test_db();
        ensure_catalog_initialized(&db).unwrap();
        let part = create_part(
            1,
            normalize_part_input(PartInput {
                description: "Catalog-only part".to_string(),
                default_unit_of_measure: "pcs".to_string(),
                ..PartInput::default()
            }),
        );
        db.put_part(&part).unwrap();

        let loaded = load_catalog_from_store(&db, shared_status(), Some(true)).unwrap();
        assert_eq!(loaded.summaries.len(), 1);
        assert!(loaded.summaries[0].totals.is_empty());
        assert_eq!(loaded.counts.no_stock, 1);
    }

    fn shared_status() -> InventorySharedStatus {
        InventorySharedStatus {
            available: false,
            can_modify: true,
            enabled: false,
            has_local_only_changes: Some(false),
            message: "Local test store".to_string(),
            mutation_mode: "local".to_string(),
            revision: None,
            last_snapshot_id: None,
            shared_root_path: None,
        }
    }

    fn test_db() -> InventoryDb {
        let root = unique_test_dir("catalog-query");
        fs::create_dir_all(&root).unwrap();
        InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap()
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
    }
}
