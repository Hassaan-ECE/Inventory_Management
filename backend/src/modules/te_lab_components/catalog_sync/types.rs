//! What the Lab catalog writes to the share and keeps in its sync state. These shapes are the
//! catalog-v2 format 0.1.x clients read, so fields and names stay as they are.

use serde::{Deserialize, Serialize};

use crate::modules::te_lab_components::catalog_model::{
    Part, StockPlacement, StorageArea, StorageContainer,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CatalogEntityType {
    Part,
    StorageArea,
    StorageContainer,
    StockPlacement,
}

impl CatalogEntityType {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Part => "part",
            Self::StorageArea => "storage_area",
            Self::StorageContainer => "storage_container",
            Self::StockPlacement => "stock_placement",
        }
    }

    pub(super) fn upsert_rank(self) -> u8 {
        match self {
            Self::StorageArea => 0,
            Self::StorageContainer => 1,
            Self::Part => 2,
            Self::StockPlacement => 3,
        }
    }

    pub(super) fn delete_rank(self) -> u8 {
        match self {
            Self::StockPlacement => 4,
            Self::Part => 5,
            Self::StorageContainer => 6,
            Self::StorageArea => 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub(crate) enum CatalogEntity {
    Part(Part),
    StorageArea(StorageArea),
    StorageContainer(StorageContainer),
    StockPlacement(StockPlacement),
}

impl CatalogEntity {
    pub(crate) fn entity_type(&self) -> CatalogEntityType {
        match self {
            Self::Part(_) => CatalogEntityType::Part,
            Self::StorageArea(_) => CatalogEntityType::StorageArea,
            Self::StorageContainer(_) => CatalogEntityType::StorageContainer,
            Self::StockPlacement(_) => CatalogEntityType::StockPlacement,
        }
    }

    pub(crate) fn entity_id(&self) -> &str {
        match self {
            Self::Part(part) => &part.entry_uuid,
            Self::StorageArea(area) => &area.area_uuid,
            Self::StorageContainer(container) => &container.container_uuid,
            Self::StockPlacement(placement) => &placement.placement_uuid,
        }
    }

    pub(crate) fn updated_at(&self) -> &str {
        match self {
            Self::Part(part) => &part.updated_at,
            Self::StorageArea(area) => &area.updated_at,
            Self::StorageContainer(container) => &container.updated_at,
            Self::StockPlacement(placement) => &placement.updated_at,
        }
    }
}

impl From<Part> for CatalogEntity {
    fn from(value: Part) -> Self {
        Self::Part(value)
    }
}

impl From<StorageArea> for CatalogEntity {
    fn from(value: StorageArea) -> Self {
        Self::StorageArea(value)
    }
}

impl From<StorageContainer> for CatalogEntity {
    fn from(value: StorageContainer) -> Self {
        Self::StorageContainer(value)
    }
}

impl From<StockPlacement> for CatalogEntity {
    fn from(value: StockPlacement) -> Self {
        Self::StockPlacement(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum CatalogOperationType {
    #[serde(rename = "catalog.entity.upsert")]
    Upsert,
    #[serde(rename = "catalog.entity.delete")]
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogOperationPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<CatalogEntity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at_utc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogOperationEnvelope {
    pub schema_version: u16,
    pub op_id: String,
    pub client_id: String,
    pub device_id: String,
    pub local_seq: u64,
    pub app_version: String,
    pub created_at_utc: String,
    #[serde(rename = "type")]
    pub operation_type: CatalogOperationType,
    pub entity_type: CatalogEntityType,
    pub entity_id: String,
    pub base_version: Option<String>,
    pub mutation_ts_utc: String,
    pub payload: CatalogOperationPayload,
    #[serde(default)]
    pub checksum: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogAppliedMarker {
    pub(super) op_id: String,
    pub(super) client_id: String,
    pub(super) local_seq: u64,
    pub(super) checksum: String,
    pub(super) applied_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogEntityState {
    pub(super) entity_type: CatalogEntityType,
    pub(super) entity_uuid: String,
    pub(super) last_op_id: String,
    pub(super) mutation_ts_utc: String,
    pub(super) deleted: bool,
    pub(super) base_version: Option<String>,
    #[serde(default)]
    pub(super) changed_fields: Vec<String>,
    pub(super) source_client_id: String,
    pub(super) source_local_seq: u64,
    pub(super) updated_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogTombstone {
    pub(super) entity_type: CatalogEntityType,
    pub(super) entity_uuid: String,
    pub(super) deleted_at_utc: String,
    pub(super) op_id: String,
    pub(super) client_id: String,
    pub(super) local_seq: u64,
    pub(super) base_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogConflictRecord {
    pub(super) conflict_id: String,
    pub(super) entity_type: CatalogEntityType,
    pub(super) entity_uuid: String,
    pub(super) incoming_op_id: String,
    pub(super) incoming_client_id: String,
    pub(super) incoming_local_seq: u64,
    pub(super) current_op_id: Option<String>,
    pub(super) reason: String,
    pub(super) detected_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogCorruptRemoteRecord {
    pub(super) record_id: String,
    pub(super) path: String,
    pub(super) detail: String,
    pub(super) detected_at_utc: String,
    pub(super) content_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogWatermark {
    pub(super) client_id: String,
    pub(super) local_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogSnapshot {
    pub(super) schema_version: u16,
    pub(super) sync_schema_version: u16,
    pub(super) snapshot_id: String,
    pub(super) app_version: String,
    pub(super) source_client_id: String,
    pub(super) created_at_utc: String,
    pub(super) parts: Vec<Part>,
    pub(super) storage_areas: Vec<StorageArea>,
    pub(super) storage_containers: Vec<StorageContainer>,
    pub(super) stock_placements: Vec<StockPlacement>,
    pub(super) tombstones: Vec<CatalogTombstone>,
    pub(super) entity_states: Vec<CatalogEntityState>,
    pub(super) watermarks: Vec<CatalogWatermark>,
    pub(super) checksum: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) auth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogManifest {
    pub(super) schema_version: u16,
    pub(super) sync_schema_version: u16,
    pub(super) snapshot_id: String,
    pub(super) snapshot_file: String,
    pub(super) snapshot_checksum: String,
    pub(super) app_version: String,
    pub(super) source_client_id: String,
    pub(super) created_at_utc: String,
    pub(super) part_count: usize,
    pub(super) area_count: usize,
    pub(super) container_count: usize,
    pub(super) placement_count: usize,
    pub(super) tombstone_count: usize,
    pub(super) watermarks: Vec<CatalogWatermark>,
    pub(super) checksum: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) auth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CatalogCutoverMarker {
    pub(super) schema_version: u16,
    pub(super) sync_schema_version: u16,
    pub(super) local_fingerprint: String,
    pub(super) source_client_id: String,
    pub(super) created_at_utc: String,
    pub(super) checksum: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) auth: Option<String>,
}
