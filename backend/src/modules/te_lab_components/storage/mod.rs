use feoxdb::FeoxStore;
use std::{fs, path::PathBuf, sync::Arc};
use tauri::Manager;

mod codec;
mod entries;
mod keys;
mod metadata;
mod sync_state;
#[cfg(test)]
mod tests;

pub(crate) use sync_state::SyncKeyspace;

const INITIAL_DB_SIZE: u64 = 64 * 1024 * 1024;
#[cfg(test)]
const TEST_DB_SIZE: u64 = 2 * 1024 * 1024;
pub(crate) const DB_FILE_NAME: &str = "te-lab-components.feox";

#[derive(Clone)]
pub(crate) struct InventoryDb {
    store: Arc<FeoxStore>,
    db_path: PathBuf,
}

impl InventoryDb {
    pub(crate) fn open(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = app.path().app_local_data_dir()?.join(DB_FILE_NAME);
        Self::open_with_size(db_path, INITIAL_DB_SIZE)
    }

    #[cfg(test)]
    pub(crate) fn open_at(db_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Self::open_with_size(db_path, TEST_DB_SIZE)
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn open_at_with_size(
        db_path: PathBuf,
        file_size: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::open_with_size(db_path, file_size)
    }

    fn open_with_size(
        db_path: PathBuf,
        file_size: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let store = FeoxStore::builder()
            .device_path(db_path.to_string_lossy().into_owned())
            .file_size(file_size)
            .build()?;

        Ok(Self {
            store: Arc::new(store),
            db_path,
        })
    }

    pub(crate) fn flush(&self) {
        self.store.flush_all();
    }

    pub(crate) fn db_path_string(&self) -> String {
        self.db_path.to_string_lossy().into_owned()
    }
}
