# Project Folder Breakdown

Last updated: 2026-10-15 (regenerated from the working tree at `7d81e17`, package `0.1.3`)

This document maps the project files to their responsibilities. It covers tracked source, config, docs, scripts, and tests. Generated folders such as `node_modules/`, `backend/target/`, `backend/gen/`, `frontend/dist/`, and `.tmp/` are summarized instead of expanded.

## How the pieces fit

- One Tauri 2 desktop app, one Rust crate, one frontend package.
- The frontend shell shows one module at a time. Every backend command takes a `module_id` (`te-test-equipment`, `te-lab-components`, `te-storage`) and `backend/src/api/commands.rs` dispatches to that module's store and sync code.
- Each module has its own FeOxDB file, shared root on S:, sync stream, and per-module sync gate (`runtime/shared_sync.rs`). Only the coordinator, the file watcher, and the frontend `AdaptiveSyncController` are shared.
- The operation-log sync engine exists as **three near-identical copies**: `backend/src/sync/` (TE Test Equipment), `backend/src/modules/te_lab_components/sync/` (Lab legacy v1), and `backend/src/modules/te_storage/sync/` (TE Storage). The same applies to the `storage/` folders. A fix in one copy must be applied to the others deliberately.
- TE Lab Components' current data model is the v2 catalog (`catalog_*.rs`), which has its **own** sync engine in `catalog_sync.rs` writing to `shared\catalog-v2\`.

## Top-Level Files

| Path | Handles |
| --- | --- |
| `.gitignore` | Ignore rules for generated output, local state, build artifacts, release staging, and private workbooks under `data/`. |
| `AGENTS.md` | Short agent rules: identity, product shape, code layout, env vars, verification expectations. |
| `README.md` | Human project entry: modules, product share layout, identity, env vars, development commands. |
| `bun.lock` | Locked frontend dependency graph. |
| `eslint.config.js` | ESLint configuration for frontend TypeScript and React code. |
| `package.json` | Workspace scripts, frontend dependencies, app version, package manager declaration. |
| `tsconfig.json` | Root TypeScript project references for app, tests, and Vite config. |
| `data/import/` | Local, gitignored source workbooks used only for import/roster dry runs. Never commit their contents. |

## Backend App Shell

| Path | Handles |
| --- | --- |
| `backend/Cargo.toml` / `Cargo.lock` | Crate metadata, dependencies, version (keep in sync with `package.json` and `tauri.conf.json`). |
| `backend/build.rs` | Tauri build script hook. |
| `backend/capabilities/default.json` | Tauri permissions for the main window (events, window geometry, title, updater). |
| `backend/icons/icon.ico` | Application and installer icon. |
| `backend/tauri.conf.json` | Product name, identifier, window, CSP/asset scope, NSIS bundle, updater pubkey and endpoint. |
| `backend/windows/nsis-hooks.nsh` | NSIS post-install hook that (re)creates Start Menu/Desktop shortcuts with the app user model id. |
| `backend/src/main.rs` | Native entry point; calls `inventory_management_lib::run()`. |
| `backend/src/lib.rs` | App setup: resolve paths, open the three stores, quarantine deprecated DBs, run per-module startup sync recovery (failures are logged, never block launch), register managed state and all Tauri commands, flush stores on exit. |
| `backend/src/inventory_stores.rs` | Owns the three isolated FeOxDB handles (`inventory.feox`, `te-lab-components.feox`, `te-storage-room.feox`) and flushes them together. |

## Backend Platform

| Path | Handles |
| --- | --- |
| `backend/src/platform/module_id.rs` | `ModuleId` enum and the stable string ids used by commands and events. |
| `backend/src/platform/shared_root.rs` | Default S: shared roots per module, plus unused pre-cutover `InventoryApps` pilot constants. Edit here if the product share moves. |
| `backend/src/platform/local_data_paths.rs` | Resolves Local/Roaming/Cache app dirs, or the isolated `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` override (must be absolute). |

## Backend API Layer

| Path | Handles |
| --- | --- |
| `backend/src/api/commands.rs` | All Tauri commands. Parses `module_id`, routes to TE / Lab / Storage, wraps writes in the module sync gate, and schedules background shared publishes after mutations. |
| `backend/src/api/mutations.rs` | TE Test Equipment create/import/update/verify/archive/delete with outbox queueing and rollback on error. |

## Backend Runtime

| Path | Handles |
| --- | --- |
| `backend/src/runtime/shared_sync.rs` | `SharedSyncCoordinator`: one mutex per module serializing mutations, sync, and publish; caches the last background shared status per module. |
| `backend/src/runtime/shared_watcher.rs` | Per-module sync sessions (opaque tokens), file watching of each module's ops folder, debounced `inventory:shared-changed` events, degraded-watcher reporting. |

## Backend Domain (TE Test Equipment)

| Path | Handles |
| --- | --- |
| `backend/src/domain/model.rs` | TE entry types, calibration requirement/health (`derive_calibration_health`, 30-day due-soon window), validation, normalization, filter/sort/query DTOs, shared-status DTOs. |
| `backend/src/domain/query.rs` | Backend filtering, sorting, paging, search, scope, and counts for TE. |
| `backend/src/domain/entry_changes.rs` | Field-level edit context: base versions, changed-field normalization, partial update projection. |

## Backend Storage (TE Test Equipment)

| Path | Handles |
| --- | --- |
| `backend/src/storage/mod.rs` | `InventoryDb` wrapper around FeOxDB, open helpers, copies a legacy Roaming `inventory.feox` into Local AppData when no local DB exists yet, flush. |
| `backend/src/storage/codec.rs` | JSON encode/decode helpers and metadata decoding. |
| `backend/src/storage/entries.rs` | Entry CRUD, UUID/numeric-id lookup, id index, scans, snapshot replacement. |
| `backend/src/storage/imports.rs` | Stored import preview batches and entry lookups used by import commit. |
| `backend/src/storage/keys.rs` | Key prefixes, sync key formatting/parsing, path segment validation, sequence formatting. |
| `backend/src/storage/metadata.rs` | Next id, sync schema, client/device identity, local sequence, revision, snapshot id. |
| `backend/src/storage/sync_state.rs` | Outbox, applied markers, watermarks, tombstones, entry states, conflicts, corrupt-file records. |
| `backend/src/storage/tests.rs` | Storage unit tests. |

## Backend Shared Sync (TE Test Equipment; template for the other copies)

Shared layout: `<shared root>\shared\inventory\{manifest.json, ops\<client>\*.op.json, snapshots\, locks\snapshot.lock, backups\}`.

| Path | Handles |
| --- | --- |
| `backend/src/sync/mod.rs` | Module wiring and exports. |
| `backend/src/sync/types.rs` | Env var names, sync schema version, operation envelopes, tombstones, entry state, conflicts, `SharedSyncPaths`. |
| `backend/src/sync/apply.rs` | Sync loop: push local ops, apply latest snapshot if no local-only changes are pending, pull newer remote ops, merge disjoint field edits, apply deletes/upserts, advance watermarks. |
| `backend/src/sync/auth.rs` | Optional HMAC signing/verification (`INVENTORY_MANAGEMENT_SYNC_HMAC_KEY`, ≥16 bytes). |
| `backend/src/sync/conflicts.rs` | Entry-state lookup, newer-wins comparison, stale conflict recording, corrupt remote markers. |
| `backend/src/sync/identity.rs` | Persistent client/device identity and local sequence allocation. |
| `backend/src/sync/queue.rs` | Building local operations, outbox persistence, bootstrap ops, tombstones, pushing pending ops. |
| `backend/src/sync/recovery.rs` | Startup repair (see `SYNC_RECOVERY_INVARIANTS.md`). Also holds an ignored, env-gated test that **repairs the live** TE DB for 0.1.0 installs (`REPAIR_TE_LEGACY=1`). |
| `backend/src/sync/scanning.rs` | Scanning op files after watermarks, skipping temp/unknown files, deterministic ordering. |
| `backend/src/sync/shared_paths.rs` | Shared root resolution, enable flag, folder creation, user-facing shared status. |
| `backend/src/sync/timestamps.rs` | UTC RFC3339 validation and comparison. |
| `backend/src/sync/operation_file/{mod,canonical,paths,validation}.rs` | Atomic op writes/reads, canonical JSON + SHA-256 checksums, file naming, envelope validation. |
| `backend/src/sync/snapshot/{types,apply,io,lock,publish}.rs` | Snapshot/manifest schema; publish after 1,000 ops or 24 h; keep 3 snapshots; compaction of covered ops; 10-minute stale lock; pending-apply marker. |

## Backend Integrations

| Path | Handles |
| --- | --- |
| `backend/src/integrations/deprecated_db_cleanup.rs` | One-time quarantine of legacy SQLite `.db` files into a backups folder. |
| `backend/src/integrations/native.rs` | Safe external URL / local path opening, picture picker, picture preview cache. |
| `backend/src/integrations/export/mod.rs` | `export_excel` (per module) and `export_lab_order_request` commands, save dialog, result stats. |
| `backend/src/integrations/export/workbook.rs` + `workbook/{columns,formats}.rs` | TE Inventory/Archive workbook. |
| `backend/src/integrations/export/catalog_workbook.rs` | Lab v2 six-sheet catalog/audit workbook. |
| `backend/src/integrations/export/lab_order_workbook.rs` | Lab "Order Request" workbook built from selected parts and requested quantities. |
| `backend/src/integrations/export/lab_workbook.rs` + `lab_workbook/{columns,formats}.rs` | Legacy Lab v1 inventory workbook (pre-catalog databases). |
| `backend/src/integrations/inventory_import/{mod,parser,mapping,reconcile,commit}.rs` | TE Excel/CSV import: parse, map headers, dry-run reconcile, full-batch commit. **No UI calls this today** (the Import button is intentionally absent; see `frontend/tests/inventory-shell-import.test.tsx`). |
| `backend/src/integrations/calibration_roster_import/{mod,types,parser,reconcile,commit}.rs` | IM-015 calibration roster preview/review/commit, used by the Calibration workspace. |

## Backend Module: TE Lab Components

| Path | Handles |
| --- | --- |
| `backend/src/modules/te_lab_components/catalog_model.rs` | v2 catalog types: Part, StorageArea, StorageContainer, StockPlacement, attributes, validation, `CATALOG_SCHEMA_VERSION`. |
| `backend/src/modules/te_lab_components/catalog_mutations.rs` | Part/area/container/placement CRUD, move and count stock. |
| `backend/src/modules/te_lab_components/catalog_query.rs` | Loads the catalog with per-part stock summaries; optional demo seed (`INVENTORY_MANAGEMENT_LAB_DEMO_SEED=1`). |
| `backend/src/modules/te_lab_components/catalog_migration.rs` | Legacy v1 → v2 migration status, preview, and commit. |
| `backend/src/modules/te_lab_components/catalog_sync.rs` | Separate v2 sync engine under `shared\catalog-v2\`, shared-root cutover preview/commit, legacy-stream blocker. Holds an ignored, env-gated test that **edits the live** Lab DB (`REPAIR_LAB_OUTBOX=1`). |
| `backend/src/modules/te_lab_components/catalog_demo_seed.rs` | Demo parts for empty catalogs (opt-in only). |
| `backend/src/modules/te_lab_components/simple_workflow.rs` | "Simple" component create/update: one Part plus one shelf placement written atomically with rollback. |
| `backend/src/modules/te_lab_components/{model,mutations,query,entry_changes}.rs` | Legacy v1 inventory-entry model, used for migration sources and pre-v2 databases. Commands reject it once the DB is on catalog v2. |
| `backend/src/modules/te_lab_components/storage/` | Copy of the TE storage layer for the Lab DB, plus `catalog.rs` (v2 catalog keyspaces). |
| `backend/src/modules/te_lab_components/sync/` | Copy of the TE sync engine for the legacy v1 Lab stream (`shared\inventory\`). Used for startup recovery of pre-v2 databases. |

## Backend Module: EVCS Storage (`te_storage`)

| Path | Handles |
| --- | --- |
| `backend/src/modules/te_storage/model.rs` | Storage entry (PN/PR/PO/manufacturer/model/description/qty/location/notes), inputs, shared status. |
| `backend/src/modules/te_storage/mutations.rs` | Create/update/delete with outbox rollback. |
| `backend/src/modules/te_storage/commands.rs` | Load, synchronize, and background publish helpers called from `api/commands.rs`. |
| `backend/src/modules/te_storage/entry_changes.rs` | Field-level edit context for Storage. |
| `backend/src/modules/te_storage/storage/` | Copy of the TE storage layer for `te-storage-room.feox`. |
| `backend/src/modules/te_storage/sync/` | Copy of the TE sync engine (sync schema v1). `shared_paths.rs` creates only the module folder, and only under an already reachable product `modules\` folder. |
| `backend/src/modules/te_storage/tests.rs` | Storage module tests. |

## Backend Tests

| Path | Handles |
| --- | --- |
| `backend/tests/shared_sync_flow.rs` | End-to-end TE sync: bootstrap, snapshots, HMAC, CRUD, watermarks, revision. |
| `backend/tests/sync_conflict_flow.rs` | Duplicates, tombstones, newer-wins, field merge, delete/restore, stale ops. |
| `backend/tests/sync_core.rs` | Root resolution, identity, checksums, op files, HMAC, timestamps, recovery, watermarks. |
| `backend/tests/import_flow.rs` | TE import dry-run/commit. Contains an opt-in one-shot that writes the **live** Local AppData DB (`TE_LOAD_LIVE_LOCAL=1`). |
| `backend/tests/calibration_roster_flow.rs` | Calibration roster preview/commit flows. |
| `backend/tests/live_db_audit.rs` | Opt-in read path from the live product shared root into an empty temp DB. |
| `backend/tests/performance_baseline.rs` | Ignored performance baseline. |
| `backend/tests/support/{backend,core_backend,sync_fixtures}.rs` | Module bridges and fixtures for integration tests. |
| `backend/tests/fixtures/te_lab_components/*.json` | Lab v1 → v2 migration fixtures. |

## Frontend App Shell

| Path | Handles |
| --- | --- |
| `frontend/index.html`, `frontend/public/favicon.ico` | Vite entry and favicon. |
| `frontend/src/app/main.tsx` | React bootstrap, fonts, installs the Tauri bridge (side-effect import), window title. |
| `frontend/src/app/App.tsx` | Renders `InventoryShell`. |
| `frontend/src/app/branding.ts` | Name, version, author, display-name constants. |
| `frontend/src/app/index.css` | Tailwind v4 theme tokens, dark mode, base styles. |
| `frontend/src/shell/InventoryShell.tsx` | Theme state, title, mounts every implemented module (each renders its own header; inactive ones return null) and the placeholder chrome. |
| `frontend/src/shell/useShellActiveModule.ts` | Active view id (persisted), mapped to its module. |
| `frontend/src/shell/InventorySystemSwitcher.tsx` | Title-menu switcher between views (TE Equipment, TE Calibration, Lab, ME, Storage). |
| `frontend/src/shell/ShellHeader.tsx`, `ShellStatusStrip.tsx`, `InventorySystemPlaceholder.tsx` | Header, status strip, and body used only for placeholder modules (ME Storage). |

## Frontend Platform

| Path | Handles |
| --- | --- |
| `frontend/src/platform/modules/types.ts` | Module/view ids, module host types, props passed to module views. |
| `frontend/src/platform/modules/registry.ts` | List of module hosts and views (Calibration is a second view of TE). |
| `frontend/src/platform/modules/persistence.ts` | Stores the active view in `localStorage` and migrates old ids. |
| `frontend/src/platform/sync/adaptiveSyncController.ts` | Completion-aware polling: 2 s when focused and active, 60 s when idle (2 min), unfocused or hidden; immediate sync on focus/visibility/activity. Used by TE, Lab, and Storage. |
| `frontend/src/platform/ui/theme.ts` | Theme storage key and reader. |

## Frontend Tauri Integration

| Path | Handles |
| --- | --- |
| `frontend/src/integrations/tauri/tauriInventoryBridge.ts` | Installs `window.inventoryDesktop`: every `invoke` call, updater check/download/install, shared-change listeners. |
| `frontend/src/integrations/tauri/desktop-bridge.d.ts` | Bridge type declarations per module. |
| `frontend/src/integrations/tauri/bridgeGuards.ts` | Runtime validation of TE/import/roster payloads from the backend. |
| `frontend/src/integrations/tauri/labBridgeGuards.ts` | Runtime validation of Lab catalog payloads. |
| `frontend/src/integrations/tauri/windowState.ts` | Window geometry save/restore helpers. **Not installed**: restore was disabled on purpose because it broke layout on some monitors (see comment in `tauriInventoryBridge.ts`). Kept with tests. |

## Frontend Module: TE Test Equipment

| Path | Handles |
| --- | --- |
| `frontend/src/modules/te-test-equipment/index.ts`, `moduleId.ts` | Module definition/host and id. |
| `frontend/src/modules/te-test-equipment/TeTestEquipmentView.tsx` | Module root for both the Equipment and Calibration views. |
| `frontend/src/modules/te-test-equipment/types.ts` | Entry, calibration, filter, column, and option types. |
| `frontend/src/modules/te-test-equipment/lib/` | Columns per workspace, counts, filtering, sorting, result labels, `calibrationHealth.ts` (30-day due-soon). |
| `frontend/src/modules/te-test-equipment/components/` | Header, search card, filter panel, virtualized table (`table/`), entry dialog (`entry-dialog/`), context menu, status strip, export menu (`header/`), updater button. |
| `frontend/src/modules/te-test-equipment/components/shell/` | Hooks: desktop load/sync lifecycle, mutations, export, external links, preferences, view model, status announcer, updates. |
| `frontend/src/modules/te-test-equipment/components/calibration/` | Calibration membership dialog and roster preview/review/commit dialog. |
| `frontend/src/modules/te-test-equipment/data/mockInventory.ts` | Browser-only mock rows. |

Export menu note: the **HTML** item only announces "HTML export is not implemented yet." Excel export works.

## Frontend Module: TE Lab Components

| Path | Handles |
| --- | --- |
| `frontend/src/modules/te-lab-components/index.ts`, `moduleId.ts`, `types.ts` | Module definition/host, id, and shared Lab types. |
| `frontend/src/modules/te-lab-components/TeLabComponentsView.tsx` | Module root: five-column simple workspace, dialogs, order selection, migration and cutover panels. |
| `frontend/src/modules/te-lab-components/catalog/useLabCatalog.ts` | Desktop load/sync lifecycle for the v2 catalog. |
| `frontend/src/modules/te-lab-components/catalog/` | Header, search card, parts table, part dialog (simple + advanced), simple fields, grid bin picker, placement/move/count dialogs, location manager, order export, migration panel, shared cutover dialog, column/filter/sort/template helpers, mock catalog. |
| `frontend/src/modules/te-lab-components/components/` | Status strip, export/update header pieces, and the status announcer and updater hooks are still used by the catalog view. |

Not loaded by the app (about 3,250 lines): the old Lab v1 table UI under `components/` (`InventoryTable`, `InventoryHeader`, `SearchCard`, `FilterPanel`, `EntryDialog` and `entry-dialog/*`, `EntryContextMenu`, `EmptyResults`, `ViewSettingsMenu`, `header/ScopeToggle`, `table/*`, `shell/` files other than `helpers.ts`, `useDesktopUpdates.ts`, and `useStatusAnnouncer.ts`), plus `data/mockInventory.ts`. These are candidates for deletion.

## Frontend Module: EVCS Storage (`te-storage`)

| Path | Handles |
| --- | --- |
| `frontend/src/modules/te-storage/index.ts` | Module definition/host (`te-storage`, shared folder `TE_Storage_Room`). |
| `frontend/src/modules/te-storage/TeStorageView.tsx` | Table, client-side search and sort, header right-click / Shift+F10 column picker, persisted column visibility (PR # and Description hidden by default). |
| `frontend/src/modules/te-storage/StorageEntryDialog.tsx` | Add/edit with unsaved-changes guard and confirmed delete. |
| `frontend/src/modules/te-storage/StorageColumnMenu.tsx` | Column visibility menu. |
| `frontend/src/modules/te-storage/useStorageInventory.ts` | Load + adaptive sync lifecycle. |
| `frontend/src/modules/te-storage/storageBridge.ts` | Calls the backend directly through `invoke` with `moduleId: "te-storage"`. |
| `frontend/src/modules/te-storage/types.ts` | Columns and DTOs. |

## Frontend Module: ME Storage

| Path | Handles |
| --- | --- |
| `frontend/src/modules/me-storage/index.ts` | Placeholder host only. |

## Frontend Shared UI And Utilities

| Path | Handles |
| --- | --- |
| `frontend/src/shared/components/ui/` | Badge, button, input, textarea, empty state, dropdown menu, scroll region, color-rows button, unsaved-changes dialog. `toggle.tsx` is currently unused. |
| `frontend/src/shared/hooks/useDropdownMenu.ts` | Dropdown open/close/keyboard handling. |
| `frontend/src/shared/lib/externalUrl.ts` | URL safety checks and implicit HTTPS. |
| `frontend/src/shared/lib/floatingMenu.ts` | Viewport-clamped placement for context menus. |
| `frontend/src/shared/lib/utils.ts` | Class-name merge and relative timestamps. |
| `frontend/src/shared/assets/hero.png` | Bundled image asset. |

Also currently unused in TE: `components/ImportDialog.tsx` (intentionally not mounted), `components/ColumnMenu.tsx`, `components/ViewSettingsMenu.tsx`, `components/entry-dialog/useMediaQuery.ts`.

## Frontend Tests

All tests live in `frontend/tests/` and run with `bun run test`. They cover the adaptive sync controller, calibration health/roster/workspace, columns, entry dialog, external URLs, import dialog (component only), entry actions, filtering, shell (core, import absence, mutations, sync, views/export), table, performance baseline, module registry, status messages, Tauri bridge, window state, Lab shell and simple-component logic, and TE Storage. `inventory-shell/helpers.tsx` and `setup.ts` are shared helpers.

## Frontend Tooling

| Path | Handles |
| --- | --- |
| `frontend/vite.config.ts` | Vite + React + Tailwind, `@/` alias, Vitest environment. |
| `frontend/tsconfig.{app,node,tests}.json` | TypeScript configs. |

## Scripts

| Path | Handles |
| --- | --- |
| `scripts/vite-dev.ts` (+ `vite-dev.test.ts`) | Starts the Vite dev server on `127.0.0.1:5173` (strict port) for `bun run dev:frontend` / Tauri dev. |
| `scripts/run-bun.mjs` | Resolves the real Bun binary when a stale PATH shim is present. |
| `scripts/smoke-sync-one-machine.ps1` | One-machine shared sync smoke against an isolated root (`INVENTORY_MANAGEMENT_SYNC_SMOKE_ROOT`). |
| `scripts/release/copy-shared-to-product-modules.ps1` | One-time 2026-07-20 copy of TE + Lab shared trees from `InventoryApps` pilots into product `modules\` folders (robocopy, no deletes). |

## Docs

| Path | Handles |
| --- | --- |
| `docs/README.md` | Docs index and trust rules. |
| `docs/SESSION_HANDOFF.md`, `SESSION_START_PROMPT.md`, `CHAT_HANDOFF.md` | Current state and new-chat prompts. |
| `docs/planning/DECISIONS.md` | IM-* decision register. |
| `docs/planning/IMPORT_PROFILE.md` | Import and calibration roster dry-run aggregates. |
| `docs/engineering/UPDATER_AND_RELEASE.md` | Signing keys, GitHub release layout, version bump checklist. |
| `docs/engineering/FEOXDB_SYNC_MIGRATION_PLAN.md` | Compact sync design note (layouts per module, behavior, thresholds). |
| `docs/engineering/SYNC_RECOVERY_INVARIANTS.md` | Startup recovery rules. |
| `docs/engineering/AGENT_RUNBOOK.md` | Build/release traps and pivots. |
| `docs/engineering/PROJECT_FOLDER_BREAKDOWN.md` | This file. |
| `docs/engineering/archive/` | Historical evidence from the ME-lineage app; not current instructions. |
| `docs/performance/PERFORMANCE_BASELINE.md` | Baseline notes. |
| `docs/releases/` | Per-version release notes. |
| `docs/runbooks/` | Lab catalog v2 migration and combined copied-data desktop smoke. |
| `docs/superpowers/{specs,plans}/` | Dated design specs and implementation plans (history and behavior authority, not open work unless stated). |

## Generated Or Local-Only Folders

| Path | Handles |
| --- | --- |
| `.git/` | Repository metadata. |
| `.tmp/` | Scratch area, including release staging (`.tmp/release-X.Y.Z/`) and smoke data. Non-canonical. |
| `.superpowers/` | Local brainstorming tool state. |
| `backend/target/` | Rust build output; release installers land in `backend/target/release/bundle/nsis/`. |
| `backend/gen/` | Tauri-generated capability schemas. |
| `frontend/dist/` | Built frontend. |
| `node_modules/` | Installed JS dependencies. |
