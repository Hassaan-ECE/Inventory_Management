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
| Local DB (shell / current TE module) | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| Product share (installers + modules) | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE module shared root | `...\Inventory_Management_App\modules\TE_Test_Equipment` |

## Stack

Tauri 2, React 19, TypeScript, Vite, Tailwind v4, Bun, Rust, FeOxDB.  
Scaffold lineage: TE Test Equipment Inventory (switcher prototype + rebrand) ← ME family.

## Product shape

One installer; in-app switcher between inventories:

1. TE Test Equipment (implemented first)
2. TE Lab Components (placeholder)
3. ME Storage (placeholder)
4. TE Storage Room (placeholder)

Modules stay **separate** under the hood (own shared roots / future per-module DBs). Do not merge row types.

## Priorities

1. Redesign monorepo (shared platform + modules) — **IM-012** (next engineering focus; start with map B1)  
2. ~~IM-011 adaptive TE sync~~ **done 2026-07-20** — plan under `docs/superpowers/plans/`  
3. ~~TE path strategy~~ **IM-013**: product `modules\TE_Test_Equipment` long-term; no default legacy pilot  
4. Port other modules (placeholders today); archive standalones only after deliberate migration  
5. Updater signing + GitHub Releases (new keys — do not reuse TE/ME keys)

## Env (shared sync)

| Env | Purpose |
|-----|---------|
| `INVENTORY_MANAGEMENT_SHARED_ROOT` | Override default TE module shared root |
| `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` | `0`/`false`/`no`/`off` to disable |
| `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` | Optional HMAC (≥16 bytes) |

**Sync is not a backup.**

## Verification

Run real lint/test/build/smoke before claiming success. Prefer `docs/SESSION_HANDOFF.md` over stale TE planning docs copied into this tree.
