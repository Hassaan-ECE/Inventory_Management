use feoxdb::FeoxStore;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::Manager;

mod codec;
mod entries;
mod imports;
mod keys;
mod metadata;
mod sync_state;
#[cfg(test)]
mod tests;

pub(crate) use sync_state::SyncKeyspace;

const INITIAL_DB_SIZE: u64 = 64 * 1024 * 1024;
#[cfg(test)]
const TEST_DB_SIZE: u64 = 2 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct InventoryDb {
    store: Arc<FeoxStore>,
    db_path: PathBuf,
}

impl InventoryDb {
    pub(crate) fn open(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let local_data_dir = app.path().app_local_data_dir()?;
        let legacy_roaming_dir = app.path().app_data_dir()?;
        let db_path = prepare_local_inventory_db_path(&local_data_dir, &legacy_roaming_dir)?;

        Self::open_with_size(db_path, INITIAL_DB_SIZE)
    }

    #[cfg(test)]
    pub(crate) fn open_at(db_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Self::open_with_size(db_path, TEST_DB_SIZE)
    }

    #[cfg(test)]
    // Used by path-included performance integration tests, not by the library test crate.
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

fn prepare_local_inventory_db_path(
    local_data_dir: &Path,
    legacy_roaming_dir: &Path,
) -> std::io::Result<PathBuf> {
    fs::create_dir_all(local_data_dir)?;
    let target = local_data_dir.join("inventory.feox");
    let legacy = legacy_roaming_dir.join("inventory.feox");

    if !target.exists() && legacy.is_file() {
        fs::copy(&legacy, &target)?;
    }

    Ok(target)
}

#[cfg(test)]
mod local_path_tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn copies_legacy_roaming_database_only_when_local_target_is_missing() {
        let root = std::env::temp_dir().join(format!(
            "te-test-equipment-db-path-{}",
            Uuid::new_v4().simple()
        ));
        let local = root.join("local");
        let roaming = root.join("roaming");
        fs::create_dir_all(&roaming).unwrap();
        let legacy = roaming.join("inventory.feox");
        fs::write(&legacy, b"legacy database").unwrap();

        let target = prepare_local_inventory_db_path(&local, &roaming).unwrap();

        assert_eq!(target, local.join("inventory.feox"));
        assert_eq!(fs::read(&target).unwrap(), b"legacy database");
        assert_eq!(fs::read(&legacy).unwrap(), b"legacy database");

        fs::write(&target, b"local database").unwrap();
        fs::write(&legacy, b"new roaming bytes").unwrap();
        prepare_local_inventory_db_path(&local, &roaming).unwrap();

        assert_eq!(fs::read(target).unwrap(), b"local database");
        assert_eq!(fs::read(legacy).unwrap(), b"new roaming bytes");
    }
}
