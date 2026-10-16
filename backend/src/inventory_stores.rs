use crate::{
    modules::{te_lab_components::store::InventoryDb as LabComponentsDb, te_storage},
    platform::{InventoryAppPaths, LocalDatabase},
    store::InventoryDb,
};

/// Every inventory's store, each a namespace of the one local database file.
pub(crate) struct InventoryStores {
    database: LocalDatabase,
    te_test_equipment: InventoryDb,
    te_lab_components: LabComponentsDb,
    te_storage: te_storage::store::InventoryDb,
}

impl InventoryStores {
    pub(crate) fn open(paths: &InventoryAppPaths) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::from_database(LocalDatabase::open(
            paths.local_data_dir(),
        )?))
    }

    #[cfg(test)]
    pub(crate) fn open_at(root: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::from_database(LocalDatabase::open(&root)?))
    }

    fn from_database(database: LocalDatabase) -> Self {
        Self {
            te_test_equipment: InventoryDb::open_in(&database),
            te_lab_components: LabComponentsDb::open_in(&database),
            te_storage: te_storage::store::InventoryDb::open_in(&database),
            database,
        }
    }

    pub(crate) fn te_test_equipment(&self) -> &InventoryDb {
        &self.te_test_equipment
    }

    pub(crate) fn te_lab_components(&self) -> &LabComponentsDb {
        &self.te_lab_components
    }

    pub(crate) fn te_storage(&self) -> &te_storage::store::InventoryDb {
        &self.te_storage
    }

    pub(crate) fn flush(&self) {
        self.database.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::InventoryStores;
    use crate::platform::DATABASE_FILE_NAME;
    use std::{env, fs};
    use uuid::Uuid;

    #[test]
    fn every_inventory_lives_in_the_one_local_database_file() {
        let root = env::temp_dir().join(format!("inventory-stores-{}", Uuid::new_v4().simple()));
        fs::create_dir_all(&root).unwrap();

        let stores = InventoryStores::open_at(root.clone()).unwrap();

        let database = root.join(DATABASE_FILE_NAME).to_string_lossy().into_owned();
        assert_eq!(stores.te_test_equipment().db_path_string(), database);
        assert_eq!(stores.te_lab_components().db_path_string(), database);
        assert_eq!(stores.te_storage().db_path_string(), database);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    }

    #[test]
    fn inventories_do_not_see_each_others_entries() {
        let root = env::temp_dir().join(format!("inventory-stores-{}", Uuid::new_v4().simple()));
        let stores = InventoryStores::open_at(root).unwrap();

        stores
            .te_test_equipment()
            .set_client_id("te-client")
            .unwrap();

        assert_eq!(
            stores.te_test_equipment().client_id().unwrap().as_deref(),
            Some("te-client")
        );
        assert_eq!(stores.te_storage().client_id().unwrap(), None);
        assert_eq!(stores.te_lab_components().client_id().unwrap(), None);
    }
}
