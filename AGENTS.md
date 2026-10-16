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
| Package | `inventory-management` `0.1.4` |
| Tauri id | `com.inventory.management` |
| Local DB — TE Test Equipment | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| Local DB — TE Lab Components | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| Local DB - EVCS Storage | `%LOCALAPPDATA%\com.inventory.management\te-storage-room.feox` |
| Product share (installers + modules) | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE shared root | `...\Inventory_Management_App\modules\TE_Test_Equipment` |
| Default Lab shared root | `...\Inventory_Management_App\modules\TE_Lab_Components` |
| Default Storage shared root | `...\Inventory_Management_App\modules\TE_Storage_Room` |

## Stack

Tauri 2, React 19, TypeScript, Vite, Tailwind v4, Bun, Rust, FeOxDB.  
Scaffold lineage: TE Test Equipment Inventory (switcher prototype + rebrand) ← ME family.

## Product shape

One installer; in-app switcher between inventories:

1. TE Test Equipment (implemented, with a Calibration view)
2. TE Lab Components (implemented)
3. EVCS Storage (implemented; module id `te-storage`, renamed from TE Storage in 0.1.4)

ME Storage and other unbuilt ideas live in `docs/engineering/REMOVED_AND_FUTURE.md`, not in code.

Modules stay **separate** under the hood (own DBs, shared roots, domain types, and sync streams). Do not merge row types.

## Current code layout

- `frontend/src/shell/` mounts the modules and owns the inventory switcher.
- `frontend/src/platform/` owns the module registry, the adaptive sync controller, the shared footer, and the updater UI/state.
- `frontend/src/modules/te-test-equipment/` owns the implemented TE UI, hooks, types, and desktop lifecycle.
- `frontend/src/modules/te-lab-components/` owns the implemented Lab UI, hooks, types, and desktop lifecycle.
- `frontend/src/modules/te-storage/` owns the implemented EVCS Storage inventory, with independent persistence and team sync.
- `backend/src/inventory_stores.rs` owns the isolated TE/Lab/Storage FeOx handles; `backend/src/platform/` resolves module IDs and roots; runtime sync/watcher state is keyed by module.
- `backend/src/api/commands.rs` receives every command with a `module_id` and routes it to the right module.
- The sync engine is **copied** per module, not shared: `backend/src/sync/` (TE), `backend/src/modules/te_lab_components/sync/` (Lab legacy v1), `backend/src/modules/te_storage/sync/` (Storage). Lab's active catalog sync is separate (`catalog_sync.rs`). Check every copy when fixing a sync bug.
- Full file map: `docs/engineering/PROJECT_FOLDER_BREAKDOWN.md`.

## Priorities

1. ~~Logical shell/platform/modules extract~~ **IM-012 done 2026-07-20** — plan: `docs/superpowers/plans/2026-07-20-platform-module-architecture-extract.md`
2. ~~IM-011 adaptive TE sync~~ **done 2026-07-20** — plan under `docs/superpowers/plans/`  
3. Shared roots: **product modules** (TE + Lab + Storage); InventoryApps is legacy-only after 2026-07-20 copy  

4. ~~Port TE Lab Components~~ **Phase C1 done 2026-07-20** — plan `docs/superpowers/plans/2026-07-20-te-lab-components-port.md`; ME remains deferred; TE Storage implemented 2026-09-28
5. **Phase D first team release** — plan: `docs/superpowers/plans/2026-07-20-first-team-release.md` (copy → product roots → installer; new updater keys only if enabling auto-update)
6. ~~IM-014 Lab Components catalog + grid storage redesign~~ **implemented 2026-07-25** — authority: `docs/superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md`. Copied-data owner acceptance and the coordinated live Lab cutover remain pending.
7. ~~IM-015 TE Test Equipment calibration workspace + roster tooling~~ **implemented 2026-07-25** — authority: `docs/superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md`. Equipment and Calibration use the same TE records; the backed-up live workbook cutover remains pending.

## Env (shared sync)

| Env | Purpose |
|-----|---------|
| `INVENTORY_MANAGEMENT_SHARED_ROOT` | Override default TE module shared root |
| `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT` | Override default Lab module shared root |
| `INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT` | Override default Storage module root (`modules/TE_Storage_Room`) |
| `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` | `0`/`false`/`no`/`off` to disable (all modules) |
| `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` | Optional HMAC (≥16 bytes), all modules |
| `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` | Absolute copied-data DB root for diagnostics/rehearsals only |

**Developer builds are isolated.** Any debug build (`bun run dev:desktop`, `cargo test`) uses `.tmp/dev-data` for databases and `.tmp/dev-share` for shared roots unless an env override above is set, and the footer shows `DEV build`. Only release builds touch `%LOCALAPPDATA%` and S:.

The only live-data tool left is the shared-snapshot restore in `backend/src/sync/snapshot/restore.rs` (ignored test, dry run unless `RESTORE_TE_CONFIRM=publish`); never run it without owner approval.

**Sync is not a backup. One writer per shared root:** do not run either standalone against its pilot share while Inventory Management is open.

## Verification

Run real lint/test/build/smoke before claiming success. Prefer `docs/SESSION_HANDOFF.md` over stale TE planning docs copied into this tree.
