use super::{keys, InventoryDb};
use crate::{
    modules::te_lab_components::catalog_model::CATALOG_SCHEMA_VERSION,
    platform::common::CommandResult,
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
        if let Some(next_id) = self.stored_next_id()? {
            return Ok(next_id);
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
        self.set_next_id(next_id)
    }

    /// Catalog sync starts its own stream (schema 2) with a fresh local sequence.
    fn reset_sync_state_for_catalog_v2(&self) -> CommandResult<()> {
        self.clear_sync_state()?;
        self.set_sync_schema_version(2)?;
        self.set_next_local_seq(1)
    }
}
