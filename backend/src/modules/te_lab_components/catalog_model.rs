use std::{collections::BTreeMap, path::Path};

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::modules::te_lab_components::model::{
    now_timestamp, CommandResult, InventorySharedStatus,
};

pub(crate) const CATALOG_SCHEMA_VERSION: u32 = 2;
pub(crate) const MAX_CATALOG_QUANTITY: f64 = 1_000_000_000.0;
const STANDARD_TEXT_LIMIT: usize = 512;
const LONG_TEXT_LIMIT: usize = 4_000;
const NOTES_TEXT_LIMIT: usize = 8_000;
const PATH_TEXT_LIMIT: usize = 2_048;
const MAX_GRID_DIMENSION: u32 = 1_000;
const IMAGE_PATH_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff"];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComponentAttributeValue {
    pub value: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub unit: String,
}

pub(crate) type ComponentAttributes = BTreeMap<String, ComponentAttributeValue>;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct LegacyPartFields {
    pub serial_number: String,
    pub project_name: String,
    pub assigned_to: String,
    pub lifecycle_status: String,
    pub working_status: String,
    pub condition: String,
    pub verified_in_survey: bool,
    pub manual_entry: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Part {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_id: Option<i64>,
    pub entry_uuid: String,
    #[serde(default)]
    pub internal_part_number: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub subcategory: String,
    #[serde(default)]
    pub manufacturer: String,
    #[serde(default)]
    pub manufacturer_part_number: String,
    #[serde(default)]
    pub display_value: String,
    #[serde(default)]
    pub mounting_type: String,
    #[serde(default)]
    pub package_type: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub attributes: ComponentAttributes,
    #[serde(default)]
    pub supplier: String,
    #[serde(default)]
    pub supplier_sku: String,
    #[serde(default)]
    pub supplier_packaging: String,
    #[serde(default)]
    pub product_url: String,
    #[serde(default)]
    pub datasheet_url: String,
    #[serde(default = "default_unit_of_measure")]
    pub default_unit_of_measure: String,
    pub reorder_point: Option<f64>,
    pub target_quantity: Option<f64>,
    #[serde(default = "default_part_status")]
    pub part_status: String,
    #[serde(default)]
    pub picture_path: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub legacy: LegacyPartFields,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct PartInput {
    pub internal_part_number: String,
    pub category: String,
    pub subcategory: String,
    pub manufacturer: String,
    pub manufacturer_part_number: String,
    pub display_value: String,
    pub mounting_type: String,
    pub package_type: String,
    pub description: String,
    pub attributes: ComponentAttributes,
    pub supplier: String,
    pub supplier_sku: String,
    pub supplier_packaging: String,
    pub product_url: String,
    pub datasheet_url: String,
    pub default_unit_of_measure: String,
    pub reorder_point: Option<f64>,
    pub target_quantity: Option<f64>,
    pub part_status: String,
    pub picture_path: Option<String>,
    pub notes: String,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageArea {
    pub area_uuid: String,
    pub name: String,
    pub area_type: String,
    pub owner: String,
    pub description: String,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct StorageAreaInput {
    pub name: String,
    pub area_type: String,
    pub owner: String,
    pub description: String,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StorageContainer {
    pub container_uuid: String,
    pub area_uuid: String,
    pub name: String,
    pub container_type: String,
    pub grid_enabled: bool,
    pub row_count: Option<u32>,
    pub column_count: Option<u32>,
    pub row_start: u32,
    pub origin: String,
    pub description: String,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct StorageContainerInput {
    pub area_uuid: String,
    pub name: String,
    pub container_type: String,
    pub grid_enabled: bool,
    pub row_count: Option<u32>,
    pub column_count: Option<u32>,
    pub row_start: Option<u32>,
    pub description: String,
    pub archived: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StockPlacement {
    pub placement_uuid: String,
    pub part_uuid: String,
    pub container_uuid: String,
    pub column_index: Option<u32>,
    pub row_index: Option<u32>,
    pub freeform_position: String,
    pub quantity: f64,
    pub unit_of_measure: String,
    pub packaging: String,
    pub lot_code: String,
    pub date_code: String,
    pub condition: String,
    pub count_state: String,
    pub last_counted_at: Option<String>,
    pub last_counted_by: String,
    pub notes: String,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct StockPlacementInput {
    pub part_uuid: String,
    pub container_uuid: String,
    pub column_index: Option<u32>,
    pub row_index: Option<u32>,
    pub freeform_position: String,
    pub quantity: f64,
    pub unit_of_measure: String,
    pub packaging: String,
    pub lot_code: String,
    pub date_code: String,
    pub condition: String,
    pub count_state: String,
    pub last_counted_at: Option<String>,
    pub last_counted_by: String,
    pub notes: String,
    pub archived: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct StockMoveInput {
    pub source_placement_uuid: String,
    pub destination_placement_uuid: Option<String>,
    pub destination: Option<StockPlacementInput>,
    pub quantity: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct StockCountInput {
    pub quantity: f64,
    pub counted_by: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuantityTotal {
    pub unit_of_measure: String,
    pub quantity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StockStatus {
    Archived,
    NoStock,
    UnitReview,
    LowStock,
    InStock,
    MixedUnits,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PartStockSummary {
    pub part_uuid: String,
    pub totals: Vec<QuantityTotal>,
    pub stock_status: StockStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogCounts {
    pub active_parts: usize,
    pub archived_parts: usize,
    pub total_parts: usize,
    pub no_stock: usize,
    pub low_stock: usize,
    pub unit_review: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogMigrationStatus {
    pub schema_version: Option<u32>,
    pub required: bool,
    pub legacy_entry_count: usize,
    pub catalog_initialized: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogSyncResult {
    pub db_path: String,
    pub parts: Vec<Part>,
    pub storage_areas: Vec<StorageArea>,
    pub storage_containers: Vec<StorageContainer>,
    pub stock_placements: Vec<StockPlacement>,
    pub summaries: Vec<PartStockSummary>,
    pub counts: CatalogCounts,
    pub migration: CatalogMigrationStatus,
    pub entries_changed: Option<bool>,
    pub shared: InventorySharedStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogMutationResult<T> {
    pub value: T,
    pub message: String,
    pub mutation_mode: String,
    pub shared: InventorySharedStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogDeleteResult {
    pub entity_uuid: String,
    pub message: String,
    pub mutation_mode: String,
    pub shared: InventorySharedStatus,
}

pub(crate) fn normalize_part_input(input: PartInput) -> PartInput {
    let mut attributes = ComponentAttributes::new();
    for (key, value) in input.attributes {
        let key = normalize_attribute_key(&key);
        let value = ComponentAttributeValue {
            value: value.value.trim().to_string(),
            unit: value.unit.trim().to_string(),
        };
        if !key.is_empty() && !value.value.is_empty() {
            attributes.insert(key, value);
        }
    }

    PartInput {
        internal_part_number: input.internal_part_number.trim().to_string(),
        category: input.category.trim().to_string(),
        subcategory: input.subcategory.trim().to_string(),
        manufacturer: input.manufacturer.trim().to_string(),
        manufacturer_part_number: input.manufacturer_part_number.trim().to_string(),
        display_value: input.display_value.trim().to_string(),
        mounting_type: input.mounting_type.trim().to_string(),
        package_type: input.package_type.trim().to_string(),
        description: input.description.trim().to_string(),
        attributes,
        supplier: input.supplier.trim().to_string(),
        supplier_sku: input.supplier_sku.trim().to_string(),
        supplier_packaging: input.supplier_packaging.trim().to_string(),
        product_url: input.product_url.trim().to_string(),
        datasheet_url: input.datasheet_url.trim().to_string(),
        default_unit_of_measure: normalize_unit(&input.default_unit_of_measure),
        reorder_point: input.reorder_point,
        target_quantity: input.target_quantity,
        part_status: normalize_enum_text(&input.part_status, "active"),
        picture_path: input.picture_path.map(|value| value.trim().to_string()),
        notes: input.notes.trim().to_string(),
        archived: input.archived,
    }
}

pub(crate) fn validate_part_input(input: &PartInput) -> CommandResult<()> {
    if input.internal_part_number.is_empty()
        && input.manufacturer_part_number.is_empty()
        && input.display_value.is_empty()
        && input.description.is_empty()
    {
        return Err(
            "Provide an internal part number, manufacturer part number, value/label, or description before saving."
                .to_string(),
        );
    }

    validate_text_length(
        "Internal part number",
        &input.internal_part_number,
        STANDARD_TEXT_LIMIT,
    )?;
    validate_text_length("Category", &input.category, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Subcategory", &input.subcategory, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Manufacturer", &input.manufacturer, STANDARD_TEXT_LIMIT)?;
    validate_text_length(
        "Manufacturer part number",
        &input.manufacturer_part_number,
        STANDARD_TEXT_LIMIT,
    )?;
    validate_text_length("Value / label", &input.display_value, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Mounting type", &input.mounting_type, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Package type", &input.package_type, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Description", &input.description, LONG_TEXT_LIMIT)?;
    validate_text_length("Supplier", &input.supplier, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Supplier SKU", &input.supplier_sku, STANDARD_TEXT_LIMIT)?;
    validate_text_length(
        "Supplier packaging",
        &input.supplier_packaging,
        STANDARD_TEXT_LIMIT,
    )?;
    validate_optional_http_url("Product URL", &input.product_url)?;
    validate_optional_http_url("Datasheet URL", &input.datasheet_url)?;
    validate_unit(&input.default_unit_of_measure)?;
    validate_optional_quantity("Reorder point", input.reorder_point)?;
    validate_optional_quantity("Target quantity", input.target_quantity)?;
    if let (Some(reorder_point), Some(target_quantity)) =
        (input.reorder_point, input.target_quantity)
    {
        if target_quantity < reorder_point {
            return Err(
                "Target quantity must be greater than or equal to the reorder point.".to_string(),
            );
        }
    }
    validate_text_length("Part status", &input.part_status, STANDARD_TEXT_LIMIT)?;
    if let Some(picture_path) = &input.picture_path {
        validate_text_length("Picture path", picture_path, PATH_TEXT_LIMIT)?;
        validate_picture_path(picture_path)?;
    }
    validate_text_length("Notes", &input.notes, NOTES_TEXT_LIMIT)?;

    for (key, value) in &input.attributes {
        if normalize_attribute_key(key) != *key {
            return Err(format!("Attribute key '{key}' is not normalized."));
        }
        validate_text_length("Attribute key", key, STANDARD_TEXT_LIMIT)?;
        validate_text_length("Attribute value", &value.value, LONG_TEXT_LIMIT)?;
        validate_text_length("Attribute unit", &value.unit, STANDARD_TEXT_LIMIT)?;
    }

    Ok(())
}

pub(crate) fn create_part(id: i64, input: PartInput) -> Part {
    let timestamp = now_timestamp();
    Part {
        id: id.to_string(),
        database_id: Some(id),
        entry_uuid: Uuid::new_v4().simple().to_string(),
        internal_part_number: input.internal_part_number,
        category: input.category,
        subcategory: input.subcategory,
        manufacturer: input.manufacturer,
        manufacturer_part_number: input.manufacturer_part_number,
        display_value: input.display_value,
        mounting_type: input.mounting_type,
        package_type: input.package_type,
        description: input.description,
        attributes: input.attributes,
        supplier: input.supplier,
        supplier_sku: input.supplier_sku,
        supplier_packaging: input.supplier_packaging,
        product_url: input.product_url,
        datasheet_url: input.datasheet_url,
        default_unit_of_measure: input.default_unit_of_measure,
        reorder_point: input.reorder_point,
        target_quantity: input.target_quantity,
        part_status: input.part_status,
        picture_path: input.picture_path.unwrap_or_default(),
        notes: input.notes,
        archived: input.archived,
        legacy: LegacyPartFields::default(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
    }
}

pub(crate) fn update_part(mut part: Part, input: PartInput) -> Part {
    part.internal_part_number = input.internal_part_number;
    part.category = input.category;
    part.subcategory = input.subcategory;
    part.manufacturer = input.manufacturer;
    part.manufacturer_part_number = input.manufacturer_part_number;
    part.display_value = input.display_value;
    part.mounting_type = input.mounting_type;
    part.package_type = input.package_type;
    part.description = input.description;
    part.attributes = input.attributes;
    part.supplier = input.supplier;
    part.supplier_sku = input.supplier_sku;
    part.supplier_packaging = input.supplier_packaging;
    part.product_url = input.product_url;
    part.datasheet_url = input.datasheet_url;
    part.default_unit_of_measure = input.default_unit_of_measure;
    part.reorder_point = input.reorder_point;
    part.target_quantity = input.target_quantity;
    part.part_status = input.part_status;
    part.picture_path = input.picture_path.unwrap_or_default();
    part.notes = input.notes;
    part.archived = input.archived;
    part.updated_at = now_timestamp();
    part
}

pub(crate) fn validate_part(part: &Part) -> CommandResult<()> {
    if part.id.trim().is_empty() || part.entry_uuid.trim().is_empty() {
        return Err("Part identity is incomplete.".to_string());
    }
    validate_part_input(&PartInput {
        internal_part_number: part.internal_part_number.clone(),
        category: part.category.clone(),
        subcategory: part.subcategory.clone(),
        manufacturer: part.manufacturer.clone(),
        manufacturer_part_number: part.manufacturer_part_number.clone(),
        display_value: part.display_value.clone(),
        mounting_type: part.mounting_type.clone(),
        package_type: part.package_type.clone(),
        description: part.description.clone(),
        attributes: part.attributes.clone(),
        supplier: part.supplier.clone(),
        supplier_sku: part.supplier_sku.clone(),
        supplier_packaging: part.supplier_packaging.clone(),
        product_url: part.product_url.clone(),
        datasheet_url: part.datasheet_url.clone(),
        default_unit_of_measure: part.default_unit_of_measure.clone(),
        reorder_point: part.reorder_point,
        target_quantity: part.target_quantity,
        part_status: part.part_status.clone(),
        picture_path: Some(part.picture_path.clone()),
        notes: part.notes.clone(),
        archived: part.archived,
    })
}

pub(crate) fn normalize_storage_area_input(input: StorageAreaInput) -> StorageAreaInput {
    StorageAreaInput {
        name: input.name.trim().to_string(),
        area_type: normalize_enum_text(&input.area_type, "other"),
        owner: input.owner.trim().to_string(),
        description: input.description.trim().to_string(),
        archived: input.archived,
    }
}

pub(crate) fn validate_storage_area_input(input: &StorageAreaInput) -> CommandResult<()> {
    if input.name.is_empty() {
        return Err("Storage area name is required.".to_string());
    }
    validate_text_length("Storage area name", &input.name, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Storage area type", &input.area_type, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Storage area owner", &input.owner, STANDARD_TEXT_LIMIT)?;
    validate_text_length(
        "Storage area description",
        &input.description,
        LONG_TEXT_LIMIT,
    )
}

pub(crate) fn create_storage_area(input: StorageAreaInput) -> StorageArea {
    let timestamp = now_timestamp();
    StorageArea {
        area_uuid: Uuid::new_v4().simple().to_string(),
        name: input.name,
        area_type: input.area_type,
        owner: input.owner,
        description: input.description,
        archived: input.archived,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    }
}

pub(crate) fn update_storage_area(mut area: StorageArea, input: StorageAreaInput) -> StorageArea {
    area.name = input.name;
    area.area_type = input.area_type;
    area.owner = input.owner;
    area.description = input.description;
    area.archived = input.archived;
    area.updated_at = now_timestamp();
    area
}

pub(crate) fn validate_storage_area(area: &StorageArea) -> CommandResult<()> {
    if area.area_uuid.trim().is_empty() {
        return Err("Storage area identity is missing.".to_string());
    }
    validate_storage_area_input(&StorageAreaInput {
        name: area.name.clone(),
        area_type: area.area_type.clone(),
        owner: area.owner.clone(),
        description: area.description.clone(),
        archived: area.archived,
    })
}

pub(crate) fn normalize_storage_container_input(
    input: StorageContainerInput,
) -> StorageContainerInput {
    StorageContainerInput {
        area_uuid: input.area_uuid.trim().to_string(),
        name: input.name.trim().to_string(),
        container_type: normalize_enum_text(&input.container_type, "other"),
        grid_enabled: input.grid_enabled,
        row_count: input.row_count,
        column_count: input.column_count,
        row_start: Some(input.row_start.unwrap_or(1)),
        description: input.description.trim().to_string(),
        archived: input.archived,
    }
}

pub(crate) fn validate_storage_container_input(input: &StorageContainerInput) -> CommandResult<()> {
    if input.area_uuid.is_empty() {
        return Err("Storage container must belong to an area.".to_string());
    }
    if input.name.is_empty() {
        return Err("Storage container name is required.".to_string());
    }
    validate_text_length("Storage container name", &input.name, STANDARD_TEXT_LIMIT)?;
    validate_text_length(
        "Storage container type",
        &input.container_type,
        STANDARD_TEXT_LIMIT,
    )?;
    validate_text_length(
        "Storage container description",
        &input.description,
        LONG_TEXT_LIMIT,
    )?;
    let row_start = input.row_start.unwrap_or(1);
    if row_start == 0 {
        return Err("Grid row numbering must start at 1 or greater.".to_string());
    }
    if input.grid_enabled {
        validate_grid_dimension("Grid rows", input.row_count)?;
        validate_grid_dimension("Grid columns", input.column_count)?;
    } else if input.row_count.is_some() || input.column_count.is_some() {
        return Err("Non-grid containers cannot define row or column counts.".to_string());
    }
    Ok(())
}

pub(crate) fn create_storage_container(input: StorageContainerInput) -> StorageContainer {
    let timestamp = now_timestamp();
    StorageContainer {
        container_uuid: Uuid::new_v4().simple().to_string(),
        area_uuid: input.area_uuid,
        name: input.name,
        container_type: input.container_type,
        grid_enabled: input.grid_enabled,
        row_count: input.row_count,
        column_count: input.column_count,
        row_start: input.row_start.unwrap_or(1),
        origin: "top_left".to_string(),
        description: input.description,
        archived: input.archived,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    }
}

pub(crate) fn update_storage_container(
    mut container: StorageContainer,
    input: StorageContainerInput,
) -> StorageContainer {
    container.area_uuid = input.area_uuid;
    container.name = input.name;
    container.container_type = input.container_type;
    container.grid_enabled = input.grid_enabled;
    container.row_count = input.row_count;
    container.column_count = input.column_count;
    container.row_start = input.row_start.unwrap_or(1);
    container.origin = "top_left".to_string();
    container.description = input.description;
    container.archived = input.archived;
    container.updated_at = now_timestamp();
    container
}

pub(crate) fn validate_storage_container(container: &StorageContainer) -> CommandResult<()> {
    if container.container_uuid.trim().is_empty() {
        return Err("Storage container identity is missing.".to_string());
    }
    if container.origin != "top_left" {
        return Err("Only top-left grid origin is supported in this release.".to_string());
    }
    validate_storage_container_input(&StorageContainerInput {
        area_uuid: container.area_uuid.clone(),
        name: container.name.clone(),
        container_type: container.container_type.clone(),
        grid_enabled: container.grid_enabled,
        row_count: container.row_count,
        column_count: container.column_count,
        row_start: Some(container.row_start),
        description: container.description.clone(),
        archived: container.archived,
    })
}

pub(crate) fn normalize_stock_placement_input(
    input: StockPlacementInput,
    default_unit: Option<&str>,
) -> StockPlacementInput {
    let requested_unit = normalize_unit(&input.unit_of_measure);
    let unit_of_measure = if requested_unit.is_empty() {
        default_unit
            .map(normalize_unit)
            .unwrap_or_else(default_unit_of_measure)
    } else {
        requested_unit
    };
    StockPlacementInput {
        part_uuid: input.part_uuid.trim().to_string(),
        container_uuid: input.container_uuid.trim().to_string(),
        column_index: input.column_index,
        row_index: input.row_index,
        freeform_position: input.freeform_position.trim().to_string(),
        quantity: input.quantity,
        unit_of_measure,
        packaging: input.packaging.trim().to_string(),
        lot_code: input.lot_code.trim().to_string(),
        date_code: input.date_code.trim().to_string(),
        condition: normalize_enum_text(&input.condition, "unknown"),
        count_state: normalize_enum_text(&input.count_state, "uncounted"),
        last_counted_at: input
            .last_counted_at
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        last_counted_by: input.last_counted_by.trim().to_string(),
        notes: input.notes.trim().to_string(),
        archived: input.archived,
    }
}

pub(crate) fn validate_stock_placement_input(
    input: &StockPlacementInput,
    container: &StorageContainer,
) -> CommandResult<()> {
    if input.part_uuid.is_empty() {
        return Err("Stock placement must reference a part.".to_string());
    }
    if input.container_uuid != container.container_uuid {
        return Err("Stock placement container does not match the selected container.".to_string());
    }
    validate_quantity("Quantity", input.quantity)?;
    validate_unit(&input.unit_of_measure)?;
    if input.unit_of_measure == "pcs" && input.quantity.fract().abs() > f64::EPSILON {
        return Err("Piece quantities must be whole numbers.".to_string());
    }
    validate_text_length(
        "Freeform position",
        &input.freeform_position,
        STANDARD_TEXT_LIMIT,
    )?;
    validate_text_length("Packaging", &input.packaging, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Lot code", &input.lot_code, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Date code", &input.date_code, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Condition", &input.condition, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Counted by", &input.last_counted_by, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Placement notes", &input.notes, NOTES_TEXT_LIMIT)?;

    match input.count_state.as_str() {
        "uncounted" | "legacy_verified" => {}
        "counted" => {
            let timestamp = input
                .last_counted_at
                .as_deref()
                .ok_or_else(|| "Counted stock requires a last-counted timestamp.".to_string())?;
            validate_rfc3339("Last counted timestamp", timestamp)?;
        }
        _ => return Err("Count state must be uncounted, legacy_verified, or counted.".to_string()),
    }

    if container.grid_enabled {
        if !input.freeform_position.is_empty() {
            return Err("Grid placements cannot use a freeform position.".to_string());
        }
        let row_index = input
            .row_index
            .ok_or_else(|| "Select a grid row for this placement.".to_string())?;
        let column_index = input
            .column_index
            .ok_or_else(|| "Select a grid column for this placement.".to_string())?;
        if row_index >= container.row_count.unwrap_or(0)
            || column_index >= container.column_count.unwrap_or(0)
        {
            return Err("Selected bin is outside the container grid.".to_string());
        }
    } else if input.row_index.is_some() || input.column_index.is_some() {
        return Err("Non-grid placements cannot use row or column indexes.".to_string());
    }

    Ok(())
}

pub(crate) fn create_stock_placement(input: StockPlacementInput) -> StockPlacement {
    let timestamp = now_timestamp();
    StockPlacement {
        placement_uuid: Uuid::new_v4().simple().to_string(),
        part_uuid: input.part_uuid,
        container_uuid: input.container_uuid,
        column_index: input.column_index,
        row_index: input.row_index,
        freeform_position: input.freeform_position,
        quantity: input.quantity,
        unit_of_measure: input.unit_of_measure,
        packaging: input.packaging,
        lot_code: input.lot_code,
        date_code: input.date_code,
        condition: input.condition,
        count_state: input.count_state,
        last_counted_at: input.last_counted_at,
        last_counted_by: input.last_counted_by,
        notes: input.notes,
        archived: input.archived,
        created_at: timestamp.clone(),
        updated_at: timestamp,
    }
}

pub(crate) fn update_stock_placement(
    mut placement: StockPlacement,
    input: StockPlacementInput,
) -> StockPlacement {
    placement.part_uuid = input.part_uuid;
    placement.container_uuid = input.container_uuid;
    placement.column_index = input.column_index;
    placement.row_index = input.row_index;
    placement.freeform_position = input.freeform_position;
    placement.quantity = input.quantity;
    placement.unit_of_measure = input.unit_of_measure;
    placement.packaging = input.packaging;
    placement.lot_code = input.lot_code;
    placement.date_code = input.date_code;
    placement.condition = input.condition;
    placement.count_state = input.count_state;
    placement.last_counted_at = input.last_counted_at;
    placement.last_counted_by = input.last_counted_by;
    placement.notes = input.notes;
    placement.archived = input.archived;
    placement.updated_at = now_timestamp();
    placement
}

pub(crate) fn validate_stock_placement(
    placement: &StockPlacement,
    container: &StorageContainer,
) -> CommandResult<()> {
    if placement.placement_uuid.trim().is_empty() {
        return Err("Stock placement identity is missing.".to_string());
    }
    validate_stock_placement_input(
        &StockPlacementInput {
            part_uuid: placement.part_uuid.clone(),
            container_uuid: placement.container_uuid.clone(),
            column_index: placement.column_index,
            row_index: placement.row_index,
            freeform_position: placement.freeform_position.clone(),
            quantity: placement.quantity,
            unit_of_measure: placement.unit_of_measure.clone(),
            packaging: placement.packaging.clone(),
            lot_code: placement.lot_code.clone(),
            date_code: placement.date_code.clone(),
            condition: placement.condition.clone(),
            count_state: placement.count_state.clone(),
            last_counted_at: placement.last_counted_at.clone(),
            last_counted_by: placement.last_counted_by.clone(),
            notes: placement.notes.clone(),
            archived: placement.archived,
        },
        container,
    )
}

pub(crate) fn summarize_part_stock(part: &Part, placements: &[StockPlacement]) -> PartStockSummary {
    let mut totals = BTreeMap::<String, f64>::new();
    for placement in placements
        .iter()
        .filter(|placement| placement.part_uuid == part.entry_uuid && !placement.archived)
    {
        *totals
            .entry(placement.unit_of_measure.clone())
            .or_insert(0.0) += placement.quantity;
    }
    let totals = totals
        .into_iter()
        .map(|(unit_of_measure, quantity)| QuantityTotal {
            unit_of_measure,
            quantity,
        })
        .collect::<Vec<_>>();
    let stock_status = derive_stock_status(part, &totals);
    PartStockSummary {
        part_uuid: part.entry_uuid.clone(),
        totals,
        stock_status,
    }
}

pub(crate) fn catalog_counts(parts: &[Part], summaries: &[PartStockSummary]) -> CatalogCounts {
    let archived_parts = parts.iter().filter(|part| part.archived).count();
    let mut counts = CatalogCounts {
        active_parts: parts.len().saturating_sub(archived_parts),
        archived_parts,
        total_parts: parts.len(),
        ..CatalogCounts::default()
    };
    for summary in summaries {
        match summary.stock_status {
            StockStatus::NoStock => counts.no_stock += 1,
            StockStatus::LowStock => counts.low_stock += 1,
            StockStatus::UnitReview | StockStatus::MixedUnits => counts.unit_review += 1,
            StockStatus::Archived | StockStatus::InStock => {}
        }
    }
    counts
}

pub(crate) fn derive_stock_status(part: &Part, totals: &[QuantityTotal]) -> StockStatus {
    if part.archived {
        return StockStatus::Archived;
    }
    if totals.iter().all(|total| total.quantity <= 0.0) {
        return StockStatus::NoStock;
    }
    if part.default_unit_of_measure == "unknown" {
        return StockStatus::UnitReview;
    }
    let default_total = totals
        .iter()
        .find(|total| total.unit_of_measure == part.default_unit_of_measure)
        .map(|total| total.quantity);
    let Some(default_total) = default_total else {
        return StockStatus::UnitReview;
    };
    if totals
        .iter()
        .any(|total| total.unit_of_measure != part.default_unit_of_measure && total.quantity > 0.0)
    {
        return StockStatus::MixedUnits;
    }
    if part
        .reorder_point
        .is_some_and(|reorder_point| default_total <= reorder_point)
    {
        StockStatus::LowStock
    } else {
        StockStatus::InStock
    }
}

pub(crate) fn column_label(mut column_index: u32) -> String {
    let mut label = String::new();
    loop {
        let remainder = (column_index % 26) as u8;
        label.push((b'A' + remainder) as char);
        if column_index < 26 {
            break;
        }
        column_index = column_index / 26 - 1;
    }
    label.chars().rev().collect()
}

pub(crate) fn grid_coordinate_label(
    container: &StorageContainer,
    row_index: u32,
    column_index: u32,
) -> String {
    format!(
        "{}{}",
        column_label(column_index),
        container.row_start + row_index
    )
}

pub(crate) fn normalized_lookup(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub(crate) fn normalized_part_number(value: &str) -> String {
    normalized_lookup(value)
}

pub(crate) fn indistinguishable_placement_key(placement: &StockPlacement) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}",
        placement.part_uuid,
        placement.container_uuid,
        placement
            .column_index
            .map(|value| value.to_string())
            .unwrap_or_default(),
        placement
            .row_index
            .map(|value| value.to_string())
            .unwrap_or_default(),
        normalized_lookup(&placement.freeform_position),
        normalized_lookup(&placement.lot_code),
        normalized_lookup(&placement.date_code),
        normalized_lookup(&placement.condition),
        normalized_lookup(&placement.unit_of_measure)
    )
}

pub(crate) fn timestamp_now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn default_unit_of_measure() -> String {
    "unknown".to_string()
}

fn default_part_status() -> String {
    "active".to_string()
}

fn normalize_unit(value: &str) -> String {
    let normalized = value.trim().to_lowercase().replace(' ', "_");
    if normalized.is_empty() {
        default_unit_of_measure()
    } else {
        normalized
    }
}

fn normalize_enum_text(value: &str, fallback: &str) -> String {
    let normalized = value.trim().to_lowercase().replace(' ', "_");
    if normalized.is_empty() {
        fallback.to_string()
    } else {
        normalized
    }
}

fn normalize_attribute_key(value: &str) -> String {
    let mut words = Vec::<String>::new();
    let mut current = String::new();
    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            current.push(character);
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    let Some(first) = words.first() else {
        return String::new();
    };
    let mut key = if words.len() == 1
        && first.chars().skip(1).any(char::is_uppercase)
        && first.chars().any(char::is_lowercase)
    {
        let mut characters = first.chars();
        let mut normalized = String::new();
        if let Some(first_character) = characters.next() {
            normalized.extend(first_character.to_lowercase());
        }
        normalized.extend(characters);
        normalized
    } else {
        first.to_ascii_lowercase()
    };
    for word in words.iter().skip(1) {
        let mut characters = word.chars();
        if let Some(first_character) = characters.next() {
            key.push(first_character.to_ascii_uppercase());
            key.extend(characters.flat_map(char::to_lowercase));
        }
    }
    key
}

fn validate_grid_dimension(label: &str, value: Option<u32>) -> CommandResult<()> {
    let value = value.ok_or_else(|| format!("{label} are required for a grid container."))?;
    if value == 0 || value > MAX_GRID_DIMENSION {
        return Err(format!(
            "{label} must be between 1 and {MAX_GRID_DIMENSION}."
        ));
    }
    Ok(())
}

fn validate_optional_quantity(label: &str, value: Option<f64>) -> CommandResult<()> {
    if let Some(value) = value {
        validate_quantity(label, value)?;
    }
    Ok(())
}

fn validate_quantity(label: &str, value: f64) -> CommandResult<()> {
    if !value.is_finite() || !(0.0..=MAX_CATALOG_QUANTITY).contains(&value) {
        return Err(format!(
            "{label} must be a number between 0 and {MAX_CATALOG_QUANTITY}."
        ));
    }
    Ok(())
}

fn validate_unit(value: &str) -> CommandResult<()> {
    if value.trim().is_empty() {
        return Err("Unit of measure is required.".to_string());
    }
    validate_text_length("Unit of measure", value, 64)
}

fn validate_text_length(field_name: &str, value: &str, max_chars: usize) -> CommandResult<()> {
    if value.chars().count() > max_chars {
        return Err(format!(
            "{field_name} must be {max_chars} characters or fewer."
        ));
    }
    Ok(())
}

fn validate_optional_http_url(field_name: &str, value: &str) -> CommandResult<()> {
    if value.trim().is_empty() {
        return Ok(());
    }
    if looks_like_windows_path(value) {
        return Err(format!("{field_name} must be an http or https URL."));
    }
    let url = Url::parse(value)
        .or_else(|_| Url::parse(&format!("https://{value}")))
        .map_err(|_| format!("{field_name} must be an http or https URL."))?;
    if matches!(url.scheme(), "http" | "https") {
        Ok(())
    } else {
        Err(format!("{field_name} must be an http or https URL."))
    }
}

fn validate_picture_path(value: &str) -> CommandResult<()> {
    let value = value.trim();
    if value.is_empty() || value.starts_with("data:image/") {
        return Ok(());
    }
    if looks_like_windows_path(value) || Path::new(value).is_absolute() {
        let extension = Path::new(value)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default();
        if IMAGE_PATH_EXTENSIONS
            .iter()
            .any(|allowed| extension.eq_ignore_ascii_case(allowed))
        {
            return Ok(());
        }
        return Err("Picture path must use a supported image extension.".to_string());
    }
    if Url::parse(value)
        .ok()
        .is_some_and(|url| matches!(url.scheme(), "http" | "https"))
    {
        return Ok(());
    }
    Err("Picture path must be an absolute image path or an http/https image URL.".to_string())
}

fn validate_rfc3339(label: &str, value: &str) -> CommandResult<()> {
    DateTime::parse_from_rfc3339(value)
        .map(|_| ())
        .map_err(|_| format!("{label} must be an RFC 3339 timestamp."))
}

fn looks_like_windows_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.starts_with(r"\\")
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'\\' | b'/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_labels_continue_after_z() {
        assert_eq!(column_label(0), "A");
        assert_eq!(column_label(25), "Z");
        assert_eq!(column_label(26), "AA");
        assert_eq!(column_label(27), "AB");
        assert_eq!(column_label(701), "ZZ");
        assert_eq!(column_label(702), "AAA");
    }

    #[test]
    fn part_attributes_are_normalized_and_sorted() {
        let input = normalize_part_input(PartInput {
            description: " Resistor ".to_string(),
            attributes: BTreeMap::from([
                (
                    " Rated Voltage ".to_string(),
                    ComponentAttributeValue {
                        value: " 250 ".to_string(),
                        unit: " V ".to_string(),
                    },
                ),
                (
                    "tolerance".to_string(),
                    ComponentAttributeValue {
                        value: " ".to_string(),
                        unit: "%".to_string(),
                    },
                ),
            ]),
            ..PartInput::default()
        });

        assert_eq!(
            input.attributes.keys().collect::<Vec<_>>(),
            vec!["ratedVoltage"]
        );
        assert_eq!(input.attributes["ratedVoltage"].value, "250");
        assert_eq!(input.attributes["ratedVoltage"].unit, "V");
        assert!(validate_part_input(&input).is_ok());
    }

    #[test]
    fn grid_and_piece_quantity_validation_enforce_invariants() {
        let container = StorageContainer {
            container_uuid: "container-1".to_string(),
            area_uuid: "area-1".to_string(),
            name: "Cabinet".to_string(),
            container_type: "cabinet".to_string(),
            grid_enabled: true,
            row_count: Some(4),
            column_count: Some(3),
            row_start: 1,
            origin: "top_left".to_string(),
            description: String::new(),
            archived: false,
            created_at: now_timestamp(),
            updated_at: now_timestamp(),
        };
        let valid = normalize_stock_placement_input(
            StockPlacementInput {
                part_uuid: "part-1".to_string(),
                container_uuid: container.container_uuid.clone(),
                column_index: Some(2),
                row_index: Some(3),
                quantity: 10.0,
                unit_of_measure: "pcs".to_string(),
                ..StockPlacementInput::default()
            },
            None,
        );
        assert!(validate_stock_placement_input(&valid, &container).is_ok());

        let mut out_of_bounds = valid.clone();
        out_of_bounds.column_index = Some(3);
        assert!(validate_stock_placement_input(&out_of_bounds, &container)
            .unwrap_err()
            .contains("outside"));

        let mut decimal_pieces = valid;
        decimal_pieces.quantity = 1.5;
        assert!(validate_stock_placement_input(&decimal_pieces, &container)
            .unwrap_err()
            .contains("whole"));
    }

    #[test]
    fn stock_status_never_sums_mixed_units() {
        let input = normalize_part_input(PartInput {
            description: "Wire".to_string(),
            default_unit_of_measure: "m".to_string(),
            reorder_point: Some(10.0),
            ..PartInput::default()
        });
        let part = create_part(1, input);
        let placements = vec![
            StockPlacement {
                placement_uuid: "p1".to_string(),
                part_uuid: part.entry_uuid.clone(),
                container_uuid: "c1".to_string(),
                column_index: None,
                row_index: None,
                freeform_position: String::new(),
                quantity: 8.0,
                unit_of_measure: "m".to_string(),
                packaging: String::new(),
                lot_code: String::new(),
                date_code: String::new(),
                condition: "new".to_string(),
                count_state: "uncounted".to_string(),
                last_counted_at: None,
                last_counted_by: String::new(),
                notes: String::new(),
                archived: false,
                created_at: now_timestamp(),
                updated_at: now_timestamp(),
            },
            StockPlacement {
                placement_uuid: "p2".to_string(),
                part_uuid: part.entry_uuid.clone(),
                container_uuid: "c1".to_string(),
                column_index: None,
                row_index: None,
                freeform_position: String::new(),
                quantity: 20.0,
                unit_of_measure: "ft".to_string(),
                packaging: String::new(),
                lot_code: String::new(),
                date_code: String::new(),
                condition: "new".to_string(),
                count_state: "uncounted".to_string(),
                last_counted_at: None,
                last_counted_by: String::new(),
                notes: String::new(),
                archived: false,
                created_at: now_timestamp(),
                updated_at: now_timestamp(),
            },
        ];
        let summary = summarize_part_stock(&part, &placements);
        assert_eq!(summary.stock_status, StockStatus::MixedUnits);
        assert_eq!(summary.totals.len(), 2);
    }
}
