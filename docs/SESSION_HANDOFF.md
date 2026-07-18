# Session handoff — Inventory Management

**Last updated:** 2026-07-18  
**State:** Scaffold `0.1.0` — new product identity + S: product share; switcher UI; TE module only.  
**Redesign (accepted, not shipped):** Adaptive per-inventory sync lifecycle **IM-011** — plan: [superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md](superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md). Product still uses fixed ~500 ms polling until that plan is implemented and verified.

**New chat:** paste [SESSION_START_PROMPT.md](SESSION_START_PROMPT.md). Move context: [CHAT_HANDOFF.md](CHAT_HANDOFF.md).

## Workspace

```text
C:\Projects\Active\Inventory_Management
```

(Not under `Inventory_Apps` — that tree holds the standalone sibling apps only.)

Product share (verified created this session):

```text
S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App
```

## Identity

| Item | Value |
|------|--------|
| Display | Inventory Management |
| Package | `inventory-management` `0.1.0` |
| Tauri id | `com.inventory.management` |
| Local DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| Product share | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE module shared root | `...\modules\TE_Test_Equipment` |

## What exists

- Full ME/TE-family scaffold copied from TE Test Equipment Inventory and rebranded.
- Header **hamburger** switcher: TE Test Equipment, TE Lab Components, ME Storage, TE Storage Room.
- Placeholders for non-TE modules.
- S: tree: `modules\*`, `release-support\`, `legacy-pointers\`, `README.md`.
- Updater **disabled** (`createUpdaterArtifacts: false`, empty endpoints) until new keys/repo.
- Decisions **IM-001…IM-012**; adaptive sync plan under `docs/superpowers/plans/`.

## What does **not** exist yet

- Adaptive sync lifecycle (IM-011 / plan above) — still fixed ~500 ms TE polling
- Modular monorepo extract (shared platform + per-module domains)
- GitHub repo / signed releases for this product (owner creating)
- Port of ME Storage / TE Lab Components / TE Storage Room data layers
- Automatic data migration from legacy InventoryApps shares into `modules\...`
- Team installer on the product share root

## Standalone apps (unchanged)

Continue to work in their own trees until cutover; do not treat this folder as TE Test Equipment Inventory.

## Env

| Env | Purpose |
|-----|---------|
| `INVENTORY_MANAGEMENT_SHARED_ROOT` | Override TE module shared root |
| `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` | Opt out of shared sync |
| `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` | Optional HMAC |

## Next slices

1. Connect new GitHub remote; initial push  
2. Implement IM-011 adaptive sync (plan above) against switcher + TE module  
3. Architecture extract: shared platform + module boundaries  
4. TE cutover path: empty `modules\TE_Test_Equipment` vs env → legacy share  
5. Port other modules; archive standalone repos after team cutover  
6. Updater keypair for `com.inventory.management` only  
