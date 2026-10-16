//! Developer builds (`tauri dev`, `cargo test`, any debug build) keep every database and
//! shared folder inside the repository's `.tmp` folder. That way a build from source can
//! never write to this computer's real inventory or to the live team share; the 2026-10-15
//! TE divergence started with an operation a dev build wrote to the live share.
//! The `INVENTORY_MANAGEMENT_*` path overrides still apply when set explicitly.

use std::path::PathBuf;

pub(crate) fn is_dev_build() -> bool {
    cfg!(debug_assertions)
}

/// `<repo>/.tmp/dev-data`: stands in for `%LOCALAPPDATA%\com.inventory.management`.
pub(crate) fn dev_local_data_dir() -> PathBuf {
    dev_workspace_dir().join("dev-data")
}

/// `<repo>/.tmp/dev-share`: stands in for the product share on S:.
pub(crate) fn dev_share_dir() -> PathBuf {
    dev_workspace_dir().join("dev-share")
}

fn dev_workspace_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR is `<repo>/backend`; `.tmp` sits next to it at the repo root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).with_file_name(".tmp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_folders_live_in_the_repository_tmp_folder() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();

        assert!(is_dev_build());
        assert_eq!(
            dev_local_data_dir(),
            repo_root.join(".tmp").join("dev-data")
        );
        assert_eq!(dev_share_dir(), repo_root.join(".tmp").join("dev-share"));
    }
}
