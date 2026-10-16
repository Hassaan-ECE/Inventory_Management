//! One-time move from the per-inventory database files of 0.1.x into the single
//! `inventory-management.feox`. The merge copies every key into a staging file, checks each
//! copied value against its source, and only then renames the staging file into place and
//! moves the old files to `pre-0.2-databases\`. Any failure leaves the old files untouched.

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use feoxdb::FeoxStore;

use super::{
    database::{LocalDatabase, Namespace, DATABASE_FILE_NAME, DATABASE_FILE_SIZE},
    ModuleId,
};

const LEGACY_FILES: [(ModuleId, &str); 3] = [
    (ModuleId::TeTestEquipment, "inventory.feox"),
    (ModuleId::TeLabComponents, "te-lab-components.feox"),
    (ModuleId::TeStorage, "te-storage-room.feox"),
];
pub(crate) const LEGACY_BACKUP_DIR: &str = "pre-0.2-databases";
const STAGING_SUFFIX: &str = ".merging";
const COPY_BATCH: usize = 512;
/// Keys are UTF-8, which never contains the byte 0xFF, so this sorts after every key.
const KEY_SPACE_END: &[u8] = &[0xFF];

pub(super) fn merge_legacy_databases_once(local_data_dir: &Path) -> Result<(), Box<dyn Error>> {
    let target = local_data_dir.join(DATABASE_FILE_NAME);
    let legacy: Vec<(ModuleId, PathBuf)> = LEGACY_FILES
        .iter()
        .map(|(module, name)| (*module, local_data_dir.join(name)))
        .filter(|(_, path)| path.is_file())
        .collect();
    if target.exists() || legacy.is_empty() {
        return Ok(());
    }

    let staging = local_data_dir.join(format!("{DATABASE_FILE_NAME}{STAGING_SUFFIX}"));
    if staging.exists() {
        fs::remove_file(&staging)?;
    }
    if let Err(error) = write_merged_file(&staging, &legacy) {
        let _ = fs::remove_file(&staging);
        return Err(format!("Could not merge the local inventory databases: {error}").into());
    }
    fs::rename(&staging, &target)?;

    let backup_dir = local_data_dir.join(LEGACY_BACKUP_DIR);
    fs::create_dir_all(&backup_dir)?;
    for (_, path) in &legacy {
        if let Some(name) = path.file_name() {
            fs::rename(path, backup_dir.join(name))?;
        }
    }
    Ok(())
}

fn write_merged_file(staging: &Path, legacy: &[(ModuleId, PathBuf)]) -> Result<(), Box<dyn Error>> {
    {
        let merged = LocalDatabase::open_file(staging.to_path_buf(), DATABASE_FILE_SIZE)?;
        for (module, path) in legacy {
            copy_all_keys(&open_legacy(path)?, &merged.namespace(*module))?;
        }
        merged.flush();
    }

    // FeOx accepts writes in memory and persists them in the background, so the copy is
    // checked against a fresh open of the staging file, not the instance that wrote it.
    let reopened = LocalDatabase::open_file(staging.to_path_buf(), DATABASE_FILE_SIZE)?;
    for (module, path) in legacy {
        verify_copy(&open_legacy(path)?, &reopened.namespace(*module), path)?;
    }
    Ok(())
}

fn open_legacy(path: &Path) -> Result<FeoxStore, Box<dyn Error>> {
    Ok(FeoxStore::builder()
        .device_path(path.to_string_lossy().into_owned())
        .build()?)
}

fn copy_all_keys(source: &FeoxStore, target: &Namespace) -> Result<(), Box<dyn Error>> {
    for_each_key(source, |key, value| {
        target.insert(key, value)?;
        Ok(())
    })
}

fn verify_copy(source: &FeoxStore, target: &Namespace, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut source_count = 0usize;
    for_each_key(source, |key, value| {
        source_count += 1;
        if target.get(key)? != value {
            return Err(format!(
                "{}: key {} did not copy exactly",
                path.display(),
                String::from_utf8_lossy(key)
            )
            .into());
        }
        Ok(())
    })?;

    let mut target_count = 0usize;
    let mut start = Vec::new();
    loop {
        let batch = target.range_query(&start, KEY_SPACE_END, COPY_BATCH)?;
        target_count += batch.len();
        match batch.last() {
            Some((key, _)) if batch.len() == COPY_BATCH => start = next_key(key),
            _ => break,
        }
    }
    if source_count != target_count {
        return Err(format!(
            "{}: copied {target_count} of {source_count} keys",
            path.display()
        )
        .into());
    }
    Ok(())
}

fn for_each_key(
    source: &FeoxStore,
    mut visit: impl FnMut(&[u8], &[u8]) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let mut start = Vec::new();
    loop {
        let batch = source.range_query(&start, KEY_SPACE_END, COPY_BATCH)?;
        for (key, value) in &batch {
            visit(key, value)?;
        }
        match batch.last() {
            Some((key, _)) if batch.len() == COPY_BATCH => start = next_key(key),
            _ => return Ok(()),
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

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("im-legacy-merge-{}", Uuid::new_v4().simple()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_legacy(dir: &Path, name: &str, rows: &[(String, String)]) {
        let store = FeoxStore::builder()
            .device_path(dir.join(name).to_string_lossy().into_owned())
            // Each record takes at least one 4 KiB block; size for the largest fixture.
            .file_size(16 * 1024 * 1024)
            .build()
            .unwrap();
        for (key, value) in rows {
            store.insert(key.as_bytes(), value.as_bytes()).unwrap();
        }
        store.flush_all();
    }

    #[test]
    fn merges_every_legacy_file_into_its_namespace_and_backs_up_the_originals() {
        let dir = temp_dir();
        // More rows than one copy batch, to cover pagination.
        let te_rows: Vec<_> = (0..1_200)
            .map(|n| (format!("entry:{n:05}"), format!("te-{n}")))
            .collect();
        let storage_rows = vec![("entry:1".to_string(), "storage".to_string())];
        write_legacy(&dir, "inventory.feox", &te_rows);
        write_legacy(&dir, "te-storage-room.feox", &storage_rows);

        merge_legacy_databases_once(&dir).unwrap();

        assert!(dir.join(DATABASE_FILE_NAME).is_file());
        assert!(!dir.join("inventory.feox").exists());
        assert!(dir.join(LEGACY_BACKUP_DIR).join("inventory.feox").is_file());
        assert!(dir
            .join(LEGACY_BACKUP_DIR)
            .join("te-storage-room.feox")
            .is_file());

        let database = LocalDatabase::open(&dir).unwrap();
        let te = database.namespace(ModuleId::TeTestEquipment);
        let storage = database.namespace(ModuleId::TeStorage);
        let lab = database.namespace(ModuleId::TeLabComponents);
        for (key, value) in &te_rows {
            assert_eq!(te.get(key.as_bytes()).unwrap(), value.as_bytes());
        }
        assert_eq!(storage.get(b"entry:1").unwrap(), b"storage");
        assert!(!lab.contains_key(b"entry:1"));
    }

    #[test]
    fn runs_once_and_never_touches_an_existing_merged_database() {
        let dir = temp_dir();
        write_legacy(
            &dir,
            "inventory.feox",
            &[("entry:1".into(), "first".into())],
        );
        merge_legacy_databases_once(&dir).unwrap();

        // A stray legacy file appearing later (for example from a 0.1.x reinstall) is ignored.
        write_legacy(
            &dir,
            "inventory.feox",
            &[("entry:1".into(), "stale".into())],
        );
        merge_legacy_databases_once(&dir).unwrap();

        let database = LocalDatabase::open(&dir).unwrap();
        let te = database.namespace(ModuleId::TeTestEquipment);
        assert_eq!(te.get(b"entry:1").unwrap(), b"first");
        assert!(dir.join("inventory.feox").exists());
    }

    #[test]
    fn a_fresh_install_starts_with_an_empty_merged_database() {
        let dir = temp_dir();
        merge_legacy_databases_once(&dir).unwrap();
        assert!(!dir.join(DATABASE_FILE_NAME).exists());
        assert!(!dir.join(LEGACY_BACKUP_DIR).exists());
    }
}
