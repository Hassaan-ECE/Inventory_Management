# Adaptive Per-Inventory Sync Lifecycle — Final Plan

**Status:** **Implemented** for TE Test Equipment in Inventory Management (2026-07-20). Still the behavioral authority for regressions and for future inventories that adopt the same lifecycle.  
**Decisions:** [IM-011](../../planning/DECISIONS.md) (this product); origin **D-029** in standalone TE Test Equipment Inventory planning  
**Recorded:** 2026-07-18  
**Implemented:** 2026-07-20  
**Home:** `C:\Projects\Active\Inventory_Management` (canonical for the unified product)

This plan is the implementation authority for adaptive, per-inventory synchronization in the Inventory Management shell. Prefer this document over informal chat summaries. Copied from the TE planning tree so the unified product owns the redesign track; implement here, not by shipping a new major TE standalone.

## Implementation record (2026-07-20)

TE-first slice shipped in this repo. Do **not** re-implement IM-011 unless regressing or extending to additional inventories.

| Area | Where it lives |
|------|----------------|
| Backend session + guarded watcher | `backend/src/api/commands.rs`, `backend/src/runtime/shared_watcher.rs` |
| Adaptive controller | `frontend/src/features/inventory/sync/adaptiveSyncController.ts` |
| Shell wiring + hard deactivate | `frontend/src/features/inventory/components/shell/useDesktopInventory.ts` |
| Switcher activation | `frontend/src/features/inventory/components/InventoryShell.tsx` |
| Bridge / guards | `frontend/src/integrations/tauri/tauriInventoryBridge.ts`, `bridgeGuards.ts` |
| Tests | `frontend/tests/adaptive-sync-controller.test.ts`, `inventory-shell-sync.test.tsx`, backend watcher lifecycle tests |

**Verification (automated):** frontend targeted + full suite, lint, production build, Rust watcher lifecycle + full suite (two external-path audits opt-in/filtered), shared-sync flow 37/37, clippy with warnings denied. Measured notes: [SESSION_HANDOFF.md](../../SESSION_HANDOFF.md) § IM-011 verification.

**Residual (manual):** live DevTools routine call-rate counts (active ≲30/min, idle/unfocused ≲1/min, deselected = 0 after drain) were not captured in the implementation session because the UI stayed on initial shared-sync loading during the smoke window. Fake-timer tests cover cadence and no-rearm. Optional follow-up when shared root is healthy and initial sync completes.

**Out of scope then and now:** monorepo extract, other modules, updater, schema/format migration, scanner optimization.

## Summary

Replace TE’s fixed 500 ms polling with a reusable, completion-aware lifecycle:

- Active TE: poll every 2 seconds after the previous sync completes.
- Idle TE: after 2 minutes without meaningful input, poll every 60 seconds.
- Hidden or unfocused: immediately use the 60-second cadence.
- Watcher event, activation, focus/visibility restoration, or idle wake: sync immediately.
- TE deselected: cancel all TE scheduling/subscriptions and stop its backend watcher.
- Use frontend generations and opaque backend session tokens to prevent stale work from reactivating TE.
- Preserve database, operation-log, snapshot, and shared-folder formats.

## Lifecycle and Data Flow

### Activation

1. Increment the frontend generation.
2. Call `activateInventorySync()`; this creates an in-memory TE session token without touching the shared path.
3. If the generation became stale while activation was pending, immediately deactivate that token and stop.
4. Subscribe to scoped TE change events using the current generation and token.
5. Keep cached rows visible; show blocking loading only if TE has never loaded.
6. Run `loadInventory()` to refresh from Local AppData.
7. If `shared.enabled === false`, unsubscribe and deactivate the session:
   - Do not call `syncInventory`.
   - Do not schedule polling.
   - Do not attach a watcher.
   - Keep local loading and mutations operational.
8. Otherwise run `syncInventory(sessionToken)`.
9. Apply `Some(result)` only if the generation remains current; treat `None` as an intentional stale no-op.
10. Begin adaptive scheduling only after initialization completes.

Events arriving during initialization are coalesced and serviced after local loading rather than starting a competing sync.

### Routine synchronization

- Activation reconciliation means `loadInventory()` followed by one `syncInventory(token)`.
- Poll, watcher, focus, activity-wake, and mutation requests call only `syncInventory(token)`.
- Do not reload all local entries on every poll.
- Preserve current optimization:
  - `entriesChanged: false` keeps existing rows.
  - `entriesChanged: true` applies returned rows.
- Use post-completion `setTimeout`, never `setInterval`.
- Coalesce simultaneous triggers into at most one follow-up request.
- No exponential error backoff:
  - Active/current session remains on the 2-second cadence.
  - Idle, hidden, or unfocused remains on the 60-second cadence.

### Activity policy

Fast polling requires all of:

- TE is selected.
- `document.visibilityState === "visible"`.
- The window is focused.
- Keyboard, pointer/button, touch, or wheel activity occurred within 120 seconds.

Window blur or hidden visibility switches immediately to slow polling. Focus or visibility restoration resets the activity deadline, performs one immediate sync, and resumes fast polling. Raw mouse movement does not count as activity.

### Hard deactivation

When TE is deselected or the shell unmounts:

1. Increment the frontend generation.
2. Clear the adaptive timer, idle deadline, 75 ms mutation timer, and pending follow-up.
3. Remove activity, focus, visibility, and shared-change listeners.
4. Fire `deactivateInventorySync(sessionToken)`.
5. Ignore every late result or error from the old generation.

An already-started sync or detached mutation publish may finish for integrity, but it cannot update the inactive UI, schedule follow-up work, or reattach the watcher. Process exit also drops backend watcher state naturally.

## Backend and Bridge Interfaces

### Tauri commands

Use the existing `CommandResult<T>` convention:

- `activate_inventory_sync() -> CommandResult<String>`
  - Generate an opaque TE-global session token.
  - Replace any prior session and drop its watcher.
  - Perform no shared-path access.

- `sync_inventory(session_id: String) -> CommandResult<Option<InventorySyncResult>>`
  - `Ok(None)` before work when the token is stale; perform no shared I/O.
  - Permit already-started valid work to finish if deactivated mid-run.
  - Recheck the token after completion.
  - Return `Ok(None)` and skip watcher attachment when no longer current.
  - Return `Ok(Some(result))` only for the current session.
  - Reserve `Err` for real database, coordinator, or sync failures.

- `deactivate_inventory_sync(session_id: String) -> CommandResult<bool>`
  - Return `true` when the matching session was deactivated.
  - Return `false` for stale cleanup.
  - Atomically stop the watcher and clear its path, debounce, and health state.

Keep token checks and watcher attach/stop under one lifecycle lock. A stale TE deactivation must never stop a newer TE session. Document that future multi-inventory backend support will replace this TE-global session with a `systemId → session` map.

### Desktop bridge

Add:

- `activateInventorySync(): Promise<string>`
- `syncInventory(sessionId: string): Promise<InventorySyncResult | null>`
- `deactivateInventorySync(sessionId: string): Promise<boolean>`

Add a nullable sync-result guard so Tauri `null` is treated as a stale no-op, not a parsing failure.

Shared-change payload:

```ts
interface InventorySharedChangedPayload {
  systemId: InventorySystemId;
}
```

Both watcher notifications and detached publish completion emit:

```json
{ "systemId": "te-test-equipment" }
```

Ignore malformed, missing, unknown, or non-TE identifiers.

### Watcher degradation

- Watcher attachment failure must not fail an otherwise successful data sync.
- Reuse `shared.message` or the existing one-shot announcer for “File watch unavailable; scheduled synchronization remains active.”
- Do not add a structured watcher-health field.
- Announce only on transition into degradation; retry attachment on later current-session syncs and clear degradation after success.

### Status cleanup

Remove `syncIntervalMs` completely from:

- Rust status models and constructors.
- TypeScript inventory status types.
- Bridge guards.
- Helpers, mocks, and clamp tests.

Do not modify updater scheduling or `useDesktopUpdates`.

## Implementation Order

All steps below were completed for TE on **2026-07-20** (see Implementation record above).

1. ~~Record D-029 in `DECISIONS.md` before behavior changes.~~ *(Done at plan acceptance as IM-011.)*
2. ~~Add failing Rust tests for token activation, stale sync, stale deactivation, stop idempotency, and in-flight deactivation.~~
3. ~~Implement the backend session lifecycle and guarded watcher attachment.~~
4. ~~Add scoped event payloads and nullable bridge signatures/guards.~~
5. ~~Add failing fake-timer tests for the reusable adaptive controller.~~
6. ~~Implement completion-aware scheduling, activity detection, generation invalidation, and unmount cleanup.~~
7. ~~Thread `teActive` from the existing switcher into `useDesktopInventory`; preserve cached state and do not recreate the owner’s WIP switcher files.~~
8. ~~Remove `syncIntervalMs` end-to-end.~~
9. ~~Run full verification and update `SESSION_HANDOFF.md` with measured results.~~

Operation-count/scanner optimization remains outside this change and should be reconsidered only after measuring the revised behavior.

## Test and Acceptance Plan

### Backend

- Activation is in-memory only and returns a token.
- Stale sync returns `Ok(None)` without touching the shared path.
- Stale `None` does not produce frontend error state.
- Deactivate during in-flight sync prevents watcher reattachment.
- Stale deactivation cannot stop a newer session.
- Rapid activation replacement leaves only the newest token current.
- Watcher stop is idempotent.
- Watcher attach happens only for the post-sync current token.
- Publish completion after deactivation preserves data and emits a scoped event without restarting the watcher.
- Disabled D-027 configuration performs no sync or watcher access.
- Watcher attachment failure remains a successful data sync with one degraded notification.

### Frontend

- Launch with TE selected follows activate → subscribe → local load → initial sync → schedule.
- Launch with a stored placeholder performs no TE load, activation, subscription, sync, or watch.
- `enabled: false` deactivates after local load and performs no poll, watch, or shared sync.
- Active polling is completion-aware and no faster than once every 2 seconds.
- Two minutes without activity changes to 60-second polling.
- Blur and hidden visibility immediately change to 60 seconds.
- Focus/visibility restoration causes one immediate sync and resumes fast polling.
- Watcher events sync immediately while TE is selected, including while idle.
- Wrong or missing `systemId` is ignored.
- Deactivation during the 75 ms mutation delay cancels the request.
- Deactivation during an in-flight sync ignores the result and schedules no follow-up.
- Unmount invokes the same hard deactivation.
- TE → placeholder → TE produces one current generation, timer, subscription, and backend session.
- Cached rows remain visible on return; local and shared changes are then reconciled.
- Routine `requestSync` does not call `loadInventory`.

### Verification

**Automated (done 2026-07-20):** see Implementation record and [SESSION_HANDOFF.md](../../SESSION_HANDOFF.md).

Run again on any lifecycle regression:

- Targeted frontend sync/switcher tests.
- `bun run test`
- `bun run lint`
- `bun run build`
- Rust tests.
- Clippy.
- Existing mutation, shared-sync, and two-database flows.

Single-instance DevTools smoke (**optional residual** — not closed 2026-07-20):

- Active TE: ≤ approximately 30 routine calls/minute.
- Idle, hidden, or unfocused TE: ≤ approximately 1 routine call/minute.
- Deselected TE: 0 new calls after in-flight work drains.
- Watcher/focus triggers remain prompt.
- Visible but unfocused TE uses the slow cadence.

Do not start another desktop instance while the owner’s current dev process is running.

## Documentation and Assumptions

### D-029 (authoritative text in DECISIONS.md)

Selected inventories use an adaptive, completion-aware sync lifecycle: approximately 2 seconds while focused and recently active, and approximately 60 seconds while idle, unfocused, or hidden, with immediate sync on activation, focus/visibility restoration, activity wake, mutation, and watcher events. Deselected inventories cancel frontend scheduling/subscriptions and stop their backend watcher. Opaque backend session tokens prevent stale sync or deactivation work from rearming an inactive inventory. D-027 default-on shared sync and its explicit opt-out remain unchanged. D-013 continues to apply: synchronization is not a backup.

### Additional assumptions

- Only TE is wired in this iteration.
- Future inventories receive independent databases, shared roots, watchers, adapters, and backend session-map entries.
- Inactive inventories initiate no new shared I/O; already-started integrity-preserving work may finish.
- Assume one running desktop instance per PC; multi-PC synchronization remains supported.
- No schema migration, shared-format migration, version bump, installer, release, updater, or scanner optimization is included.
- Preserve and merge around all current modified/untracked switcher work; do not reset or overwrite it.
- Update the session handoff only after measured verification.
