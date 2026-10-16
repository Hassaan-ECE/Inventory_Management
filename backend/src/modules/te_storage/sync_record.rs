//! How EVCS Storage entries plug into the shared sync engine.

use super::{
    entry_changes::{normalize_changed_entry_fields, ENTRY_FIELDS},
    model::{validate_inventory_entry, CommandResult, InventoryEntry},
};
use crate::platform::sync::{MergeRule, SyncRecord};

impl SyncRecord for InventoryEntry {
    const LABEL: &'static str = "EVCS Storage";
    const SHARED_ROOT_ENV: &'static str = "INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT";
    const SYNC_SCHEMA_VERSION: u16 = 1;
    const ENTITY_TYPE: &'static str = "te_storage_entry";
    const OPERATION_AUTH_DOMAIN: &'static str = "te-storage.operation.v1";
    const MERGE_RULE: MergeRule = MergeRule::PerFieldVersions;
    const ENTRY_FIELDS: &'static [&'static str] = ENTRY_FIELDS;

    fn validate(&self) -> CommandResult<()> {
        validate_inventory_entry(self)
    }

    fn normalize_changed_fields(fields: &[String]) -> Vec<String> {
        normalize_changed_entry_fields(fields.to_vec())
    }

    fn copy_fields(&mut self, source: &Self, fields: &[String]) {
        for field in fields {
            match field.as_str() {
                "pn" => self.pn = source.pn.clone(),
                "pr" => self.pr = source.pr.clone(),
                "qty" => self.qty = source.qty,
                "manufacturer" => self.manufacturer = source.manufacturer.clone(),
                "model" => self.model = source.model.clone(),
                "description" => self.description = source.description.clone(),
                "po" => self.po = source.po.clone(),
                "location" => self.location = source.location.clone(),
                "notes" => self.notes = source.notes.clone(),
                "archived" => self.archived = source.archived,
                _ => {}
            }
        }
    }

    fn set_updated_at(&mut self, updated_at: String) {
        self.updated_at = updated_at;
    }

    fn take_local_id_from(&mut self, existing: &Self) {
        self.id.clone_from(&existing.id);
        self.database_id = existing.database_id;
    }

    fn set_local_id(&mut self, id: i64) {
        self.id = id.to_string();
        self.database_id = Some(id);
    }
}

#[cfg(test)]
mod tests {
    use super::InventoryEntry;
    use crate::platform::sync::test_support::{
        entry_state_keys, resolve_shared_root_from_env_value,
    };
    use crate::platform::{default_shared_root, ModuleId};
    use std::{ffi::OsString, path::PathBuf};

    /// EVCS Storage entry states have always carried their field versions and recovery
    /// entry, including in published snapshots.
    #[test]
    fn storage_entry_states_keep_field_versions_and_recovery_entry() {
        let keys = entry_state_keys::<InventoryEntry>();
        assert!(keys.iter().any(|key| key == "fieldVersions"));
        assert!(keys.iter().any(|key| key == "recoveryEntry"));
    }

    #[test]
    fn shared_root_defaults_to_the_storage_folder_and_honours_its_override() {
        assert_eq!(
            resolve_shared_root_from_env_value::<InventoryEntry>(None),
            default_shared_root(ModuleId::TeStorage)
        );
        assert_eq!(
            resolve_shared_root_from_env_value::<InventoryEntry>(Some(OsString::from(
                r"S:\Storage\Override"
            ))),
            PathBuf::from(r"S:\Storage\Override")
        );
    }
}
