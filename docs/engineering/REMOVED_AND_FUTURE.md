# Removed code and future ideas

Code that was deleted because nothing used it, or because it was a placeholder for a feature that does not exist. The code is gone from the tree; this page records what it was, why it went, and where to get it back.

**To restore anything below:** every item still exists at commit `119ca8a` (`main` at 0.1.4 plus dev-build isolation). For example, `git show 119ca8a:frontend/src/modules/te-test-equipment/components/ImportDialog.tsx`.

## Future ideas (not built)

| Idea | Notes |
|------|-------|
| **ME Storage inventory** | A switcher entry that only showed "module not connected yet". The S: folder `modules\ME_Storage\` is still reserved. Build it as a real module when there is a spec; EVCS Storage (`te-storage`) is the closest template. |
| **HTML export (TE Test Equipment)** | The Export menu had an "HTML" item that only said "HTML export is not implemented yet." Excel export is the supported format. |
| **Remember window size and position** | `windowState.ts` saved and restored window geometry. It was switched off because restoring broke the layout on some monitors, and was never turned back on. A future version would need to clamp restored bounds to the current monitors. |
| **Configurable calibration "due soon" window** | Still parked as IM-F01 in `docs/planning/DECISIONS.md`. |

## Removed in 0.2 (2026-10-16)

### Frontend

| What | Why |
|------|-----|
| ME Storage placeholder module, `placeholderHost`, `ShellHeader`, `InventorySystemPlaceholder` | Placeholder only. The shell now just mounts the three real inventories. |
| TE spreadsheet import: `ImportDialog`, bridge `pickImportFile`/`previewImport`/`commitImport`, their guards and types | Never mounted after the 0.1 cutover (the cutover import was done offline). The **calibration roster** import is separate and stays. |
| `windowState.ts` and its tests | Disabled feature (see above). |
| TE `ColumnMenu`, `ViewSettingsMenu`, `useMediaQuery`; shared `toggle.tsx` | Unreferenced. |
| Lab v1 inventory UI (~3,250 lines: old table, entry dialog, filters, header, search card, view settings, mock data, `lib/`) | Replaced by the catalog UI in 0.1.2; nothing rendered it. |
| Copies of the update button, update polling hook, update-state helpers, and status announcer in TE and Lab | Merged into one copy each: `platform/updates/` and `shared/hooks/useStatusAnnouncer.ts`. |
| Three copied footers | Merged into `platform/ui/StatusFooter.tsx`. |
| Unused exports and helpers found by `knip` (old guard functions, constants, type aliases, `formatRelativeTimestamp`, the `INVENTORY_COLUMNS` alias, test-only registry helpers) | Unreferenced. |

### Backend

| What | Why |
|------|-----|
| TE spreadsheet import: `integrations/inventory_import/`, commands `pick_import_file` / `preview_import` / `commit_import`, import-batch storage keys, `tests/import_flow.rs` | No UI since the 0.1 cutover. Calibration roster storage moved from `storage/imports.rs` to `storage/calibration_roster.rs`. Entries keep their `importProvenance` field, since existing records carry it. |
| `integrations/deprecated_db_cleanup.rs` | One-time quarantine of pre-0.1 SQLite `.db` files; it has already run on every install. |
| Roaming → Local copy of `inventory.feox` | One-time move for the earliest builds. `InventoryAppPaths` now only resolves the local data folder. |
| Live-data one-shot tests: `REPAIR_TE_LEGACY`, `REPAIR_LAB_OUTBOX`, `REPAIR_LIVE_APPDATA`, `SEED_LAB_SHARED`, `RESEED_LAB_DEMO`, `TE_LOAD_LIVE_LOCAL`, and `tests/live_db_audit.rs` (`TE_LEGACY_AUDIT_XLSX`) | Repairs and audits for 0.1.0-era data that are finished. They wrote to real AppData or S:, which developer builds should never do. The 0.1.4 shared-snapshot restore tool (`sync/snapshot/restore.rs`) stays. |
| Lab demo catalog seed (`catalog_demo_seed.rs`, `INVENTORY_MANAGEMENT_LAB_DEMO_SEED`) | Layout-review data. The dev share fixtures (0.2 plan, Phase 2) replace it. |
| InventoryApps pilot root constants | Unreferenced since the 2026-07-20 move to the product share. |
| Lab v1 backend: `te_lab_components/{model,mutations,query,entry_changes}.rs`, the copied v1 sync engine `te_lab_components/sync/`, v1 entry storage, the v1 workbook export `export/lab_workbook*` | Every Lab install is on the catalog (schema v2) since 0.1.2. Lab entry commands (`create_entry` etc. with the Lab module id) now return an error pointing at the catalog commands. |
| Lab v1 → v2 migration (`catalog_migration.rs`, commands `preview_/commit_lab_catalog_migration`, `MigrationPanel`) and its fixtures in `backend/tests/fixtures/te_lab_components/` | A namespace that is not on schema v2 now drops its v1 rows and starts empty, then pulls the shared catalog (`InventoryDb::ensure_catalog_schema`). The 0.1.x database file stays in `pre-0.2-databases\`. |
| Lab shared-root cutover (commands `preview_/commit_lab_shared_cutover`, `SharedCutoverDialog`, the header's "Set Up Lab Shared Catalog" button) | The team share was cut over in 0.1.2. A brand-new shared root (for example the dev share) is now set up automatically by the first sync; a root that still holds the v1 stream stays local-only. |
| Path-included integration test crates (`backend/tests/`) | They compiled source files by path, so any file move broke them. The same tests now run inside the crate (`sync/flow_tests/`, `calibration_roster_import/flow_tests.rs`, `domain/performance_baseline.rs`). |
