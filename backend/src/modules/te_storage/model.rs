use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub(crate) type CommandResult<T> = Result<T, String>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventoryEntry {
    pub id: String,
    pub database_id: Option<i64>,
    pub entry_uuid: String,
    pub pn: String,
    pub pr: String,
    pub po: String,
    pub manufacturer: String,
    pub model: String,
    pub description: String,
    pub location: String,
    pub notes: String,
    pub qty: Option<f64>,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct InventoryEntryInput {
    pub pn: String,
    pub pr: String,
    pub po: String,
    pub manufacturer: String,
    pub model: String,
    pub description: String,
    pub location: String,
    pub notes: String,
    pub qty: Option<f64>,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct InventoryEntryEditContext {
    pub base_version: Option<String>,
    pub changed_fields: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventorySharedStatus {
    pub available: bool,
    pub can_modify: bool,
    pub enabled: bool,
    pub has_local_only_changes: Option<bool>,
    pub message: String,
    pub mutation_mode: String,
    pub revision: Option<String>,
    pub last_snapshot_id: Option<String>,
    pub shared_root_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventorySyncResult {
    pub db_path: String,
    pub entries: Vec<InventoryEntry>,
    pub entries_changed: Option<bool>,
    pub shared: InventorySharedStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventoryEntryMutationResult {
    pub entry: InventoryEntry,
    pub message: String,
    pub mutation_mode: String,
    pub shared: InventorySharedStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventoryDeleteMutationResult {
    pub entry_id: String,
    pub message: String,
    pub mutation_mode: String,
    pub shared: InventorySharedStatus,
}

pub(crate) fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}
pub(crate) fn numeric_id(id: &str) -> i64 {
    id.parse().unwrap_or(0)
}
pub(crate) fn now_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub(crate) fn timestamp_after(previous: &str) -> String {
    let now = Utc::now();
    let next = chrono::DateTime::parse_from_rfc3339(previous)
        .map(|time| now.max(time.with_timezone(&Utc) + chrono::Duration::milliseconds(1)))
        .unwrap_or(now);
    next.to_rfc3339_opts(SecondsFormat::Millis, true)
}
pub(crate) fn normalize_entry_input(input: InventoryEntryInput) -> InventoryEntryInput {
    InventoryEntryInput {
        pn: input.pn.trim().to_string(),
        pr: input.pr.trim().to_string(),
        po: input.po.trim().to_string(),
        manufacturer: input.manufacturer.trim().to_string(),
        model: input.model.trim().to_string(),
        description: input.description.trim().to_string(),
        location: input.location.trim().to_string(),
        notes: input.notes.trim().to_string(),
        qty: input.qty,
    }
}
pub(crate) fn validate_entry_input(input: &InventoryEntryInput) -> CommandResult<()> {
    if [
        &input.pn,
        &input.pr,
        &input.po,
        &input.manufacturer,
        &input.model,
        &input.description,
    ]
    .iter()
    .all(|v| v.trim().is_empty())
    {
        return Err(
            "Enter a PN #, PR #, PO #, manufacturer, model, or description before saving.".into(),
        );
    }
    if input
        .qty
        .is_some_and(|q| !q.is_finite() || !(0.0..=1_000_000.0).contains(&q))
    {
        return Err("Qty must be a number between 0 and 1000000.".into());
    }
    for (name, value, limit) in [
        ("PN #", &input.pn, 512),
        ("PR #", &input.pr, 512),
        ("PO #", &input.po, 512),
        ("Manufacturer", &input.manufacturer, 512),
        ("Model", &input.model, 512),
        ("Description", &input.description, 4000),
        ("Location", &input.location, 512),
        ("Notes", &input.notes, 8000),
    ] {
        if value.chars().count() > limit {
            return Err(format!("{name} must be {limit} characters or fewer."));
        }
    }
    Ok(())
}
pub(crate) fn create_entry_from_input(id: i64, input: InventoryEntryInput) -> InventoryEntry {
    let now = now_timestamp();
    InventoryEntry {
        id: id.to_string(),
        database_id: Some(id),
        entry_uuid: Uuid::new_v4().simple().to_string(),
        pn: input.pn,
        pr: input.pr,
        po: input.po,
        manufacturer: input.manufacturer,
        model: input.model,
        description: input.description,
        location: input.location,
        notes: input.notes,
        qty: input.qty,
        archived: false,
        created_at: now.clone(),
        updated_at: now,
    }
}
pub(crate) fn update_entry_from_input(
    mut entry: InventoryEntry,
    input: InventoryEntryInput,
) -> InventoryEntry {
    entry.pn = input.pn;
    entry.pr = input.pr;
    entry.po = input.po;
    entry.manufacturer = input.manufacturer;
    entry.model = input.model;
    entry.description = input.description;
    entry.location = input.location;
    entry.notes = input.notes;
    entry.qty = input.qty;
    entry.updated_at = now_timestamp();
    entry
}
pub(crate) fn validate_inventory_entry(entry: &InventoryEntry) -> CommandResult<()> {
    validate_entry_input(&InventoryEntryInput {
        pn: entry.pn.clone(),
        pr: entry.pr.clone(),
        po: entry.po.clone(),
        manufacturer: entry.manufacturer.clone(),
        model: entry.model.clone(),
        description: entry.description.clone(),
        location: entry.location.clone(),
        notes: entry.notes.clone(),
        qty: entry.qty,
    })
}
