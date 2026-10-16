#[cfg(test)]
use super::sync_state::SyncMetadata;
use super::{keys, InventoryDb};
use crate::{
    modules::te_lab_components::catalog_model::CATALOG_SCHEMA_VERSION,
    platform::common::{db_error, CommandResult},
};

impl InventoryDb {
    /// Moves a namespace that is not yet on catalog schema v2 onto it. Rows left by the
    /// retired v1 Lab model are dropped rather than migrated: the shared catalog is the source
    /// of truth, and the 0.1.x database file is kept in `pre-0.2-databases\`.
    pub(crate) fn ensure_catalog_schema(&self) -> CommandResult<()> {
        if self.schema_version()? == Some(CATALOG_SCHEMA_VERSION) {
            return Ok(());
        }
        for prefix in keys::LEGACY_ENTRY_PREFIXES {
            self.delete_prefix(prefix)?;
        }
        self.reset_sync_state_for_catalog_v2()?;
        self.set_schema_version(CATALOG_SCHEMA_VERSION)?;
        self.flush();
        Ok(())
    }

    /// Next numeric part id. The key keeps its v1 name so existing counters carry over.
    pub(crate) fn next_part_id(&self) -> CommandResult<i64> {
        if self.store.contains_key(keys::META_NEXT_ID) {
            let bytes = self.store.get(keys::META_NEXT_ID).map_err(db_error)?;
            let text = String::from_utf8(bytes).map_err(db_error)?;
            if let Ok(next_id) = text.parse::<i64>() {
                return Ok(next_id.max(1));
            }
        }

        let max_id = self
            .load_parts()?
            .iter()
            .filter_map(|part| part.id.parse::<i64>().ok())
            .max()
            .unwrap_or(0);
        Ok(max_id + 1)
    }

    pub(crate) fn set_next_part_id(&self, next_id: i64) -> CommandResult<()> {
        self.put_bytes(keys::META_NEXT_ID, next_id.to_string().as_bytes())
    }

    #[cfg(test)]
    pub(crate) fn sync_metadata(&self) -> CommandResult<SyncMetadata> {
        Ok(SyncMetadata {
            schema_version: self.schema_version()?,
            sync_schema_version: self.sync_schema_version()?,
            client_id: self.client_id()?,
            device_id: self.device_id()?,
            next_local_seq: self.next_local_seq()?,
            last_snapshot_id: self.last_snapshot_id()?,
        })
    }

    pub(crate) fn schema_version(&self) -> CommandResult<Option<u32>> {
        self.get_u32(keys::META_SCHEMA_VERSION, "schema_version")
    }

    pub(crate) fn set_schema_version(&self, version: u32) -> CommandResult<()> {
        self.put_u32(keys::META_SCHEMA_VERSION, "schema_version", version)
    }

    #[cfg(test)]
    pub(crate) fn sync_schema_version(&self) -> CommandResult<Option<u32>> {
        self.get_u32(keys::META_SYNC_SCHEMA_VERSION, "sync_schema_version")
    }

    pub(crate) fn set_sync_schema_version(&self, version: u32) -> CommandResult<()> {
        self.put_u32(
            keys::META_SYNC_SCHEMA_VERSION,
            "sync_schema_version",
            version,
        )
    }

    pub(crate) fn client_id(&self) -> CommandResult<Option<String>> {
        self.get_string(keys::META_CLIENT_ID)
    }

    pub(crate) fn set_client_id(&self, client_id: &str) -> CommandResult<()> {
        let client_id = keys::normalized_sync_key_segment("client_id", client_id)?;
        self.put_bytes(keys::META_CLIENT_ID, client_id.as_bytes())
    }

    pub(crate) fn get_or_create_client_id(&self) -> CommandResult<String> {
        if let Some(client_id) = self.client_id()? {
            return Ok(client_id);
        }

        let client_id = uuid::Uuid::new_v4().simple().to_string();
        self.set_client_id(&client_id)?;
        Ok(client_id)
    }

    pub(crate) fn device_id(&self) -> CommandResult<Option<String>> {
        self.get_string(keys::META_DEVICE_ID)
    }

    pub(crate) fn set_device_id(&self, device_id: &str) -> CommandResult<()> {
        let device_id = keys::normalized_sync_key_segment("device_id", device_id)?;
        self.put_bytes(keys::META_DEVICE_ID, device_id.as_bytes())
    }

    pub(crate) fn get_or_create_device_id(&self) -> CommandResult<String> {
        if let Some(device_id) = self.device_id()? {
            return Ok(device_id);
        }

        let device_id = uuid::Uuid::new_v4().simple().to_string();
        self.set_device_id(&device_id)?;
        Ok(device_id)
    }

    pub(crate) fn next_local_seq(&self) -> CommandResult<u64> {
        match self.get_u64(keys::META_NEXT_LOCAL_SEQ, "next_local_seq")? {
            Some(next_local_seq) => {
                keys::validate_local_seq(next_local_seq)?;
                Ok(next_local_seq)
            }
            None => Ok(1),
        }
    }

    pub(crate) fn set_next_local_seq(&self, next_local_seq: u64) -> CommandResult<()> {
        keys::validate_local_seq(next_local_seq)?;
        self.put_bytes(
            keys::META_NEXT_LOCAL_SEQ,
            next_local_seq.to_string().as_bytes(),
        )
    }

    pub(crate) fn reserve_next_local_seq(&self) -> CommandResult<u64> {
        let local_seq = self.next_local_seq()?;
        let next_local_seq = local_seq
            .checked_add(1)
            .ok_or_else(|| "next_local_seq overflowed".to_string())?;
        self.set_next_local_seq(next_local_seq)?;
        Ok(local_seq)
    }

    pub(crate) fn sync_revision(&self) -> CommandResult<u64> {
        Ok(self
            .get_u64(keys::META_SYNC_REVISION, "sync_revision")?
            .unwrap_or(0))
    }

    pub(crate) fn increment_sync_revision(&self) -> CommandResult<u64> {
        let next_revision = self
            .sync_revision()?
            .checked_add(1)
            .ok_or_else(|| "sync_revision overflowed".to_string())?;
        self.put_bytes(
            keys::META_SYNC_REVISION,
            next_revision.to_string().as_bytes(),
        )?;
        Ok(next_revision)
    }

    pub(crate) fn last_snapshot_id(&self) -> CommandResult<Option<String>> {
        self.get_string(keys::META_LAST_SNAPSHOT_ID)
    }

    pub(crate) fn set_last_snapshot_id(&self, snapshot_id: &str) -> CommandResult<()> {
        let snapshot_id = keys::normalized_sync_key_segment("last_snapshot_id", snapshot_id)?;
        self.put_bytes(keys::META_LAST_SNAPSHOT_ID, snapshot_id.as_bytes())
    }

    #[cfg(test)]
    pub(crate) fn clear_last_snapshot_id(&self) -> CommandResult<()> {
        self.delete_key(keys::META_LAST_SNAPSHOT_ID)
    }
}
