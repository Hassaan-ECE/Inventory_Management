use serde::{de::DeserializeOwned, Serialize};

use super::{keys, InventoryDb};
use crate::model::{CommandResult, InventoryEntry};

impl InventoryDb {
    pub(crate) fn find_import_entry_by_uuid(
        &self,
        entry_uuid: &str,
    ) -> CommandResult<Option<InventoryEntry>> {
        let key = keys::entry_key(entry_uuid);
        self.get_json(key.as_bytes())
    }

    pub(crate) fn put_import_preview<T: Serialize>(
        &self,
        batch_id: &str,
        preview: &T,
    ) -> CommandResult<()> {
        let key = keys::import_preview_key(batch_id)?;
        self.put_json(key.as_bytes(), preview)?;
        self.flush();
        Ok(())
    }

    pub(crate) fn import_preview<T: DeserializeOwned>(
        &self,
        batch_id: &str,
    ) -> CommandResult<Option<T>> {
        let key = keys::import_preview_key(batch_id)?;
        self.get_json(key.as_bytes())
    }

    pub(crate) fn mark_import_completed<T: Serialize>(
        &self,
        batch_id: &str,
        result: &T,
    ) -> CommandResult<()> {
        let key = keys::import_completed_key(batch_id)?;
        self.put_json(key.as_bytes(), result)?;
        self.flush();
        Ok(())
    }

    pub(crate) fn completed_import<T: DeserializeOwned>(
        &self,
        batch_id: &str,
    ) -> CommandResult<Option<T>> {
        let key = keys::import_completed_key(batch_id)?;
        self.get_json(key.as_bytes())
    }

    pub(crate) fn has_import_row_marker(
        &self,
        batch_id: &str,
        source_row: u64,
    ) -> CommandResult<bool> {
        let key = keys::import_row_key(batch_id, source_row)?;
        Ok(self.store.contains_key(key.as_bytes()))
    }

    pub(crate) fn mark_import_row_complete<T: Serialize>(
        &self,
        batch_id: &str,
        source_row: u64,
        marker: &T,
    ) -> CommandResult<()> {
        let key = keys::import_row_key(batch_id, source_row)?;
        self.put_json(key.as_bytes(), marker)?;
        self.flush();
        Ok(())
    }
}
