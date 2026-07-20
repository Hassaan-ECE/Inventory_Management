# Agent notes — Inventory Management

Read `docs/SESSION_HANDOFF.md` before non-trivial work.  
New chat: owner pastes `docs/SESSION_START_PROMPT.md`. See also `docs/CHAT_HANDOFF.md`.

## Workspace

- **Active app:** `C:\Projects\Active\Inventory_Management`
- **Standalone siblings (keep until cutover):** under `C:\Projects\Active\Inventory_Apps\`  
  - `TE\TE_Test_Equipment_Inventory`  
  - `TE\TE_Parts_Inventory`  
  - `ME\ME_Inventory`
- **Not the app:** older Lab-named leftover trees

## Identity (stable — do not change after first team install without a migration plan)

| Item | Value |
|------|--------|
| Name | Inventory Management |
| Package | `inventory-management` `0.1.0` |
| Tauri id | `com.inventory.management` |
| Local DB — TE Test Equipment | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| Local DB — TE Lab Components | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| Product share (installers + modules) | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE shared root (pilot) | `...\InventoryApps\TE_Test_Equipment_Inventory` — flip to product `modules\TE_Test_Equipment` at release |
| Default Lab shared root (pilot) | `...\InventoryApps\TE` — flip to product `modules\TE_Lab_Components` at release |

## Stack

Tauri 2, React 19, TypeScript, Vite, Tailwind v4, Bun, Rust, FeOxDB.  
Scaffold lineage: TE Test Equipment Inventory (switcher prototype + rebrand) ← ME family.

## Product shape

One installer; in-app switcher between inventories:

1. TE Test Equipment (implemented)
2. TE Lab Components (implemented)
3. ME Storage (placeholder)
4. TE Storage Room (placeholder)

Modules stay **separate** under the hood (own DBs, shared roots, domain types, and sync streams). Do not merge row types.

## Current code layout

- `frontend/src/shell/` owns product chrome, module selection, placeholders, and the module viewport.
- `frontend/src/platform/` owns module registry/persistence and the reusable adaptive sync controller.
- `frontend/src/modules/te-test-equipment/` owns the implemented TE UI, hooks, types, and desktop lifecycle.
- `frontend/src/modules/te-lab-components/` owns the implemented Lab UI, hooks, types, and desktop lifecycle.
- `frontend/src/modules/{me-storage,te-storage}/` remain placeholder hosts.
- `backend/src/inventory_stores.rs` owns the isolated TE/Lab FeOx handles; `backend/src/platform/` resolves module IDs and roots; runtime sync/watcher state is keyed by module.

## Priorities

1. ~~Logical shell/platform/modules extract~~ **IM-012 done 2026-07-20** — plan: `docs/superpowers/plans/2026-07-20-platform-module-architecture-extract.md`
2. ~~IM-011 adaptive TE sync~~ **done 2026-07-20** — plan under `docs/superpowers/plans/`  
3. TE path: **pilot default = legacy InventoryApps TE share** so `bun run desktop` has data; **release** = product `modules\TE_Test_Equipment` after copy  
4. ~~Port TE Lab Components~~ **Phase C1 done 2026-07-20** — plan `docs/superpowers/plans/2026-07-20-te-lab-components-port.md`; ME/Storage Room remain deferred
5. **Next:** owner live CRUD/restart QA, then release-path cutover and Phase D installer/updater work (new keys — do not reuse TE/ME keys)

## Env (shared sync)

| Env | Purpose |
|-----|---------|
| `INVENTORY_MANAGEMENT_SHARED_ROOT` | Override default TE module shared root |
| `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT` | Override default Lab module shared root |
| `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` | `0`/`false`/`no`/`off` to disable |
| `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` | Optional HMAC (≥16 bytes) |

**Sync is not a backup. One writer per shared root:** do not run either standalone against its pilot share while Inventory Management is open.

## Verification

Run real lint/test/build/smoke before claiming success. Prefer `docs/SESSION_HANDOFF.md` over stale TE planning docs copied into this tree.
