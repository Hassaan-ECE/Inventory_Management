# TE Test Equipment Inventory v0.1 Design

**Date:** 2026-07-13
**Status:** Principal-approved initial-version contract
**Authority:** `docs/planning/DECISIONS.md`, with the provisional defaults approved in the initial-version request

## Outcome

Version 0.1 is a usable Windows desktop inventory app for TE test equipment on the existing ME-family Tauri 2, React 19, TypeScript, Rust, and FeOxDB scaffold. Calibration is a first-class current-state domain. This is not the production lab cutover.

The implementation remains in this repository. Its recorded lineage is ME at `e092c73`; TE Parts at `e444389` is a read-only UI reference. The application UUID remains stable record identity. Manufacturer plus model never causes an unattended merge.

## Scope and non-goals

V0.1 includes identity cleanup, calibration domain/storage/sync/export, a fixture-backed Excel/CSV importer, calibration UI in the existing shell, documentation, and verified builds/tests.

V0.1 does not include a `CalibrationEvent` history store, managed certificate/media vault, production shared-sync enablement, two-machine cutover proof, Python-app retirement, TanStack/Reicon migration, dashboard-heavy reporting, ownership/rental expansion, or release publication.

## Product identity

| Concern | Required value |
|---|---|
| Display name | `TE Test Equipment Inventory` |
| Package | `te-test-equipment-inventory` |
| Version | `0.1.0` |
| Tauri identifier | `com.te.test.equipment.inventory` |
| Local database | `%LOCALAPPDATA%\com.te.test.equipment.inventory\inventory.feox` |

`InventoryDb::open` uses Tauri `app_local_data_dir()`. Preference keys use `teTestEquipmentInventory.*`. HTML title, UI copy, export name, environment names, shared-root fallback, tests, README, and handoff stop claiming ME identity.

The inherited ME updater endpoint and public key are unsafe. V0.1 disables the updater completely: remove runtime registration, frontend updater calls and controls, updater capabilities, updater dependencies/configuration, and updater artifact generation. No code may contact an ME release URL.

## Calibration record

The entry projection stores these fields:

| TypeScript | Rust | Type |
|---|---|---|
| `calibrationRequirement` | `calibration_requirement` | `required | reference_only | not_required | unknown` |
| `outToCalibration` | `out_to_calibration` | boolean |
| `lastCalibratedAt` | `last_calibrated_at` | optional `YYYY-MM-DD` |
| `calibrationDueAt` | `calibration_due_at` | optional `YYYY-MM-DD` |
| `calibrationIntervalMonths` | `calibration_interval_months` | optional positive integer |
| `certificateRef` | `certificate_ref` | simple optional string |
| `calibrationVendor` | `calibration_vendor` | simple optional string |
| `calibrationNotes` | `calibration_notes` | simple optional string |
| `verifiedAt` | `verified_at` | optional RFC 3339 timestamp |
| `verifiedBy` | `verified_by` | optional free text |

The due date is explicit source of truth. An interval may suggest a date in the UI but never overwrites `calibrationDueAt` without a user action. A due date earlier than the last-calibrated date is invalid.

`verifiedAt` replaces `verifiedInSurvey`. Compatibility decoding accepts legacy JSON without pretending certainty: missing or false maps to no timestamp. Legacy true plus a valid legacy `updatedAt` maps that timestamp to `verifiedAt` and sets `verifiedBy` to `Legacy verified flag — timestamp approximated from updatedAt`. Legacy true without a valid `updatedAt` maps to no timestamp and requires re-verification. Migration round-trip tests prove the approximation is labeled and never becomes an unlabeled verification fact.

## Derived health

`CalibrationHealth` is computed, never persisted:

`missing_due | overdue | due_soon | current | not_applicable | unknown | out_to_cal`

For active records, evaluate with workstation local date in this order:

1. Archived records are excluded from active calibration counts.
2. `reference_only` or `not_required` becomes `not_applicable`.
3. `unknown` becomes `unknown`.
4. `outToCalibration` becomes `out_to_cal`.
5. `required` with no valid due date becomes `missing_due`.
6. A due date before today becomes `overdue`.
7. A due date from today through today plus 30 calendar days, inclusive, becomes `due_soon`.
8. A later due date becomes `current`.

UI labels are `Reference only` and `Not required`. Tests pass an explicit local date to the health helper.

## Persistence, sync, and export

The existing `entry:{entry_uuid}` FeOxDB representation remains. New calibration and verification fields participate in normalization, validation, create/update, field-scoped changes, bridge guards, serialization, operation payloads, canonical checksums, merge/diff, snapshots, bootstrap, recovery, conflicts, export, and tests.

The sync schema version is bumped for the changed projection. Shared production mode stays disabled unless `TE_TEST_EQUIPMENT_SHARED_SYNC_ENABLED` is explicitly true. `TE_TEST_EQUIPMENT_SHARED_ROOT` selects the root and `TE_TEST_EQUIPMENT_SYNC_HMAC_KEY` optionally authenticates artifacts. The staging sketch remains `S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE\Test_Equipment`; it is not production approval. Local use remains available while shared mode is disabled or unavailable. Sync artifacts are not backups.

Archive remains separate from lifecycle. Scrapped or missing equipment is not automatically archived.

## Importer

V0.1 provides a native picker for `.xlsx`, `.xls`, and `.csv`, a dry run, and a separately confirmed commit. Rust owns parsing and reconciliation so it is testable without UI.

At design time, `data/import/` contained only `.gitkeep`. Implementation records `profile blocked — no export` in `docs/planning/IMPORT_PROFILE.md` and uses synthetic fixtures representing the documented Python-reference shape. Real lab workbooks remain ignored and uncommitted.

Every dry-run row has one classification: `inserted`, `matched`, `conflicted`, `rejected`, or `ignored`. Auto-match is allowed only through one unique normalized asset or serial. Duplicate keys, multiple candidates, or asset/serial disagreement are conflicts. Manufacturer plus model is a review hint only.

The batch ID is deterministic from file content, selected sheet, and mapping version. Provenance includes batch ID, source filename, sheet, one-based row, and original identifiers. Completed batch identity and row provenance are stored. Recommitting a completed batch is a no-op result, not duplicate insertion.

All source columns are reported as mapped or intentionally ignored. Unknown columns and nonblank unmapped cells produce report entries. No row or column is silently dropped. Commit requires the dry-run batch ID and explicit confirmation, and refuses if the source fingerprint or reconciliation basis changed.

## Existing-shell UI

The current inventory/archive shell remains. V0.1 adds:

- default-visible health and due-date columns;
- requirement, health, and due-window filters plus health/due sorting;
- overdue, due soon, missing due, and out-to-cal count chips;
- add/edit fields for requirement, dates, workflow flag, interval, certificate, vendor, notes, and optional verifier;
- verification action that sets the current timestamp and a clear action;
- import picker, dry-run reconciliation, and guarded commit;
- accessible text labels in addition to badge colors.

There is no history UI because there is no event store. Current-state edits replace prior calibration details.

## Errors and validation

Validation failures return field-specific messages without partial FeOxDB mutation. Import errors identify file, sheet, row, and column when possible. Conflicted or rejected dry-run rows block commit unless the user explicitly resolves or excludes them. Sync gate failures remain local and state the reason. Updater behavior cannot execute because it is removed.

## Verification gates

Automated evidence covers health boundaries, date validation, legacy verification compatibility, storage round trips, sync merge/conflict/snapshots, importer reconciliation/idempotence, bridge guards, filters/sorts/counts, dialog behavior, and export columns.

Integrated commands are:

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

An unavailable dependency is reported with its exact error. A distinct read-only Reviewer inspects the integrated patch, followed by Boss verification against acceptance areas A through G.

## Acceptance traceability

| Area | Coverage |
|---|---|
| A | Product/package/Tauri/database/preference/updater identity |
| B | Approved defaults, deferred history/media, base SHAs, optional attribution, due-date authority, labels, blocked live profile |
| C | Calibration fields, exact health, verified timestamp, persistence/sync/export, UUID identity |
| D | Excel/CSV, five classifications, deterministic identity, provenance, guarded idempotent commit, no silent loss |
| E | Existing shell, columns, badges, filters/sorts/counts, dialog, verification, archive separation |
| F | Shared gate, truthful docs, sync-not-backup, updater removal |
| G | Frontend/backend/build evidence, independent review, Boss verification, limitations |

## Limitations

V0.1 is a current-state reminder board, not an audit-history system. Certificates and pictures remain references, not managed media. Import mapping is fixture-backed until a live export is profiled. Shared sync remains non-production; backup/restore, two-machine proof, cutover rehearsal, and Python retirement remain later work.
