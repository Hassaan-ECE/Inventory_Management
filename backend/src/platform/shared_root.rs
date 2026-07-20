use std::path::PathBuf;

use super::ModuleId;

/// Pre-release pilot default: live team TE share so `bun run desktop` loads real inventory.
/// Before team release, flip this to the product module path and copy latest shared data there:
/// `...\Inventory_Management_App\modules\TE_Test_Equipment`
pub(crate) const DEFAULT_SHARED_ROOT: &str =
    r"S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE_Test_Equipment_Inventory";

/// Pre-release pilot default for the separate TE Lab Components shared stream.
/// Before team release, flip this to the product module path and copy latest shared data there:
/// `...\Inventory_Management_App\modules\TE_Lab_Components`
pub(crate) const DEFAULT_LAB_COMPONENTS_SHARED_ROOT: &str =
    r"S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE";

/// Post-cutover product root (not the active default until release).
#[allow(dead_code)]
pub(crate) const PRODUCT_TE_SHARED_ROOT: &str =
    r"S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules\TE_Test_Equipment";

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
    fn te_default_root_matches_the_active_shared_root() {
        let resolved = default_shared_root(ModuleId::TeTestEquipment);

        assert_eq!(resolved, PathBuf::from(DEFAULT_SHARED_ROOT));
        assert!(resolved.ends_with("TE_Test_Equipment_Inventory"));
    }

    #[test]
    fn lab_components_default_root_matches_the_active_pilot_share() {
        let resolved = default_shared_root(ModuleId::TeLabComponents);

        assert_eq!(resolved, PathBuf::from(DEFAULT_LAB_COMPONENTS_SHARED_ROOT));
        assert_eq!(
            resolved,
            PathBuf::from(r"S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE")
        );
        assert_ne!(resolved, default_shared_root(ModuleId::TeTestEquipment));
    }
}
