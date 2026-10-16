//! The one local database file. Every inventory reads and writes through its own
//! [`Namespace`], which stores each key as `<module id>/<key>`, so the inventories share a
//! file without being able to see each other's records.

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use feoxdb::{FeoxError, FeoxStore};

use super::{legacy_databases::merge_legacy_databases_once, ModuleId};

pub(crate) const DATABASE_FILE_NAME: &str = "inventory-management.feox";

/// FeOx files never grow after creation (writes fail with `OutOfSpace` once full), so the
/// file is sized well above the three 64 MiB files it replaces.
pub(super) const DATABASE_FILE_SIZE: u64 = 256 * 1024 * 1024;

pub(crate) struct LocalDatabase {
    store: Arc<FeoxStore>,
    path: Arc<PathBuf>,
}

impl LocalDatabase {
    /// Opens `<local data dir>/inventory-management.feox`, first merging the per-inventory
    /// files that 0.1.x used if this computer has not been merged yet.
    pub(crate) fn open(local_data_dir: &Path) -> Result<Self, Box<dyn Error>> {
        merge_legacy_databases_once(local_data_dir)?;
        Self::open_file(local_data_dir.join(DATABASE_FILE_NAME), DATABASE_FILE_SIZE)
    }

    /// Opens (or creates at `file_size`) a database file at an explicit path.
    pub(crate) fn open_file(path: PathBuf, file_size: u64) -> Result<Self, Box<dyn Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let store = FeoxStore::builder()
            .device_path(path.to_string_lossy().into_owned())
            .file_size(file_size)
            .build()?;
        Ok(Self {
            store: Arc::new(store),
            path: Arc::new(path),
        })
    }

    pub(crate) fn namespace(&self, module: ModuleId) -> Namespace {
        Namespace {
            store: Arc::clone(&self.store),
            prefix: format!("{}/", module.as_system_id_str()).into_bytes(),
            path: Arc::clone(&self.path),
        }
    }

    pub(crate) fn flush(&self) {
        self.store.flush_all();
    }
}

/// Key/value pairs returned by a range query.
pub(crate) type KeyValueRows = Vec<(Vec<u8>, Vec<u8>)>;

/// One inventory's view of the local database. Mirrors the `FeoxStore` methods the storage
/// layers use; keys passed in and returned are relative to the namespace.
#[derive(Clone)]
pub(crate) struct Namespace {
    store: Arc<FeoxStore>,
    prefix: Vec<u8>,
    path: Arc<PathBuf>,
}

impl Namespace {
    pub(crate) fn contains_key(&self, key: &[u8]) -> bool {
        self.store.contains_key(&self.full_key(key))
    }

    pub(crate) fn get(&self, key: &[u8]) -> Result<Vec<u8>, FeoxError> {
        self.store.get(&self.full_key(key))
    }

    pub(crate) fn insert(&self, key: &[u8], value: &[u8]) -> Result<bool, FeoxError> {
        self.store.insert(&self.full_key(key), value)
    }

    pub(crate) fn delete(&self, key: &[u8]) -> Result<(), FeoxError> {
        self.store.delete(&self.full_key(key))
    }

    /// Inclusive range over this namespace; returned keys have the namespace removed.
    pub(crate) fn range_query(
        &self,
        start_key: &[u8],
        end_key: &[u8],
        limit: usize,
    ) -> Result<KeyValueRows, FeoxError> {
        let rows =
            self.store
                .range_query(&self.full_key(start_key), &self.full_key(end_key), limit)?;
        Ok(rows
            .into_iter()
            .filter_map(|(key, value)| {
                key.strip_prefix(self.prefix.as_slice())
                    .map(|key| (key.to_vec(), value))
            })
            .collect())
    }

    pub(crate) fn flush_all(&self) {
        self.store.flush_all();
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    fn full_key(&self, key: &[u8]) -> Vec<u8> {
        [self.prefix.as_slice(), key].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn temp_database() -> LocalDatabase {
        let path = std::env::temp_dir()
            .join(format!("im-database-{}", Uuid::new_v4().simple()))
            .join(DATABASE_FILE_NAME);
        LocalDatabase::open_file(path, 2 * 1024 * 1024).unwrap()
    }

    #[test]
    fn namespaces_share_one_file_without_seeing_each_other() {
        let database = temp_database();
        let te = database.namespace(ModuleId::TeTestEquipment);
        let storage = database.namespace(ModuleId::TeStorage);

        te.insert(b"entry:1", b"te").unwrap();
        storage.insert(b"entry:1", b"storage").unwrap();
        storage.insert(b"entry:2", b"storage-2").unwrap();

        assert_eq!(te.get(b"entry:1").unwrap(), b"te");
        assert_eq!(storage.get(b"entry:1").unwrap(), b"storage");
        assert!(!te.contains_key(b"entry:2"));
        assert_eq!(te.path(), storage.path());

        let rows = storage.range_query(b"entry:", b"entry:\xff", 10).unwrap();
        assert_eq!(
            rows,
            vec![
                (b"entry:1".to_vec(), b"storage".to_vec()),
                (b"entry:2".to_vec(), b"storage-2".to_vec()),
            ]
        );

        storage.delete(b"entry:1").unwrap();
        assert!(!storage.contains_key(b"entry:1"));
        assert!(te.contains_key(b"entry:1"));
    }
}
