# TE Test Equipment Inventory v0.1 Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver the Principal-approved usable v0.1 desktop inventory with first-class current-state calibration, a real dry-run importer, and safe local-only operations.

**Architecture:** Extend the existing `InventoryEntry` projection through Rust storage/sync/export and the TypeScript bridge/UI. Keep health derived, import reconciliation deterministic, updater disabled, and shared production mode config-gated. Full semantics are in `docs/superpowers/specs/2026-07-13-te-test-equipment-v0.1-design.md`.

**Tech stack:** Tauri 2, Rust 2021, FeOxDB, React 19, TypeScript 6, Vite 8, Tailwind 4, Bun/Vitest.

## Global constraints

- Preserve `com.te.test.equipment.inventory` and `%LOCALAPPDATA%\com.te.test.equipment.inventory\inventory.feox`.
- Workers author tracked content. Boss, Managers, and Reviewers remain read-only.
- No remote, push, deploy, release, production shared-mode enablement, or lab-data deletion.
- Never commit a live workbook or anything under `data/import/` except `.gitkeep`.
- No `CalibrationEvent` store, media vault, UI redesign, table/icon migration, or cutover work.
- Every task uses red/green verification and a focused commit; final claims require independent review and Boss verification.

## Ownership and serialization

| Workstream | Exclusive writable files | Dependency |
|---|---|---|
| Decisions/profile | `docs/planning/DECISIONS.md`, `docs/planning/IMPORT_PROFILE.md` | none |
| Identity/ops | identity/config files listed in Task 2 | decisions/profile |
| Domain | model/query/type files listed in Task 3 | decisions/profile |
| Storage/sync/export | backend files listed in Task 4 | domain |
| Importer backend | importer files and fixtures listed in Task 5 | domain, storage |
| Frontend bridge/UI | frontend files listed in Tasks 6–7 | domain, importer backend |
| Project truth | `README.md`, `docs/SESSION_HANDOFF.md` | all implementation |

Tasks 2 and 3 may run in parallel because ownership does not overlap. Tasks 3–7 are serialized at shared files such as `frontend/src/features/inventory/types.ts`, `backend/src/lib.rs`, `backend/Cargo.toml`, and bridge declarations. Reassign a file before dispatch if a discovered edit crosses ownership.

## Task 1: Close provisional decisions and record blocked profile

**Worker files:** `docs/planning/DECISIONS.md`, create `docs/planning/IMPORT_PROFILE.md`.

**Acceptance:** O-001 through O-007 become Accepted exactly as the design states; O-008 records `profile blocked — no export`, headers/value-shape analysis deferred, and fixture mapping allowed. Record D-016 lineage ME `e092c73` and TE Parts reference `e444389`.

1. Add a failing document assertion command:

   ```powershell
   rg -n "O-001.*Open|O-007.*Open|profile blocked — no export" docs/planning
   ```

   Red evidence: open rows remain and the profile sentence is absent.

2. Edit only the owned files, preserving decision history and explicitly marking current-state calibration as a v1 limitation.
3. Run the green assertion:

   ```powershell
   rg -n "current-state|Reference only|Not required|profile blocked — no export|e092c73|e444389" docs/planning/DECISIONS.md docs/planning/IMPORT_PROFILE.md
   ```

4. Commit: `docs: accept v0.1 calibration defaults`.

## Task 2: Finish identity, Local AppData, updater, and sync gates

**Worker files:** `package.json`, `bun.lock`, `scripts/run-bun.mjs`, `scripts/smoke-sync-one-machine.ps1`, `frontend/index.html`, `frontend/src/app/branding.ts`, `frontend/src/features/inventory/components/InventoryHeader.tsx`, `frontend/src/features/inventory/components/InventoryShell.tsx`, `frontend/src/features/inventory/components/header/UpdateActionButton.tsx`, `frontend/src/features/inventory/components/shell/helpers.ts`, `useDesktopInventory.ts`, `useDesktopUpdates.ts`, `useInventoryEntryMutations.ts`, `frontend/src/integrations/tauri/windowState.ts`, `tauriInventoryBridge.ts`, `desktop-bridge.d.ts`, all affected updater/identity tests, `backend/src/storage/mod.rs`, `backend/src/integrations/deprecated_db_cleanup.rs`, `backend/src/sync/types.rs`, `shared_paths.rs`, `auth.rs`, `backend/src/api/mutations.rs`, `backend/src/lib.rs`, `backend/capabilities/default.json`, `backend/tauri.conf.json`, `backend/Cargo.toml`, `backend/Cargo.lock`.

**Acceptance:** no user-facing ME name; TE preference keys and exact sync keys `TE_TEST_EQUIPMENT_SHARED_SYNC_ENABLED`, `TE_TEST_EQUIPMENT_SHARED_ROOT`, `TE_TEST_EQUIPMENT_SYNC_HMAC_KEY`; updater runtime/config/capabilities/dependencies are removed; Tauri package scripts work through Bun because Node is unavailable; shared stays disabled unless the enabled gate is explicitly true. `InventoryDb::open` uses `app_local_data_dir()`. If the TE identifier's old Roaming database exists and the Local target does not, copy it to Local, retain the source, and test target-present/no-source cases; never inspect, copy, move, or delete ME or TE Parts data.

1. Add failing assertions to existing bridge/shell/backend unit tests for TE copy, preference keys, updater absence, Bun-compatible Tauri scripts, target-missing-only Roaming-to-Local copy, and disabled shared default.
2. Run red tests:

   ```powershell
   bun run test -- frontend/tests/inventory-shell.test.tsx frontend/tests/inventory-shell-views-export.test.tsx frontend/tests/tauri-inventory-bridge.test.ts
   Set-Location backend; cargo test storage::tests deprecated_db_cleanup sync::shared_paths; Set-Location ..
   ```

3. Implement the minimal identity/gate changes and non-destructive TE-only LocalAppData copy. Do not create a TE release endpoint or signing key.
4. Run green tests and residue scan:

   ```powershell
   bun run test -- frontend/tests/inventory-shell.test.tsx frontend/tests/inventory-shell-views-export.test.tsx frontend/tests/tauri-inventory-bridge.test.ts
   Set-Location backend; cargo test storage::tests deprecated_db_cleanup sync::shared_paths; Set-Location ..
   rg -n "ME Inventory|com\.me\.inventory|ME_LAB|ME_Inventory|ME_Inventory_App_Tauri_v2|meInventory" frontend backend package.json scripts
   ```

   Green evidence: tests pass and residue scan has no product-identity hits outside deliberately named migration compatibility tests.

5. Commit: `fix: isolate TE application identity`.

## Task 3: Add calibration and verification domain

**Worker files:** `frontend/src/features/inventory/types.ts`, `frontend/src/features/inventory/lib/calibrationHealth.ts`, `frontend/tests/calibration-health.test.ts`, `backend/src/domain/model.rs`, `backend/src/domain/query.rs`, `backend/src/domain/entry_changes.rs`.

**Interfaces produced:** TypeScript `CalibrationRequirement`, `CalibrationHealth`, expanded `InventoryEntry`/`InventoryEntryInput`, and `deriveCalibrationHealth(entry, localDate, dueSoonDays = 30)`. Rust `CalibrationRequirement`, `CalibrationHealth`, expanded entry/input structs, `derive_calibration_health(entry, local_date, due_soon_days)`, and date validation.

1. Write boundary tests for exact precedence, due yesterday/today/+30/+31, required missing due, interval non-authority, impossible date order, and migration round trips: legacy true uses valid `updatedAt` only with the explicit approximate `verifiedBy` label, while invalid/missing `updatedAt` leaves `verifiedAt` empty.
2. Run red tests:

   ```powershell
   bun run test -- frontend/tests/calibration-health.test.ts
   Set-Location backend; cargo test domain::model::tests domain::query::tests; Set-Location ..
   ```

3. Implement the interfaces and field normalization/change detection. `verifiedAt` is optional RFC 3339; calibration dates are `YYYY-MM-DD`.
4. Re-run the same commands for green evidence, then run:

   ```powershell
   bun run build
   Set-Location backend; cargo check; Set-Location ..
   ```

5. Commit: `feat: add calibration domain model`.

## Task 4: Wire storage, sync, snapshots, conflicts, and export

**Worker files:** `backend/src/storage/codec.rs`, `backend/src/storage/entries.rs`, `backend/src/storage/tests.rs`, `backend/src/sync/types.rs`, `backend/src/sync/apply.rs`, `backend/src/sync/queue.rs`, `backend/src/sync/snapshot/types.rs`, sync fixture/integration tests under `backend/tests/`, `backend/src/integrations/export/workbook.rs`, `backend/src/integrations/export/workbook/columns.rs`, `backend/src/integrations/export/mod.rs`.

**Acceptance:** calibration/verification round-trip; sync changed-field normalization and disjoint merge; same-field conflict; snapshot bootstrap; export includes stored calibration, derived health, and verified timestamp; sync schema version increments. A merged entry is revalidated before persistence, and a disjoint merge that creates an invalid date combination is routed to conflict handling.

1. Add failing round-trip, merge/conflict, invalid merged-date combination, snapshot, and workbook-contract tests.
2. Run red tests:

   ```powershell
   Set-Location backend
   cargo test storage::tests
   cargo test --test sync_conflict_flow
   cargo test --test shared_sync_flow
   cargo test integrations::export::tests
   Set-Location ..
   ```

3. Extend serialization and field maps without storing derived health; validate the merged entry before `put_entry` and record/refuse invalid concurrent combinations through the existing conflict path.
4. Re-run all commands for green evidence and inspect one canonical operation/snapshot fixture for the new fields.
5. Commit: `feat: persist and sync calibration fields`.

## Task 5: Implement deterministic Excel/CSV dry run and commit

**Worker files:** `backend/Cargo.toml`, `backend/Cargo.lock`, create `backend/src/integrations/import/mod.rs`, `mapping.rs`, `parser.rs`, `reconcile.rs`, `commit.rs`, create synthetic fixtures under `backend/tests/fixtures/import/`, create `backend/tests/import_flow.rs`, update `backend/src/integrations/mod.rs`, `backend/src/lib.rs`, `backend/src/api/commands.rs`, and importer metadata helpers under `backend/src/storage/`.

**Interfaces produced:** serialized `ImportClassification`; `ImportDryRunReport` with five totals, columns, row outcomes, batch/source fingerprint; `ImportCommitInput { batch_id, confirmed }`; `ImportCommitResult`; Tauri commands `preview_import(path)` and `commit_import(input)`. Commit calls the normal mutation/orchestration layer so every imported change creates its durable outbox operation; importer code never calls `put_entry` directly.

1. Add synthetic CSV/XLSX fixtures for unique asset match, unique serial match, duplicate conflict, identifier disagreement, invalid date, ignored blank row, unknown nonblank column, repeat batch, and injected failure after a durable subset.
2. Write failing parser/reconciliation/idempotence tests and run:

   ```powershell
   Set-Location backend; cargo test --test import_flow; Set-Location ..
   ```

3. Add only parsing dependencies required for `.xlsx`, `.xls`, and `.csv`; implement normalization, five-way accounting, content-based batch identity, provenance, and guarded commit through normal mutations/durable outbox. Persist per-row completion so partial failure followed by rerun finishes once without duplicate entries or operations.
4. Run green tests, then prove no real data is tracked:

   ```powershell
   Set-Location backend; cargo test --test import_flow; cargo check; Set-Location ..
   git ls-files data/import backend/tests/fixtures/import
   ```

5. Commit: `feat: add guarded inventory importer`.

## Task 6: Add importer bridge and guarded UI

**Worker files:** `frontend/src/features/inventory/types.ts`, `frontend/src/integrations/tauri/desktop-bridge.d.ts`, `frontend/src/integrations/tauri/bridgeGuards.ts`, `frontend/src/integrations/tauri/tauriInventoryBridge.ts`, create `frontend/src/features/inventory/components/ImportDialog.tsx`, update `InventoryHeader.tsx` and `InventoryShell.tsx`, create/update focused frontend tests.

**Acceptance:** picker/preview renders batch, column accounting, all five totals, row issues, provenance; commit button requires confirmation and is disabled for unresolved conflicts/rejections or stale preview.

1. Write failing bridge-guard and dialog tests with malformed results and each classification.
2. Run red tests:

   ```powershell
   bun run test -- frontend/tests/tauri-inventory-bridge.test.ts frontend/tests/import-dialog.test.tsx
   ```

3. Implement typed bridge methods and the minimal existing-shell dialog.
4. Re-run for green evidence and run `bun run build`.
5. Commit: `feat: add import dry-run workflow`.

## Task 7: Add calibration editing, table, filters, sorting, and counts

**Worker files:** `frontend/src/features/inventory/components/EntryDialog.tsx`, entry-dialog helpers, `InventoryTable.tsx`, table body/header files, `FilterPanel.tsx`, `StatusStrip.tsx`, shell view-model/helpers, `frontend/src/features/inventory/lib/filtering.ts`, `sorting.ts`, `counts.ts`, `columns.ts`, and focused tests under `frontend/tests/`.

**Acceptance:** all design fields edit safely; interval only suggests; verify sets timestamp; health/due columns and accessible badges; requirement/health/due filters; health/due sorts; four count chips; archive excluded from active calibration counts.

1. Add failing entry-dialog, filtering, sorting, counts, table, and mutation tests.
2. Run red tests:

   ```powershell
   bun run test -- frontend/tests/entry-dialog.test.tsx frontend/tests/inventory-filtering.test.ts frontend/tests/inventory-table.test.tsx frontend/tests/inventory-shell-mutations.test.tsx
   ```

3. Implement within the existing shell and reuse the domain health helper.
4. Re-run for green evidence, then `bun run lint` and `bun run build`.
5. Commit: `feat: surface calibration health in inventory UI`.

## Task 8: Reconcile project truth and run integrated gates

**Worker files:** `README.md`, `docs/SESSION_HANDOFF.md`; production corrections discovered by review return to the owning Worker.

1. Update current state, identity, importer limitation, shared gate, current-state-only calibration, and `sync ≠ backup` language.
2. Run the full gate:

   ```powershell
   bun install
   bun run lint
   bun run test
   bun run build
   Set-Location backend
   cargo fmt --check
   cargo test
   cargo check
   cargo clippy --all-targets -- -D warnings
   Set-Location ..
   bun run tauri build --debug --no-bundle
   ```

3. Run residue/data checks:

   ```powershell
   rg -n "ME Inventory|com\.me\.inventory|ME_LAB|ME_Inventory_App_Tauri_v2|meInventory" frontend backend package.json README.md docs/SESSION_HANDOFF.md
   Get-ChildItem data/import -Force | Select-Object Name,PSIsContainer,Length
   git status --short
   ```

4. Commit: `docs: update v0.1 operational truth`.
5. Dispatch a read-only Reviewer independent of every final-patch author. Route corrections to Workers, rerun affected and full gates, then have the Boss verify A–G, diff scope, no remote/push/deploy/delete, limitations, and exact command evidence.

## A–G completion matrix

| Area | Required task evidence |
|---|---|
| A | Task 2 identity/updater residue scan and tests |
| B | Task 1 accepted decisions/profile record |
| C | Tasks 3–4 domain/storage/sync/export tests |
| D | Tasks 5–6 importer reconciliation/idempotence/UI tests |
| E | Task 7 shell/dialog/filter/sort/count tests |
| F | Tasks 2 and 8 shared gate, docs, sync-not-backup |
| G | Task 8 full commands, independent review, Boss verification, limitations |
