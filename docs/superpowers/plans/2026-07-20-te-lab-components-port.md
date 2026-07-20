# TE Lab Components Port (Phase C1) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port **TE Lab Components** into Inventory Management as a second real module (same shell style as TE Test Equipment), backed by its own local DB + shared root, without merging schemas into Test Equipment and without shipping ME Storage / TE Storage Room yet.

**Architecture:** Keep one Tauri app. Add `ModuleId::TeLabComponents` end-to-end. Lab Components is a **different domain** from Test Equipment (no calibration; `verifiedInSurvey` instead of cal fields; sync schema v1). Frontend: new desktop host under `frontend/src/modules/te-lab-components/` cloned/adapted from standalone `TE_Parts_Inventory` and restyled to match the unified shell (single top bar, switcher, Shared pill, theme). Backend: second FeOx DB + second shared-root + session map entry; inventory commands become **module-scoped**. Do **not** force Lab Components rows through the Test Equipment entry model.

**Tech Stack:** Existing Inventory Management stack. Source of truth for Lab Components domain/UX:  
`C:\Projects\Active\Inventory_Apps\TE\TE_Parts_Inventory`  
Product shell/style reference:  
`C:\Projects\Active\Inventory_Management` (`shell/`, `modules/te-test-equipment/`).

**Decisions:** IM-004 (separate modules), IM-005 (labels), IM-012 (platform/modules), IM-013 (pilot on live InventoryApps path OK until release copy), IM-011 (reuse adaptive sync controller per active module).

**Status:** Ready for implementation (2026-07-20).

**Release note (owner intent):** After Lab Components works, owner may ship a team release with **TE Test Equipment + TE Lab Components** only; ME Storage and TE Storage Room stay placeholders until a later update.

---

## 0. Non-goals

- Porting ME Storage or TE Storage Room in this plan.
- Merging Lab Components and Test Equipment into one table/schema/DB.
- Changing Tauri id / package name / AppData product id (`com.inventory.management` stays).
- Multi-crate Cargo / npm workspaces.
- Updater keys / installer packaging (Phase D; separate).
- Importing Test Equipment calibration UI into Lab Components.
- Dual-writing with standalone TE Lab Components Inventory while piloting (one writer per shared root).

---

## 1. Domain differences (must preserve)

| Item | TE Test Equipment (already in app) | TE Lab Components (to port) |
|------|------------------------------------|-----------------------------|
| Standalone tree | TE_Test_Equipment_Inventory lineage | `Inventory_Apps\TE\TE_Parts_Inventory` |
| Product label | TE Test Equipment | TE Lab Components |
| `ModuleId` / systemId | `te-test-equipment` | `te-lab-components` |
| Pilot shared root | `...\InventoryApps\TE_Test_Equipment_Inventory` | `...\InventoryApps\TE` |
| Product share folder (release) | `modules\TE_Test_Equipment` | `modules\TE_Lab_Components` |
| Entry model | Calibration fields, import provenance, verified timestamps | **No calibration**; `verifiedInSurvey: bool` |
| Columns | Includes cal health etc. | Verified, qty, maker, model, desc, location, links… (parts columns) |
| Sync schema version (standalone) | 2 | **1** |
| Shared env (standalone) | was TE_TEST_EQUIPMENT_* | `TE_LAB_COMPONENTS_SHARED_ROOT` |
| Local AppData DB (unified) | Keep existing TE file | **New** second DB file (see Task 2) |

**Shared op format:** Treat Lab Components shared folder as its own stream. Do not mix ops with Test Equipment.

---

## 2. Target layout

### Frontend

```text
frontend/src/modules/te-lab-components/
  index.ts                    # desktop host, implemented: true
  TeLabComponentsView.tsx     # same role as TeTestEquipmentView
  types.ts                    # from standalone parts types (no cal)
  data/mockInventory.ts
  lib/                        # columns, filtering, sorting, counts (parts-shaped)
  components/                 # table, dialogs, header, search — shell style match
  components/shell/
    useDesktopInventory.ts    # moduleId te-lab-components; own bridge calls
    ...
```

### Backend

```text
backend/src/
  platform/module_id.rs       # add TeLabComponents
  platform/shared_root.rs     # root per ModuleId
  modules/                    # NEW logical split (Rust modules, one crate)
    te_test_equipment/        # optional extract of current domain — only if needed
    te_lab_components/        # model, storage keys, sync schema constants
  storage/                    # InventoryDb becomes multi-handle OR registry
  api/commands.rs             # module_id on inventory commands
  runtime/shared_watcher.rs   # already map-keyed; populate Lab entry
```

Prefer the **smallest** backend change that gives two isolated stores. Recommended:

```rust
// Conceptual
struct InventoryStores {
    te_test_equipment: InventoryDb,      // existing open path
    te_lab_components: InventoryDb,      // new path
}
```

DB filenames under `%LOCALAPPDATA%\com.inventory.management\`:

| Module | Suggested file |
|--------|----------------|
| TE Test Equipment | keep current `inventory.feox` (no migration) |
| TE Lab Components | `te-lab-components.feox` (new) |

Do **not** rename the existing TE file (avoids wiping owner pilot data).

---

## 3. Shared roots (pilot)

```rust
// platform/shared_root.rs — illustrative
ModuleId::TeTestEquipment => // existing pilot path
  r"...InventoryApps\TE_Test_Equipment_Inventory"
ModuleId::TeLabComponents =>
  r"S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE"
```

Optional env overrides (diagnostic):

| Env | Scope |
|-----|--------|
| `INVENTORY_MANAGEMENT_SHARED_ROOT` | **TE Test Equipment only** (keep current behavior) |
| `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT` | Lab Components only (new) |

Do not use one env for both modules.

Release later: both defaults flip to product `modules\...` after data copy (owner-driven).

---

## 4. Command / bridge contract

All inventory lifecycle and CRUD commands take `moduleId: string` (system id).

Examples:

```ts
activateInventorySync(moduleId: ModuleId): Promise<string>
loadInventory(moduleId: ModuleId): Promise<InventorySyncResult>
syncInventory(moduleId: ModuleId, sessionId: string): Promise<InventorySyncResult | null>
deactivateInventorySync(moduleId: ModuleId, sessionId: string): Promise<boolean>
createEntry(moduleId: ModuleId, input: LabEntryInput): Promise<...>
// same for update/delete/archive/toggleVerified/query/export as applicable
```

**Compatibility:** TE Test Equipment frontend must pass `"te-test-equipment"` on every call. Update existing bridge + TE hooks.

Shared-change events already carry `systemId` — Lab frontend ignores non-`te-lab-components` events; TE ignores non-TE.

Import commands: Lab Components standalone may lack the TE Test Equipment Excel import pipeline — **port only what Lab Components already has**. If standalone has no import UI, do not invent TE import for Lab. Check standalone shell for Import; Test Equipment has import — Lab may not. Prefer parity with standalone Lab, not with Test Equipment.

Excel export: port Lab export if present in standalone; match TE shell placement (Export menu).

---

## 5. UI / style rules (owner requirement)

Match Inventory Management shell styling already used by TE Test Equipment:

- Single unified top bar when module active: **Switcher · Shared/Local · Update · Scope · Theme · Export · Add**
- Same fonts, spacing, Shared pill, dark theme toggle behavior
- Placeholder modules (ME, Storage Room) unchanged
- Lab-specific: **no calibration filters/columns/dialog sections**
- Verified column uses `verifiedInSurvey` (checkbox semantics from standalone)
- Loading copy: “Loading TE Lab Components…” not Test Equipment wording
- Preference keys **per module** (theme can stay global; column visibility / filters storage keys must not collide with TE)

Copy shell chrome patterns from:

- `frontend/src/modules/te-test-equipment/components/InventoryHeader.tsx`
- `frontend/src/shell/*`

Copy domain UI behavior from:

- `Inventory_Apps\TE\TE_Parts_Inventory\frontend\src\features\inventory\**`

---

## 6. Adaptive sync (IM-011)

- Reuse `AdaptiveSyncController` from platform (do not reimplement).
- Each desktop module runs its own controller only while `active === true`.
- Shell already keeps TE mounted with `active={false}` for cache; do the **same** for Lab Components (mount both desktop hosts; only active module schedules sync).
- Backend session map: separate tokens per `ModuleId` (already designed in IM-012).

---

## 7. Implementation tasks

### Task 1: Expand ModuleId + shared roots (backend)

**Files:**
- Modify: `backend/src/platform/module_id.rs`
- Modify: `backend/src/platform/shared_root.rs`
- Tests: unit tests for parse + default roots

- [ ] Add `TeLabComponents` variant; `as_system_id_str` → `"te-lab-components"`; parse both modules.
- [ ] Default shared root for Lab → InventoryApps `\TE` (pilot).
- [ ] Keep TE Test Equipment root as current pilot constant.
- [ ] `cargo test --lib platform::`

### Task 2: Dual InventoryDb registry

**Files:**
- Modify: `backend/src/storage/*` open paths
- Modify: `backend/src/lib.rs` setup/manage/flush

- [ ] Open existing TE DB unchanged.
- [ ] Open/create `te-lab-components.feox` under same app data dir.
- [ ] Manage a registry (`InventoryStores` or similar) instead of single `State<InventoryDb>` where needed.
- [ ] All command handlers resolve DB by `ModuleId`.
- [ ] Flush **both** on exit.

### Task 3: Lab Components domain + storage + sync apply path

**Files:**
- Port model from standalone `TE_Parts_Inventory` backend domain/storage/sync pieces needed for CRUD + shared sync schema v1.
- Isolate so TE Test Equipment calibration model is untouched.

Minimum vertical slice:

- [ ] Serialize/deserialize Lab entry
- [ ] load_entries / create / update / delete / archive / toggle verified_in_survey
- [ ] run_shared_sync for Lab root (schema v1, Lab DB)
- [ ] Unit/integration tests adapted from standalone shared_sync_flow **or** new focused tests for Lab module

**Hard rule:** Lab shared sync must not read/write TE Test Equipment ops directory.

### Task 4: Module-scoped Tauri commands + bridge

**Files:**
- `backend/src/api/commands.rs` (+ export if needed)
- `frontend/src/integrations/tauri/tauriInventoryBridge.ts`
- `frontend/src/integrations/tauri/desktop-bridge.d.ts`
- `frontend/src/integrations/tauri/bridgeGuards.ts`
- Update TE Test Equipment hooks to pass `moduleId: "te-test-equipment"`

- [ ] Add `module_id` argument to activate/load/sync/deactivate/CRUD/query (and export if module-specific).
- [ ] Reject unknown module ids.
- [ ] Frontend bridge wrappers accept `ModuleId`.
- [ ] Existing TE shell tests updated for new signatures.
- [ ] `bun run test` green for TE suite before adding Lab UI.

### Task 5: Frontend Lab Components module (desktop host)

**Files:** create under `frontend/src/modules/te-lab-components/**`

- [ ] Port types/columns/filters/table/dialog from standalone (no cal).
- [ ] `TeLabComponentsView` mirrors TE Test Equipment view structure + `DesktopModuleViewProps`.
- [ ] Unified header like TE (switcher + Shared + actions + theme).
- [ ] `useDesktopInventory({ active, moduleId: "te-lab-components" })` using AdaptiveSyncController.
- [ ] `index.ts`: `implemented: true`, `kind: "desktop"`.
- [ ] Registry already imports host — ensure order: TE Test Equipment, Lab Components, ME, Storage.

### Task 6: Shell integration smoke paths

- [ ] Switching TE ↔ Lab: only active module syncs; inactive hard-deactivates.
- [ ] Cached rows preserved when switching away (same pattern as TE ↔ placeholder).
- [ ] ME / Storage Room still placeholders.
- [ ] Document title / switcher labels correct.

### Task 7: Tests

Frontend:

- [ ] Registry: Lab `implemented: true`, host `kind: "desktop"`.
- [ ] Shell: switch to Lab Components shows Lab UI (mock or bridge mocks).
- [ ] Lab filtering/columns/dialog smoke tests (port/adapt from standalone if cheap).
- [ ] TE Test Equipment regression suite still passes.

Backend:

- [ ] ModuleId parse
- [ ] Lab shared root default
- [ ] Session map: TE and Lab tokens independent (stale TE cannot kill Lab)
- [ ] Lab CRUD + sync smoke (temp dirs)

### Task 8: Verification gate + docs

```powershell
cd C:\Projects\Active\Inventory_Management
bun run test
bun run lint
bun run build
cd backend
cargo test --lib
cargo test --test shared_sync_flow
cargo clippy --all-targets --all-features -- -D warnings
```

Desktop (single instance):

```powershell
bun run desktop
```

Check:

1. TE Test Equipment still loads live data (Shared).
2. Switch to **TE Lab Components** — loads live Lab share (`InventoryApps\TE`), table usable.
3. Add/edit one Lab entry; persists after restart.
4. Switch away → placeholder or TE; no dual watcher thrash; no console Illegal invocation.
5. One writer: do not run standalone Lab Components app against `InventoryApps\TE` during pilot.

Update:

- `docs/SESSION_HANDOFF.md` — Lab implemented; verification notes
- `docs/SESSION_START_PROMPT.md` / `AGENTS.md` — two modules live
- This plan status → Implemented
- Roadmap Phase C1 done; ME/Storage deferred to later update

---

## 8. Risk register

| Risk | Mitigation |
|------|------------|
| Schema cross-contamination | Separate DBs, separate shared roots, separate domain types |
| Breaking TE after command signature change | Pass moduleId from TE first; green TE tests before Lab UI |
| Sync schema v1 vs v2 mix-up | Explicit schema constant per module in sync code paths |
| Preference key collisions | Prefix localStorage keys with module id |
| Scope creep (ME/Storage) | Explicit non-goal; leave placeholder hosts |
| Dual writers on InventoryApps\TE | Document; owner closes standalone Lab app |

---

## 9. Definition of done

1. Switcher: TE Test Equipment + **TE Lab Components** both real; ME + Storage Room placeholders.
2. Lab uses live pilot share `InventoryApps\TE` with Shared status when S: healthy.
3. TE Test Equipment behavior and data path unchanged except moduleId plumbing.
4. Same visual language as current unified app.
5. Automated gates green; desktop smoke documented in handoff.
6. No ME/Storage port; no identity change; no multi-crate.

---

## 10. Implementor paste prompt

```text
You are the IMPLEMENTATION agent for Inventory Management — Phase C1 TE Lab Components port.

## Workspace
C:\Projects\Active\Inventory_Management

## Source reference (Lab Components domain/UI)
C:\Projects\Active\Inventory_Apps\TE\TE_Parts_Inventory
(read-only reference; implement in Inventory_Management only)

## Authority
1. AGENTS.md
2. docs/SESSION_HANDOFF.md
3. docs/planning/DECISIONS.md (IM-004, IM-011, IM-012, IM-013)
4. docs/superpowers/plans/2026-07-20-te-lab-components-port.md  ← FOLLOW THIS PLAN
5. Style reference: frontend/src/shell + frontend/src/modules/te-test-equipment

## Goal
Port TE Lab Components as a second real module with its own FeOx DB, shared root (pilot: InventoryApps\TE), domain model (no calibration; verifiedInSurvey), and UI matching the unified app shell style. TE Test Equipment must keep working. Leave ME Storage and TE Storage Room as placeholders.

## Method
- Tasks 1→8 in the plan order.
- Module-scoped commands (moduleId on load/sync/CRUD).
- Separate DBs: keep existing inventory.feox for TE; new te-lab-components.feox for Lab.
- Reuse AdaptiveSyncController; do not reimplement IM-011.
- Do not merge schemas. Do not port ME/Storage. Do not change Tauri id.
- One writer per shared root (warn in docs; do not dual-run standalone Lab).

## Verify before done
bun run test && bun run lint && bun run build
cargo test --lib; cargo test --test shared_sync_flow; clippy -D warnings
bun run desktop smoke: TE + Lab both load; switcher; CRUD; placeholders still placeholders
Update SESSION_HANDOFF with results; mark plan implemented

## First reply
Restate goal and domain differences (no cal vs TE), then start Task 1 (ModuleId + shared roots) with failing tests.
```

---

## 11. Owner release intent (context only)

After this port is verified:

1. Owner may proceed toward **first team release** with TE Test Equipment + TE Lab Components.
2. Near release: copy latest shared data into product `modules\TE_*` paths and point defaults there.
3. ME Storage + TE Storage Room → later update (new plan then).
