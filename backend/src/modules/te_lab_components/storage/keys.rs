//! Key layout of the Lab catalog records.

pub(super) use crate::platform::store::keys::is_numeric_id;

/// Row keys of the retired v1 Lab model, removed when a namespace moves to the catalog.
pub(super) const LEGACY_ENTRY_PREFIXES: [&str; 2] = ["entry:", "entry_id:"];
pub(super) const PART_PREFIX: &str = "part:";
pub(super) const PART_ID_PREFIX: &str = "part_id:";
pub(super) const PART_NUMBER_PREFIX: &str = "part_number:";
pub(super) const STORAGE_AREA_PREFIX: &str = "storage-area:";
pub(super) const STORAGE_AREA_NAME_PREFIX: &str = "storage-area-name:";
pub(super) const STORAGE_CONTAINER_PREFIX: &str = "storage-container:";
pub(super) const STORAGE_CONTAINER_NAME_PREFIX: &str = "storage-container-name:";
pub(super) const STOCK_PLACEMENT_PREFIX: &str = "stock-placement:";
pub(super) const PLACEMENT_PART_INDEX_PREFIX: &str = "placement-part:";
pub(super) const PLACEMENT_CONTAINER_INDEX_PREFIX: &str = "placement-container:";

pub(super) fn part_key(part_uuid: &str) -> String {
    format!("{PART_PREFIX}{part_uuid}")
}

pub(super) fn part_id_key(part_id: &str) -> String {
    format!("{PART_ID_PREFIX}{part_id}")
}

pub(super) fn part_number_key(normalized_part_number: &str) -> String {
    format!("{PART_NUMBER_PREFIX}{normalized_part_number}")
}

pub(super) fn storage_area_key(area_uuid: &str) -> String {
    format!("{STORAGE_AREA_PREFIX}{area_uuid}")
}

pub(super) fn storage_area_name_key(normalized_name: &str) -> String {
    format!("{STORAGE_AREA_NAME_PREFIX}{normalized_name}")
}

pub(super) fn storage_container_key(container_uuid: &str) -> String {
    format!("{STORAGE_CONTAINER_PREFIX}{container_uuid}")
}

pub(super) fn storage_container_name_key(area_uuid: &str, normalized_name: &str) -> String {
    format!("{STORAGE_CONTAINER_NAME_PREFIX}{area_uuid}:{normalized_name}")
}

pub(super) fn stock_placement_key(placement_uuid: &str) -> String {
    format!("{STOCK_PLACEMENT_PREFIX}{placement_uuid}")
}

pub(super) fn placement_part_index_prefix(part_uuid: &str) -> String {
    format!("{PLACEMENT_PART_INDEX_PREFIX}{part_uuid}:")
}

pub(super) fn placement_part_index_key(part_uuid: &str, placement_uuid: &str) -> String {
    format!("{}{placement_uuid}", placement_part_index_prefix(part_uuid))
}

pub(super) fn placement_container_index_prefix(container_uuid: &str) -> String {
    format!("{PLACEMENT_CONTAINER_INDEX_PREFIX}{container_uuid}:")
}

pub(super) fn placement_container_index_key(container_uuid: &str, placement_uuid: &str) -> String {
    format!(
        "{}{placement_uuid}",
        placement_container_index_prefix(container_uuid)
    )
}
