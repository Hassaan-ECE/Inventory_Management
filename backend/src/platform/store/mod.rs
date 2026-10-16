//! One inventory's namespace of the local database: its entries, sync state, and metadata.
//!
//! [`ModuleStore<R>`] is generic over the module's record type, so each inventory gets its
//! own store type (`ModuleStore<te::InventoryEntry>`, `ModuleStore<LabCatalog>`, ...) and
//! adds its module-specific methods in its own `impl ModuleStore<...>` block.

use std::marker::PhantomData;
#[cfg(test)]
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

use super::{LocalDatabase, ModuleId, Namespace};

mod codec;
mod entries;
pub(crate) mod keys;
mod metadata;
mod sync_state;
#[cfg(test)]
mod tests;

pub(crate) use sync_state::SyncKeyspace;

/// Test databases hold a few hundred records at most (each takes at least one 4 KiB block).
#[cfg(test)]
const TEST_DB_SIZE: u64 = 2 * 1024 * 1024;

/// Ties a store to the inventory module whose namespace it uses.
pub(crate) trait ModuleRecord {
    const MODULE: ModuleId;
}

/// A record kept under `entry:<uuid>`, with an `entry_id:<id>` index for its display id.
pub(crate) trait StoredEntry:
    ModuleRecord + Serialize + DeserializeOwned + Clone + PartialEq
{
    fn id(&self) -> &str;
    fn entry_uuid(&self) -> &str;
    fn updated_at(&self) -> &str;
}

pub(crate) struct ModuleStore<R> {
    store: Namespace,
    record: PhantomData<fn() -> R>,
}

impl<R> Clone for ModuleStore<R> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            record: PhantomData,
        }
    }
}

impl<R: ModuleRecord> ModuleStore<R> {
    pub(crate) fn open_in(database: &LocalDatabase) -> Self {
        Self {
            store: database.namespace(R::MODULE),
            record: PhantomData,
        }
    }

    /// Opens a standalone database file for tests, using this module's namespace.
    #[cfg(test)]
    pub(crate) fn open_at(db_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Self::open_at_with_size(db_path, TEST_DB_SIZE)
    }

    #[cfg(test)]
    pub(crate) fn open_at_with_size(
        db_path: PathBuf,
        file_size: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::open_in(&LocalDatabase::open_file(
            db_path, file_size,
        )?))
    }
}

impl<R> ModuleStore<R> {
    pub(crate) fn flush(&self) {
        self.store.flush_all();
    }

    pub(crate) fn db_path_string(&self) -> String {
        self.store.path().to_string_lossy().into_owned()
    }
}
