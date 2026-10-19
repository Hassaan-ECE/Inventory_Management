use super::types::*;
use super::*;
use crate::modules::te_lab_components::catalog_model::{
    create_part, create_stock_placement, create_storage_area, create_storage_container,
    normalize_part_input, normalize_stock_placement_input, normalize_storage_area_input,
    normalize_storage_container_input, ComponentAttributeValue, PartInput, StockPlacementInput,
    StorageAreaInput, StorageContainerInput,
};
use crate::modules::te_lab_components::catalog_model::{
    Part, StockPlacement, StorageArea, StorageContainer,
};
use crate::platform::{
    common::now_timestamp,
    sync::{operation_file_name, sha256_hex},
};
use std::{env, fs, path::PathBuf};
use uuid::Uuid;

/// The exact bytes Lab writes to the share. 0.1.4 clients verify these checksums, so any
/// refactor of the catalog file code must leave every value here unchanged.
#[test]
fn shared_catalog_files_keep_their_exact_bytes() {
    let part = golden_part();
    let mut operation = CatalogOperationEnvelope {
        schema_version: CATALOG_SYNC_SCHEMA_VERSION,
        op_id: "op-golden".to_string(),
        client_id: "client-golden".to_string(),
        device_id: "device-golden".to_string(),
        local_seq: 7,
        app_version: "0.1.4".to_string(),
        created_at_utc: "2026-10-19T12:00:00.000Z".to_string(),
        operation_type: CatalogOperationType::Upsert,
        entity_type: CatalogEntityType::Part,
        entity_id: part.entry_uuid.clone(),
        base_version: Some("2026-10-18T12:00:00.000Z".to_string()),
        mutation_ts_utc: "2026-10-19T12:00:00.000Z".to_string(),
        payload: CatalogOperationPayload {
            entity: Some(part.clone().into()),
            changed_fields: vec!["notes".to_string()],
            deleted_at_utc: None,
        },
        checksum: String::new(),
        auth: None,
    };
    operation.checksum = checksum(&operation).unwrap();
    let snapshot = CatalogSnapshot {
        schema_version: CATALOG_SNAPSHOT_SCHEMA_VERSION,
        sync_schema_version: CATALOG_SYNC_SCHEMA_VERSION,
        snapshot_id: "snapshot-golden".to_string(),
        app_version: "0.1.4".to_string(),
        source_client_id: "client-golden".to_string(),
        created_at_utc: "2026-10-19T12:00:00.000Z".to_string(),
        parts: vec![part],
        storage_areas: Vec::new(),
        storage_containers: Vec::new(),
        stock_placements: Vec::new(),
        tombstones: Vec::new(),
        entity_states: Vec::new(),
        watermarks: vec![CatalogWatermark {
            client_id: "client-golden".to_string(),
            local_seq: 7,
        }],
        checksum: String::new(),
        auth: None,
    };

    let digests = [
        operation_file_name(7),
        operation.checksum.clone(),
        sha256_hex(&canonical_json(&operation).unwrap()),
        checksum(&snapshot).unwrap(),
        sha256_hex(&canonical_json(&snapshot).unwrap()),
    ];
    assert_eq!(digests, GOLDEN_CATALOG_FILES);
}

const GOLDEN_CATALOG_FILES: [&str; 5] = [
    "000000000007.op.json",
    "sha256:bf1f327acbdd4f0177b87cfe8a7dc1d1bd7822c1ce7887facaa372baec342c89",
    "2da563325f5009b4e47456f80c944b4d567635cb8faf627df2fe94229b68453d",
    "sha256:94f542a45801df2df18019c1a04bfcd1990bd5803ea44746bd35eca660b95ce2",
    "38029c103f6f29dc8aad09fd3024d460ca7d94ea6670e789cc7f06c9d047b1cf",
];

fn golden_part() -> Part {
    Part {
        id: "1".to_string(),
        database_id: Some(1),
        entry_uuid: "part-golden".to_string(),
        internal_part_number: "R-10K".to_string(),
        category: "Resistor".to_string(),
        subcategory: String::new(),
        manufacturer: "Yageo".to_string(),
        manufacturer_part_number: "CFR-25JB-52-10K".to_string(),
        display_value: "10kΩ".to_string(),
        mounting_type: "through_hole".to_string(),
        package_type: "Axial".to_string(),
        description: "10 kΩ ±5% 1/4 W".to_string(),
        attributes: [(
            "resistance".to_string(),
            ComponentAttributeValue {
                value: "10".to_string(),
                unit: "kΩ".to_string(),
            },
        )]
        .into_iter()
        .collect(),
        supplier: String::new(),
        supplier_sku: String::new(),
        supplier_packaging: String::new(),
        product_url: String::new(),
        datasheet_url: String::new(),
        default_unit_of_measure: "pcs".to_string(),
        reorder_point: Some(10.0),
        target_quantity: None,
        part_status: "active".to_string(),
        picture_path: String::new(),
        notes: "Golden".to_string(),
        archived: false,
        legacy: Default::default(),
        created_at: "2026-10-18T12:00:00.000Z".to_string(),
        updated_at: "2026-10-19T12:00:00.000Z".to_string(),
    }
}

#[test]
fn first_sync_sets_up_a_new_shared_catalog_and_blocks_the_v1_stream() {
    let db = test_db("catalog-share-db");
    let root = unique_test_dir("catalog-share-root");
    fs::create_dir_all(&root).unwrap();

    let result = run_shared_sync_with_root(&db, root.clone()).unwrap();

    assert!(result.shared.available);
    let shared = root.join("shared");
    assert!(shared.join("inventory").is_file());
    assert!(shared.join("catalog-v2").join("cutover.json").is_file());
    assert!(shared.join("catalog-v2").join("manifest.json").is_file());
}

#[test]
fn a_share_still_holding_the_v1_stream_stays_local_only() {
    let db = test_db("catalog-v1-share-db");
    let root = unique_test_dir("catalog-v1-share-root");
    let legacy = root.join("shared").join("inventory");
    fs::create_dir_all(&legacy).unwrap();

    let result = run_shared_sync_with_root(&db, root.clone()).unwrap();

    assert!(!result.shared.available);
    assert!(legacy.is_dir());
    assert!(!root.join("shared").join("catalog-v2").exists());
}

#[test]
fn two_databases_sync_every_catalog_entity_and_dependency_safe_deletes() {
    let first = test_db("catalog-sync-first");
    let second = test_db("catalog-sync-second");
    let root = unique_test_dir("catalog-sync-root");
    fs::create_dir_all(&root).unwrap();
    let (part, area, container, placement) = seed_catalog(&first);
    queue_upsert_operation(&first, area.clone().into(), Vec::new(), None).unwrap();
    queue_upsert_operation(&first, container.clone().into(), Vec::new(), None).unwrap();
    queue_upsert_operation(&first, part.clone().into(), Vec::new(), None).unwrap();
    queue_upsert_operation(&first, placement.clone().into(), Vec::new(), None).unwrap();
    run_shared_sync_with_root(&first, root.clone()).unwrap();

    run_shared_sync_with_root(&second, root.clone()).unwrap();
    assert_eq!(second.load_parts().unwrap(), vec![part.clone()]);
    assert_eq!(second.load_storage_areas().unwrap(), vec![area.clone()]);
    assert_eq!(
        second.load_storage_containers().unwrap(),
        vec![container.clone()]
    );
    assert_eq!(
        second.load_stock_placements().unwrap(),
        vec![placement.clone()]
    );

    first.delete_stock_placement(&placement).unwrap();
    queue_delete_operation(
        &first,
        CatalogEntityType::StockPlacement,
        &placement.placement_uuid,
        now_timestamp(),
        Some(placement.updated_at.clone()),
    )
    .unwrap();
    first.delete_part(&part).unwrap();
    queue_delete_operation(
        &first,
        CatalogEntityType::Part,
        &part.entry_uuid,
        now_timestamp(),
        Some(part.updated_at.clone()),
    )
    .unwrap();
    first.delete_storage_container(&container).unwrap();
    queue_delete_operation(
        &first,
        CatalogEntityType::StorageContainer,
        &container.container_uuid,
        now_timestamp(),
        Some(container.updated_at.clone()),
    )
    .unwrap();
    first.delete_storage_area(&area).unwrap();
    queue_delete_operation(
        &first,
        CatalogEntityType::StorageArea,
        &area.area_uuid,
        now_timestamp(),
        Some(area.updated_at.clone()),
    )
    .unwrap();
    publish_pending_local_changes_with_root(&first, root.clone()).unwrap();
    run_shared_sync_with_root(&second, root).unwrap();

    assert!(second.load_parts().unwrap().is_empty());
    assert!(second.load_storage_areas().unwrap().is_empty());
    assert!(second.load_storage_containers().unwrap().is_empty());
    assert!(second.load_stock_placements().unwrap().is_empty());
}

#[test]
fn simultaneous_quantity_edits_converge_without_additive_merge() {
    let first = test_db("catalog-conflict-first");
    let second = test_db("catalog-conflict-second");
    let root = unique_test_dir("catalog-conflict-root");
    fs::create_dir_all(&root).unwrap();
    let (part, area, container, placement) = seed_catalog(&first);
    queue_upsert_operation(&first, area.into(), Vec::new(), None).unwrap();
    queue_upsert_operation(&first, container.into(), Vec::new(), None).unwrap();
    queue_upsert_operation(&first, part.into(), Vec::new(), None).unwrap();
    queue_upsert_operation(&first, placement.clone().into(), Vec::new(), None).unwrap();
    run_shared_sync_with_root(&first, root.clone()).unwrap();
    run_shared_sync_with_root(&second, root.clone()).unwrap();

    let base_version = placement.updated_at.clone();
    let mutation_timestamp = "2026-07-25T12:00:00.000Z".to_string();
    let mut first_edit = first
        .find_stock_placement(&placement.placement_uuid)
        .unwrap()
        .unwrap();
    first_edit.quantity = 11.0;
    first_edit.updated_at = mutation_timestamp.clone();
    first.put_stock_placement(&first_edit).unwrap();
    queue_upsert_operation(
        &first,
        first_edit.into(),
        vec!["quantity".to_string()],
        Some(base_version.clone()),
    )
    .unwrap();

    let mut second_edit = second
        .find_stock_placement(&placement.placement_uuid)
        .unwrap()
        .unwrap();
    second_edit.quantity = 29.0;
    second_edit.updated_at = mutation_timestamp;
    second.put_stock_placement(&second_edit).unwrap();
    queue_upsert_operation(
        &second,
        second_edit.into(),
        vec!["quantity".to_string()],
        Some(base_version),
    )
    .unwrap();

    publish_pending_local_changes_with_root(&first, root.clone()).unwrap();
    publish_pending_local_changes_with_root(&second, root.clone()).unwrap();
    run_shared_sync_with_root(&first, root.clone()).unwrap();
    run_shared_sync_with_root(&second, root.clone()).unwrap();
    run_shared_sync_with_root(&first, root.clone()).unwrap();
    run_shared_sync_with_root(&second, root).unwrap();

    let first_quantity = first
        .find_stock_placement(&placement.placement_uuid)
        .unwrap()
        .unwrap()
        .quantity;
    let second_quantity = second
        .find_stock_placement(&placement.placement_uuid)
        .unwrap()
        .unwrap()
        .quantity;
    assert_eq!(first_quantity, second_quantity);
    assert!(matches!(first_quantity, 11.0 | 29.0));
    assert_ne!(first_quantity, 40.0);
    assert!(conflict_count(&first) + conflict_count(&second) > 0);
}

fn seed_catalog(db: &InventoryDb) -> (Part, StorageArea, StorageContainer, StockPlacement) {
    let part = create_part(
        1,
        normalize_part_input(PartInput {
            internal_part_number: "LAB-001".to_string(),
            description: "1 kOhm resistor".to_string(),
            default_unit_of_measure: "pcs".to_string(),
            ..PartInput::default()
        }),
    );
    let area = create_storage_area(normalize_storage_area_input(StorageAreaInput {
        name: "Main Lab".to_string(),
        area_type: "lab".to_string(),
        ..StorageAreaInput::default()
    }));
    let container =
        create_storage_container(normalize_storage_container_input(StorageContainerInput {
            area_uuid: area.area_uuid.clone(),
            name: "Cabinet 1".to_string(),
            container_type: "cabinet".to_string(),
            grid_enabled: true,
            row_count: Some(4),
            column_count: Some(4),
            ..StorageContainerInput::default()
        }));
    let placement = create_stock_placement(normalize_stock_placement_input(
        StockPlacementInput {
            part_uuid: part.entry_uuid.clone(),
            container_uuid: container.container_uuid.clone(),
            row_index: Some(1),
            column_index: Some(2),
            quantity: 10.0,
            unit_of_measure: "pcs".to_string(),
            ..StockPlacementInput::default()
        },
        Some("pcs"),
    ));
    db.put_storage_area(&area).unwrap();
    db.put_storage_container(&container).unwrap();
    db.put_part(&part).unwrap();
    db.put_stock_placement(&placement).unwrap();
    db.set_next_part_id(2).unwrap();
    (part, area, container, placement)
}

fn conflict_count(db: &InventoryDb) -> usize {
    let mut count = 0usize;
    db.scan_sync_conflict_records::<CatalogConflictRecord, _>(usize::MAX, |_, _| {
        count += 1;
        Ok(true)
    })
    .unwrap();
    count
}

fn test_db(prefix: &str) -> InventoryDb {
    let root = unique_test_dir(prefix);
    fs::create_dir_all(&root).unwrap();
    let db = InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap();
    db.ensure_catalog_schema().unwrap();
    db
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
}
