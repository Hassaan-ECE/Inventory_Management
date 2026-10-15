# Session start prompt — Inventory Management

Copy everything below the line into a **new chat** with workspace:

```text
C:\Projects\Active\Inventory_Management
```

---

You are working on **Inventory Management**, a unified multi-inventory Windows desktop product.

## Open this workspace first

```text
C:\Projects\Active\Inventory_Management
```

Work on `main`. All earlier feature branches (Lab simple workspace, IM-014, IM-015) are merged.

Do **not** use `C:\Projects\Active\Inventory_Apps\...` as the active app tree. Those are the old standalone apps and remain legacy until owner-controlled cutover.

## Read first

1. `AGENTS.md`
2. `docs/SESSION_HANDOFF.md`
3. `docs/planning/DECISIONS.md`
4. `README.md`
5. `docs/engineering/PROJECT_FOLDER_BREAKDOWN.md` for where code lives
6. `docs/superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md` and `docs/runbooks/te-lab-components-catalog-v2-migration.md` for IM-014
7. `docs/superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md` for IM-015
8. `docs/superpowers/specs/2026-09-28-te-storage-room-design.md` and `docs/superpowers/plans/2026-09-28-te-storage-room.md` for EVCS Storage (formerly TE Storage)
9. `docs/runbooks/im-014-im-015-combined-desktop-smoke.md` for safe combined copied-data testing
10. `docs/superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md` for IM-011 sync behavior

Prefer live code and these current documents over older TE/ME planning copies.

## Product intent

- One install with a title-menu switcher across separate inventories.
- Labels: **TE Test Equipment** (with a **Calibration** view), **TE Lab Components**, **ME Storage**, **EVCS Storage** (module id `te-storage`).
- Modules keep separate databases, domain types, shared roots, and sync streams.
- Shared data stays per module on S:; sync is not a backup.

## Stable identity

| Item | Value |
|------|-------|
| Display | Inventory Management |
| Package | `inventory-management` `0.1.4` |
| Tauri id | `com.inventory.management` |
| TE Test Equipment DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| TE Lab Components DB | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| EVCS Storage DB | `%LOCALAPPDATA%\com.inventory.management\te-storage-room.feox` |
| Product share | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE shared root | `...\Inventory_Management_App\modules\TE_Test_Equipment` |
| Default Lab shared root | `...\Inventory_Management_App\modules\TE_Lab_Components` |
| Default Storage shared root | `...\Inventory_Management_App\modules\TE_Storage_Room` |
| Env prefix | `INVENTORY_MANAGEMENT_*` |

## Current state — 2026-10-15

**On `main`**

- IM-011 adaptive per-inventory sync lifecycle (TE, Lab, Storage).
- IM-012 shell/platform/module architecture.
- TE Test Equipment, TE Lab Components, and EVCS Storage are implemented; ME Storage is a placeholder.
- IM-014 generalized Lab parts catalog (sync schema v2 under `shared\catalog-v2\`) with the simple five-column workspace, bin picker, and order-request export.
- IM-015 TE Equipment/Calibration workspace and guarded calibration-roster tooling.
- Release **0.1.4** published (sync divergence fix + EVCS Storage rename). TE shared data was restored to the most complete copy on 2026-10-15; see SESSION_HANDOFF.

**Not completed**

- Live IM-014 Lab schema-v2 cutover and live IM-015 calibration roster cutover (separate owner ops).
- ME Storage implementation.

**Known code facts worth knowing**

- The operation-log sync engine is copied three times (`backend/src/sync/`, `modules/te_lab_components/sync/`, `modules/te_storage/sync/`); a fix in one is not a fix in the others.
- The TE Import button is intentionally hidden; the TE export menu's HTML option is a placeholder; window-geometry restore is intentionally disabled.

## Rules

- Verify critical paths before claiming success.
- Sync is **not** a backup.
- Use one writer per shared root.
- Do not rename the Tauri id without a migration plan.
- Do not run either live cutover during a normal development smoke.
- For copied-data desktop testing, set the absolute `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT`, redirect all three shared roots, and set `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED=0`; changing `LOCALAPPDATA` alone is insufficient on Windows.
- Do not point verification env vars at live `%LOCALAPPDATA%\com.inventory.management` or `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App`.
- Do not run the ignored `REPAIR_*` / `TE_LOAD_LIVE_LOCAL` tests unless the owner asks; they write live data.

## First reply in a new chat

Summarize the stable product identity, confirm 0.1.4 is the published release with TE Test Equipment, TE Lab Components, and EVCS Storage implemented, and note that the IM-014 / IM-015 live cutovers remain pending, separate owner operations.
