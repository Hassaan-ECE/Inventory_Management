# Session handoff — Inventory Management

**Last updated:** 2026-07-20  
**State:** Scaffold `0.1.0` — new product identity + S: product share; switcher UI; TE module only; adaptive TE sync lifecycle implemented.  
**Implemented redesign:** Adaptive per-inventory sync lifecycle **IM-011** — authority (status **implemented**): [superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md](superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md).  
**Docs hygiene:** SESSION_START_PROMPT, AGENTS priorities, DECISIONS IM-011, CHAT_HANDOFF, and docs index refreshed 2026-07-20 so new chats do not re-open IM-011 as open work.  
**Desktop runtime fixes (2026-07-20):** `devUrl` uses `http://127.0.0.1:5173` (avoid localhost→IPv6); capability `core:window:allow-set-title`; adaptive controller wraps `setTimeout`/`clearTimeout` to avoid WebView `Illegal invocation`. Full restart of `bun run desktop` required after capability change.

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
- TE uses completion-aware post-sync scheduling: approximately 2 seconds while selected, visible, focused, and recently active; approximately 60 seconds while idle, hidden, or unfocused.
- Activation, focus/visibility restoration, activity wake, mutation, and scoped watcher events request immediate TE synchronization.
- TE deselection/unmount clears timers and listeners, invalidates late frontend work, and deactivates the opaque backend sync session without clobbering cached rows.
- Backend sync sessions guard watcher attachment/deactivation; stale sync returns `null`, stale cleanup cannot stop a newer session, and shared-change events are scoped to `te-test-equipment`.
- S: tree: `modules\*`, `release-support\`, `legacy-pointers\`, `README.md`.
- Updater **disabled** (`createUpdaterArtifacts: false`, empty endpoints) until new keys/repo.
- Decisions **IM-001…IM-012**; adaptive sync plan under `docs/superpowers/plans/`.
- Git: `main` → `https://github.com/Hassaan-ECE/Inventory_Management.git` (initial scaffold push 2026-07-18).

## What does **not** exist yet

- Modular monorepo extract (shared platform + per-module domains)
- Signed GitHub Releases / updater keys for this product
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

## IM-011 verification — 2026-07-20

- Targeted frontend sync/switcher gate: **4 files, 57 tests passed**.
- Full frontend suite: **17 files passed, 1 skipped; 142 tests passed, 1 skipped**.
- `bun run lint`: passed.
- `bun run build`: passed (`tsc -b` plus Vite production build).
- Targeted Rust watcher lifecycle: **9 passed**.
- Full Rust suite with only the two external-path audits filtered: passed. `shared_sync_flow` passed **37/37**, including `two_databases_push_and_pull_create_update_and_delete` and the signed two-database flow.
- Raw `cargo test` reaches the same green code paths but the two opt-in live-audit cases require unset `TE_LEGACY_AUDIT_XLSX` and `TE_INVENTORY_AUDIT_DB`; they were not pointed at owner data during this implementation session.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Static lifecycle check: no inventory sync `setInterval` or `syncIntervalMs` remains; the only `setInterval` is the unchanged updater scheduler in `useDesktopUpdates.ts`.
- Single-instance desktop smoke: confirmed no owner instance was running, launched one Tauri instance, verified the TE WebView and bridge, measured `activateInventorySync` at **3 ms** and local `loadInventory` at **17 ms**, then fully stopped the process and DevTools listener.
- Residual manual verification: post-initial-sync DevTools call counts for active, slow, and deselected TE were not captured because the live UI remained in initial shared-sync loading during the observation window and WebView2 did not retain the pre-page CDP instrumentation. Fake-timer coverage verifies the 2-second, 120-second idle transition, 60-second, immediate-trigger, and no-rearm behavior deterministically.

## Next slices

1. ~~Connect new GitHub remote; initial push~~ done (`origin` / `main`)  
2. ~~Implement IM-011 adaptive sync against switcher + TE module~~ done and verified 2026-07-20 (optional residual: live DevTools cadence counts when owner soak-tests)  
3. **Capability map + roadmap:** [superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md](superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md)  
4. ~~Phase A TE path: shared health + cutover~~ **done** — owner confirmed **Shared** on product module path; **IM-013** = long-term `modules\TE_Test_Equipment` (not default legacy pilot)  
5. **Next (ready for implementor):** Phase B architecture extract (IM-012) — plan: [superpowers/plans/2026-07-20-platform-module-architecture-extract.md](superpowers/plans/2026-07-20-platform-module-architecture-extract.md) (shell + platform + modules; TE only; paste prompt in §8)  
6. Phase C: Port Lab Components / ME Storage / TE Storage Room (each under own `modules\<Name>\`)  
7. Phase D: Updater keys, GitHub Releases, team installer; archive standalones after deliberate migration  
