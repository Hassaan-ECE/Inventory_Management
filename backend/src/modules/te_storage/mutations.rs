use crate::modules::te_storage::{
    entry_changes::{
        changed_entry_fields, entry_base_version, normalize_changed_entry_fields,
        update_entry_from_input_fields,
    },
    model::{
        create_entry_from_input, normalize_entry_input, update_entry_from_input,
        validate_entry_input, validate_inventory_entry, CommandResult,
        InventoryDeleteMutationResult, InventoryEntry, InventoryEntryEditContext,
        InventoryEntryInput, InventoryEntryMutationResult, InventorySharedStatus,
    },
    store::InventoryDb,
    sync::{self, SyncOperationType},
};

fn create_entry_in_store_inner(
    input: InventoryEntryInput,
    db: &InventoryDb,
) -> CommandResult<InventoryEntryMutationResult> {
    let input = normalize_entry_input(input);
    validate_entry_input(&input)?;

    let id = db.next_entry_id()?;
    let entry = create_entry_from_input(id, input);
    db.put_entry(&entry)?;
    db.set_next_entry_id(id + 1)?;
    let sync_state = match queue_entry_sync_operation_before_flush(
        db,
        SyncOperationType::InventoryEntryCreate,
        entry.clone(),
        Vec::new(),
        None,
    ) {
        Ok(sync_state) => sync_state,
        Err(error) => {
            let _ = db.delete_entry(&entry);
            let _ = db.set_next_entry_id(id);
            db.flush();
            return Err(error);
        }
    };
    db.flush();

    Ok(InventoryEntryMutationResult {
        entry,
        message: "Entry added to the TE Storage database.".to_string(),
        mutation_mode: sync_state.mutation_mode,
        shared: sync_state.shared,
    })
}

fn update_entry_in_store_inner(
    entry_id: &str,
    input: InventoryEntryInput,
    edit_context: Option<InventoryEntryEditContext>,
    db: &InventoryDb,
) -> CommandResult<InventoryEntryMutationResult> {
    let input = normalize_entry_input(input);
    validate_entry_input(&input)?;

    let existing = db
        .find_entry(entry_id)?
        .ok_or_else(|| "The selected entry could not be found.".to_string())?;
    let has_edit_context = edit_context.is_some();
    let edit_context = edit_context.unwrap_or_default();
    let context_changed_fields = normalize_changed_entry_fields(edit_context.changed_fields);
    let base_version = edit_context
        .base_version
        .filter(|version| !version.trim().is_empty())
        .or_else(|| entry_base_version(&existing));
    let mut entry = if !has_edit_context {
        update_entry_from_input(existing.clone(), input)
    } else {
        update_entry_from_input_fields(existing.clone(), &input, &context_changed_fields)
    };
    validate_inventory_entry(&entry)?;
    entry.updated_at = super::model::timestamp_after(&existing.updated_at);
    let changed_fields = changed_entry_fields(&existing, &entry);
    if changed_fields.is_empty() {
        return Ok(InventoryEntryMutationResult {
            entry: existing,
            message: "Entry was already up to date.".to_string(),
            mutation_mode: "local".to_string(),
            shared: sync::shared_inventory_status(db, "FeOxDB local store ready."),
        });
    }
    db.put_entry(&entry)?;
    let sync_state = match queue_entry_sync_operation_before_flush(
        db,
        SyncOperationType::InventoryEntryUpdate,
        entry.clone(),
        changed_fields,
        base_version,
    ) {
        Ok(sync_state) => sync_state,
        Err(error) => {
            let _ = db.put_entry(&existing);
            db.flush();
            return Err(error);
        }
    };
    db.flush();

    Ok(InventoryEntryMutationResult {
        entry,
        message: "Entry updated in the TE Storage database.".to_string(),
        mutation_mode: sync_state.mutation_mode,
        shared: sync_state.shared,
    })
}

fn delete_entry_in_store_inner(
    entry_id: &str,
    db: &InventoryDb,
) -> CommandResult<InventoryDeleteMutationResult> {
    let entry = db
        .find_entry(entry_id)?
        .ok_or_else(|| "The selected entry could not be found.".to_string())?;
    let deleted_at_utc = super::model::timestamp_after(&entry.updated_at);
    db.delete_entry(&entry)?;
    let sync_state = match queue_delete_sync_operation_before_flush(
        db,
        &entry.entry_uuid,
        deleted_at_utc,
        entry_base_version(&entry),
    ) {
        Ok(sync_state) => sync_state,
        Err(error) => {
            let _ = db.put_entry(&entry);
            db.flush();
            return Err(error);
        }
    };
    db.flush();

    Ok(InventoryDeleteMutationResult {
        entry_id: entry.id,
        message: "Entry deleted.".to_string(),
        mutation_mode: sync_state.mutation_mode,
        shared: sync_state.shared,
    })
}

#[derive(Debug, Clone)]
struct QueuedMutationState {
    mutation_mode: String,
    shared: InventorySharedStatus,
}

fn queue_entry_sync_operation_before_flush(
    db: &InventoryDb,
    operation_type: SyncOperationType,
    entry: InventoryEntry,
    changed_fields: Vec<String>,
    base_version: Option<String>,
) -> CommandResult<QueuedMutationState> {
    sync::queue_entry_operation(db, operation_type, entry, changed_fields, base_version)?;

    Ok(QueuedMutationState {
        mutation_mode: "local".to_string(),
        shared: sync::queued_local_status(db),
    })
}

fn queue_delete_sync_operation_before_flush(
    db: &InventoryDb,
    entry_uuid: &str,
    deleted_at_utc: String,
    base_version: Option<String>,
) -> CommandResult<QueuedMutationState> {
    sync::queue_delete_operation(db, entry_uuid, deleted_at_utc, base_version)?;

    Ok(QueuedMutationState {
        mutation_mode: "local".to_string(),
        shared: sync::queued_local_status(db),
    })
}

// FeOx operations are individually fallible. Roll back the row and its entire
// outbox/metadata unit together, so a reported failure cannot later publish.
fn with_mutation_rollback<T>(
    db: &InventoryDb,
    entry_id: &str,
    mutate: impl FnOnce() -> CommandResult<T>,
) -> CommandResult<T> {
    let prior_entry = db.find_entry(entry_id)?;
    let prior_next_id = db.next_entry_id()?;
    let sync_backup = db.backup_sync_state()?;
    match mutate() {
        Ok(value) => Ok(value),
        Err(error) => {
            let rollback = (|| -> CommandResult<()> {
                db.restore_sync_state(sync_backup)?;
                if let Some(entry) = prior_entry {
                    db.put_entry(&entry)?;
                } else if let Some(entry) = db.find_entry(entry_id)? {
                    db.delete_entry(&entry)?;
                }
                db.set_next_entry_id(prior_next_id)?;
                Ok(())
            })();
            db.flush();
            match rollback {
                Ok(()) => Err(error),
                Err(rollback_error) => Err(format!("{error}. Could not restore the previous state: {rollback_error}. Stop editing and contact support.")),
            }
        }
    }
}

pub(crate) fn create_entry_in_store(
    input: InventoryEntryInput,
    db: &InventoryDb,
) -> CommandResult<InventoryEntryMutationResult> {
    let entry_id = db.next_entry_id()?.to_string();
    with_mutation_rollback(db, &entry_id, || create_entry_in_store_inner(input, db))
}

pub(crate) fn update_entry_in_store(
    entry_id: &str,
    input: InventoryEntryInput,
    context: Option<InventoryEntryEditContext>,
    db: &InventoryDb,
) -> CommandResult<InventoryEntryMutationResult> {
    with_mutation_rollback(db, entry_id, || {
        update_entry_in_store_inner(entry_id, input, context, db)
    })
}

pub(crate) fn delete_entry_in_store(
    entry_id: &str,
    db: &InventoryDb,
) -> CommandResult<InventoryDeleteMutationResult> {
    with_mutation_rollback(db, entry_id, || delete_entry_in_store_inner(entry_id, db))
}
