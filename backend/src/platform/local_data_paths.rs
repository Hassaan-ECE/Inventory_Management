use std::{
    env,
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

use tauri::Manager;

use super::dev_build::{dev_local_data_dir, is_dev_build};

pub(crate) const LOCAL_DATA_ROOT_ENV: &str = "INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT";

/// Where this app keeps its local databases: `%LOCALAPPDATA%\com.inventory.management` in
/// release builds, `<repo>/.tmp/dev-data` in developer builds, or the absolute
/// `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` override when set.
#[derive(Debug, Clone)]
pub(crate) struct InventoryAppPaths {
    local_data_dir: PathBuf,
}

impl InventoryAppPaths {
    pub(crate) fn resolve(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let local_data_dir = match validated_override_path(env::var_os(LOCAL_DATA_ROOT_ENV))? {
            Some(root) => root,
            None if is_dev_build() => dev_local_data_dir(),
            None => app.path().app_local_data_dir()?,
        };
        Ok(Self { local_data_dir })
    }

    pub(crate) fn local_data_dir(&self) -> &Path {
        &self.local_data_dir
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
    fn relative_or_empty_override_is_rejected() {
        let relative_error =
            validated_override_path(Some(OsString::from("relative-data"))).unwrap_err();
        let empty_error = validated_override_path(Some(OsString::new())).unwrap_err();

        assert_eq!(relative_error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(empty_error.kind(), io::ErrorKind::InvalidInput);
    }
}
