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

Do **not** use `C:\Projects\Active\Inventory_Apps\TE\TE_Test_Equipment_Inventory` as the active app tree. Sibling standalones under `Inventory_Apps\` remain legacy until owner-controlled cutover.

## Read first

1. `AGENTS.md`
2. `docs/SESSION_HANDOFF.md`
3. `docs/planning/DECISIONS.md`
4. `README.md`
5. `docs/superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md` and `docs/runbooks/te-lab-components-catalog-v2-migration.md` for IM-014
6. `docs/superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md` for IM-015
7. `docs/runbooks/im-014-im-015-combined-desktop-smoke.md` for safe combined copied-data testing
8. `docs/superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md` for IM-011 sync behavior

Prefer live code and these current documents over older TE planning copies.

## Product intent

- One install with a hamburger switcher across separate inventories.
- Labels: **TE Test Equipment**, **TE Lab Components**, **ME Storage**, **TE Storage Room**.
- Modules keep separate databases, domain types, shared roots, and sync streams.
- Shared data stays per module on S:; sync is not a backup.

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
| Env prefix | `INVENTORY_MANAGEMENT_*` |

## Current state — 2026-07-25

**Implemented**

- IM-011 adaptive per-inventory sync lifecycle.
- IM-012 shell/platform/module architecture.
- Real TE Test Equipment and TE Lab Components modules; ME Storage and TE Storage Room remain placeholders.
- First team release `0.1.0`, signed installer, GitHub Release, and updater metadata.
- IM-014 generalized Lab parts catalog: flexible attributes, multi-location stock, grid bins, migration, Lab sync schema v2, and six-sheet export.
- IM-015 TE Equipment/Calibration workspace and guarded calibration-roster preview/review/commit tooling.
- Combined integration branch: `feature/im-014-im-015-integration` in `C:\Projects\Active\Inventory_Management_IM014_IM015`.
- Combined automated gates and an explicit-root isolated desktop startup smoke pass; manual owner workflow QA remains.

**Not completed**

- Merge of the combined integration branch into `main`.
- Owner copied-data rehearsal and live IM-014 Lab schema-v2 cutover.
- Owner-backed-up single-writer IM-015 calibration roster cutover.
- ME Storage and TE Storage Room implementations.

## Next priorities

1. Run the prepared combined manual smoke with `.tmp\run-combined-desktop.ps1` in the integration worktree.
2. Verify TE Equipment/Calibration and Lab catalog/grid/migration workflows coexist across module switching and restart.
3. Merge into `main` only after owner acceptance.
4. Perform IM-014 and IM-015 live cutovers separately under their owner runbooks, backups, and one-writer rules.

## Rules

- Verify critical paths before claiming success.
- Sync is **not** a backup.
- Use one writer per shared root.
- Do not rename the Tauri id without a migration plan.
- Do not run either live cutover during a normal development smoke.
- For copied-data desktop testing, set the absolute `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT`; changing `LOCALAPPDATA` alone is insufficient on Windows.

## First reply in a new chat

Summarize the stable product identity, that IM-011/IM-012/C1/IM-014/IM-015 implementation is complete, that the combined integration branch is the current test target, and that both live data cutovers remain pending and separate.
