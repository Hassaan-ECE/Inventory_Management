# Project Folder Breakdown

Last updated: 2026-10-16 (backend regenerated after the 0.2 Phase 4 restructure on `feature/0.2-cleanup`; the frontend sections are refreshed after the Phase 5 file splits)

This document maps the project files to their responsibilities. It covers tracked source, config, docs, scripts, and tests. Generated folders such as `node_modules/`, `backend/target/`, `backend/gen/`, `frontend/dist/`, and `.tmp/` are summarized instead of expanded. Code removed in 0.2 is listed in `REMOVED_AND_FUTURE.md`.

## How the pieces fit

- One Tauri 2 desktop app, one Rust crate, one frontend package.
- The frontend shell shows one module at a time. Every backend command takes a `module_id` (`te-test-equipment`, `te-lab-components`, `te-storage`) and `backend/src/api/commands/` dispatches to that module.
- The backend mirrors the frontend: `platform/` is what every inventory shares, `modules/<inventory>/` is one inventory.
- **One local database per PC** (`inventory-management.feox`, 256 MiB). Each module reads and writes its own namespace (`<module id>/<key>`) through `ModuleStore<R>`, so modules cannot see each other's records.
- **One entry sync engine** (`platform/sync/`) for TE Test Equipment and EVCS Storage. Each plugs in through `SyncRecord`, which carries its constants, field copy, validation, and `MergeRule`. Each rule, and the entry-state fields it serializes, stays exactly as 0.1.4 shipped it, so mixed 0.1.4/0.2 teams converge.
- Each module still has its own shared root on S:, sync stream, and sync gate. TE Lab Components syncs its catalog with its own engine in `catalog_sync.rs`, writing to `shared\catalog-v2\`.

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
| `backend/.cargo/config.toml` | Where `cargo test` writes the generated frontend types (`TS_RS_EXPORT_DIR`), and integers as `number`. |
| `backend/build.rs` | Tauri build script hook. |
| `backend/capabilities/default.json` | Tauri permissions for the main window (events, window geometry, title, updater). |
| `backend/icons/icon.ico` | Application and installer icon. |
| `backend/tauri.conf.json` | Product name, identifier, window, CSP/asset scope, NSIS bundle, updater pubkey and endpoint. |
| `backend/windows/nsis-hooks.nsh` | NSIS post-install hook that (re)creates Start Menu/Desktop shortcuts with the app user model id. |
| `backend/src/main.rs` | Native entry point; calls `inventory_management_lib::run()`. |
| `backend/src/lib.rs` | App setup: resolve paths, open the local database, run per-module startup sync recovery (failures are logged, never block launch), register managed state and all Tauri commands, flush on exit. |
| `backend/src/inventory_stores.rs` | Opens the one local database and each module's store (its namespace of that database); flushes the database on exit. |

## Backend Developer Test Share (test-only)

| Path | Handles |
| --- | --- |
| `backend/src/dev_share/mod.rs` | Builds the developer test share (`bun run dev:share`): three fake computers, the live-share refusal, `DEV-SHARE.md`, and the field-coverage check. |
| `backend/src/dev_share/te_test_equipment.rs` | TE fixtures and timeline: every status and calibration state, limits, unicode, duplicates, verify/archive/delete, merged and disputed edits. |
| `backend/src/dev_share/te_storage.rs` | EVCS Storage fixtures and timeline: filled/empty/hidden columns, quantities, limits, unicode, duplicates, delete, per-field merges. |
| `backend/src/dev_share/te_lab_components.rs` | Lab fixtures and timeline: every area and container type, simple and full parts, every stock status and count state, move, count, edit, delete. |

## Backend Platform

| Path | Handles |
| --- | --- |
| `backend/src/platform/mod.rs` | Platform module root and its shared exports. |
| `backend/src/platform/common.rs` | `CommandResult`, `InventorySharedStatus` and its `SyncWarning`, `ExcelExportStats`, timestamp and id helpers. |
| `backend/src/platform/database.rs` | `LocalDatabase` (the one `inventory-management.feox`) and `Namespace` (one module's view of it); database health (estimated percent used, FeOx write failures). |
| `backend/src/platform/legacy_databases.rs` | One-time merge of the 0.1.x per-module files into the single database, verified key by key; originals move to `pre-0.2-databases\`. |
| `backend/src/platform/dev_build.rs` | Developer builds use `.tmp/dev-data` and `.tmp/dev-share` instead of live data. |
| `backend/src/platform/local_data_paths.rs` | Resolves the local data folder (env override, dev folder, or `%LOCALAPPDATA%`). |
| `backend/src/platform/module_id.rs` | `ModuleId` and its stable string ids. |
| `backend/src/platform/shared_root.rs` | Default shared root per module (team share, or the dev share in dev builds). |
| `backend/src/platform/native.rs` | Native dialogs and openers: picture picker and preview, open path, open external link. |
| `backend/src/platform/runtime/shared_sync.rs` | `SharedSyncCoordinator`: one gate per module for mutations and sync, plus the last background status. |
| `backend/src/platform/runtime/shared_watcher.rs` | Sync sessions and the shared-folder file watcher that triggers syncs. |

## Backend Platform Store

| Path | Handles |
| --- | --- |
| `backend/src/platform/store/mod.rs` | `ModuleStore<R>`, generic over the module's record type; `ModuleRecord` and `StoredEntry`. |
| `backend/src/platform/store/codec.rs` | Raw key/value access: JSON, strings, numbers, prefix scans and deletes. |
| `backend/src/platform/store/keys.rs` | Key layout for entries, metadata, and sync state. |
| `backend/src/platform/store/metadata.rs` | Schema versions, client and device ids, local sequence, sync revision, last snapshot, next display id. |
| `backend/src/platform/store/entries.rs` | Entry records (`entry:<uuid>`) with the display-id index (`entry_id:<id>`). |
| `backend/src/platform/store/sync_state.rs` | Outbox, applied and sequence markers, watermarks, tombstones, entry states, conflicts, corrupt-file records; sync-state backup and restore. |
| `backend/src/platform/store/tests.rs` | Store tests against a small test record type. |

## Backend Platform Sync Engine

| Path | Handles |
| --- | --- |
| `backend/src/platform/sync/mod.rs` | Engine exports and the test-support surface. |
| `backend/src/platform/sync/record.rs` | `SyncRecord` (what an inventory supplies) and `MergeRule` (`DisjointFieldsFromSameBase` for TE, `PerFieldVersions` for EVCS Storage). |
| `backend/src/platform/sync/types.rs` | Operation envelope and payload, entry state (serialized per inventory exactly as 0.1.4), tombstones, conflicts, corrupt-file records, shared paths. |
| `backend/src/platform/sync/apply.rs` | Full sync and background publish; pulling and applying remote operations under the merge rule. |
| `backend/src/platform/sync/queue.rs` | Building and queueing local operations, bootstrap of pre-sync entries, pushing the outbox. |
| `backend/src/platform/sync/conflicts.rs` | Entry-state ordering, stale-operation conflicts, corrupt-file records. |
| `backend/src/platform/sync/recovery.rs` | Startup repair of local sync state after an interrupted write. |
| `backend/src/platform/sync/scanning.rs` | Scanning operation files above the watermarks. |
| `backend/src/platform/sync/operation_file/` | Operation file names, canonical JSON and checksums, validation, reading and writing. |
| `backend/src/platform/sync/snapshot/` | Snapshot publish, apply, locking, I/O, compaction; `restore.rs` is the owner-run TE snapshot restore tool (ignored test). |
| `backend/src/platform/sync/files.rs` | What every shared file shares, for both sync streams: canonical JSON, `sha256:` checksums, SHA-256, and write-to-temp-then-rename writes. |
| `backend/src/platform/sync/auth.rs` | Optional HMAC signing and verification. |
| `backend/src/platform/sync/identity.rs` | Client identity and local sequence helpers. |
| `backend/src/platform/sync/shared_paths.rs` | Shared root resolution and provisioning, sync on/off, status building (also used by the Lab catalog). |
| `backend/src/platform/sync/timestamps.rs` | Timestamp validation and ordering. |
| `backend/src/platform/sync/warning.rs` | What the last sync found (`SyncFindings`) and the footer warning derived from it and database health. |

## Backend API Layer

| Path | Handles |
| --- | --- |
| `backend/src/api/commands/mod.rs` | Command helpers shared by every command file: `module_id` parsing, JSON input and output, background TE publish. |
| `backend/src/api/commands/inventory.rs` | Commands routed by `module_id`: load, sync session (activate, sync, deactivate), and entry create/update/verify/archive/delete for TE and EVCS Storage. |
| `backend/src/api/commands/lab_catalog.rs` | Lab catalog commands: parts, simple components, areas, containers, placements, moves, counts. |
| `backend/src/api/commands/calibration_roster.rs` | TE calibration roster pick, preview, and commit. |
| `backend/src/api/export.rs` | Excel export commands (save dialog, then the module's workbook writer) and workbook tests. |

## Backend Module: TE Test Equipment

| Path | Handles |
| --- | --- |
| `backend/src/modules/te_test_equipment/model.rs` | Entry, input, query, status and result types; normalization and validation; calibration health. |
| `backend/src/modules/te_test_equipment/entry_changes.rs` | Changed-field detection and field-level updates for edits. |
| `backend/src/modules/te_test_equipment/mutations.rs` | Create, update, verify, archive, delete, each queued for sync. |
| `backend/src/modules/te_test_equipment/store/` | The TE store type and calibration roster batch storage. |
| `backend/src/modules/te_test_equipment/sync_record.rs` | TE's `SyncRecord`: constants, field names and copy, `DisjointFieldsFromSameBase`. |
| `backend/src/modules/te_test_equipment/sync_tests/` | End-to-end sync scenarios (conflicts, divergence regression, shared flow, warnings). |
| `backend/src/modules/te_test_equipment/calibration_roster_import/` | Calibration roster workbook preview, reconcile, and commit. |
| `backend/src/modules/te_test_equipment/workbook.rs` and `workbook/` | Equipment workbook export: columns and formats. |
| `backend/src/modules/te_test_equipment/performance_baseline.rs` | Ignored benchmark harness (`cargo test --lib performance_baseline -- --ignored`). |

## Backend Module: TE Lab Components

| Path | Handles |
| --- | --- |
| `backend/src/modules/te_lab_components/catalog_model.rs` | Parts, storage areas and containers, stock placements; validation and summaries. |
| `backend/src/modules/te_lab_components/catalog_mutations.rs` | Catalog create, update, delete, move, and count. |
| `backend/src/modules/te_lab_components/simple_workflow.rs` | The simple one-dialog component workflow on top of the catalog. |
| `backend/src/modules/te_lab_components/catalog_query.rs` | Loading the catalog with derived stock summaries and counts. |
| `backend/src/modules/te_lab_components/catalog_sync/mod.rs` | Catalog sync (`shared\catalog-v2\`): the sync run, publish, and status entry points. Whole-entity newest-wins; the file format is what 0.1.x reads. |
| `backend/src/modules/te_lab_components/catalog_sync/types.rs` | Operation, entity state, tombstone, snapshot, manifest, and cutover-marker shapes. |
| `backend/src/modules/te_lab_components/catalog_sync/outbox.rs` | Queueing local changes, pushing them, counting what is still local, startup recovery. |
| `backend/src/modules/te_lab_components/catalog_sync/operation_files.rs` | Reading, writing, validating, and scanning operation files. |
| `backend/src/modules/te_lab_components/catalog_sync/apply.rs` | Pulling remote operations and applying the newest per entity, with conflict records. |
| `backend/src/modules/te_lab_components/catalog_sync/snapshot.rs` | Publishing snapshots and bootstrapping an empty computer from one. |
| `backend/src/modules/te_lab_components/catalog_sync/share.rs` | Setting up a new share (v1 blocker, cutover marker), recognising a ready one, canonical JSON writes. |
| `backend/src/modules/te_lab_components/catalog_sync/tests.rs` | Two-database sync, dependency-safe deletes, conflicts, and the golden-bytes test that pins the shared file format. |
| `backend/src/modules/te_lab_components/catalog_workbook.rs` | Catalog workbook export. |
| `backend/src/modules/te_lab_components/order_workbook.rs` | Order-request workbook export. |
| `backend/src/modules/te_lab_components/store/` | The Lab store type, catalog records and indexes, schema setup. |

## Backend Module: EVCS Storage (`te_storage`)

| Path | Handles |
| --- | --- |
| `backend/src/modules/te_storage/model.rs` | Entry, input, and result types; validation. |
| `backend/src/modules/te_storage/entry_changes.rs` | Field list, changed-field detection, field-level updates. |
| `backend/src/modules/te_storage/mutations.rs` | Create, update, delete, each queued for sync. |
| `backend/src/modules/te_storage/commands.rs` | Load, sync, and background publish for EVCS Storage. |
| `backend/src/modules/te_storage/store.rs` | The EVCS Storage store type. |
| `backend/src/modules/te_storage/sync_record.rs` | EVCS Storage's `SyncRecord`: constants, field copy, `PerFieldVersions`. |
| `backend/src/modules/te_storage/tests.rs` | Persistence, two-client sync, and crash-recovery tests. |

## Frontend App Shell

| Path | Handles |
| --- | --- |
| `frontend/index.html`, `frontend/public/favicon.ico` | Vite entry and favicon. |
| `frontend/src/app/main.tsx` | React bootstrap, fonts, installs the Tauri bridge (side-effect import), window title. |
| `frontend/src/app/App.tsx` | Renders `InventoryShell`. |
| `frontend/src/app/branding.ts` | Name, version, author, display-name constants. |
| `frontend/src/app/index.css` | Tailwind v4 theme tokens, dark mode, base styles. |
| `frontend/src/shell/InventoryShell.tsx` | Theme state, title, mounts every implemented module (each renders its own header; inactive ones return null). |
| `frontend/src/shell/useShellActiveModule.ts` | Active view id (persisted), mapped to its module. |
| `frontend/src/shell/InventorySystemSwitcher.tsx` | Title-menu switcher between views (TE Equipment, TE Calibration, Lab, ME, Storage). |

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
| `frontend/src/integrations/tauri/tauriInventoryBridge.ts` | Installs `window.inventoryDesktop`: every typed `invoke` call for TE and Lab, updater check/download/install, shared-change listeners. |
| `frontend/src/integrations/tauri/desktop-bridge.d.ts` | The bridge's method signatures, in terms of the generated payload types. |
| `frontend/src/integrations/tauri/generated/` | **Generated, do not edit.** TypeScript for every payload the backend sends or receives (`platform/`, `te-test-equipment/`, `te-lab-components/`, `te-storage/`), written by `cargo test` from the Rust types that derive `ts_rs::TS` (`backend/.cargo/config.toml` sets the folder). Commit the regenerated files with the Rust change. |

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
| `frontend/src/modules/te-storage/storageBridge.ts` | Calls the backend directly through typed `invoke` with `moduleId: "te-storage"`. |
| `frontend/src/modules/te-storage/types.ts` | Columns and DTOs. |

## Frontend Shared UI And Utilities

| Path | Handles |
| --- | --- |
| `frontend/src/shared/components/ui/` | Badge, button, input, textarea, empty state, dropdown menu, scroll region, color-rows button, unsaved-changes dialog. `toggle.tsx` is currently unused. |
| `frontend/src/shared/hooks/useDropdownMenu.ts` | Dropdown open/close/keyboard handling. |
| `frontend/src/shared/lib/externalUrl.ts` | URL safety checks and implicit HTTPS. |
| `frontend/src/shared/lib/floatingMenu.ts` | Viewport-clamped placement for context menus. |
| `frontend/src/shared/lib/utils.ts` | Class-name merge and relative timestamps. |
| `frontend/src/shared/assets/hero.png` | Bundled image asset. |


## Frontend Tests

All tests live in `frontend/tests/` and run with `bun run test`. They cover the adaptive sync controller, calibration health/roster/workspace, columns, entry dialog, external URLs, entry actions, filtering, shell (core, mutations, sync, views/export), table, performance baseline, module registry, status messages, Tauri bridge, Lab shell and simple-component logic, and TE Storage. `inventory-shell/helpers.tsx` and `setup.ts` are shared helpers.

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
