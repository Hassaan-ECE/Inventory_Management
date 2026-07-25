# TE Test Equipment Calibration Workspace + Workbook Cutover — Implementation Plan

- **Decision:** IM-015
- **Status:** Implementation and pre-cutover verification complete; controlled live cutover pending
- **Approved:** 2026-07-24
- **Runtime baseline:** Inventory Management `0.1.0`; TE Test Equipment and TE Lab Components implemented; IM-015 code is implemented and verified in the isolated `feature/im-015-calibration` worktree; no live roster migration has been applied.

> This document is the implementation and verification authority for IM-015. Do not mark a phase complete without the listed tests/evidence, and do not treat planned behavior as current runtime behavior until the acceptance criteria are verified.

## Goal

Add a dedicated **Calibration** workspace inside **TE Test Equipment** that is a second projection of the existing equipment records, not a duplicate database/table. Equipment and calibration edits therefore update the same `InventoryEntry` and appear immediately in both views and through the existing TE shared-sync stream.

Use the current calibration workbook as the one-time baseline for calibration membership, explicitly resolve every unmatched/conflicted/duplicate/junk row, mark approved workbook records verified at commit time, and remove workbook-absent active records from the calibration group without deleting their prior calibration details. The copied-data rehearsal does not hardcode the final live create/ignore counts; those require a fresh owner-reviewed preview.

## Implementation Tracker

- [x] Phase 1 — Calibration workspace UI
- [x] Phase 2 — Focused editing and manual membership
- [x] Phase 3 — Isolated calibration roster reconciliation backend
- [x] Phase 4 — Calibration roster preview/review wizard
- [x] Phase 5 — Frontend and backend automated coverage
- [x] Phase 6 — Decision, handoff, and operator documentation updates
- [x] Phase 7 — Full lint/test/build/clippy/desktop verification
- [ ] Phase 8 — Backed-up, single-writer live workbook cutover
- [ ] Final acceptance criteria verified and evidence recorded

## Confirmed Product Decisions

- Calibration lives inside the existing **TE Test Equipment** module.
- The header gains an **Equipment / Calibration** workspace switch; the top-level app module switcher does not change.
- The Equipment table keeps only **Calibration due** and **Calibration health** as calibration columns.
- Calibration remains **current-state only** for this release; a future `CalibrationEvent` history store remains possible.
- The supplied workbook is a **one-time cutover seed**; Inventory Management becomes the source of truth afterward.
- The workbook defines the initial calibration roster. Approved workbook rows become the baseline; more equipment can be added manually later.
- Approved workbook records are re-verified at commit time.
- The owner initially expected four legitimate workbook-only units. The final copied-data preview instead surfaces eight create candidates and four ignored-junk rows; five create candidates have review hints only. The live operator must explicitly choose use-existing, create, or ignore for every blocking row rather than relying on the preliminary count.
- Workbook-absent active entries currently marked `required` leave the calibration group. Set them to `unknown`, not `not_required`, because absence from the workbook does not prove calibration is unnecessary. Preserve their existing dates/vendor/certificate/notes; clear only stale `outToCalibration` workflow state.

## Read-Only Workbook / Inventory Profile

Source workbook:

`S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\TE Lab Equip Calibration Data for New List.xlsx`

- Workbook last modified: **July 22, 2026**.
- `MAY 2026 Calibration Equip`: 69 parsed nonblank source rows.
- `OCT 2026 Calibration Equip`: 28 parsed nonblank source rows.
- Total parsed nonblank source rows: 97.
- Final copied-data preview: 78 matched updates, 8 create candidates, 3 conflicts, 4 duplicate source rows, and 4 ignored label/junk rows, leaving 93 equipment-like rows.
- All eight create candidates contain both source identity cells but have no accepted unique identifier match. Review hints are absent for three candidates, unique for one, and multiple for four; hints never auto-match.
- The source includes duplicate/placeholder identities (`NSN`, repeated serials, duplicate equipment rows), two swapped asset/serial pairs, and one duplicate database asset.
- Semantic exception rows include explicit **Reference only**, **No calibration needed**, **Inactive/scrapped**, **Needs calibration**, **vendor has unit/out to calibration**, and **failed calibration** cases. These must be proposed explicitly in preview rather than flattened into a single `required` value.
- Copied database used for the final rehearsal: 542 TE records (529 active, 13 archived).
- The final copied preview reports 110 current active-required entries absent from the workbook and a pre-resolution prospective active-required count of 86. Both values must be recomputed after fresh live sync and owner resolutions.

Do not commit live workbook contents, row identifiers, or hashes. Keep the copied workbook under gitignored `data/import/` only for local cutover work; all automated fixtures must be synthetic.

## Architecture

### Same Record, Two Views

Keep the existing TE `InventoryEntry` calibration fields as the single source of truth:

- `calibrationRequirement`
- `outToCalibration`
- `lastCalibratedAt`
- `calibrationDueAt`
- `calibrationIntervalMonths`
- `certificateRef`
- `calibrationVendor`
- `calibrationNotes`
- `verifiedAt` / `verifiedBy`

Do **not** create a second FeOx equipment record, second TE database, second shared root, or duplicated calibration row. The Calibration workspace filters and presents the same in-memory records loaded by `useDesktopInventory`.

No entry-schema or sync-schema bump is required because the persisted fields already exist and already participate in CRUD, conflict merging, snapshots, and shared sync. New roster preview/commit metadata remains local administrative state.

### Future History Seam

Refactor the edit UI into focused **Equipment** and **Calibration** sections/tabs, but keep one save contract. This creates a clean future insertion point for `CalibrationEvent` history without implementing events now.

## Phase 1 — Calibration Workspace UI

### 1. Add workspace state and navigation

Primary files:

- Modify `frontend/src/modules/te-test-equipment/TeTestEquipmentView.tsx`
- Modify `frontend/src/modules/te-test-equipment/components/InventoryHeader.tsx`
- Add `frontend/src/modules/te-test-equipment/components/header/WorkspaceToggle.tsx`
- Modify `frontend/src/modules/te-test-equipment/types.ts`

Implementation:

- [x] Add `TeTestEquipmentWorkspace = "equipment" | "calibration"`.
- [x] Add a segmented **Equipment / Calibration** switch beside the existing Inventory/Archive scope control.
- [x] Preserve the existing `InventoryScope` behavior so either workspace can show active or archived records.
- [x] Keep the TE module mounted and on the same adaptive sync controller; switching workspaces must not activate/deactivate another backend module.
- [x] Preserve separate query, filter, sort, and column-visibility state for each workspace so switching back restores the prior view.

### 2. Define view-specific columns and preferences

Primary files:

- Modify `frontend/src/modules/te-test-equipment/types.ts`
- Modify `frontend/src/modules/te-test-equipment/lib/columns.ts`
- Modify `frontend/src/modules/te-test-equipment/components/shell/helpers.ts`
- Modify `frontend/src/modules/te-test-equipment/components/shell/useInventoryPreferences.ts`
- Modify `frontend/src/modules/te-test-equipment/components/InventoryTable.tsx`
- Modify `frontend/src/modules/te-test-equipment/components/table/InventoryTableBody.tsx`
- Modify `frontend/src/modules/te-test-equipment/components/table/InventoryTableHeader.tsx`
- Modify `frontend/src/modules/te-test-equipment/components/table/columnStyles.ts`

Equipment columns:

- [x] Keep the existing general inventory columns.
- [x] Remove `calibrationRequirement` and `outToCalibration` from the Equipment view’s available/default columns.
- [x] Keep `calibrationDueAt` and `calibrationHealth` default-visible.
- [x] Preserve existing user visibility preferences through a backward-compatible merge.

Calibration columns:

- [x] Verified
- [x] Asset number
- [x] Serial number
- [x] Manufacturer
- [x] Model
- [x] Description
- [x] Calibration health
- [x] Last calibrated
- [x] Calibration due
- [x] Interval months
- [x] Out to calibration
- [x] Vendor
- [x] Certificate reference
- [x] Location
- [x] Assigned to
- [x] Calibration notes

Use a separate localStorage key for Calibration column visibility. Extend table cell rendering/sorting/widths for the added current-state calibration fields.

When row coloring is enabled:

- [x] Equipment workspace colors by lifecycle, as today.
- [x] Calibration workspace colors by derived calibration health (overdue/missing, due soon/out, current) while retaining accessible text badges.

### 3. Add workspace-aware filtering and summaries

Primary files:

- Modify `frontend/src/modules/te-test-equipment/lib/filtering.ts`
- Modify `frontend/src/modules/te-test-equipment/lib/sorting.ts`
- Modify `frontend/src/modules/te-test-equipment/lib/resultLabels.ts`
- Modify `frontend/src/modules/te-test-equipment/components/FilterPanel.tsx`
- Modify `frontend/src/modules/te-test-equipment/components/SearchCard.tsx`
- Modify `frontend/src/modules/te-test-equipment/components/shell/useInventoryViewModel.ts`
- Modify `frontend/src/modules/te-test-equipment/components/StatusStrip.tsx`

Behavior:

- [x] Calibration workspace defaults to active entries with `calibrationRequirement === "required"`.
- [x] Add an opt-in filter for `reference_only`; never silently include `unknown` or `not_required` in the default roster.
- [x] Keep calibration search across identity, description, dates, vendor, certificate, notes, location, and assignment.
- [x] Keep due-window and health filters prominent in Calibration; keep general filters in Equipment.
- [x] Calibration results/status copy says “calibration equipment,” not generic inventory entries.
- [x] Reuse the existing overdue, due-soon, missing-due, and out-to-cal counts; add a visible tracked/result count without changing the persisted model.

## Phase 2 — Focused Editing and Manual Membership

### 4. Split the entry editor into Equipment and Calibration sections

Primary files:

- Modify `frontend/src/modules/te-test-equipment/components/EntryDialog.tsx`
- Modify `frontend/src/modules/te-test-equipment/components/entry-dialog/fieldMetadata.ts`
- Modify `frontend/src/modules/te-test-equipment/components/entry-dialog/form.ts`
- Modify `frontend/src/modules/te-test-equipment/components/shell/useInventoryEntryMutations.ts`

Behavior:

- [x] Keep one `InventoryEntryInput` save and existing field-scoped edit context.
- [x] Equipment table opens the Equipment section by default.
- [x] Calibration table opens the Calibration section by default.
- [x] Calibration section contains requirement, workflow flag, last/due dates, interval, certificate, vendor, and calibration notes, with equipment identity shown for context.
- [x] The due-date suggestion remains explicit; it never overwrites a supplied due date automatically.
- [x] Saving from either section updates the same record and immediately refreshes both views.

### 5. Add manual calibration membership actions

Primary files:

- Modify `frontend/src/modules/te-test-equipment/components/EntryContextMenu.tsx`
- Add `frontend/src/modules/te-test-equipment/components/calibration/CalibrationMembershipDialog.tsx`
- Modify `frontend/src/modules/te-test-equipment/TeTestEquipmentView.tsx`
- Reuse `updateEntry` through `useInventoryEntryMutations`

Behavior:

- [x] Equipment row context action **Add to Calibration** sets `calibrationRequirement = required` without auto-verifying the row.
- [x] Calibration header action **Add Equipment** opens a searchable picker of active non-required TE equipment and supports selecting one or more existing records.
- [x] Picker also offers **Create New Equipment**, opening the entry dialog with `calibrationRequirement = required` preselected.
- [x] Removing a row from Calibration requires choosing `reference_only`, `not_required`, or `unknown`; do not use an ambiguous boolean toggle.
- [x] Manual membership changes use normal update mutations and shared sync.

## Phase 3 — Isolated Calibration Roster Reconciliation

### 6. Add a calibration-specific importer without changing the approved general importer

New backend area:

- Add `backend/src/integrations/calibration_roster_import/mod.rs`
- Add `backend/src/integrations/calibration_roster_import/parser.rs`
- Add `backend/src/integrations/calibration_roster_import/reconcile.rs`
- Add `backend/src/integrations/calibration_roster_import/commit.rs`
- Add `backend/src/integrations/calibration_roster_import/types.rs`
- Modify `backend/src/integrations/mod.rs`
- Modify `backend/src/storage/imports.rs` and related key helpers for a namespaced roster-preview/completion store

Hard boundary:

- [x] Leave `backend/src/integrations/inventory_import/**` behavior unchanged: single selected inventory sheet, matched rows are no-ops, and its existing full-batch contract/tests remain authoritative.
- [x] The new roster importer is TE-only, `.xlsx`-only for this cutover, and allowed to update matched records because that is its explicit purpose.

Parser behavior:

- [x] Read all visible sheets and locate header rows by required header aliases rather than hardcoding row 1/8.
- [x] Merge the May and October calibration sheets into one normalized row model while preserving source sheet and one-based source row.
- [x] Ignore fully blank rows.
- [x] Classify obvious workbook labels and punctuation-only separators as ignored when they provide no usable equipment context; report the reason.
- [x] Normalize Excel date cells to `YYYY-MM-DD`.
- [x] Treat placeholder identities such as `NSN`, `N/A`, `NONE`, `UNKNOWN`, and repeated `X` values as missing identities for roster matching, while reporting that the placeholder was ignored.
- [x] Do not depend on Excel fill colors. Derive proposed semantics from explicit cell text, dates, status, comments, and condition; any unclear case remains review-required.

Proposed semantic mappings:

- [x] Normal dated rows: `required`, valid last/due dates.
- [x] `REF ONLY` / `NO CAL NEEDED`: `reference_only`, no fabricated dates.
- [x] `Needs Cal` / unknown due: `required` with missing due date.
- [x] Vendor-has-unit text such as `ACCURA` / `Accura Has`: `required`, `outToCalibration = true`.
- [x] Failed-calibration notes: remain `required`; retain valid overdue dates and surface notes.
- [x] Inactive/scrapped text: propose `not_required` and require explicit review; do not silently change lifecycle/archive state.
- [x] Never infer `calibrationIntervalMonths` from the dates.

### 7. Reconcile workbook rows against current TE entries

Matching rules:

- [x] Auto-match only by unique normalized asset number or serial number.
- [x] If both identifiers resolve, they must resolve to the same entry.
- [x] Placeholder serials do not participate in matching.
- [x] Manufacturer/model/description are review hints only; hinted creates do not receive a default resolution.
- [x] Source duplicates that resolve to the same entry must be merged or one row explicitly ignored; never apply two competing updates silently.
- [x] Duplicate database candidates, cross-field/swapped identifiers, asset/serial disagreement, and unclear semantic rows are blocking until resolved.

Preview classifications:

- [x] Matched update
- [x] Create candidate
- [x] Conflict/review required
- [x] Duplicate source row
- [x] Ignored junk/label
- [x] Current required entry absent from the approved workbook roster

Preview content:

- [x] Workbook fingerprint and filename
- [x] All contributing sheets
- [x] Current inventory reconciliation basis
- [x] Aggregate counts
- [x] Source sheet/row
- [x] Candidate entry UUID and safe identity/description context
- [x] Field-level proposed changes
- [x] Explicit warnings for cleared values or requirement changes
- [x] Prospective count of active `required` records after commit

### 8. Add explicit resolution and guarded commit types

Primary backend files:

- Modify `backend/src/api/commands.rs`
- Modify `backend/src/lib.rs`
- Reuse `backend/src/api/mutations.rs` field-scoped mutation helpers

New commands:

- [x] `pick_calibration_roster_file`
- [x] `preview_calibration_roster`
- [x] `commit_calibration_roster`

Commit input:

- [x] Batch ID
- [x] Explicit confirmation
- [x] Per-conflict resolution (target entry, create, or ignore)
- [x] Per-create edited input for approved unmatched equipment
- [x] Confirmation that the workbook replaces the active required roster
- [x] Verification attribution text (default: `Calibration roster cutover — TE Lab Equip Calibration Data for New List.xlsx`)

Commit safeguards:

- [x] Re-read and fingerprint the workbook.
- [x] Recompute inventory basis and refuse stale previews.
- [x] Refuse unresolved conflicts, duplicate targets, or unreviewed destructive changes.
- [x] Use deterministic UUIDs and source provenance for newly created rows.
- [x] Keep matched entries’ existing original import provenance; store calibration-roster row-to-entry audit records in the roster batch metadata.
- [x] Apply matched updates through normal validation and field-scoped sync operations.
- [x] For matched entries, update calibration fields only; preserve general equipment identity/location/description by default. Workbook comments may be appended to calibration notes only through an explicit proposed change.
- [x] For approved new rows, populate available identity/manufacturer/model/description/location/assignment/condition fields, leave quantity unset unless reviewed, and mark calibration state from the normalized row.
- [x] Set approved workbook entries’ `verifiedAt` to the actual commit timestamp and `verifiedBy` to the confirmed attribution. Do not use the workbook modification time as a verification timestamp.
- [x] Build the final approved workbook entry UUID set. For active entries currently `required` but absent from that set, set `calibrationRequirement = unknown` and `outToCalibration = false`; preserve dates, vendor, certificate, notes, verification, lifecycle, and archive state.
- [x] Use per-row/per-entry completion markers so a failed commit can resume idempotently without duplicate creates or operations.
- [x] Schedule one TE shared publish after the batch completes.

## Phase 4 — Calibration Roster Wizard

### 9. Add the desktop preview/review UI

Primary frontend files:

- Add `frontend/src/modules/te-test-equipment/components/calibration/CalibrationRosterDialog.tsx`
- Add roster report/input types in `frontend/src/modules/te-test-equipment/types.ts`
- Modify `frontend/src/integrations/tauri/desktop-bridge.d.ts`
- Modify `frontend/src/integrations/tauri/tauriInventoryBridge.ts`
- Modify `frontend/src/integrations/tauri/bridgeGuards.ts`
- Modify `frontend/src/modules/te-test-equipment/components/InventoryHeader.tsx`
- Modify `frontend/src/modules/te-test-equipment/TeTestEquipmentView.tsx`

Behavior:

- [x] Expose **Initialize from Calibration Workbook** only in the Calibration workspace and desktop runtime.
- [x] Keep the existing general `ImportDialog` unexposed and unchanged.
- [x] Preview first; nothing writes during file selection or reconciliation.
- [x] Show summary cards for matched updates, create candidates, conflicts, ignored junk, duplicate source rows, and currently-required app-only rows that will leave the group.
- [x] Default unhinted create candidates to “Create after review” and junk rows to “Ignore,” while leaving hinted create candidates unresolved until the operator explicitly chooses use-existing, create, or ignore. Every reviewed row still requires confirmation.
- [x] Provide candidate selection for swapped/duplicate identifier conflicts.
- [x] Show semantic exception rows separately (reference-only, inactive/scrapped, needs calibration, out-to-calibration, failed calibration).
- [x] Show field-level before/after values for requirement, workflow, dates, vendor, notes, and verification.
- [x] Require final checkboxes confirming roster replacement and re-verification before enabling commit.
- [x] After commit, refresh entries, switch to Calibration active scope, and show the final tracked/due counts.

## Phase 5 — Tests

### 10. Frontend tests

Add/update:

- `frontend/tests/inventory-shell-views-export.test.tsx`
- `frontend/tests/inventory-table.test.tsx`
- `frontend/tests/columns.test.ts`
- `frontend/tests/inventory-filtering.test.ts`
- `frontend/tests/entry-dialog.test.tsx`
- `frontend/tests/inventory-entry-actions.test.tsx`
- Add `frontend/tests/calibration-workspace.test.tsx`
- Add `frontend/tests/calibration-roster-dialog.test.tsx`
- Update `frontend/tests/tauri-inventory-bridge.test.ts`

Required coverage:

- [x] Equipment/Calibration switch preserves independent search/filter/sort/columns.
- [x] Equipment columns retain only due + health calibration fields.
- [x] Calibration defaults to active required rows and can opt into reference-only.
- [x] Same entry edit is reflected in both workspaces.
- [x] Calibration dialog section opens by default from Calibration.
- [x] Manual add/remove membership semantics.
- [x] Calibration-specific row coloring and accessible health text.
- [x] Roster preview groups all classifications and blocks unresolved commit.
- [x] Reviewed create and junk resolutions are explicit; hinted create candidates do not default to creation.
- [x] Replacement confirmation and verification attribution reach the bridge.
- [x] Bridge guards reject malformed roster reports/results.

### 11. Backend tests

Add `backend/tests/calibration_roster_flow.rs` using synthetic workbooks only.

Required coverage:

- [x] Finds non-first-row headers on two sheets and merges them.
- [x] Parses Excel dates and explicit semantic tokens.
- [x] Ignores placeholder serials for identity matching while reporting them.
- [x] Detects source duplicates, database duplicates, cross-field/swapped identifiers, and asset/serial disagreement.
- [x] Never auto-matches on manufacturer/model/description hints alone.
- [x] Produces matched updates, create candidates, conflicts, and ignored junk.
- [x] Requires explicit resolution and rejects stale workbook/inventory basis.
- [x] Updates only calibration/verification fields on matched records.
- [x] Creates reviewed unmatched rows with deterministic UUID/provenance.
- [x] Re-verifies approved workbook records at commit time.
- [x] Resets workbook-absent active required rows to `unknown` without clearing historical calibration details.
- [x] Leaves archived records and non-required/reference/unknown records unchanged unless directly represented by an approved workbook row.
- [x] Queues field-scoped outbox operations and publishes through normal TE sync.
- [x] Is idempotent and resumes correctly after injected mid-batch failure.
- [x] Existing `backend/tests/import_flow.rs` remains green and proves the general importer contract did not change.
- [x] Existing `backend/tests/shared_sync_flow.rs` remains green; no sync schema bump or cross-module leakage.

## Phase 6 — Documentation and Decision Record

Update:

- [x] Add IM-015 to `docs/planning/DECISIONS.md`.
- [x] Add this implementation authority under `docs/superpowers/plans/`.
- [x] Update `docs/planning/IMPORT_PROFILE.md` with aggregate-only calibration workbook observations during implementation/cutover; do not include row identifiers, file hash, or live row data.
- [x] Update `docs/SESSION_HANDOFF.md` and `docs/SESSION_START_PROMPT.md` after implementation with actual evidence and the still-pending live-cutover status.
- [x] Document that calibration sync is not a backup and the standalone TE writer must remain closed during cutover.

## Phase 7 — Verification Before Any Live Commit

Run targeted first, then full gates:

```powershell
bun run test -- calibration-workspace calibration-roster-dialog inventory-table entry-dialog inventory-filtering columns tauri-inventory-bridge
bun run test
bun run lint
bun run build
Set-Location backend
cargo fmt --all -- --check
cargo test --test calibration_roster_flow
cargo test --test import_flow
cargo test --test shared_sync_flow
cargo test
cargo clippy --all-targets --all-features -- -D warnings
Set-Location ..
```

Desktop smoke before live mutation:

- [x] Confirm no unified app, standalone TE app, or other writer is running.
- [x] Launch one Inventory Management instance against isolated copied data with shared sync disabled.
- [x] Verify Equipment ↔ Calibration switching, workspace-specific columns, roster preview-only UI, editor routing, and no runtime errors. Automated coverage verifies independent preferences and add/remove membership.
- [x] Run the workbook preview only and compare its aggregate counts to the read-only profile; do not commit.
- [x] Stop the app and confirm the desktop process/listeners close.

## Phase 8 — Controlled Live Cutover

The gitignored workbook/database copies and aggregate preview used for rehearsal do **not** satisfy any live-cutover step below.

1. [ ] Copy the workbook to gitignored `data/import/calibration-tracking.xlsx` for the cutover session; keep the S: source unchanged.
2. [ ] Confirm the source fingerprint and modification date still match the reviewed workbook. If it changed, repeat profiling/review.
3. [ ] Ensure Inventory Management and all standalone writers are closed.
4. [ ] Take a dated backup of `%LOCALAPPDATA%\com.inventory.management\inventory.feox` and a separate dated copy of the TE product shared root. Sync artifacts alone are not a backup.
5. [ ] Launch one Inventory Management instance and wait for a clean TE shared sync.
6. [ ] Preview the workbook against the current database.
7. [ ] Resolve all remaining identifier conflicts and duplicate source rows.
8. [ ] Resolve every live create candidate and junk row. The copied rehearsal currently shows eight create candidates and four junk rows; five creates have review hints. Approve only the legitimate new units after owner review and do not assume these counts if the source or database changed.
9. [ ] Confirm roster replacement and workbook-based re-verification.
10. [ ] Commit once. Let the app queue and publish the resulting TE operations; do not open another writer during publication.
11. [ ] Verify after commit:
    - Calibration workspace contains only approved workbook-required rows plus later/manual additions.
    - Reference-only rows are available through the reference filter but absent from the default required roster.
    - Workbook-absent formerly-required active rows now read `unknown`, retain prior details, and no longer appear by default.
    - Every owner-approved new equipment record exists in both Equipment and Calibration views; the final count agrees with the resolved live preview.
    - All approved workbook records have the new verification timestamp/attribution.
    - Due/health counts agree with the resolved preview.
12. [ ] Wait for shared publish, confirm no pending local-only changes, restart the app, and verify persistence.
13. [ ] Perform a second-client/read-only verification only after the first writer is closed or fully idle under the one-writer rule.
14. [ ] Retain the backup, reviewed workbook, and aggregate reconciliation report through the rollback window.

## Acceptance Criteria

- [x] One TE equipment record is the source for both Equipment and Calibration views; no duplicated data or second TE store exists.
- [x] Changes made from either view appear immediately in the other and use the existing sync path.
- [x] Equipment table shows only Calibration due + Calibration health from the calibration domain.
- [x] Calibration workspace defaults to active `required` equipment, with reference-only available explicitly.
- [x] Users can add existing/new equipment to calibration and remove it with an explicit semantic requirement choice.
- [x] The current workbook can be previewed and resolved without changing the approved general inventory importer.
- [ ] A fresh live preview is fully resolved so only owner-approved equipment-like rows affect the cutover; no fixed create/ignore count is assumed before that review.
- [ ] Approved workbook records are verified at the actual commit time with source attribution.
- [ ] Active required entries absent from the approved workbook set leave the group as `unknown` without losing their previous calibration details.
- [x] Full frontend/Rust/sync tests, lint, build, clippy, and desktop smoke pass before live mutation.
- [ ] Live cutover is backed up, single-writer, explicitly confirmed, restart-verified, and reversible.

## Completion Evidence

Fill this section during implementation; do not replace planned checks with claims.

| Evidence | Result / artifact |
|----------|-------------------|
| Targeted frontend tests | Passed: calibration workspace 7/7; calibration roster dialog 4/4; inventory table 9/9. |
| Full frontend suite | Passed: 21 files, 1 skipped; 166 tests, 1 skipped. Existing unrelated `act(...)` warning remains non-failing. |
| Frontend lint | Passed: `npm run lint`. |
| Frontend production build | Passed: `npm run build` (`tsc -b` + Vite). |
| Calibration roster Rust flow | Passed: 43/43. |
| Existing general import regression | Passed: 60/60. |
| Shared sync regression | Passed: 40/40. |
| Full Rust suite | Passed with only `inspect_legacy_headers_only` and `audit_live_database_snapshot_aggregates_only` filtered because they require owner paths; benchmark remained intentionally ignored. |
| Rustfmt | Passed: `cargo fmt --all`; final format check clean. |
| Clippy `-D warnings` | Passed across all targets/features. |
| Single-instance desktop smoke | Passed against copied data with shared sync disabled; no runtime exceptions; app and ports 5173/9222 fully closed afterward. |
| Workbook preview reconciliation | Passed aggregate-only/no-write rehearsal: 97 rows = 78 matched + 8 create + 3 conflict + 4 duplicate + 4 junk; 110 absent; prospective required 86; blocking `true`; copied entry/outbox counts unchanged. |
| Backup paths / rollback evidence | Pending |
| Live commit and shared publish | Pending |
| Restart verification | Pending |
