use std::path::{Path, PathBuf};

use super::{
    dev_build::{dev_share_dir, is_dev_build},
    ModuleId,
};

/// Product share that release builds sync through.
const TEAM_SHARE_ROOT: &str = r"S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App";

/// Each inventory's folder under `modules\`. These names are part of the live share layout
/// and stay fixed when display names change (EVCS Storage still lives in `TE_Storage_Room`).
fn module_folder(module: ModuleId) -> &'static str {
    match module {
        ModuleId::TeTestEquipment => "TE_Test_Equipment",
        ModuleId::TeLabComponents => "TE_Lab_Components",
        ModuleId::TeStorage => "TE_Storage_Room",
    }
}

/// Shared root for a module when no env override is set: the team share in release
/// builds, `<repo>/.tmp/dev-share` in developer builds.
pub(crate) fn default_shared_root(module: ModuleId) -> PathBuf {
    let share_root = if is_dev_build() {
        dev_share_dir()
    } else {
        PathBuf::from(TEAM_SHARE_ROOT)
    };
    module_shared_root(&share_root, module)
}

/// A module's folder in a share laid out like the team share: `<share>\modules\<folder>`.
pub(crate) fn module_shared_root(share_root: &Path, module: ModuleId) -> PathBuf {
    share_root.join("modules").join(module_folder(module))
}

/// Live team share folder for a module, regardless of build type.
#[cfg(test)]
pub(crate) fn team_shared_root(module: ModuleId) -> PathBuf {
    module_shared_root(Path::new(TEAM_SHARE_ROOT), module)
}

/// The live team share's root folder, regardless of build type.
#[cfg(test)]
pub(crate) fn team_share_root() -> &'static Path {
    Path::new(TEAM_SHARE_ROOT)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::platform::{dev_build::dev_share_dir, ModuleId};

    use super::{default_shared_root, team_shared_root};

    #[test]
    fn team_roots_match_the_live_share_layout() {
        let modules = PathBuf::from(
            r"S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules",
        );

        assert_eq!(
            team_shared_root(ModuleId::TeTestEquipment),
            modules.join("TE_Test_Equipment")
        );
        assert_eq!(
            team_shared_root(ModuleId::TeLabComponents),
            modules.join("TE_Lab_Components")
        );
        assert_eq!(
            team_shared_root(ModuleId::TeStorage),
            modules.join("TE_Storage_Room")
        );
    }

    #[test]
    fn dev_builds_default_to_the_dev_share_with_the_same_layout() {
        for module in [
            ModuleId::TeTestEquipment,
            ModuleId::TeLabComponents,
            ModuleId::TeStorage,
        ] {
            let resolved = default_shared_root(module);

            assert!(resolved.starts_with(dev_share_dir()));
            assert!(resolved.ends_with(
                team_shared_root(module)
                    .strip_prefix(
                        r"S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App"
                    )
                    .unwrap()
            ));
        }
    }
}
