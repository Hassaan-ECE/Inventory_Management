# Session start prompt — Inventory Management

Copy everything below the line into a **new chat** with workspace:

```text
C:\Projects\Active\Inventory_Management
```

---

You are working on **Inventory Management**, a new unified multi-inventory Windows desktop product.

## Open this workspace first

```text
C:\Projects\Active\Inventory_Management
```

Do **not** use `C:\Projects\Active\Inventory_Apps\TE\TE_Test_Equipment_Inventory` as the active app tree (that is the standalone TE product). Sibling standalones under `Inventory_Apps\` stay legacy until cutover.

## Read first (in order)

1. `AGENTS.md`
2. `docs/SESSION_HANDOFF.md`
3. `docs/planning/DECISIONS.md` (IM-* decisions)
4. `docs/superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md` (**IM-011** — adaptive per-inventory sync; accepted, not implemented)
5. `README.md`

Prefer live code + those docs over stale TE planning copies still sitting under `docs/planning/` (many are lineage from TE).

## Product intent

- **One install**, hamburger title switcher across separate inventories (not one merged table/schema).
- Labels: **TE Test Equipment**, **TE Lab Components**, **ME Storage**, **TE Storage Room**.
- **Whole redesign monorepo**: shared platform + domain modules; TE quality as UX/ops baseline; room to add modules later.
- **New GitHub repo** for this product; archive old standalone repos **after** cutover works.
- Shared data stays **per module** on S:, not one shared ops stream for everything.

## Identity (do not casually change)

| Item | Value |
|------|--------|
| Display | Inventory Management |
| Package | `inventory-management` `0.1.0` |
| Tauri id | `com.inventory.management` |
| Local DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| Product share | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE module shared root | `...\Inventory_Management_App\modules\TE_Test_Equipment` |
| Env prefix | `INVENTORY_MANAGEMENT_*` |

Updater is **off** until new signing keys + GitHub Releases exist. Do not reuse TE/ME updater keys or endpoints.

## Current state (2026-07-18)

**Exists**

- Scaffold from TE Test Equipment (rebranded) at the path above.
- Switcher UI (menu icon + short labels; placeholders for non-TE modules).
- S: product tree created (`modules\*`, `release-support\`, `legacy-pointers\`, README).
- Decisions IM-001…IM-012; adaptive sync plan copied in as implementation authority.

**Not done**

- IM-011 adaptive sync (still ~500 ms fixed polling from TE scaffold).
- Modular folder extract (`modules/` platform split).
- GitHub remote (owner creates and connects).
- Real ports of Lab Components / ME Storage / TE Storage Room.
- Team installer on S:; data cutover from legacy `InventoryApps\...` shares.

## Priorities (next work)

1. Confirm git remote after owner creates the new repo; initial push if needed.
2. Implement **IM-011** adaptive per-inventory sync lifecycle (plan file is authority; TE-first; session tokens; hard deactivate on deselect).
3. Architecture: shared shell/platform + per-module domains (expandable).
4. TE module data cutover strategy (new `modules\TE_Test_Equipment` vs temporary env to legacy TE share).
5. Port other inventories one by one; then archive standalones.

## Rules

- Verify critical paths before claiming success (lint/test/build/smoke as appropriate).
- Sync is **not** a backup.
- One writer client per inventory shared root at a time (don’t dual-run standalone + unified against the same live root).
- Do not rename Tauri id after team installs without a migration plan.
- Preserve switcher UX intent; improve architecture under it.

## First reply in the new chat

Summarize: workspace path, product identity, IM-011 status, and ask what to do first (usually: wire git remote **or** start adaptive sync implementation). Wait for the owner if the new GitHub repo URL is not yet known.
