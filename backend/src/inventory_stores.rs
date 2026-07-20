use crate::{
    modules::te_lab_components::store::InventoryDb as LabComponentsDb, store::InventoryDb,
};

pub(crate) struct InventoryStores {
    te_test_equipment: InventoryDb,
    te_lab_components: LabComponentsDb,
}

impl InventoryStores {
    pub(crate) fn open(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            te_test_equipment: InventoryDb::open(app)?,
            te_lab_components: LabComponentsDb::open(app)?,
        })
    }

    #[cfg(test)]
    pub(crate) fn open_at(root: std::path::PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            te_test_equipment: InventoryDb::open_at(root.join("inventory.feox"))?,
            te_lab_components: LabComponentsDb::open_at(
                root.join(crate::modules::te_lab_components::storage::DB_FILE_NAME),
            )?,
        })
    }

    pub(crate) fn te_test_equipment(&self) -> &InventoryDb {
        &self.te_test_equipment
    }

    pub(crate) fn te_lab_components(&self) -> &LabComponentsDb {
        &self.te_lab_components
    }

    pub(crate) fn flush(&self) {
        self.te_test_equipment.flush();
        self.te_lab_components.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::InventoryStores;
    use std::{env, fs, path::PathBuf};
    use uuid::Uuid;

    #[test]
    fn opens_separate_database_files_without_renaming_te() {
        let root = unique_test_dir("inventory-stores");
        fs::create_dir_all(&root).unwrap();

        let stores = InventoryStores::open_at(root.clone()).unwrap();

        assert_eq!(
            PathBuf::from(stores.te_test_equipment().db_path_string()),
            root.join("inventory.feox")
        );
        assert_eq!(
            PathBuf::from(stores.te_lab_components().db_path_string()),
            root.join("te-lab-components.feox")
        );
        assert_ne!(
            stores.te_test_equipment().db_path_string(),
            stores.te_lab_components().db_path_string()
        );
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
    }
}
