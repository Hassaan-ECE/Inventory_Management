//! The one local database file. Every inventory reads and writes through its own
//! [`Namespace`], which stores each key as `<module id>/<key>`, so the inventories share a
//! file without being able to see each other's records.

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use feoxdb::{FeoxError, FeoxStore};

use super::{legacy_databases::merge_legacy_databases_once, ModuleId};

pub(crate) const DATABASE_FILE_NAME: &str = "inventory-management.feox";

/// FeOx files never grow after creation (writes fail with `OutOfSpace` once full), so the
/// file is sized well above the three 64 MiB files it replaces.
pub(super) const DATABASE_FILE_SIZE: u64 = 256 * 1024 * 1024;

/// FeOx writes each record as a 30-byte header plus key and value, rounded up to whole
/// 4 KiB blocks, after 16 blocks reserved for its own metadata.
const BLOCK_SIZE: u64 = 4096;
const RECORD_OVERHEAD: u64 = 30;
const RESERVED_BLOCKS: u64 = 16;
/// Measuring usage reads every record, so it is refreshed at most this often.
const USAGE_REFRESH: Duration = Duration::from_secs(5 * 60);

const SCAN_BATCH: usize = 512;
/// Keys are UTF-8, which never contains the byte 0xFF, so this sorts after every key.
const KEY_SPACE_END: &[u8] = &[0xFF];

/// How full the database file is, and whether FeOx has failed to write any record to disk
/// since the app started (FeOx keeps such records in memory only, so they are lost on exit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DatabaseHealth {
    pub percent_used: u8,
    pub write_failures: u64,
}

pub(crate) struct LocalDatabase {
    store: Arc<FeoxStore>,
    path: Arc<PathBuf>,
    usage: Arc<UsageCache>,
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
            usage: Arc::new(UsageCache {
                file_size,
                measured: Mutex::new(None),
            }),
        })
    }

    pub(crate) fn namespace(&self, module: ModuleId) -> Namespace {
        Namespace {
            store: Arc::clone(&self.store),
            prefix: format!("{}/", module.as_system_id_str()).into_bytes(),
            path: Arc::clone(&self.path),
            usage: Arc::clone(&self.usage),
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
    usage: Arc<UsageCache>,
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

    /// Health of the whole database file, which every inventory shares.
    pub(crate) fn database_health(&self) -> DatabaseHealth {
        DatabaseHealth {
            percent_used: self.usage.percent_used(&self.store),
            write_failures: self.store.stats().write_failures,
        }
    }

    fn full_key(&self, key: &[u8]) -> Vec<u8> {
        [self.prefix.as_slice(), key].concat()
    }
}

struct UsageCache {
    file_size: u64,
    measured: Mutex<Option<(Instant, u8)>>,
}

impl UsageCache {
    fn percent_used(&self, store: &FeoxStore) -> u8 {
        let Ok(mut measured) = self.measured.lock() else {
            return 0;
        };
        if let Some((at, percent)) = *measured {
            if at.elapsed() < USAGE_REFRESH {
                return percent;
            }
        }
        let percent = measure_percent_used(store, self.file_size);
        *measured = Some((Instant::now(), percent));
        percent
    }
}

fn measure_percent_used(store: &FeoxStore, file_size: u64) -> u8 {
    let total_blocks = (file_size / BLOCK_SIZE).max(1);
    let mut used_blocks = RESERVED_BLOCKS;
    let scanned = for_each_key(store, |key, value| {
        let record_bytes = RECORD_OVERHEAD + key.len() as u64 + value.len() as u64;
        used_blocks += record_bytes.div_ceil(BLOCK_SIZE);
        Ok(())
    });
    if scanned.is_err() {
        return 0;
    }
    (used_blocks * 100 / total_blocks).min(100) as u8
}

/// Visits every key in the store in order.
pub(super) fn for_each_key(
    store: &FeoxStore,
    mut visit: impl FnMut(&[u8], &[u8]) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let mut start = Vec::new();
    loop {
        let batch = store.range_query(&start, KEY_SPACE_END, SCAN_BATCH)?;
        for (key, value) in &batch {
            visit(key, value)?;
        }
        match batch.last() {
            Some((key, _)) if batch.len() == SCAN_BATCH => start = next_key(key),
            _ => return Ok(()),
        }
    }
}

/// Counts every key in a namespace.
pub(super) fn count_keys(namespace: &Namespace) -> Result<usize, Box<dyn Error>> {
    let mut count = 0usize;
    let mut start = Vec::new();
    loop {
        let batch = namespace.range_query(&start, KEY_SPACE_END, SCAN_BATCH)?;
        count += batch.len();
        match batch.last() {
            Some((key, _)) if batch.len() == SCAN_BATCH => start = next_key(key),
            _ => return Ok(count),
        }
    }
}

/// Smallest key strictly after `key` (range queries are inclusive).
fn next_key(key: &[u8]) -> Vec<u8> {
    let mut next = key.to_vec();
    next.push(0);
    next
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

    #[test]
    fn usage_counts_whole_blocks_per_record_across_every_namespace() {
        let database = temp_database();
        let te = database.namespace(ModuleId::TeTestEquipment);
        let storage = database.namespace(ModuleId::TeStorage);
        // 2 MiB holds 512 blocks; 16 are reserved.
        assert_eq!(te.database_health().percent_used, 3);

        for n in 0..100 {
            te.insert(format!("entry:{n}").as_bytes(), b"small")
                .unwrap();
        }
        // One record spanning two blocks.
        storage.insert(b"entry:big", &vec![b'x'; 5000]).unwrap();
        *database.usage.measured.lock().unwrap() = None;

        let health = storage.database_health();
        assert_eq!(health.percent_used, ((16 + 100 + 2) * 100 / 512) as u8);
        assert_eq!(health.write_failures, 0);
    }
}
