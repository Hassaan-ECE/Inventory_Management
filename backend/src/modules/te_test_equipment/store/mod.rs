//! TE Test Equipment's store: the shared [`ModuleStore`] holding TE entries, plus the
//! calibration roster batches kept beside them.

use crate::{
    modules::te_test_equipment::model::InventoryEntry,
    platform::{
        store::{ModuleRecord, ModuleStore, StoredEntry},
        ModuleId,
    },
};

mod calibration_roster;
#[cfg(test)]
mod tests;

pub(crate) type InventoryDb = ModuleStore<InventoryEntry>;

impl ModuleRecord for InventoryEntry {
    const MODULE: ModuleId = ModuleId::TeTestEquipment;
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
