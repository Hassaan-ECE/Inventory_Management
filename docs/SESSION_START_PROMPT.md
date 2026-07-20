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
2. `docs/SESSION_HANDOFF.md` (**current state / next slices**)
3. `docs/planning/DECISIONS.md` (IM-* decisions)
4. `README.md`
5. As needed for sync behavior or regressions: `docs/superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md` (**IM-011** — **implemented** for TE 2026-07-20; still the behavioral authority)

Prefer live code + those docs over stale TE planning copies still sitting under `docs/planning/` (many are lineage from TE).

## Product intent

- **One install**, hamburger title switcher across separate inventories (not one merged table/schema).
- Labels: **TE Test Equipment**, **TE Lab Components**, **ME Storage**, **TE Storage Room**.
- **Whole redesign monorepo**: shared platform + domain modules; TE quality as UX/ops baseline; room to add modules later.
- **New GitHub repo** for this product (`origin` already connected); archive old standalone repos **after** cutover works.
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

## Current state (2026-07-20)

**Exists**

- Scaffold from TE Test Equipment (rebranded) at the path above.
- Switcher UI (menu icon + short labels; placeholders for non-TE modules).
- S: product tree created (`modules\*`, `release-support\`, `legacy-pointers\`, README).
- Decisions IM-001…IM-012.
- GitHub: `https://github.com/Hassaan-ECE/Inventory_Management.git` (`origin` / `main`).
- **IM-011 adaptive TE sync lifecycle** — completion-aware 2s/60s scheduling, session tokens, hard deactivate on deselect, `syncIntervalMs` removed. Plan + verification notes in handoff.

**Not done**

- Modular folder extract (`modules/` platform split) / monorepo architecture (IM-012).
- Real ports of Lab Components / ME Storage / TE Storage Room.
- TE data cutover from legacy `InventoryApps\...` shares into `modules\TE_Test_Equipment`.
- Team installer on S:; updater keys + GitHub Releases.
- Optional residual: live DevTools call-rate smoke for adaptive cadence (automated/fake-timer coverage already green).

## Priorities (next work)

1. Architecture: shared shell/platform + per-module domains (expandable) — IM-012.
2. TE module data cutover strategy (new `modules\TE_Test_Equipment` vs temporary env to legacy TE share).
3. Port other inventories one by one; then archive standalones.
4. Updater keypair + GitHub Releases for `com.inventory.management` only.
5. Optional: close IM-011 live cadence smoke when shared root is healthy.

Do **not** restart IM-011 implementation unless fixing a regression or extending the lifecycle to another inventory.

## Rules

- Verify critical paths before claiming success (lint/test/build/smoke as appropriate).
- Sync is **not** a backup.
- One writer client per inventory shared root at a time (don’t dual-run standalone + unified against the same live root).
- Do not rename Tauri id after team installs without a migration plan.
- Preserve switcher UX intent; improve architecture under it.

## First reply in the new chat

Summarize: workspace path, product identity, that **IM-011 is done for TE**, and the next open slices (architecture extract, TE cutover, other modules, updater). Ask the owner what to do first.
