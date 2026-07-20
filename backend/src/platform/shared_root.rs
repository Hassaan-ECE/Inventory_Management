use std::path::PathBuf;

use super::ModuleId;

/// Product module shared root for TE Test Equipment (team release default).
pub(crate) const DEFAULT_SHARED_ROOT: &str =
    r"S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules\TE_Test_Equipment";

/// Product module shared root for TE Lab Components (team release default).
pub(crate) const DEFAULT_LAB_COMPONENTS_SHARED_ROOT: &str =
    r"S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules\TE_Lab_Components";

/// Pre-cutover InventoryApps pilots (diagnostic / env override only).
#[allow(dead_code)]
pub(crate) const PILOT_TE_SHARED_ROOT: &str =
    r"S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE_Test_Equipment_Inventory";

#[allow(dead_code)]
pub(crate) const PILOT_LAB_COMPONENTS_SHARED_ROOT: &str =
    r"S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE";

pub(crate) fn default_shared_root(module: ModuleId) -> PathBuf {
    match module {
        ModuleId::TeTestEquipment => PathBuf::from(DEFAULT_SHARED_ROOT),
        ModuleId::TeLabComponents => PathBuf::from(DEFAULT_LAB_COMPONENTS_SHARED_ROOT),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::platform::ModuleId;

    use super::{default_shared_root, DEFAULT_LAB_COMPONENTS_SHARED_ROOT, DEFAULT_SHARED_ROOT};

    #[test]
    fn te_default_root_matches_the_product_module_path() {
        let resolved = default_shared_root(ModuleId::TeTestEquipment);

        assert_eq!(resolved, PathBuf::from(DEFAULT_SHARED_ROOT));
        assert!(resolved.ends_with(PathBuf::from("modules").join("TE_Test_Equipment")));
    }

    #[test]
    fn lab_components_default_root_matches_the_product_module_path() {
        let resolved = default_shared_root(ModuleId::TeLabComponents);

        assert_eq!(resolved, PathBuf::from(DEFAULT_LAB_COMPONENTS_SHARED_ROOT));
        assert!(resolved.ends_with(PathBuf::from("modules").join("TE_Lab_Components")));
        assert_ne!(resolved, default_shared_root(ModuleId::TeTestEquipment));
    }
}
