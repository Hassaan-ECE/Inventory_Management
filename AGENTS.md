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
| Package | `inventory-management` `0.2.0` |
| Tauri id | `com.inventory.management` |
| Local DB (all inventories, from 0.2) | `%LOCALAPPDATA%\com.inventory.management\inventory-management.feox`, one namespace per module id. On first 0.2 launch the 0.1.x files (`inventory.feox`, `te-lab-components.feox`, `te-storage-room.feox`) are merged in and moved to `pre-0.2-databases\` |
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

Modules stay **separate** under the hood: each has its own namespace in the one local database, its own shared root and sync stream on S:, and its own domain types. Do not merge row types.

## Current code layout

- `frontend/src/shell/` mounts the modules and owns the inventory switcher.
- `frontend/src/platform/` owns the module registry, the adaptive sync controller, the shared footer, and the updater UI/state.
- `frontend/src/modules/te-test-equipment/` owns the implemented TE UI, hooks, types, and desktop lifecycle.
- `frontend/src/modules/te-lab-components/` owns the implemented Lab UI, hooks, types, and desktop lifecycle.
- `frontend/src/modules/te-storage/` owns the implemented EVCS Storage inventory, with independent persistence and team sync.
- The backend mirrors the frontend. `backend/src/platform/` holds what every inventory shares: the one local database (`database.rs`), the generic store (`store/`), the entry sync engine (`sync/`), module ids and roots, the sync coordinator and watcher (`runtime/`), and native dialogs. `backend/src/modules/<inventory>/` holds each inventory's model, store, mutations, sync plug-in, and workbook export (TE also owns the calibration roster import). `backend/src/inventory_stores.rs` opens each module's namespace.
- `backend/src/api/commands/` receives every command with a `module_id` and routes it to the right module (`inventory.rs`, `lab_catalog.rs`, `calibration_roster.rs`).
- `backend/src/platform/store/` is the one storage layer (`ModuleStore<R>`, generic over the module's record type) and `backend/src/platform/sync/` is the one entry sync engine, used by TE Test Equipment and EVCS Storage. Each inventory plugs in through `SyncRecord` (its constants, field copy, validation, and `MergeRule`); the merge rules and the serialized entry-state fields stay exactly as 0.1.4 shipped them so mixed 0.1.4/0.2 teams converge. Lab's catalog sync is separate (`modules/te_lab_components/catalog_sync/`) but shares the platform's file helpers (`platform/sync/files.rs`) and status building.
- The frontend's bridge payload types are generated from the Rust types: `cargo test` writes `frontend/src/integrations/tauri/generated/` (ts-rs). Commit them with the Rust change; never edit them by hand.
- Full file map: `docs/engineering/PROJECT_FOLDER_BREAKDOWN.md`.

## Priorities

1. ~~Logical shell/platform/modules extract~~ **IM-012 done 2026-07-20** — plan: `docs/superpowers/plans/2026-07-20-platform-module-architecture-extract.md`
2. ~~IM-011 adaptive TE sync~~ **done 2026-07-20** — plan under `docs/superpowers/plans/`  
3. Shared roots: **product modules** (TE + Lab + Storage); InventoryApps is legacy-only after 2026-07-20 copy  

4. ~~Port TE Lab Components~~ **Phase C1 done 2026-07-20** — plan `docs/superpowers/plans/2026-07-20-te-lab-components-port.md`; ME remains deferred; TE Storage implemented 2026-09-28
5. **Phase D first team release** — plan: `docs/superpowers/plans/2026-07-20-first-team-release.md` (copy → product roots → installer; new updater keys only if enabling auto-update)
6. ~~IM-014 Lab Components catalog + grid storage redesign~~ **implemented 2026-07-25** — authority: `docs/superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md`. Copied-data owner acceptance and the coordinated live Lab cutover remain pending.
7. ~~IM-015 TE Test Equipment calibration workspace + roster tooling~~ **implemented 2026-07-25** — authority: `docs/superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md`. Equipment and Calibration use the same TE records; the backed-up live workbook cutover remains pending.
8. ~~0.2.0 cleanup and consolidation~~ **released 2026-10-19** — plan `docs/superpowers/plans/2026-10-16-v0.2-cleanup-and-consolidation.md`; merged to `main`, tag `v0.2.0`, published on GitHub and S:.

## Env (shared sync)

| Env | Purpose |
|-----|---------|
| `INVENTORY_MANAGEMENT_SHARED_ROOT` | Override default TE module shared root |
| `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT` | Override default Lab module shared root |
| `INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT` | Override default Storage module root (`modules/TE_Storage_Room`) |
| `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` | `0`/`false`/`no`/`off` to disable (all modules) |
| `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` | Optional HMAC (≥16 bytes), all modules |
| `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` | Absolute copied-data DB root for diagnostics/rehearsals only |
| `INVENTORY_MANAGEMENT_DEV_SHARE_TARGET` | Where `bun run dev:share` builds the fixture share (default `.tmp/dev-share`; must be empty or a previous dev share; the live share is refused) |

**Developer builds are isolated.** Any debug build (`bun run dev:desktop`, `cargo test`) uses `.tmp/dev-data` for databases and `.tmp/dev-share` for shared roots unless an env override above is set, and the footer shows `DEV build`. Only release builds touch `%LOCALAPPDATA%` and S:. `bun run dev:share` rebuilds `.tmp/dev-share` with fixtures for every inventory (`backend/src/dev_share/`); every new field or feature adds fixtures there in the same change.

The only live-data tool left is the shared-snapshot restore in `backend/src/sync/snapshot/restore.rs` (ignored test, dry run unless `RESTORE_TE_CONFIRM=publish`); never run it without owner approval.

**Sync is not a backup. One writer per shared root:** do not run either standalone against its pilot share while Inventory Management is open.

## Verification

Run real lint/test/build/smoke before claiming success. Prefer `docs/SESSION_HANDOFF.md` over stale TE planning docs copied into this tree.
