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
5. As needed for sync behavior or regressions: `docs/superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md` (**IM-011** — implemented for TE and reused by Lab; still the behavioral authority)
6. For Lab catalog work or rollout: `docs/superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md` (**IM-014**) and `docs/runbooks/te-lab-components-catalog-v2-migration.md`

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
| TE Test Equipment DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| TE Lab Components DB | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| Product share | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE shared root | `...\Inventory_Management_App\modules\TE_Test_Equipment` |
| Default Lab shared root | `...\Inventory_Management_App\modules\TE_Lab_Components` |
| Env prefix | `INVENTORY_MANAGEMENT_*` |

Updater is **enabled in config** (product-specific pubkey + GitHub Releases `latest.json`). Sign with private key outside the repo — see `docs/engineering/UPDATER_AND_RELEASE.md`. Do not reuse TE/ME keys.

## Current state (2026-07-25)

**Exists**

- Scaffold from TE Test Equipment (rebranded) at the path above.
- Switcher UI with two real desktop modules (TE Test Equipment + TE Lab Components) and placeholders for ME Storage + TE Storage Room.
- S: product tree created (`modules\*`, `release-support\`, `legacy-pointers\`, README).
- Decisions IM-001…IM-014.
- GitHub: `https://github.com/Hassaan-ECE/Inventory_Management.git` (`origin` / `main`).
- **IM-011 adaptive TE sync lifecycle** — completion-aware 2s/60s scheduling, session tokens, hard deactivate on deselect, `syncIntervalMs` removed. Plan + verification notes in handoff.
- **IM-012 logical architecture extract** — product shell under `frontend/src/shell`, registry/sync under `frontend/src/platform`, TE under `frontend/src/modules/te-test-equipment`, placeholder hosts beside it, and backend `ModuleId`/root/session-map seams under `backend/src/platform` plus `backend/src/runtime`.
- **Phase C1 TE Lab Components port** — separate `te-lab-components.feox`, module-scoped commands, isolated TE/Lab sessions, and unified shell styling.
- **IM-014 generalized Lab catalog implementation** — Parts separate from StockPlacements; flexible specifications; area/desk → container → Excel-style bins; multiple locations and shared bins; moves/counts; detailed search/filter/sort; reviewed lossless migration; Lab-only sync schema v2; six-sheet export.

**Not done**

- Real ports of ME Storage / TE Storage Room.
- Owner copied-data rehearsal and coordinated live IM-014 Lab migration/cutover.
- Single-instance copied-data desktop smoke covering restart persistence and Lab stock workflows.
- Optional residual: live DevTools call-rate smoke for adaptive cadence (automated/fake-timer coverage already green).
- Real ports of ME Storage / TE Storage Room.

## Priorities (next work)

Full “what works on desktop vs remaining work” map:
`docs/superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md`

1. ~~Phase A TE path~~ **done** — product `modules\TE_Test_Equipment` default.
2. ~~Phase B architecture extract (IM-012)~~ **done 2026-07-20** — shell/platform/modules; plan under `docs/superpowers/plans/`.
3. ~~Phase C1 TE Lab Components~~ **done and verified 2026-07-20**:
   `docs/superpowers/plans/2026-07-20-te-lab-components-port.md`
4. **IM-014 implementation complete; rollout active** — rehearse on copied owner data, run copied-data desktop smoke, then coordinate live Lab schema-v2 cutover using the migration runbook.
5. First release artifacts and updater are configured; team adoption and standalone retirement remain owner-coordinated.
6. ME Storage + TE Storage Room remain placeholders until a later post-release plan; optional A2 live cadence soak remains non-blocking.

Do **not** restart IM-011 implementation unless fixing a regression or extending the lifecycle to another inventory.  
Do **not** casually change either active pilot default or release target; cutover requires an owner-driven data copy and one-writer transition.

## Rules

- Verify critical paths before claiming success (lint/test/build/smoke as appropriate).
- Sync is **not** a backup.
- One writer client per inventory shared root at a time (don’t dual-run standalone + unified against the same live root).
- Do not rename Tauri id after team installs without a migration plan.
- Preserve switcher UX intent; improve architecture under it.

## First reply in the new chat

Summarize: workspace path, stable product identity, that **IM-011, IM-012, C1, and IM-014 implementation are done**, that TE + Lab are isolated on product module roots, and that **IM-014 copied-data rehearsal/desktop smoke/live Lab cutover** is the next guarded step. ME Storage and TE Storage Room remain placeholders.
