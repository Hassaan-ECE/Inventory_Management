#[cfg(test)]
use std::path::PathBuf;

use crate::platform::Namespace;
#[cfg(test)]
use crate::platform::{LocalDatabase, ModuleId};

mod catalog;
mod codec;
#[allow(dead_code)]
mod entries;
mod keys;
mod metadata;
mod sync_state;
#[cfg(test)]
mod tests;

pub(crate) use sync_state::SyncKeyspace;

#[cfg(test)]
const TEST_DB_SIZE: u64 = 2 * 1024 * 1024;
/// This inventory's records, stored in its own namespace of the shared local database.
#[derive(Clone)]
pub(crate) struct InventoryDb {
    store: Namespace,
}

impl InventoryDb {
    pub(crate) fn new(store: Namespace) -> Self {
        Self { store }
    }

    /// Opens a standalone database file for tests, using this inventory's namespace.
    #[cfg(test)]
    pub(crate) fn open_at(db_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Self::open_at_with_size(db_path, TEST_DB_SIZE)
    }

    #[cfg(test)]
    // Used by path-included integration tests, not by the library test crate.
    #[allow(dead_code)]
    pub(crate) fn open_at_with_size(
        db_path: PathBuf,
        file_size: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let database = LocalDatabase::open_file(db_path, file_size)?;
        Ok(Self::new(database.namespace(ModuleId::TeLabComponents)))
    }

    pub(crate) fn flush(&self) {
        self.store.flush_all();
    }

    pub(crate) fn db_path_string(&self) -> String {
        self.store.path().to_string_lossy().into_owned()
    }
}
