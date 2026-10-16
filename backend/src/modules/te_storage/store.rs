//! EVCS Storage's store: the shared [`ModuleStore`] holding EVCS Storage entries.

use super::model::InventoryEntry;
use crate::platform::{
    store::{ModuleRecord, ModuleStore, StoredEntry},
    ModuleId,
};

pub(crate) type InventoryDb = ModuleStore<InventoryEntry>;

impl ModuleRecord for InventoryEntry {
    const MODULE: ModuleId = ModuleId::TeStorage;
}

impl StoredEntry for InventoryEntry {
    fn id(&self) -> &str {
        &self.id
    }

    fn entry_uuid(&self) -> &str {
        &self.entry_uuid
    }

    fn updated_at(&self) -> &str {
        &self.updated_at
    }
}
