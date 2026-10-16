use serde::{de::DeserializeOwned, Serialize};

use super::super::model::CommandResult;
use super::InventoryDb;

const CALIBRATION_ROSTER_PREVIEW_PREFIX: &str = "calibration_roster:preview:";
const CALIBRATION_ROSTER_COMMIT_PREFIX: &str = "calibration_roster:commit:";
const CALIBRATION_ROSTER_COMPLETED_PREFIX: &str = "calibration_roster:completed:";
const CALIBRATION_ROSTER_OPERATION_PREFIX: &str = "calibration_roster:operation:";

impl InventoryDb {
    pub(crate) fn put_calibration_roster_preview<T: Serialize>(
        &self,
        batch_id: &str,
        preview: &T,
    ) -> CommandResult<()> {
        let key = calibration_roster_preview_key(batch_id)?;
        self.put_json(key.as_bytes(), preview)?;
        self.flush();
        Ok(())
    }

    pub(crate) fn calibration_roster_preview<T: DeserializeOwned>(
        &self,
        batch_id: &str,
    ) -> CommandResult<Option<T>> {
        let key = calibration_roster_preview_key(batch_id)?;
        self.get_json(key.as_bytes())
    }

    pub(crate) fn put_calibration_roster_commit_state<T: Serialize>(
        &self,
        batch_id: &str,
        state: &T,
    ) -> CommandResult<()> {
        let key = calibration_roster_commit_key(batch_id)?;
        self.put_json(key.as_bytes(), state)?;
        self.flush();
        Ok(())
    }

    pub(crate) fn calibration_roster_commit_state<T: DeserializeOwned>(
        &self,
        batch_id: &str,
    ) -> CommandResult<Option<T>> {
        let key = calibration_roster_commit_key(batch_id)?;
        self.get_json(key.as_bytes())
    }

    pub(crate) fn mark_calibration_roster_completed<T: Serialize>(
        &self,
        batch_id: &str,
        result: &T,
    ) -> CommandResult<()> {
        let key = calibration_roster_completed_key(batch_id)?;
        self.put_json(key.as_bytes(), result)?;
        self.flush();
        Ok(())
    }

    pub(crate) fn completed_calibration_roster<T: DeserializeOwned>(
        &self,
        batch_id: &str,
    ) -> CommandResult<Option<T>> {
        let key = calibration_roster_completed_key(batch_id)?;
        self.get_json(key.as_bytes())
    }

    pub(crate) fn has_calibration_roster_operation_marker(
        &self,
        batch_id: &str,
        operation_id: &str,
    ) -> CommandResult<bool> {
        let key = calibration_roster_operation_key(batch_id, operation_id)?;
        Ok(self.contains_key(key.as_bytes()))
    }

    pub(crate) fn mark_calibration_roster_operation_complete<T: Serialize>(
        &self,
        batch_id: &str,
        operation_id: &str,
        marker: &T,
    ) -> CommandResult<()> {
        let key = calibration_roster_operation_key(batch_id, operation_id)?;
        self.put_json(key.as_bytes(), marker)?;
        self.flush();
        Ok(())
    }
}

fn calibration_roster_preview_key(batch_id: &str) -> CommandResult<String> {
    import_key(CALIBRATION_ROSTER_PREVIEW_PREFIX, batch_id)
}

fn calibration_roster_commit_key(batch_id: &str) -> CommandResult<String> {
    import_key(CALIBRATION_ROSTER_COMMIT_PREFIX, batch_id)
}

fn calibration_roster_completed_key(batch_id: &str) -> CommandResult<String> {
    import_key(CALIBRATION_ROSTER_COMPLETED_PREFIX, batch_id)
}

fn calibration_roster_operation_key(batch_id: &str, operation_id: &str) -> CommandResult<String> {
    let batch_id = normalized_import_batch_id(batch_id)?;
    let operation_id = normalized_calibration_operation_id(operation_id)?;
    Ok(format!(
        "{CALIBRATION_ROSTER_OPERATION_PREFIX}{batch_id}:{operation_id}"
    ))
}

fn import_key(prefix: &str, batch_id: &str) -> CommandResult<String> {
    Ok(format!("{prefix}{}", normalized_import_batch_id(batch_id)?))
}

fn normalized_import_batch_id(batch_id: &str) -> CommandResult<String> {
    let value = batch_id.trim();
    if value.len() < 16
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err("import batch_id has an invalid format".to_string());
    }
    Ok(value.to_string())
}

fn normalized_calibration_operation_id(operation_id: &str) -> CommandResult<String> {
    let value = operation_id.trim();
    if value.len() < 8
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err("calibration roster operation_id has an invalid format".to_string());
    }
    Ok(value.to_string())
}
