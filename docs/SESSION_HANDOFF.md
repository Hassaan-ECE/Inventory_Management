# Session handoff — Inventory Management

**Last updated:** 2026-07-25
**State:** Product `0.1.0` — stable identity + S: product share; TE Test Equipment and TE Lab Components are implemented as isolated modules. **IM-015 Calibration workspace and roster tooling are implemented and pre-cutover verified; no live calibration workbook commit has occurred.** ME Storage and TE Storage Room remain placeholders.
**Implemented redesigns:** Adaptive per-inventory sync lifecycle **IM-011** ([plan](superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md)), logical platform/module architecture **IM-012** ([plan](superpowers/plans/2026-07-20-platform-module-architecture-extract.md)), Phase **C1 TE Lab Components** ([plan](superpowers/plans/2026-07-20-te-lab-components-port.md)), and the implementation side of **IM-015** ([plan](superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md)).
**Docs hygiene:** SESSION_START_PROMPT, AGENTS, capability roadmap, and this handoff describe the implemented architecture so new chats do not re-open IM-011, IM-012, C1, or IM-015 implementation as greenfield work. IM-015 Phase 8 remains an owner-controlled live operation.
**Desktop runtime fixes (2026-07-20):** `devUrl` uses `http://127.0.0.1:5173` (avoid localhost→IPv6); capability `core:window:allow-set-title`; adaptive controller wraps `setTimeout`/`clearTimeout` to avoid WebView `Illegal invocation`. Full restart of `bun run desktop` required after capability change.

**New chat:** paste [SESSION_START_PROMPT.md](SESSION_START_PROMPT.md). Move context: [CHAT_HANDOFF.md](CHAT_HANDOFF.md).

## Workspace

```text
C:\Projects\Active\Inventory_Management
```

(Not under `Inventory_Apps` — that tree holds the standalone sibling apps only.)

IM-015 was developed and verified in the isolated checkout below so concurrent work could proceed safely. Check branch/worktree integration status before continuing from `main`:

```text
C:\Projects\Active\Inventory_Management_IM015
branch: feature/im-015-calibration
```

Product share (verified created this session):

```text
S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App
```

## Identity

| Item | Value |
|------|--------|
| Display | Inventory Management |
| Package | `inventory-management` `0.1.0` |
| Tauri id | `com.inventory.management` |
| TE Test Equipment DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| TE Lab Components DB | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| Product share | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE shared root | `...\Inventory_Management_App\modules\TE_Test_Equipment` |
| Default Lab shared root | `...\Inventory_Management_App\modules\TE_Lab_Components` |

## What exists

- Full ME/TE-family scaffold copied from TE Test Equipment Inventory and rebranded.
- Header **hamburger** switcher: TE Test Equipment, TE Lab Components, ME Storage, TE Storage Room.
- Real TE Test Equipment and TE Lab Components desktop modules; placeholders only for ME Storage and TE Storage Room.
- Product chrome and switching live under `frontend/src/shell/`; module registry/persistence and adaptive sync live under `frontend/src/platform/`.
- TE UI/domain remains under `frontend/src/modules/te-test-equipment/` and the existing `inventory.feox`. It now exposes **Equipment** and **Calibration** as two projections of the same records; the persisted calibration fields and sync schema v2 remain unchanged.
- Lab UI/domain lives under `frontend/src/modules/te-lab-components/`; it has no calibration fields, uses `verifiedInSurvey`, sync schema v1, and the separate `te-lab-components.feox` DB.
- The shell keeps both desktop hosts mounted, runs lifecycle work only for the active module, hard-deactivates the inactive session, and preserves each module's cached rows across switches.
- Both real modules reuse the completion-aware IM-011 schedule: approximately 2 seconds while selected/active and approximately 60 seconds while idle, hidden, or unfocused; activation, restoration, activity, mutation, and correctly scoped watcher events can request immediate sync.
- Backend inventory lifecycle, query, CRUD, and export commands are module-scoped. The Tauri JSON boundary dispatches into distinct TE and Lab types instead of merging schemas; TE import remains TE-only.
- `backend/src/inventory_stores.rs` owns isolated FeOx handles. Shared roots, sync gates, watcher sessions, opaque tokens, statuses, and events are keyed by `ModuleId`; stale work for one module cannot deactivate the other.
- TE retains its calibration workbook export; Lab has a separate standalone-compatible 19-column workbook without calibration columns.
- TE now has a calibration-specific `.xlsx` roster preview/review/commit flow with explicit resolutions, stale-preview guards, deterministic creates, field-scoped updates, resumable completion markers, and one final shared publish. The approved general inventory importer remains unchanged.
- S: tree: `modules\*`, `release-support\`, `legacy-pointers\`, `README.md`.
- Updater **configured** for this product (pubkey + GitHub `latest.json` endpoint; `createUpdaterArtifacts: true`). Private key on build PC only — see `docs/engineering/UPDATER_AND_RELEASE.md`.
- Decisions **IM-001…IM-013** plus **IM-015** in this branch; implementation plans live under `docs/superpowers/plans/`.
- Git: `main` → `https://github.com/Hassaan-ECE/Inventory_Management.git` (initial scaffold push 2026-07-18).

## First team release (v0.1.0) — 2026-07-20

- Signed NSIS: `backend/target/release/bundle/nsis/Inventory Management_0.1.0_x64-setup.exe` (+ `.sig`)
- Product share installer: `S:\...\Inventory_Management_App\Inventory Management_0.1.0_x64-setup.exe`
- Archive: `S:\...\Inventory_Management_App\release-support\v0.1.0\`
- GitHub Release: https://github.com/Hassaan-ECE/Inventory_Management/releases/tag/v0.1.0  
  (setup + `.sig` + `latest.json` for in-app Update)

## What does **not** exist yet

- Port of ME Storage / TE Storage Room data layers
- Team cutover completed (standalone writers retired after team is stable on unified)
- IM-015 live calibration roster cutover: no live backup, owner row resolutions, commit, shared publish, or restart verification has been performed.
- Final integration of the concurrent implementation branches into `main`; resolve shared-document conflicts carefully and rerun gates after integration.

## Standalone apps (unchanged)

Continue to work in their own trees until cutover; do not treat this folder as TE Test Equipment Inventory.

## Env

| Env | Purpose |
|-----|---------|
| `INVENTORY_MANAGEMENT_SHARED_ROOT` | Override TE module shared root |
| `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT` | Override Lab module shared root |
| `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` | Opt out of shared sync |
| `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` | Optional HMAC |

## Shared roots (product defaults — cutover copy 2026-07-20)

**Copy performed 2026-07-20 ~16:42 local** (robocopy `/E`, no `/MIR`) from InventoryApps pilots into product modules:

| Module | Product shared inventory | Spot-check |
|--------|--------------------------|------------|
| TE Test Equipment | `...\modules\TE_Test_Equipment\shared\inventory` | manifest present; ops=1; snaps=4 |
| TE Lab Components | `...\modules\TE_Lab_Components\shared\inventory` | manifest present; ops=1; snaps=3 |

App code defaults now point at those product roots (see `backend/src/platform/shared_root.rs`).

**Rules**

- One writer per root — do **not** run standalone TE Test Equipment against its pilot share, or standalone TE Parts/Lab Components against `InventoryApps\TE`, while this app is open.
- **Before team release:** copy the latest TE and Lab shared data into `...\Inventory_Management_App\modules\TE_Test_Equipment` and `...\Inventory_Management_App\modules\TE_Lab_Components`, deliberately flip both defaults, then ship.

## IM-011 verification — 2026-07-20

- Targeted frontend sync/switcher gate: **4 files, 57 tests passed**.
- Full frontend suite: **17 files passed, 1 skipped; 142 tests passed, 1 skipped**.
- `bun run lint`: passed.
- `bun run build`: passed (`tsc -b` plus Vite production build).
- Targeted Rust watcher lifecycle: **9 passed**.
- Full Rust suite with only the two external-path audits filtered: passed. `shared_sync_flow` passed **37/37**, including `two_databases_push_and_pull_create_update_and_delete` and the signed two-database flow.
- Raw `cargo test` reaches the same green code paths but the two opt-in live-audit cases require unset `TE_LEGACY_AUDIT_XLSX` and `TE_INVENTORY_AUDIT_DB`; they were not pointed at owner data during this implementation session.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Static lifecycle check: no inventory sync `setInterval` or `syncIntervalMs` remains; the only `setInterval` is the unchanged updater scheduler in `useDesktopUpdates.ts`.
- Single-instance desktop smoke: confirmed no owner instance was running, launched one Tauri instance, verified the TE WebView and bridge, measured `activateInventorySync` at **3 ms** and local `loadInventory` at **17 ms**, then fully stopped the process and DevTools listener.
- Residual manual verification: post-initial-sync DevTools call counts for active, slow, and deselected TE were not captured because the live UI remained in initial shared-sync loading during the observation window and WebView2 did not retain the pre-page CDP instrumentation. Fake-timer coverage verifies the 2-second, 120-second idle transition, 60-second, immediate-trigger, and no-rearm behavior deterministically.

## IM-012 verification — 2026-07-20

- Full frontend suite: **18 files passed, 1 skipped; 145 tests passed, 1 skipped**. The existing unrelated React `act(...)` warning in `entry-dialog.test.tsx` remains non-failing.
- `bun run lint`: passed.
- `bun run build`: passed (`tsc -b` plus Vite production build; JS **351.22 kB**, CSS **49.63 kB** before gzip).
- Rust library suite: **73/73 passed**, including the new `ModuleId`, shared-root, token/session-map, stale-deactivation, idempotent stop, and in-flight deactivation cases.
- `cargo test --test shared_sync_flow`: **39/39 passed**, including unsigned/signed two-database push/pull and existing mutation/bootstrap/snapshot flows.
- `cargo fmt --all -- --check`: passed after applying rustfmt-only layout changes.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Single-instance desktop smoke: confirmed no existing app/Vite listener, launched one instrumented Tauri instance, verified the Inventory Management title and all four switcher labels, selected TE Lab Components before its C1 port, returned to TE, retained the empty local table state (**0 → 0 rows**), observed no runtime errors or `Illegal invocation`, then stopped the app and confirmed ports **5173/9222** and the desktop process were closed.
- Residual risk: the live desktop cache check used an empty local DB; the seeded-row path is covered by `inventory-shell-sync.test.tsx` (`keeps cached TE rows visible across placeholder switch`). Optional IM-011 live cadence soak A2 remains separate and non-blocking.

## Phase C1 verification — 2026-07-20

- Full frontend suite: **19 files passed, 1 skipped; 148 tests passed, 1 skipped**. The existing unrelated React `act(...)` warning in `entry-dialog.test.tsx` remains non-failing.
- `bun run lint`: passed.
- `bun run build`: passed (`tsc -b` plus Vite production build; JS **419.81 kB**, CSS **49.85 kB** before gzip).
- Rust library suite: **107/107 passed**, including Lab model/storage/query/mutations, schema-v1 sync isolation, dual-store filenames, module roots, independent gates/sessions, and the Lab workbook contract.
- `cargo test --test shared_sync_flow`: **40/40 passed**.
- `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Automated shell/bridge coverage verifies TE→Lab→TE session isolation, hard deactivation, scoped events, cached Lab rows, correct titles, module IDs on every command, Lab CRUD/query/export contracts, and absence of calibration UI in Lab.
- Single-instance live desktop smoke: preflight found no unified/standalone writer and no listeners on **5173/9222**; Lab opened **Shared** with **1 inventory / 0 archive** and its known row; TE switched in **Shared** with **529 inventory / 13 archive**; ME Storage remained a placeholder; returning to Lab preserved its row and no runtime exception or `Illegal invocation` was observed. The process and both listeners were fully stopped afterward.
- The smoke was intentionally non-mutating on live shared data. Automated tests cover Lab create/update/delete/archive/verify; owner release QA should still add/edit one Lab entry and confirm persistence after restart.

## IM-015 verification — 2026-07-25

- Dedicated TE **Equipment / Calibration** workspace implemented against the same `InventoryEntry` records. Equipment retains calibration due + health; Calibration owns detailed columns, independent search/filter/sort/visibility state, current-state row coloring, focused editing, and explicit manual membership changes.
- Calibration roster desktop flow implemented under `backend/src/integrations/calibration_roster_import/` and `frontend/src/modules/te-test-equipment/components/calibration/`. The general importer was not changed.
- Focused frontend gates: calibration workspace **7/7**, roster dialog **4/4**, and inventory table **9/9** passed.
- Full frontend suite: **21 files passed, 1 skipped; 166 tests passed, 1 skipped**. The existing unrelated React `act(...)` warning in `entry-dialog.test.tsx` remains non-failing.
- `npm run lint` and `npm run build` passed.
- Calibration roster Rust flow: **43/43 passed**. Existing general import regression: **60/60 passed**. Shared sync regression: **40/40 passed**.
- Full Rust suite passed with only the two documented owner-path audits filtered: `inspect_legacy_headers_only` and `audit_live_database_snapshot_aggregates_only`. The performance benchmark remained intentionally ignored.
- `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings` passed.
- Isolated single-instance desktop smoke passed with shared sync disabled against copied data: Equipment/Calibration switching, workspace-specific columns, roster dialog preview-only copy, Calibration-default editor section, and return to Equipment all worked without runtime exceptions. The app and listeners on **5173/9222** were fully stopped afterward.
- Aggregate-only copied-data preview passed without changing copied entry or outbox counts: **97** source rows = **78 matched + 8 create candidates + 3 conflicts + 4 duplicates + 4 junk**; **110** current required entries were absent; pre-resolution prospective required count was **86**; preview remained blocking. Of the eight create candidates, review hints were absent for three, unique for one, and multiple for four. No row values, identifiers, UUIDs, or hashes were documented.
- No live database, live shared root, or source workbook was mutated. Sync is not a backup. Phase 8 requires dated local/shared backups, one writer, a fresh preview, explicit owner resolutions, one commit, publish completion, restart verification, and retained rollback artifacts.

## Next slices

1. Integrate the concurrent feature worktrees into `main`, preserving each plan's ownership boundaries and rerunning frontend tests/lint/build plus Rust tests/clippy after conflict resolution.
2. Perform IM-015 Phase 8 only in an owner-controlled maintenance window: verify the workbook is still current, close every writer, take dated local/shared backups, run a fresh preview, resolve every blocking row, commit once, wait for publish, restart, and verify persistence.
3. Keep the copied-data preview aggregate as rehearsal evidence only; do not treat **8 create candidates**, **4 junk**, **110 absent**, or **86 prospective required** as final live outcomes before fresh owner resolutions.
4. Complete remaining owner QA for live Lab persistence if still outstanding; ME Storage and TE Storage Room remain deferred.
