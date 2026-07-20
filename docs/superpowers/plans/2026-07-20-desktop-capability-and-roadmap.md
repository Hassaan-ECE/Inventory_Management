# What works on `bun run desktop` + product roadmap

**Status:** Planning authority for “current capability vs remaining work”  
**Date:** 2026-07-20  
**Product:** Inventory Management `0.1.0` (`com.inventory.management`)  
**Workspace:** `C:\Projects\Active\Inventory_Management`  
**Related:** [SESSION_HANDOFF.md](../../SESSION_HANDOFF.md), [DECISIONS.md](../../planning/DECISIONS.md), IM-011 plan (implemented for TE)

---

## 1. What `bun run desktop` is

```powershell
cd C:\Projects\Active\Inventory_Management
bun run desktop
```

That runs **Tauri dev** (`backend` + Vite frontend). It is a **developer desktop instance**, not a team installer.

| You get | You do not get |
|---------|----------------|
| Live React UI + Rust backend | NSIS installer on S: |
| Local FeOx DB under AppData | Auto-update from GitHub |
| Optional shared sync to S: module path | Ports of Lab / ME / Storage Room |
| Hot reload during development | Production signing / release pipeline |

**Local DB path:**

```text
%LOCALAPPDATA%\com.inventory.management\inventory.feox
```

**Default shared root (TE only):**

```text
S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules\TE_Test_Equipment
```

Override with `INVENTORY_MANAGEMENT_SHARED_ROOT`. Opt out of shared sync with `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED=0` (or `false` / `no` / `off`).

**Hard rule:** do not run this app and a standalone inventory app as writers against the **same** live shared root at the same time.

---

## 2. What should work today (when desktop starts)

### Shell / product chrome

| Capability | Expected behavior |
|------------|-------------------|
| App window | Opens as **Inventory Management** |
| Hamburger switcher | Lists TE Test Equipment, TE Lab Components, ME Storage, TE Storage Room |
| Active system memory | Last selected system restored from `localStorage` |
| Theme / preferences | Local UI preferences (theme, columns, etc.) from TE lineage |
| Version strip | Shows app version from branding |

### TE Test Equipment (only fully wired module)

| Capability | Expected behavior |
|------------|-------------------|
| Local inventory | Load / show entries from local FeOx DB |
| CRUD | Add, edit, archive, delete (when `canModify` allows) |
| Search / filter / sort | Client-side view model over loaded rows |
| Scopes | Inventory vs related scopes from TE shell |
| Excel export / external actions | TE-family export paths (filenames may still say TE Test Equipment lineage) |
| Adaptive shared sync (**IM-011**) | Activate session → local load → initial sync → completion-aware schedule (~2 s active, ~60 s idle/unfocused/hidden) |
| Immediate sync triggers | Activate, focus/visibility restore, activity wake, mutation, TE watcher events |
| Hard deactivate | Switching away from TE or unmount cancels timers/listeners and deactivates backend session; cached TE rows kept in memory |
| Shared-change events | Scoped to `te-test-equipment`; wrong/missing ids ignored |
| Status strip | Shared available / local-only / disabled messaging |

### Placeholder modules (switcher works; data does not)

Selecting **TE Lab Components**, **ME Storage**, or **TE Storage Room** shows a **placeholder panel** (“not connected yet”). No separate DB, no shared root, no CRUD for those modules.

### Shared sync outcomes (depends on environment)

| Situation | What you should see |
|-----------|---------------------|
| S: reachable and default TE module root **exists** | Shared mode can run; ops/snapshots under that tree after layout ensure |
| S: missing / root does not exist | App still runs **local-only**; shared status reflects unavailable path |
| Sync disabled via env | Local CRUD only; no poll/watch/shared I/O after deactivate path |
| Empty new module folder on S: | First sync may create layout; **no automatic import** of legacy InventoryApps data |

If the UI sits on **initial shared-sync loading** for a long time, treat that as an **environment / share-path** issue first (S: down, path wrong, permissions, lock contention)—not as “adaptive scheduling is broken.” Cadence policy is covered by automated tests; live call-rate smoke is still an optional residual.

### Explicitly not working / not configured

| Item | Status |
|------|--------|
| In-app auto-updater | **Off** (no product keys / release endpoints) |
| Team installer on product share root | **Not shipped** |
| Lab Components / ME / Storage Room inventory | **Placeholders only** |
| Auto-migration from legacy `InventoryApps\...` shares | **Not built** |
| Multi-inventory backend sessions | `ModuleId`-keyed map exists; only TE commands/session are wired until Phase C |
| Physical npm/Cargo workspaces | **Not planned**; IM-012 intentionally uses logical folders in one frontend package and one Rust crate |

---

## 3. Quick smoke checklist (owner / QA)

Run once after `bun run desktop` (single instance only):

1. Window opens; title/switcher show Inventory Management modules.
2. TE: table loads (empty OK if fresh DB).
3. TE: create an entry → appears after save; edit → persists after restart.
4. Status strip: either shared path healthy or clear local/unavailable message (not a silent hang).
5. Switch to a placeholder module → placeholder text; switch back to TE → previous TE rows still visible, then reconcile.
6. Optional: with healthy S: root, confirm sync finishes and idle cadence is calm (manual residual for IM-011).

---

## 4. What we still need to do (roadmap)

Ordered for product value and risk. Each slice should end with **verify + update SESSION_HANDOFF**.

### Phase A — Make TE desktop trustworthy for daily use (ops)

| ID | Slice | Status | Done means |
|----|-------|--------|------------|
| A1 | **TE shared-root health** | **Done** (owner 2026-07-20: desktop Shared mode works on product module path) | S: product module path exists; first sync completes |
| A2 | **Optional IM-011 live cadence smoke** | Open (owner will soak-test later) | DevTools: active ≲30/min, idle ≲1/min, deselected = 0 after drain |
| A3 | **TE cutover decision** | **Done — Option A** (**IM-013**): long-term product module path only; no default legacy pilot | Written choice recorded in DECISIONS |
| A4 | **Pilot rules** | Follow IM-013 | One writer per shared root; standalones stay on legacy InventoryApps until deliberate migration; roll-back = standalone apps unchanged |

**Out of A:** full team install, other modules. **Phase B is complete**; the next engineering focus is one deliberate Phase C module port.

### Phase B — Architecture extract (IM-012 foundation)

**Plan (authority):** [2026-07-20-platform-module-architecture-extract.md](./2026-07-20-platform-module-architecture-extract.md)

| ID | Slice | Status | Done means |
|----|-------|--------|------------|
| B1 | **Target logical architecture + implementor plan** | **Done 2026-07-20** | Design + tasks recorded in authority plan |
| B2 | **Extract shared shell / platform / TE module folders** | **Done 2026-07-20** | Tasks 1–3 in extract plan |
| B3 | **Per-module seams (ModuleId, session map skeleton)** | **Done 2026-07-20** | Tasks 4–5 in extract plan |
| B4 | **Docs/agent update + verification** | **Done 2026-07-20** | Tasks 6–7; measured results in SESSION_HANDOFF |

**Why before ports:** avoids copying TE three times and re-fighting sync lifecycle per module.

### Phase C — Port modules one by one

| ID | Slice | Done means |
|----|-------|------------|
| C1 | **TE Lab Components** | Real data path + own shared root under `modules\TE_Lab_Components`; switcher `implemented: true` |
| C2 | **ME Storage** | Same for ME under `modules\ME_Storage` |
| C3 | **TE Storage Room** | Same for storage room under `modules\TE_Storage_Room` |
| C4 | **Multi-module session wiring** | Add non-TE entries/commands to the existing `ModuleId → session` map; hard deactivate per module |

Each port: own shared root, no merged tables, adaptive lifecycle pattern from IM-011.

### Phase D — Ship to team

| ID | Slice | Done means |
|----|-------|------------|
| D1 | **New updater keypair** | Keys for `com.inventory.management` only (never reuse TE/ME) |
| D2 | **GitHub Releases + updater endpoints** | Signed artifacts; updater enabled in config |
| D3 | **Installer on product share** | NSIS at `S:\...\Inventory_Management_App\` + `release-support\vX.Y.Z\` |
| D4 | **Cutover playbook** | Per-module: install unified → stop standalone writer → migrate/point shared root → archive standalone when stable |
| D5 | **Archive standalones** | Only after team no longer needs Inventory_Apps writers |

### Explicit non-goals until later

- Merging all inventories into one schema/table
- Treating shared sync as backup
- Auto-pushing installer via old TE/ME updaters
- Renaming Tauri id / AppData after first team install without a migration plan

---

## 5. Suggested next action (pick one)

**Current default (2026-07-20):** **Phase C** — choose one module port after the owner confirms its data model, local DB, and product shared-root requirements.

| If you care about… | Do next |
|--------------------|---------|
| Next inventory capability | Choose one **Phase C** module port after owner confirmation |
| Confidence in adaptive sync in the real window | **A2** soak-test (optional, not blocking) |
| Seeding TE product share with real rows later | Planned **migration/import** into `modules\TE_Test_Equipment` (not default env→legacy; IM-013) |
| Shipping to others | Phase D after TE pilot on product path + green architecture seams |

**Default recommendation:** choose the first Phase C port; optional A2 remains non-blocking, and TE migration/import stays separate from legacy dual-writing.

---

## 6. Env cheat sheet

| Env | Purpose |
|-----|---------|
| `INVENTORY_MANAGEMENT_SHARED_ROOT` | Override TE shared root |
| `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` | `0`/`false`/`no`/`off` disables shared sync |
| `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` | Optional HMAC (≥16 bytes) for signed ops |

---

## 7. Doc maintenance

When a phase slice ships: update this plan’s status rows, [SESSION_HANDOFF.md](../../SESSION_HANDOFF.md) next slices, and [SESSION_START_PROMPT.md](../../SESSION_START_PROMPT.md) priorities. Do not re-open IM-011 as greenfield work unless regressing or extending to another inventory.
