use super::model::{now_timestamp, InventoryEntry, InventoryEntryInput};
pub(crate) const ENTRY_FIELDS: &[&str] = &[
    "pn",
    "pr",
    "po",
    "manufacturer",
    "model",
    "description",
    "qty",
    "location",
    "notes",
];
pub(crate) fn entry_base_version(entry: &InventoryEntry) -> Option<String> {
    Some(entry.updated_at.clone())
}
pub(crate) fn normalize_changed_entry_fields(mut fields: Vec<String>) -> Vec<String> {
    fields.retain(|f| {
        matches!(
            f.as_str(),
            "pn" | "pr"
                | "po"
                | "manufacturer"
                | "model"
                | "description"
                | "qty"
                | "location"
                | "notes"
        )
    });
    fields.sort();
    fields.dedup();
    fields
}
pub(crate) fn update_entry_from_input_fields(
    mut entry: InventoryEntry,
    input: &InventoryEntryInput,
    changed: &[String],
) -> InventoryEntry {
    for field in changed {
        match field.as_str() {
            "pn" => entry.pn.clone_from(&input.pn),
            "pr" => entry.pr.clone_from(&input.pr),
            "po" => entry.po.clone_from(&input.po),
            "manufacturer" => entry.manufacturer.clone_from(&input.manufacturer),
            "model" => entry.model.clone_from(&input.model),
            "description" => entry.description.clone_from(&input.description),
            "location" => entry.location.clone_from(&input.location),
            "notes" => entry.notes.clone_from(&input.notes),
            "qty" => entry.qty = input.qty,
            _ => {}
        }
    }
    entry.updated_at = now_timestamp();
    entry
}
pub(crate) fn changed_entry_fields(before: &InventoryEntry, after: &InventoryEntry) -> Vec<String> {
    let mut fields = Vec::new();
    if before.pn != after.pn {
        fields.push("pn".to_string());
    }
    if before.pr != after.pr {
        fields.push("pr".to_string());
    }
    if before.po != after.po {
        fields.push("po".to_string());
    }
    if before.manufacturer != after.manufacturer {
        fields.push("manufacturer".to_string());
    }
    if before.model != after.model {
        fields.push("model".to_string());
    }
    if before.description != after.description {
        fields.push("description".to_string());
    }
    if before.location != after.location {
        fields.push("location".to_string());
    }
    if before.notes != after.notes {
        fields.push("notes".to_string());
    }
    if before.qty != after.qty {
        fields.push("qty".to_string());
    }
    fields
}
