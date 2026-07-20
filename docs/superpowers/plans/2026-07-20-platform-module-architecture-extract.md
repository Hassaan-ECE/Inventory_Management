# Platform + Module Architecture Extract (IM-012) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure Inventory Management so a **product shell + shared platform** hosts **separate inventory modules**, with TE Test Equipment as the first real module and placeholders for the rest — without merging schemas or changing product identity.

**Architecture:** Keep one Tauri app and one Git repo. Extract **logical** boundaries first (folders, registry, lifecycle seams). Do **not** invent multi-crate Cargo workspaces or multi-package npm workspaces in this plan unless a later task proves a hard need. Backend remains one crate; introduce `ModuleId` + session/root resolution hooks so multi-module is possible without rewriting sync formats. Frontend gains `shell` / `platform` / `modules/*` layout and a single module registry the switcher already implies.

**Tech Stack:** Tauri 2, React 19, TypeScript, Vite, Bun, Rust, FeOxDB (unchanged). Identity: `com.inventory.management` / package `inventory-management` `0.1.0` — **do not change**.

**Decisions:** [IM-004](../../planning/DECISIONS.md), [IM-012](../../planning/DECISIONS.md), [IM-013](../../planning/DECISIONS.md), [IM-011](../../planning/DECISIONS.md) (adaptive sync already shipped for TE).  
**Roadmap context:** [2026-07-20-desktop-capability-and-roadmap.md](./2026-07-20-desktop-capability-and-roadmap.md) Phase **B**.

**Status:** Implemented and verified 2026-07-20 (Tasks 1–7 complete; measured results in `docs/SESSION_HANDOFF.md`).

---

## 0. Non-goals (do not do in this plan)

- Porting Lab Components / ME Storage / TE Storage Room data layers (Phase C).
- Pointing TE at legacy InventoryApps shares by default (**IM-013** forbids this as product default).
- Schema merge across modules; one shared table for all inventories.
- Updater keys, installer, GitHub Releases (Phase D).
- Renaming Tauri id, AppData path, or package name.
- Multi-package monorepo tooling (pnpm/bun workspaces, cargo workspace members) **unless** a thin path alias is insufficient — prefer folder boundaries first.
- Changing shared op-log / snapshot file formats.
- Re-implementing IM-011 adaptive scheduling from scratch — **reuse** `AdaptiveSyncController`.

---

## 1. Target layout (logical monorepo)

### Frontend (`frontend/src/`)

```text
frontend/src/
  app/                          # entry, branding, global CSS (exists)
  shell/                        # product chrome that hosts modules
    InventoryShell.tsx          # moved/refactored from features/inventory/components/
    InventorySystemPlaceholder.tsx
    useShellActiveModule.ts     # active system id + persistence (thin)
  platform/                     # reusable across modules
    sync/
      adaptiveSyncController.ts # moved from features/inventory/sync/
    modules/
      types.ts                  # ModuleId, InventoryModuleDefinition, InventoryModuleHost
      registry.ts               # INVENTORY_MODULES list + lookups
      persistence.ts            # localStorage active module (from inventorySystems.ts)
    ui/                         # optional later: re-export shared/components/ui
  modules/
    te-test-equipment/          # ONLY fully wired module this plan
      index.ts                  # public module surface for shell
      types.ts                  # TE entry types (from features/inventory/types.ts)
      components/               # TE table, dialogs, shell hooks currently under features/inventory
      lib/                      # filtering, columns, calibration, etc.
      data/                     # mock data if still needed
    te-lab-components/
      index.ts                  # placeholder module definition only
    me-storage/
      index.ts
    te-storage/
      index.ts
  integrations/tauri/           # product bridge (stays); may gain moduleId on payloads later
  shared/                       # keep existing UI primitives for now (or re-export via platform/ui)
```

**Path alias:** keep `@/` → `frontend/src/`. Prefer new imports like `@/platform/...` and `@/modules/te-test-equipment/...`. Temporary re-export shims under old `@/features/inventory/...` paths are allowed for one PR to keep tests green, then delete shims in a follow-up task.

### Backend (`backend/src/`)

Stay **one crate**. Introduce clear modules (Rust modules, not crates):

```text
backend/src/
  platform/                     # NEW (or rename gradually)
    module_id.rs                # ModuleId enum / constants matching frontend
    shared_root.rs              # resolve root by ModuleId (TE → DEFAULT path; others Err/unsupported)
  runtime/
    shared_watcher.rs           # evolve TE-global session → HashMap<ModuleId, Session> (TE-only entries OK)
  ... existing domain/storage/sync/api ...
```

Do **not** split into `backend/crates/*` in this plan.

### Product share (S:) — already decided

```text
...\Inventory_Management_App\modules\
  TE_Test_Equipment\            # IM-013 — TE only
  TE_Lab_Components\            # reserved
  ME_Storage\
  TE_Storage_Room\
```

Code defaults must keep TE on `TE_Test_Equipment` only.

---

## 2. Core interfaces (implement these exactly)

### 2.1 Frontend module registry

Create `frontend/src/platform/modules/types.ts`:

```ts
export type ModuleId =
  | "te-test-equipment"
  | "te-lab-components"
  | "me-storage"
  | "te-storage";

export interface InventoryModuleDefinition {
  id: ModuleId;
  label: string;
  /** True only when module owns real UI + desktop lifecycle. */
  implemented: boolean;
  /** Product share folder name under modules\ (documentation + future root resolution). */
  sharedFolderName: string;
}

/**
 * Host contract the shell uses to mount a module.
 * Placeholders return kind: "placeholder".
 * TE returns kind: "desktop" and renders the TE surface.
 */
export type InventoryModuleHost =
  | { kind: "placeholder"; definition: InventoryModuleDefinition }
  | {
      kind: "desktop";
      definition: InventoryModuleDefinition;
      /** Render function or component reference used by shell. */
      MainView: React.ComponentType<{ active: boolean }>;
    };
```

Create `frontend/src/platform/modules/registry.ts`:

```ts
import type { InventoryModuleDefinition, InventoryModuleHost, ModuleId } from "./types";
import { teTestEquipmentHost } from "@/modules/te-test-equipment";
import { teLabComponentsHost } from "@/modules/te-lab-components";
import { meStorageHost } from "@/modules/me-storage";
import { teStorageHost } from "@/modules/te-storage";

export const INVENTORY_MODULE_HOSTS: readonly InventoryModuleHost[] = [
  teTestEquipmentHost,
  teLabComponentsHost,
  meStorageHost,
  teStorageHost,
] as const;

export function getModuleHost(id: ModuleId): InventoryModuleHost {
  const found = INVENTORY_MODULE_HOSTS.find((h) => h.definition.id === id);
  if (!found) {
    throw new Error(`Unknown module id: ${id}`);
  }
  return found;
}

export function listModuleDefinitions(): readonly InventoryModuleDefinition[] {
  return INVENTORY_MODULE_HOSTS.map((h) => h.definition);
}
```

Placeholder host factory:

```ts
export function placeholderHost(
  definition: InventoryModuleDefinition,
): InventoryModuleHost {
  return { kind: "placeholder", definition };
}
```

### 2.2 Shell wiring rules

- Switcher lists `listModuleDefinitions()` (labels unchanged from today).
- When `host.kind === "placeholder"`, show existing placeholder panel.
- When `host.kind === "desktop"`, render `host.MainView` with `active={activeModuleId === host.definition.id}`.
- TE lifecycle (`useDesktopInventory` / adaptive sync) must run **only** while TE is active (`active === true`), matching current `teActive` behavior.
- Preserve cached TE rows when switching away (already implemented — do not regress).

### 2.3 Backend `ModuleId` (Rust)

Create `backend/src/platform/module_id.rs` (and `platform/mod.rs`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleId {
    TeTestEquipment,
    // Future: TeLabComponents, MeStorage, TeStorage,
}

impl ModuleId {
    pub const TE_SYSTEM_ID_STR: &'static str = "te-test-equipment";

    pub fn as_system_id_str(self) -> &'static str {
        match self {
            ModuleId::TeTestEquipment => Self::TE_SYSTEM_ID_STR,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "te-test-equipment" => Some(ModuleId::TeTestEquipment),
            _ => None,
        }
    }
}
```

Shared root resolution for TE only (move/wrap `DEFAULT_SHARED_ROOT` logic):

```rust
pub fn default_shared_root(module: ModuleId) -> PathBuf {
    match module {
        ModuleId::TeTestEquipment => PathBuf::from(DEFAULT_SHARED_ROOT),
        // No other arms until ports exist.
    }
}
```

**Session map direction (Task 5):** replace TE-global single session with `HashMap<ModuleId, SessionState>` (or equivalent). Only TE is inserted/activated in this plan. Stale deactivation must still not kill a newer TE session (existing tests must stay green).

---

## 3. Current → target mapping (what moves)

| Today | After |
|-------|--------|
| `frontend/src/features/inventory/lib/inventorySystems.ts` | `platform/modules/*` + module `index.ts` hosts |
| `frontend/src/features/inventory/sync/adaptiveSyncController.ts` | `platform/sync/adaptiveSyncController.ts` |
| `frontend/src/features/inventory/components/InventoryShell.tsx` | `shell/InventoryShell.tsx` (or shell hosts + TE main view split) |
| `frontend/src/features/inventory/**` TE UI/hooks/types | `modules/te-test-equipment/**` |
| `backend/src/sync/types.rs` `DEFAULT_SHARED_ROOT` | still TE default; resolution goes through `platform` helper |
| `backend/src/runtime/shared_watcher.rs` TE-global session | `ModuleId → session` map (TE-only entries) |

---

## 4. Implementation tasks

### Task 1: Platform module types + registry tests (frontend-first)

**Files:**
- Create: `frontend/src/platform/modules/types.ts`
- Create: `frontend/src/platform/modules/registry.ts`
- Create: `frontend/src/platform/modules/persistence.ts` (move storage helpers from `inventorySystems.ts`)
- Create: `frontend/src/modules/te-lab-components/index.ts`
- Create: `frontend/src/modules/me-storage/index.ts`
- Create: `frontend/src/modules/te-storage/index.ts`
- Create: `frontend/src/modules/te-test-equipment/index.ts` (temporary host pointing at existing shell until Task 3)
- Create: `frontend/tests/platform-module-registry.test.ts`
- Modify: keep `inventorySystems.ts` as thin re-export shim exporting old names → new types (compat)

- [ ] **Step 1: Add failing registry test**

```ts
// frontend/tests/platform-module-registry.test.ts
import { describe, expect, it } from "vitest";
import { getModuleHost, listModuleDefinitions } from "@/platform/modules/registry";

describe("platform module registry", () => {
  it("lists four modules with only TE implemented", () => {
    const defs = listModuleDefinitions();
    expect(defs).toHaveLength(4);
    expect(defs.filter((d) => d.implemented)).toEqual([
      expect.objectContaining({ id: "te-test-equipment", sharedFolderName: "TE_Test_Equipment" }),
    ]);
  });

  it("returns placeholder hosts for non-TE modules", () => {
    expect(getModuleHost("me-storage").kind).toBe("placeholder");
    expect(getModuleHost("te-test-equipment").kind).toBe("desktop");
  });
});
```

- [ ] **Step 2: Run test — expect FAIL** (module not found)

```powershell
cd C:\Projects\Active\Inventory_Management
bun run test -- frontend/tests/platform-module-registry.test.ts
```

- [ ] **Step 3: Implement types, placeholder hosts, TE stub host, registry, persistence**

TE stub host may temporarily use a noop `MainView` **only if** shell is not yet switched; preferred: Task 1 TE host still imports current inventory surface so app keeps working. Safer sequence: Task 1 registry + placeholders + **TE host wraps existing `InventoryShell` body** only after Task 3 — for Task 1, TE host can be `kind: "desktop"` with `MainView` importing a tiny `TeModuleRoot` that re-exports current tree.

Minimum viable Task 1: registry + placeholders + TE definition `implemented: true` with `MainView` that renders `null` **is not acceptable** if it breaks app. Keep `InventoryShell` as entry until Task 3; registry tests alone are enough for Task 1 if TE host is defined but not yet mounted by App.

- [ ] **Step 4: Re-export shim** from `features/inventory/lib/inventorySystems.ts` so existing imports compile:

```ts
export type { ModuleId as InventorySystemId } from "@/platform/modules/types";
// re-export getInventorySystem, INVENTORY_SYSTEMS mapped from listModuleDefinitions, etc.
```

- [ ] **Step 5: Run registry test + full frontend tests; commit**

```powershell
bun run test -- frontend/tests/platform-module-registry.test.ts
bun run test
git add frontend/src/platform frontend/src/modules frontend/src/features/inventory/lib/inventorySystems.ts frontend/tests/platform-module-registry.test.ts
git commit -m "feat: add platform inventory module registry (IM-012)"
```

---

### Task 2: Move adaptive sync controller to platform

**Files:**
- Move: `frontend/src/features/inventory/sync/adaptiveSyncController.ts` → `frontend/src/platform/sync/adaptiveSyncController.ts`
- Update imports in `useDesktopInventory.ts` and tests
- Optional shim re-export at old path for one commit

- [ ] **Step 1: Move file; fix imports**
- [ ] **Step 2: Run**

```powershell
bun run test -- frontend/tests/adaptive-sync-controller.test.ts frontend/tests/inventory-shell-sync.test.tsx
```

Expected: PASS (same behavior; timer wrap for Illegal invocation must remain).

- [ ] **Step 3: Commit**

```powershell
git add frontend/src/platform/sync frontend/src/features/inventory frontend/tests
git commit -m "refactor: move AdaptiveSyncController to platform/sync"
```

---

### Task 3: Shell hosts modules; TE lives under `modules/te-test-equipment`

**Files:**
- Create: `frontend/src/shell/InventoryShell.tsx` (product chrome: header switcher, status region, module viewport)
- Create: `frontend/src/modules/te-test-equipment/TeTestEquipmentView.tsx` (current TE table/hooks composition currently inside InventoryShell)
- Move TE-specific components/hooks/types under `modules/te-test-equipment/`
- Update: `frontend/src/app/App.tsx` → import shell from `@/shell/InventoryShell`
- Update: switcher to read `listModuleDefinitions()` / hosts
- Delete or shim old paths

**Split rule for InventoryShell (critical):**

1. **Shell owns:** active module id state, switcher, theme chrome if global, placeholder rendering, which host is active.
2. **TE module owns:** `useDesktopInventory`, mutations, table, dialogs, TE status details, export/import.

Suggested TE main view signature:

```tsx
// modules/te-test-equipment/TeTestEquipmentView.tsx
export function TeTestEquipmentView({ active }: { active: boolean }) {
  // move existing TE hooks here; pass active into useDesktopInventory({ teActive: active })
  ...
}
```

Shell sketch:

```tsx
export function InventoryShell() {
  const [activeId, setActiveId] = useState(readStoredModuleId);
  const host = getModuleHost(activeId);
  // header switcher bound to listModuleDefinitions()
  return (
    <>
      <InventoryHeader ... systems={listModuleDefinitions()} activeId={activeId} onChange={...} />
      {host.kind === "placeholder" ? (
        <InventorySystemPlaceholder system={host.definition} />
      ) : (
        <host.MainView active={activeId === host.definition.id} />
      )}
    </>
  );
}
```

- [ ] **Step 1: Extract TE view without behavior change** (same hooks, same props) under `modules/te-test-equipment`
- [ ] **Step 2: Shell only switches hosts**
- [ ] **Step 3: Run full frontend suite**

```powershell
bun run test
bun run lint
bun run build
```

Expected: all previous tests pass (update import paths in tests as needed).

- [ ] **Step 4: Commit**

```powershell
git commit -m "refactor: shell hosts TE module via registry (IM-012)"
```

**Acceptance for Task 3:**
- Switching to placeholder still shows placeholder.
- Switching back to TE keeps cached rows + reactivates sync (existing shell-sync tests).
- No TE lifecycle when placeholder selected.

---

### Task 4: Backend `ModuleId` + shared root helper

**Files:**
- Create: `backend/src/platform/mod.rs`, `backend/src/platform/module_id.rs`, `backend/src/platform/shared_root.rs`
- Modify: `backend/src/lib.rs` / `sync/shared_paths.rs` to call platform resolver for TE default
- Modify: tests if paths are hard-coded (keep env override `INVENTORY_MANAGEMENT_SHARED_ROOT` as **process-level TE override** for diagnostics only — document IM-013)

- [ ] **Step 1: Unit test ModuleId parse + TE default root string ends with `modules\TE_Test_Equipment` or equals `DEFAULT_SHARED_ROOT`**
- [ ] **Step 2: Implement platform module**
- [ ] **Step 3: Wire `resolve_shared_root` to TE module id** (behavior unchanged for current app)
- [ ] **Step 4: Run**

```powershell
cd C:\Projects\Active\Inventory_Management\backend
cargo test --lib
cargo test --test shared_sync_flow
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] **Step 5: Commit**

```powershell
git commit -m "feat: backend ModuleId and TE shared-root resolution (IM-012)"
```

---

### Task 5: Session map skeleton (TE-only entries)

**Files:**
- Modify: `backend/src/runtime/shared_watcher.rs` (or successor) so session state is keyed by `ModuleId`
- Keep command signatures **backward compatible** for this plan if possible:
  - Option A (preferred for smaller diff): internal map with single key `TeTestEquipment`; public commands unchanged.
  - Option B: add optional `module_id` argument later (Phase C) — **not required now**.

**Must preserve IM-011 tests:**
- stale sync → `Ok(None)`
- stale deactivate does not kill newer session
- watcher attach only for current session

- [ ] **Step 1: Extend / adjust Rust lifecycle tests for map semantics** (same TE scenarios)
- [ ] **Step 2: Implement map; TE-only activate path**
- [ ] **Step 3: Run watcher lifecycle + shared_sync_flow + clippy**
- [ ] **Step 4: Commit**

```powershell
git commit -m "refactor: per-module sync session map (TE-only wiring)"
```

---

### Task 6: Docs + agent entry points

**Files:**
- Modify: `docs/SESSION_HANDOFF.md` — Phase B status, new paths
- Modify: `docs/SESSION_START_PROMPT.md` — architecture layout blurb
- Modify: `AGENTS.md` — point at `shell` / `platform` / `modules`
- Modify: this plan header **Status** → Implemented when done
- Modify: `docs/superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md` Phase B rows

- [ ] **Step 1: Update docs only after Tasks 1–5 green**
- [ ] **Step 2: Commit**

```powershell
git commit -m "docs: record platform/module layout after IM-012 extract"
```

---

### Task 7: Full verification gate

- [ ] **Frontend**

```powershell
cd C:\Projects\Active\Inventory_Management
bun run test
bun run lint
bun run build
```

- [ ] **Backend**

```powershell
cd backend
cargo test --lib
cargo test --test shared_sync_flow
cargo clippy --all-targets --all-features -- -D warnings
```

- [ ] **Desktop smoke (single instance)**

```powershell
bun run desktop
```

Check:
1. Window opens; title works (set-title permission already present).
2. TE loads; Shared status if S: module path healthy.
3. Placeholder modules still placeholders.
4. TE → placeholder → TE: rows cached; no console `Illegal invocation`.
5. Adaptive sync still schedules (optional DevTools soak = A2, not blocking).

- [ ] **Update SESSION_HANDOFF** with measured results
- [ ] **Final commit + push** (owner/implementor)

---

## 5. Risk register

| Risk | Mitigation |
|------|------------|
| Massive import churn breaks tests | Compat shims under `features/inventory/*` for one PR; delete in Task 3 cleanup |
| Accidental behavior change in shell split | Run `inventory-shell-*.test.tsx` after every shell commit |
| Session map regression | Keep IM-011 Rust tests green; no command rename in this plan |
| Over-scoping to multi-package monorepo | Explicit non-goal; folders + registry only |
| Env override confusion | Document IM-013: override is diagnostic only |

---

## 6. Definition of done (Phase B extract)

1. Frontend code is organized under `shell/`, `platform/`, `modules/*` with TE as the only `implemented` desktop host.
2. Switcher driven by platform registry; placeholders unchanged in UX.
3. Adaptive sync controller lives under platform; TE still uses it only when active.
4. Backend has `ModuleId` + TE root resolution + session map keyed by module (TE-only populated).
5. All automated gates green; desktop smoke OK.
6. Docs/handoff updated; this plan marked implemented.
7. **No** legacy InventoryApps default; **no** identity change; **no** other modules ported.

---

## 7. Out of scope follow-ons (do not mix in)

| Later | Plan |
|-------|------|
| Port Lab / ME / Storage | Phase C — new module folders become `kind: "desktop"` one at a time |
| Multi-module concurrent sessions | Extend session map + per-module DBs/roots |
| Physical npm/cargo workspaces | Only if folder boundaries prove insufficient |
| TE historical data import into product share | Separate migration plan under IM-013 |

---

## 8. Implementor paste prompt

Copy everything below into the implementor chat (workspace `C:\Projects\Active\Inventory_Management`):

```text
You are the IMPLEMENTATION agent for Inventory Management (IM-012 architecture extract).

## Workspace
C:\Projects\Active\Inventory_Management

## Authority
1. AGENTS.md
2. docs/SESSION_HANDOFF.md
3. docs/planning/DECISIONS.md (IM-004, IM-012, IM-013, IM-011)
4. docs/superpowers/plans/2026-07-20-platform-module-architecture-extract.md  ← FOLLOW THIS PLAN TASK BY TASK
5. docs/superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md (Phase B context)

## Goal
Logical monorepo extract: product shell + shared platform + per-module folders. TE Test Equipment is the only real module. Placeholders stay placeholders. Do not change Tauri id, AppData, package name, or default TE shared root (product modules\TE_Test_Equipment). Do not re-implement IM-011; move/reuse AdaptiveSyncController.

## Method
- Execute Tasks 1→7 in order from the plan.
- Prefer thin vertical slices; TDD where the plan shows failing tests first.
- Compat re-export shims OK temporarily; remove before calling the extract done.
- No multi-crate Cargo workspace / npm workspaces unless the plan’s non-goals change (they must not).
- Do not default INVENTORY_MANAGEMENT_SHARED_ROOT to legacy InventoryApps (IM-013).
- Do not port Lab/ME/Storage Room data in this work.

## Verification before “done”
- bun run test && bun run lint && bun run build
- cargo test --lib; cargo test --test shared_sync_flow; clippy -D warnings
- bun run desktop smoke: TE works, placeholders work, switch cache OK, no Illegal invocation
- Update docs/SESSION_HANDOFF.md with results; mark plan status implemented

## First reply
Restate goal, list Task 1 files you will create, then start Task 1 failing registry test.
```

---

## 9. Planning self-check

| Requirement | Task coverage |
|-------------|---------------|
| Shared platform + domain modules (IM-012) | Tasks 1–3 |
| No merged schemas | Non-goals + module hosts |
| TE product path IM-013 | Task 4 root helper; non-goals |
| Adaptive sync preserved | Task 2 move; Task 3 active flag |
| Future multi-module sessions | Task 5 map skeleton |
| Placeholders remain | Task 1 hosts; Task 3 shell |
| Docs/agent update | Task 6 |
| Verification | Task 7 |
