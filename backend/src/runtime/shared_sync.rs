use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::{model::CommandResult, platform::ModuleId};

/// Serializes every backend critical section that can change inventory rows,
/// local sync metadata, operation files, or shared snapshot state.
///
/// Mutations take the same gate as manual sync and background publish because
/// entry records, outbox operations, applied markers, tombstones, and revisions
/// are committed as one recoverable unit.
#[derive(Clone)]
pub(crate) struct SharedSyncCoordinator {
    gates: Arc<HashMap<ModuleId, Arc<Mutex<()>>>>,
    last_background_statuses: Arc<Mutex<HashMap<ModuleId, serde_json::Value>>>,
}

impl Default for SharedSyncCoordinator {
    fn default() -> Self {
        Self {
            gates: Arc::new(HashMap::from([
                (ModuleId::TeTestEquipment, Arc::new(Mutex::new(()))),
                (ModuleId::TeLabComponents, Arc::new(Mutex::new(()))),
            ])),
            last_background_statuses: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl SharedSyncCoordinator {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn run_exclusive<T>(
        &self,
        module: ModuleId,
        operation_name: &str,
        operation: impl FnOnce() -> CommandResult<T>,
    ) -> CommandResult<T> {
        let gate =
            self.gates.get(&module).cloned().ok_or_else(|| {
                format!("No shared sync coordinator exists for {operation_name}.")
            })?;
        let _guard = gate.lock().map_err(|_| {
            format!("Shared sync coordinator is unavailable during {operation_name}.")
        })?;
        operation()
    }

    pub(crate) fn set_background_status<T: Serialize>(
        &self,
        module: ModuleId,
        status: T,
    ) -> CommandResult<()> {
        let status = serde_json::to_value(status)
            .map_err(|error| format!("Could not store shared sync status: {error}"))?;
        let mut last_background_statuses = self
            .last_background_statuses
            .lock()
            .map_err(|_| "Shared sync status state is unavailable.".to_string())?;
        last_background_statuses.insert(module, status);
        Ok(())
    }

    pub(crate) fn background_status<T: DeserializeOwned>(
        &self,
        module: ModuleId,
    ) -> CommandResult<Option<T>> {
        let last_background_statuses = self
            .last_background_statuses
            .lock()
            .map_err(|_| "Shared sync status state is unavailable.".to_string())?;
        last_background_statuses
            .get(&module)
            .cloned()
            .map(serde_json::from_value)
            .transpose()
            .map_err(|error| format!("Could not read shared sync status: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InventorySharedStatus;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        thread,
        time::Duration,
    };

    #[test]
    fn shared_sync_operations_are_serialized() {
        let coordinator = SharedSyncCoordinator::new();
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));
        let mut handles = Vec::new();

        for _ in 0..4 {
            let coordinator = coordinator.clone();
            let active = Arc::clone(&active);
            let max_active = Arc::clone(&max_active);
            handles.push(thread::spawn(move || {
                coordinator
                    .run_exclusive(ModuleId::TeTestEquipment, "test", || {
                        let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                        max_active.fetch_max(current, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(5));
                        active.fetch_sub(1, Ordering::SeqCst);
                        Ok(())
                    })
                    .unwrap();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(max_active.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn different_modules_use_independent_sync_gates() {
        let coordinator = SharedSyncCoordinator::new();
        let (te_started_tx, te_started_rx) = std::sync::mpsc::channel();
        let (release_te_tx, release_te_rx) = std::sync::mpsc::channel();
        let te_coordinator = coordinator.clone();
        let te_worker = thread::spawn(move || {
            te_coordinator
                .run_exclusive(ModuleId::TeTestEquipment, "TE sync", || {
                    te_started_tx.send(()).unwrap();
                    release_te_rx.recv().unwrap();
                    Ok(())
                })
                .unwrap();
        });

        te_started_rx.recv().unwrap();
        let (lab_started_tx, lab_started_rx) = std::sync::mpsc::channel();
        let lab_coordinator = coordinator.clone();
        let lab_worker = thread::spawn(move || {
            lab_coordinator
                .run_exclusive(ModuleId::TeLabComponents, "Lab sync", || {
                    lab_started_tx.send(()).unwrap();
                    Ok(())
                })
                .unwrap();
        });

        let lab_started_while_te_was_held = lab_started_rx
            .recv_timeout(Duration::from_millis(250))
            .is_ok();
        release_te_tx.send(()).unwrap();
        te_worker.join().unwrap();
        lab_worker.join().unwrap();

        assert!(lab_started_while_te_was_held);
    }

    #[test]
    fn manual_sync_and_mutation_triggered_publish_share_the_same_gate() {
        let coordinator = SharedSyncCoordinator::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let manual_coordinator = coordinator.clone();
        let manual_events = Arc::clone(&events);

        let manual_sync = thread::spawn(move || {
            manual_coordinator
                .run_exclusive(ModuleId::TeTestEquipment, "manual sync", || {
                    manual_events.lock().unwrap().push("manual-start");
                    started_tx.send(()).unwrap();
                    thread::sleep(Duration::from_millis(10));
                    manual_events.lock().unwrap().push("manual-end");
                    Ok(())
                })
                .unwrap();
        });

        started_rx.recv().unwrap();
        coordinator
            .run_exclusive(
                ModuleId::TeTestEquipment,
                "mutation-triggered publish",
                || {
                    events.lock().unwrap().push("publish");
                    Ok(())
                },
            )
            .unwrap();
        manual_sync.join().unwrap();

        assert_eq!(
            events.lock().unwrap().as_slice(),
            ["manual-start", "manual-end", "publish"]
        );
    }

    #[test]
    fn background_publish_status_is_available_to_later_loads() {
        let coordinator = SharedSyncCoordinator::new();
        let status = InventorySharedStatus {
            available: false,
            can_modify: true,
            enabled: true,
            has_local_only_changes: Some(true),
            last_snapshot_id: None,
            message: "Background shared publish failed: disk full".to_string(),
            mutation_mode: "local".to_string(),
            revision: Some("3".to_string()),
            shared_root_path: Some("S:\\TE\\Test_Equipment".to_string()),
        };

        coordinator
            .set_background_status(ModuleId::TeTestEquipment, status)
            .unwrap();

        let stored: crate::model::InventorySharedStatus = coordinator
            .background_status(ModuleId::TeTestEquipment)
            .unwrap()
            .unwrap();
        assert_eq!(
            stored.message,
            "Background shared publish failed: disk full"
        );
        assert_eq!(stored.has_local_only_changes, Some(true));
        assert_eq!(stored.mutation_mode, "local");
        assert!(coordinator
            .background_status::<crate::modules::te_lab_components::model::InventorySharedStatus>(
                ModuleId::TeLabComponents
            )
            .unwrap()
            .is_none());
    }
}
