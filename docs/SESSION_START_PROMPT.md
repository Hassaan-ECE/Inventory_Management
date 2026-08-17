# Session start prompt — Inventory Management

Copy everything below the line into a **new chat** with workspace:

```text
C:\Projects\Active\Inventory_Management
```

For the Lab simple-workspace feature branch/worktree instead:

```text
C:\Users\Syed.h.Shah\.grok\worktrees\active-inventory-management\subagent-01a01092-c1e3-7292-9e15-0f9ec316cc30
```

Branch: `feature/lab-simple-workspace`

---

You are working on **Inventory Management**, a unified multi-inventory Windows desktop product.

## Open this workspace first

```text
C:\Projects\Active\Inventory_Management
```

For unfinished Lab simple-workspace work, open the feature worktree above on `feature/lab-simple-workspace` instead.

Do **not** use `C:\Projects\Active\Inventory_Apps\TE\TE_Test_Equipment_Inventory` as the active app tree. Sibling standalones under `Inventory_Apps\` remain legacy until owner-controlled cutover.

## Read first

1. `AGENTS.md`
2. `docs/SESSION_HANDOFF.md`
3. `docs/planning/DECISIONS.md`
4. `README.md`
5. `docs/superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md` and `docs/runbooks/te-lab-components-catalog-v2-migration.md` for IM-014
6. `docs/superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md` for IM-015
7. `docs/superpowers/specs/2026-07-28-te-lab-components-simple-workspace-and-order-export-design.md` and `docs/superpowers/plans/2026-08-17-te-lab-components-simple-workspace-and-order-export.md` for the Lab simple workspace + order export
8. `docs/runbooks/im-014-im-015-combined-desktop-smoke.md` for safe combined copied-data testing
9. `docs/superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md` for IM-011 sync behavior

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
| Package | `inventory-management` `0.1.2` |
| Tauri id | `com.inventory.management` |
| TE Test Equipment DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| TE Lab Components DB | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| Product share | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE shared root | `...\Inventory_Management_App\modules\TE_Test_Equipment` |
| Default Lab shared root | `...\Inventory_Management_App\modules\TE_Lab_Components` |
| Env prefix | `INVENTORY_MANAGEMENT_*` |

## Current state — 2026-08-17

**On `main` (active product checkout)**

- IM-011 adaptive per-inventory sync lifecycle.
- IM-012 shell/platform/module architecture.
- Real TE Test Equipment and TE Lab Components modules; ME Storage and TE Storage Room remain placeholders.
- Signed team installer **0.1.2** on S: (`Inventory Management_0.1.2_x64-setup.exe`).
- IM-014 generalized Lab parts catalog: flexible attributes, multi-location stock, grid bins, migration, Lab sync schema v2, and six-sheet export.
- IM-015 TE Equipment/Calibration workspace and guarded calibration-roster preview/review/commit tooling.
- Simple Lab workspace: five-column table, on-demand bin picker, review mode, order-request Excel.

**Not completed**

- Live IM-014 Lab schema-v2 cutover and live IM-015 calibration roster cutover (separate owner ops).
- ME Storage and TE Storage Room implementations.

## Next priorities

1. Team install from S: or in-app Update to **0.1.2**.
2. Perform IM-014 and IM-015 live cutovers separately under their owner runbooks, backups, and one-writer rules.

## Rules

- Verify critical paths before claiming success.
- Sync is **not** a backup.
- Use one writer per shared root.
- Do not rename the Tauri id without a migration plan.
- Do not run either live cutover during a normal development smoke.
- For copied-data desktop testing, set the absolute `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT`; changing `LOCALAPPDATA` alone is insufficient on Windows.
- Do not point verification env vars at live `%LOCALAPPDATA%\com.inventory.management` or `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App`.

## First reply in a new chat

Summarize the stable product identity; that IM-011/IM-012/C1/IM-014/IM-015 are on main; that Lab simple workspace + order export is implemented on `feature/lab-simple-workspace` with automated gates green and isolated startup smoke only; and that owner acceptance, version bump, installer staging, and live cutovers remain pending and separate.
