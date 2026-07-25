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
5. `docs/superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md` (**IM-015** — implementation verified; live Phase 8 pending)
6. As needed for sync behavior or regressions: `docs/superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md` (**IM-011** — implemented for TE and reused by Lab; still the behavioral authority)

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
- Decisions IM-001…IM-013 plus IM-015 in the calibration feature branch.
- GitHub: `https://github.com/Hassaan-ECE/Inventory_Management.git` (`origin` / `main`).
- **IM-011 adaptive TE sync lifecycle** — completion-aware 2s/60s scheduling, session tokens, hard deactivate on deselect, `syncIntervalMs` removed. Plan + verification notes in handoff.
- **IM-012 logical architecture extract** — product shell under `frontend/src/shell`, registry/sync under `frontend/src/platform`, TE under `frontend/src/modules/te-test-equipment`, placeholder hosts beside it, and backend `ModuleId`/root/session-map seams under `backend/src/platform` plus `backend/src/runtime`.
- **Phase C1 TE Lab Components port** — distinct no-calibration domain (`verifiedInSurvey`), sync schema v1, separate `te-lab-components.feox`, Lab pilot root `InventoryApps\TE`, module-scoped commands, isolated TE/Lab sessions, Lab export, and unified shell styling.
- **First team release v0.1.0** — signed NSIS, GitHub Release, and updater metadata were completed July 20, 2026.
- **IM-015 Calibration workspace + roster tooling** — implemented and pre-cutover verified in isolated worktree `C:\Projects\Active\Inventory_Management_IM015` on branch `feature/im-015-calibration`. Equipment and Calibration are two views of the same TE records; the roster importer is calibration-specific and leaves the general importer unchanged.

**Not done**

- Real ports of ME Storage / TE Storage Room.
- Integration of the concurrent feature worktrees into `main`.
- IM-015 live calibration roster cutover: backups, fresh preview, owner resolutions, one live commit/publish, and restart verification.
- Optional residual: live DevTools call-rate smoke for adaptive cadence (automated/fake-timer coverage already green).
- Owner release QA: create/edit one live Lab row and confirm persistence after restart; implementation smoke intentionally did not mutate live rows.

## Priorities (next work)

Full “what works on desktop vs remaining work” map:
`docs/superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md`

1. Integrate the concurrent implementation branches into `main`; resolve shared-document conflicts deliberately and rerun all gates.
2. Run IM-015 controlled live cutover only after integration: fresh workbook/database preview, dated local/shared backups, exactly one writer, explicit row resolutions, one commit, publish completion, restart verification, and retained rollback artifacts.
3. Treat the copied-data preview as rehearsal only: **97 rows = 78 matched + 8 creates + 3 conflicts + 4 duplicates + 4 junk**, with **110 absent** and a pre-resolution prospective count of **86**. No live writes occurred.
4. Complete any remaining owner Lab persistence QA; ME Storage + TE Storage Room remain placeholders for later work.

Do **not** restart IM-011 implementation unless fixing a regression or extending the lifecycle to another inventory.  
Do **not** casually change either active pilot default or release target; cutover requires an owner-driven data copy and one-writer transition.

## Rules

- Verify critical paths before claiming success (lint/test/build/smoke as appropriate).
- Sync is **not** a backup.
- One writer client per inventory shared root at a time (don’t dual-run standalone + unified against the same live root).
- Do not rename Tauri id after team installs without a migration plan.
- Preserve switcher UX intent; improve architecture under it.

## First reply in the new chat

Summarize: workspace path, stable product identity, that **IM-011, IM-012, C1, and IM-015 implementation are done**, that TE + Lab are real modules, that the v0.1.0 team release exists, and that **feature integration plus the backed-up single-writer IM-015 live cutover are next**. State clearly that no live calibration roster commit has occurred and ME/Storage remain placeholders.
