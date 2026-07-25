# Session handoff — Inventory Management

**Last updated:** 2026-07-25
**State:** Product `0.1.0`; IM-014 generalized Lab catalog and IM-015 TE Calibration workspace/roster tooling are integrated on an isolated combined branch. Automated combined verification and copied-data desktop smoke are the next gates. Neither live cutover has occurred.

**New chat:** paste [SESSION_START_PROMPT.md](SESSION_START_PROMPT.md).

## Workspaces and branches

| Purpose | Path / branch |
|---------|---------------|
| Main checkout | `C:\Projects\Active\Inventory_Management` / `main` — contains unrelated local documentation edits; do not overwrite |
| IM-014 source | `C:\Projects\Active\Inventory_Management_IM014` / `feature/im-014-lab-components` / commit `226c679` |
| IM-015 source | `C:\Projects\Active\Inventory_Management_IM015` / `feature/im-015-calibration` / commit `ee7fa70` |
| Combined test | `C:\Projects\Active\Inventory_Management_IM014_IM015` / `feature/im-014-im-015-integration` |

## Stable identity

| Item | Value |
|------|-------|
| Display | Inventory Management |
| Package | `inventory-management` `0.1.0` |
| Tauri id | `com.inventory.management` |
| TE Test Equipment DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| TE Lab Components DB | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| Product share | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE shared root | `...\Inventory_Management_App\modules\TE_Test_Equipment` |
| Default Lab shared root | `...\Inventory_Management_App\modules\TE_Lab_Components` |

## Implemented product shape

- One desktop product with isolated TE Test Equipment and TE Lab Components modules; ME Storage and TE Storage Room remain placeholders.
- Product shell/module switching and IM-011 adaptive sync remain module-scoped.
- TE uses the existing `inventory.feox` and sync schema v2.
- Lab uses the separate `te-lab-components.feox` and IM-014 Lab-only catalog sync schema v2.
- Shared roots, watcher sessions, sync gates, tokens, statuses, and events remain keyed by module.
- First team release `0.1.0`, installer, updater metadata, and product roots already exist.

## IM-014 — generalized Lab catalog

- Parts are separate from StockPlacements.
- Flexible category attributes and detailed persisted filters/sorting are supported.
- Storage hierarchy is area/desk → container → Excel-style grid bin.
- Parts can occupy multiple locations; shared bins are allowed with warnings; duplicate placements merge safely.
- Quantity totals/status are derived by unit; moves and counts are explicit.
- Reviewed lossless migration preserves legacy identity, quantities, locations, compatibility fields, and rerun behavior.
- Lab-only sync schema v2 blocks old v1 writers after cutover.
- Catalog export contains Parts, Archived Parts, Stock Placements, Storage Layout, Attributes, and Legacy Fields sheets.
- Authority: [IM-014 plan](superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md) and [migration runbook](runbooks/te-lab-components-catalog-v2-migration.md).

## IM-015 — TE Calibration workspace and roster

- Equipment and Calibration are two projections of the same TE `InventoryEntry` records; no second equipment table/database/root exists.
- Equipment keeps Calibration due + Calibration health; Calibration owns detailed calibration columns and independent view state.
- Focused Equipment/Calibration editor sections and explicit membership actions update the same record and normal TE sync stream.
- Calibration roster flow is TE-only and `.xlsx`-only: preview, classify, review, resolve, stale-check, commit, resume, and publish.
- Manufacturer/model/description are review hints only and never auto-match; hinted create candidates require an explicit action.
- The approved general inventory importer remains unchanged.
- Authority: [IM-015 plan](superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md).

## Feature verification before integration

### IM-014

- Lab shell + bridge focused gates: **32/32 passed**.
- Other frontend tests: **146 passed, 1 skipped**; lint and production build passed.
- Raw full frontend suite had one unrelated date-sensitive TE count failure on July 25, 2026.
- Rust: **347 passed, 1 ignored**; rustfmt and strict Clippy passed.
- No live database, workbook, shared root, or IM-015 file was modified.

### IM-015

- Full frontend suite: **166 passed, 1 skipped**; lint and production build passed.
- Calibration roster Rust flow: **43/43**; general import **60/60**; shared sync **40/40**; full filtered Rust suite and strict Clippy passed.
- Isolated desktop smoke passed with shared sync disabled.
- Copied-data calibration preview remained blocking and no-write: **97 = 78 matched + 8 create + 3 conflict + 4 duplicate + 4 junk**; 110 absent; prospective required 86.
- No live database, workbook, or shared root was modified.

## Combined integration verification

- Merge order: IM-014 first, IM-015 second.
- Source code auto-merged; only handoff/start/decision documents required manual combination.
- Combined frontend tests, lint, build, Rust tests, rustfmt, Clippy, and desktop smoke: **pending**.

## Live-data boundaries

- Sync is **not** a backup.
- Do not use the product shared roots during the combined development smoke.
- Run desktop testing with a temporary `LOCALAPPDATA` and `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED=0`.
- IM-014 live Lab migration and IM-015 live calibration roster commit are separate owner operations with separate backups and review steps.
- Do not run standalone writers against the corresponding shared roots while either live cutover is active.

## Next steps

1. Finish the combined merge commit after conflict resolution.
2. Run combined automated quality gates.
3. Prepare copied TE/Lab databases under temporary Local AppData and run a single-instance `bun run desktop` smoke.
4. Verify module switching, TE Equipment/Calibration behavior, Lab catalog/grid behavior, and restart persistence.
5. Merge the combined branch into `main` only after owner acceptance.
6. Schedule IM-014 and IM-015 live cutovers separately under their plans/runbooks.
