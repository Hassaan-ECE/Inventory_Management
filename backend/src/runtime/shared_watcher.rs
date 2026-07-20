use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use uuid::Uuid;

use crate::{model::CommandResult, platform::ModuleId};

pub(crate) const SHARED_INVENTORY_CHANGED_EVENT: &str = "inventory:shared-changed";
const WATCHER_EMIT_DEBOUNCE: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventorySharedChangedPayload {
    pub(crate) system_id: &'static str,
}

pub(crate) struct SharedSyncWatcher {
    state: Mutex<SharedSyncLifecycleState>,
}

struct SharedSyncLifecycleState {
    sessions: HashMap<ModuleId, ModuleSyncSessionState>,
}

struct ModuleSyncSessionState {
    current_session_id: Option<String>,
    watched_path: Option<PathBuf>,
    watcher: Option<RecommendedWatcher>,
    last_emit: Arc<Mutex<Option<Instant>>>,
    watcher_degraded: bool,
}

impl ModuleSyncSessionState {
    fn new() -> Self {
        Self {
            current_session_id: None,
            watched_path: None,
            watcher: None,
            last_emit: Arc::new(Mutex::new(None)),
            watcher_degraded: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SyncSessionCompletion {
    pub(crate) current: bool,
    pub(crate) watcher_degradation_started: bool,
}

impl SharedSyncWatcher {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(SharedSyncLifecycleState {
                sessions: HashMap::new(),
            }),
        }
    }

    pub(crate) fn activate_session(&self) -> CommandResult<String> {
        self.activate_session_for(ModuleId::TeTestEquipment)
    }

    fn activate_session_for(&self, module: ModuleId) -> CommandResult<String> {
        let session_id = Uuid::new_v4().to_string();
        let mut state = self.lock_state()?;
        let session = state
            .sessions
            .entry(module)
            .or_insert_with(ModuleSyncSessionState::new);
        stop_watching(session);
        session.current_session_id = Some(session_id.clone());
        Ok(session_id)
    }

    pub(crate) fn begin_sync(&self, session_id: &str) -> CommandResult<bool> {
        self.begin_sync_for(ModuleId::TeTestEquipment, session_id)
    }

    fn begin_sync_for(&self, module: ModuleId, session_id: &str) -> CommandResult<bool> {
        let state = self.lock_state()?;
        Ok(state
            .sessions
            .get(&module)
            .is_some_and(|session| session_is_current(session, session_id)))
    }

    pub(crate) fn complete_sync<R: Runtime>(
        &self,
        app: AppHandle<R>,
        session_id: &str,
        ops_dir: &Path,
    ) -> CommandResult<SyncSessionCompletion> {
        let module = ModuleId::TeTestEquipment;
        self.complete_sync_with_emit_for(module, session_id, ops_dir, move || {
            emit_module_shared_inventory_changed(&app, module);
        })
    }

    pub(crate) fn complete_sync_without_watcher(
        &self,
        session_id: &str,
    ) -> CommandResult<SyncSessionCompletion> {
        self.complete_sync_without_watcher_for(ModuleId::TeTestEquipment, session_id)
    }

    fn complete_sync_without_watcher_for(
        &self,
        module: ModuleId,
        session_id: &str,
    ) -> CommandResult<SyncSessionCompletion> {
        let state = self.lock_state()?;
        Ok(SyncSessionCompletion {
            current: state
                .sessions
                .get(&module)
                .is_some_and(|session| session_is_current(session, session_id)),
            watcher_degradation_started: false,
        })
    }

    pub(crate) fn deactivate_session(&self, session_id: &str) -> CommandResult<bool> {
        self.deactivate_session_for(ModuleId::TeTestEquipment, session_id)
    }

    fn deactivate_session_for(&self, module: ModuleId, session_id: &str) -> CommandResult<bool> {
        let mut state = self.lock_state()?;
        if !state
            .sessions
            .get(&module)
            .is_some_and(|session| session_is_current(session, session_id))
        {
            return Ok(false);
        }

        let mut session = state
            .sessions
            .remove(&module)
            .expect("current module session must exist");
        session.current_session_id = None;
        stop_watching(&mut session);
        Ok(true)
    }

    #[cfg(test)]
    fn complete_sync_with_emit<F>(
        &self,
        session_id: &str,
        ops_dir: &Path,
        emit: F,
    ) -> CommandResult<SyncSessionCompletion>
    where
        F: Fn() + Send + 'static,
    {
        self.complete_sync_with_emit_for(ModuleId::TeTestEquipment, session_id, ops_dir, emit)
    }

    fn complete_sync_with_emit_for<F>(
        &self,
        module: ModuleId,
        session_id: &str,
        ops_dir: &Path,
        emit: F,
    ) -> CommandResult<SyncSessionCompletion>
    where
        F: Fn() + Send + 'static,
    {
        self.complete_sync_with_attach_for(module, session_id, ops_dir, emit, ensure_watching)
    }

    #[cfg(test)]
    fn complete_sync_with_attach<F, A>(
        &self,
        session_id: &str,
        ops_dir: &Path,
        emit: F,
        attach: A,
    ) -> CommandResult<SyncSessionCompletion>
    where
        F: Fn() + Send + 'static,
        A: FnOnce(&mut ModuleSyncSessionState, &Path, F) -> CommandResult<()>,
    {
        self.complete_sync_with_attach_for(
            ModuleId::TeTestEquipment,
            session_id,
            ops_dir,
            emit,
            attach,
        )
    }

    fn complete_sync_with_attach_for<F, A>(
        &self,
        module: ModuleId,
        session_id: &str,
        ops_dir: &Path,
        emit: F,
        attach: A,
    ) -> CommandResult<SyncSessionCompletion>
    where
        F: Fn() + Send + 'static,
        A: FnOnce(&mut ModuleSyncSessionState, &Path, F) -> CommandResult<()>,
    {
        let mut state = self.lock_state()?;
        let Some(session) = state.sessions.get_mut(&module) else {
            return Ok(SyncSessionCompletion {
                current: false,
                watcher_degradation_started: false,
            });
        };
        if !session_is_current(session, session_id) {
            return Ok(SyncSessionCompletion {
                current: false,
                watcher_degradation_started: false,
            });
        }

        let watcher_degradation_started = match attach(session, ops_dir, emit) {
            Ok(()) => {
                session.watcher_degraded = false;
                false
            }
            Err(_) => {
                let degradation_started = !session.watcher_degraded;
                stop_watcher_resources(session);
                session.watcher_degraded = true;
                degradation_started
            }
        };

        Ok(SyncSessionCompletion {
            current: true,
            watcher_degradation_started,
        })
    }

    fn lock_state(&self) -> CommandResult<std::sync::MutexGuard<'_, SharedSyncLifecycleState>> {
        self.state
            .lock()
            .map_err(|_| "Shared sync watcher state is unavailable.".to_string())
    }

    #[cfg(test)]
    pub(crate) fn watched_path_for_test(&self) -> Option<PathBuf> {
        self.state.lock().ok().and_then(|state| {
            state
                .sessions
                .get(&ModuleId::TeTestEquipment)
                .and_then(|session| session.watched_path.clone())
        })
    }

    #[cfg(test)]
    pub(crate) fn session_count_for_test(&self) -> usize {
        self.state
            .lock()
            .map(|state| state.sessions.len())
            .unwrap_or(0)
    }

    #[cfg(test)]
    pub(crate) fn current_session_for_test(&self, module: ModuleId) -> Option<String> {
        self.state.lock().ok().and_then(|state| {
            state
                .sessions
                .get(&module)
                .and_then(|session| session.current_session_id.clone())
        })
    }
}

pub(crate) fn emit_shared_inventory_changed<R: Runtime>(app: &AppHandle<R>) {
    emit_module_shared_inventory_changed(app, ModuleId::TeTestEquipment);
}

fn emit_module_shared_inventory_changed<R: Runtime>(app: &AppHandle<R>, module: ModuleId) {
    let _ = app.emit(
        SHARED_INVENTORY_CHANGED_EVENT,
        InventorySharedChangedPayload {
            system_id: module.as_system_id_str(),
        },
    );
}

fn session_is_current(state: &ModuleSyncSessionState, session_id: &str) -> bool {
    state.current_session_id.as_deref() == Some(session_id)
}

fn ensure_watching<F>(
    state: &mut ModuleSyncSessionState,
    ops_dir: &Path,
    emit: F,
) -> CommandResult<()>
where
    F: Fn() + Send + 'static,
{
    if !ops_dir.exists() {
        return Ok(());
    }

    let normalized_path = normalize_watched_path(ops_dir);
    if state
        .watched_path
        .as_ref()
        .is_some_and(|current| paths_match(current, &normalized_path))
    {
        return Ok(());
    }

    stop_watcher_resources(state);
    let last_emit = Arc::clone(&state.last_emit);
    let mut watcher = RecommendedWatcher::new(
        move |result: notify::Result<notify::Event>| {
            let Ok(event) = result else {
                return;
            };
            if !event_kind_should_emit(&event.kind) {
                return;
            }
            if should_debounce_emit(&last_emit) {
                return;
            }
            emit();
        },
        Config::default(),
    )
    .map_err(|error| format!("Could not start shared sync watcher: {error}"))?;

    watcher
        .watch(&normalized_path, RecursiveMode::Recursive)
        .map_err(|error| format!("Could not watch shared sync operations: {error}"))?;
    state.watched_path = Some(normalized_path);
    state.watcher = Some(watcher);
    Ok(())
}

fn stop_watching(state: &mut ModuleSyncSessionState) {
    stop_watcher_resources(state);
    state.watcher_degraded = false;
}

fn stop_watcher_resources(state: &mut ModuleSyncSessionState) {
    state.watcher.take();
    state.watched_path = None;
    state.last_emit = Arc::new(Mutex::new(None));
}

fn event_kind_should_emit(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Any | EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

fn should_debounce_emit(last_emit: &Arc<Mutex<Option<Instant>>>) -> bool {
    let Ok(mut last_emit) = last_emit.lock() else {
        return true;
    };
    if last_emit
        .as_ref()
        .is_some_and(|instant| instant.elapsed() < WATCHER_EMIT_DEBOUNCE)
    {
        return true;
    }
    *last_emit = Some(Instant::now());
    false
}

fn normalize_watched_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn paths_match(left: &Path, right: &Path) -> bool {
    if cfg!(windows) {
        left.to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy())
    } else {
        left == right
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, thread};

    #[test]
    fn activation_is_in_memory_and_replaces_the_previous_session() {
        let watcher = SharedSyncWatcher::new();
        let untouched_shared_path = unique_ops_dir("activation");

        let first_session = watcher.activate_session().unwrap();
        let second_session = watcher.activate_session().unwrap();

        assert!(!first_session.is_empty());
        assert!(!second_session.is_empty());
        assert_ne!(first_session, second_session);
        assert!(!untouched_shared_path.exists());
        assert!(!watcher.begin_sync(&first_session).unwrap());
        assert!(watcher.begin_sync(&second_session).unwrap());
    }

    #[test]
    fn lifecycle_sessions_are_keyed_by_module_and_removed_on_deactivation() {
        let watcher = SharedSyncWatcher::new();

        assert_eq!(watcher.session_count_for_test(), 0);
        let session = watcher.activate_session().unwrap();
        assert_eq!(watcher.session_count_for_test(), 1);
        assert_eq!(
            watcher.current_session_for_test(ModuleId::TeTestEquipment),
            Some(session.clone())
        );

        assert!(watcher.deactivate_session(&session).unwrap());
        assert_eq!(watcher.session_count_for_test(), 0);
        assert_eq!(
            watcher.current_session_for_test(ModuleId::TeTestEquipment),
            None
        );
    }

    #[test]
    fn shared_change_payload_is_scoped_to_te_test_equipment() {
        let payload = InventorySharedChangedPayload {
            system_id: ModuleId::TeTestEquipment.as_system_id_str(),
        };

        assert_eq!(
            serde_json::to_value(payload).unwrap(),
            serde_json::json!({ "systemId": "te-test-equipment" })
        );
    }

    #[test]
    fn stale_sync_is_rejected_before_shared_work_starts() {
        let watcher = SharedSyncWatcher::new();
        let current_session = watcher.activate_session().unwrap();
        let mut shared_work_started = false;

        if watcher.begin_sync("stale-session").unwrap() {
            shared_work_started = true;
        }

        assert!(!shared_work_started);
        assert!(watcher.begin_sync(&current_session).unwrap());
    }

    #[test]
    fn stale_deactivation_cannot_stop_a_newer_session() {
        let watcher = SharedSyncWatcher::new();
        let stale_session = watcher.activate_session().unwrap();
        let current_session = watcher.activate_session().unwrap();

        assert!(!watcher.deactivate_session(&stale_session).unwrap());
        assert!(watcher.begin_sync(&current_session).unwrap());
        assert!(watcher.deactivate_session(&current_session).unwrap());
    }

    #[test]
    fn watcher_stop_is_idempotent_through_repeated_deactivation() {
        let watcher = SharedSyncWatcher::new();
        let session = watcher.activate_session().unwrap();
        let ops_dir = unique_ops_dir("stop-idempotent");
        std::fs::create_dir_all(&ops_dir).unwrap();

        let completion = watcher
            .complete_sync_with_emit(&session, &ops_dir, || {})
            .unwrap();
        assert!(completion.current);
        assert!(watcher.watched_path_for_test().is_some());

        assert!(watcher.deactivate_session(&session).unwrap());
        assert!(watcher.watched_path_for_test().is_none());
        assert!(!watcher.deactivate_session(&session).unwrap());
        assert!(watcher.watched_path_for_test().is_none());

        std::fs::remove_dir_all(ops_dir).unwrap();
    }

    #[test]
    fn deactivation_during_in_flight_sync_prevents_watcher_attachment() {
        let watcher = Arc::new(SharedSyncWatcher::new());
        let session = watcher.activate_session().unwrap();
        let ops_dir = unique_ops_dir("in-flight-deactivation");
        std::fs::create_dir_all(&ops_dir).unwrap();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (finish_tx, finish_rx) = std::sync::mpsc::channel();
        let worker_watcher = Arc::clone(&watcher);
        let worker_session = session.clone();
        let worker_ops_dir = ops_dir.clone();

        let worker = thread::spawn(move || {
            assert!(worker_watcher.begin_sync(&worker_session).unwrap());
            started_tx.send(()).unwrap();
            finish_rx.recv().unwrap();
            worker_watcher
                .complete_sync_with_emit(&worker_session, &worker_ops_dir, || {})
                .unwrap()
        });

        started_rx.recv().unwrap();
        assert!(watcher.deactivate_session(&session).unwrap());
        finish_tx.send(()).unwrap();

        let completion = worker.join().unwrap();
        assert!(!completion.current);
        assert!(watcher.watched_path_for_test().is_none());

        std::fs::remove_dir_all(ops_dir).unwrap();
    }

    #[test]
    fn missing_ops_dir_is_ignored() {
        let watcher = SharedSyncWatcher::new();
        let missing_path = unique_ops_dir("missing");
        let session = watcher.activate_session().unwrap();

        watcher
            .complete_sync_with_emit(&session, &missing_path, || {})
            .unwrap();

        assert!(watcher.watched_path_for_test().is_none());
    }

    #[test]
    fn watcher_starts_after_ops_dir_becomes_available() {
        let watcher = SharedSyncWatcher::new();
        let session = watcher.activate_session().unwrap();
        let ops_dir = unique_ops_dir("available");

        watcher
            .complete_sync_with_emit(&session, &ops_dir, || {})
            .unwrap();
        assert!(watcher.watched_path_for_test().is_none());

        std::fs::create_dir_all(&ops_dir).unwrap();
        watcher
            .complete_sync_with_emit(&session, &ops_dir, || {})
            .unwrap();
        assert!(watcher.watched_path_for_test().is_some());

        watcher.deactivate_session(&session).unwrap();
        std::fs::remove_dir_all(ops_dir).unwrap();
    }

    #[test]
    fn watcher_attachment_failure_degrades_once_and_recovers_after_success() {
        let watcher = SharedSyncWatcher::new();
        let session = watcher.activate_session().unwrap();
        let ops_dir = unique_ops_dir("degradation");

        let first_failure = watcher
            .complete_sync_with_attach(
                &session,
                &ops_dir,
                || {},
                |_, _, _| Err("watcher unavailable".to_string()),
            )
            .unwrap();
        let repeated_failure = watcher
            .complete_sync_with_attach(
                &session,
                &ops_dir,
                || {},
                |_, _, _| Err("watcher unavailable".to_string()),
            )
            .unwrap();

        assert!(first_failure.current);
        assert!(first_failure.watcher_degradation_started);
        assert!(repeated_failure.current);
        assert!(!repeated_failure.watcher_degradation_started);
        assert!(watcher.watched_path_for_test().is_none());

        std::fs::create_dir_all(&ops_dir).unwrap();
        let recovered = watcher
            .complete_sync_with_emit(&session, &ops_dir, || {})
            .unwrap();

        assert!(recovered.current);
        assert!(!recovered.watcher_degradation_started);
        assert!(watcher.watched_path_for_test().is_some());

        watcher.deactivate_session(&session).unwrap();
        std::fs::remove_dir_all(ops_dir).unwrap();
    }

    fn unique_ops_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "inventory-management-{prefix}-{}",
            Uuid::new_v4().simple()
        ))
    }
}
