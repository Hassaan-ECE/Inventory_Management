use std::{
    env,
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

use tauri::Manager;

pub(crate) const LOCAL_DATA_ROOT_ENV: &str = "INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT";

#[derive(Debug, Clone)]
pub(crate) struct InventoryAppPaths {
    local_data_dir: PathBuf,
    roaming_data_dir: PathBuf,
    cache_dir: PathBuf,
}

impl InventoryAppPaths {
    pub(crate) fn resolve(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(root) = validated_override_path(env::var_os(LOCAL_DATA_ROOT_ENV))? {
            return Ok(Self::isolated(root));
        }

        Ok(Self {
            local_data_dir: app.path().app_local_data_dir()?,
            roaming_data_dir: app.path().app_data_dir()?,
            cache_dir: app.path().app_cache_dir()?,
        })
    }

    pub(crate) fn local_data_dir(&self) -> &Path {
        &self.local_data_dir
    }

    pub(crate) fn roaming_data_dir(&self) -> &Path {
        &self.roaming_data_dir
    }

    pub(crate) fn cleanup_roots(&self) -> [&Path; 3] {
        [
            &self.roaming_data_dir,
            &self.local_data_dir,
            &self.cache_dir,
        ]
    }

    fn isolated(root: PathBuf) -> Self {
        Self {
            roaming_data_dir: root.join("legacy-roaming"),
            cache_dir: root.join("cache"),
            local_data_dir: root,
        }
    }
}

fn validated_override_path(value: Option<OsString>) -> io::Result<Option<PathBuf>> {
    let Some(value) = value else {
        return Ok(None);
    };

    if value.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{LOCAL_DATA_ROOT_ENV} cannot be empty"),
        ));
    }

    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{LOCAL_DATA_ROOT_ENV} must be an absolute path"),
        ));
    }

    Ok(Some(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_override_isolates_local_roaming_and_cache_paths() {
        let root = env::temp_dir().join("inventory-management-isolated-paths");
        let resolved = validated_override_path(Some(root.clone().into_os_string()))
            .unwrap()
            .unwrap();
        let paths = InventoryAppPaths::isolated(resolved);

        assert_eq!(paths.local_data_dir(), root);
        assert_eq!(paths.roaming_data_dir(), root.join("legacy-roaming"));
        assert_eq!(paths.cleanup_roots()[2], root.join("cache"));
    }

    #[test]
    fn relative_or_empty_override_is_rejected() {
        let relative_error =
            validated_override_path(Some(OsString::from("relative-data"))).unwrap_err();
        let empty_error = validated_override_path(Some(OsString::new())).unwrap_err();

        assert_eq!(relative_error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(empty_error.kind(), io::ErrorKind::InvalidInput);
    }
}
