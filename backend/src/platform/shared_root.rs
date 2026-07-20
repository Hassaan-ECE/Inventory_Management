use std::path::PathBuf;

use super::ModuleId;

pub(crate) const DEFAULT_SHARED_ROOT: &str =
    r"S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules\TE_Test_Equipment";

pub(crate) fn default_shared_root(module: ModuleId) -> PathBuf {
    match module {
        ModuleId::TeTestEquipment => PathBuf::from(DEFAULT_SHARED_ROOT),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::platform::ModuleId;

    use super::{default_shared_root, DEFAULT_SHARED_ROOT};

    #[test]
    fn te_default_root_matches_the_product_module_path() {
        let resolved = default_shared_root(ModuleId::TeTestEquipment);

        assert_eq!(resolved, PathBuf::from(DEFAULT_SHARED_ROOT));
        assert!(resolved.ends_with(PathBuf::from("modules").join("TE_Test_Equipment")));
    }
}
