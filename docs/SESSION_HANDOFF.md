# Session handoff — Inventory Management

**Last updated:** 2026-07-20  
**State:** Product `0.1.0` — stable identity + S: product share; TE Test Equipment and TE Lab Components are implemented as isolated modules with separate DBs, roots, domain types, and sync streams; ME Storage and TE Storage Room remain placeholders.
**Implemented redesigns:** Adaptive per-inventory sync lifecycle **IM-011** ([plan](superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md)), logical platform/module architecture **IM-012** ([plan](superpowers/plans/2026-07-20-platform-module-architecture-extract.md)), and Phase **C1 TE Lab Components** ([plan](superpowers/plans/2026-07-20-te-lab-components-port.md)).
**Docs hygiene:** SESSION_START_PROMPT, AGENTS, capability roadmap, and this handoff describe the implemented architecture so new chats do not re-open IM-011, IM-012, or C1 as greenfield work.
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
| TE Test Equipment DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| TE Lab Components DB | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| Product share | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE shared root (**pilot**) | `...\InventoryApps\TE_Test_Equipment_Inventory` (live data for `bun run desktop`) |
| Product TE root (**release target**) | `...\Inventory_Management_App\modules\TE_Test_Equipment` |
| Default Lab shared root (**pilot**) | `...\InventoryApps\TE` (live data for `bun run desktop`) |
| Product Lab root (**release target**) | `...\Inventory_Management_App\modules\TE_Lab_Components` |

## What exists

- Full ME/TE-family scaffold copied from TE Test Equipment Inventory and rebranded.
- Header **hamburger** switcher: TE Test Equipment, TE Lab Components, ME Storage, TE Storage Room.
- Real TE Test Equipment and TE Lab Components desktop modules; placeholders only for ME Storage and TE Storage Room.
- Product chrome and switching live under `frontend/src/shell/`; module registry/persistence and adaptive sync live under `frontend/src/platform/`.
- TE UI/domain remains under `frontend/src/modules/te-test-equipment/` and the existing `inventory.feox`; its calibration schema and sync schema v2 are unchanged except for module-scoped bridge calls.
- Lab UI/domain lives under `frontend/src/modules/te-lab-components/`; it has no calibration fields, uses `verifiedInSurvey`, sync schema v1, and the separate `te-lab-components.feox` DB.
- The shell keeps both desktop hosts mounted, runs lifecycle work only for the active module, hard-deactivates the inactive session, and preserves each module's cached rows across switches.
- Both real modules reuse the completion-aware IM-011 schedule: approximately 2 seconds while selected/active and approximately 60 seconds while idle, hidden, or unfocused; activation, restoration, activity, mutation, and correctly scoped watcher events can request immediate sync.
- Backend inventory lifecycle, query, CRUD, and export commands are module-scoped. The Tauri JSON boundary dispatches into distinct TE and Lab types instead of merging schemas; TE import remains TE-only.
- `backend/src/inventory_stores.rs` owns isolated FeOx handles. Shared roots, sync gates, watcher sessions, opaque tokens, statuses, and events are keyed by `ModuleId`; stale work for one module cannot deactivate the other.
- TE retains its calibration workbook export; Lab has a separate standalone-compatible 19-column workbook without calibration columns.
- S: tree: `modules\*`, `release-support\`, `legacy-pointers\`, `README.md`.
- Updater **disabled** (`createUpdaterArtifacts: false`, empty endpoints) until new keys/repo.
- Decisions **IM-001…IM-013**; implementation plans under `docs/superpowers/plans/`.
- Git: `main` → `https://github.com/Hassaan-ECE/Inventory_Management.git` (initial scaffold push 2026-07-18).

## What does **not** exist yet

- Signed GitHub Releases / updater keys for this product
- Port of ME Storage / TE Storage Room data layers
- Automatic data migration from legacy InventoryApps shares into `modules\...`
- Team installer on the product share root

## Standalone apps (unchanged)

Continue to work in their own trees until cutover; do not treat this folder as TE Test Equipment Inventory.

## Env

| Env | Purpose |
|-----|---------|
| `INVENTORY_MANAGEMENT_SHARED_ROOT` | Override TE module shared root |
| `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT` | Override Lab module shared root |
| `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` | Opt out of shared sync |
| `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` | Optional HMAC |

## Pilot shared roots (current vs release)

**TE Test Equipment active default (no env needed):**
`S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE_Test_Equipment_Inventory`  

**TE Lab Components active default (no env needed):**
`S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE`

So plain `bun run desktop` loads the live pilot data for both implemented modules.

**Rules**

- One writer per root — do **not** run standalone TE Test Equipment against its pilot share, or standalone TE Parts/Lab Components against `InventoryApps\TE`, while this app is open.
- **Before team release:** copy the latest TE and Lab shared data into `...\Inventory_Management_App\modules\TE_Test_Equipment` and `...\Inventory_Management_App\modules\TE_Lab_Components`, deliberately flip both defaults, then ship.

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

## IM-012 verification — 2026-07-20

- Full frontend suite: **18 files passed, 1 skipped; 145 tests passed, 1 skipped**. The existing unrelated React `act(...)` warning in `entry-dialog.test.tsx` remains non-failing.
- `bun run lint`: passed.
- `bun run build`: passed (`tsc -b` plus Vite production build; JS **351.22 kB**, CSS **49.63 kB** before gzip).
- Rust library suite: **73/73 passed**, including the new `ModuleId`, shared-root, token/session-map, stale-deactivation, idempotent stop, and in-flight deactivation cases.
- `cargo test --test shared_sync_flow`: **39/39 passed**, including unsigned/signed two-database push/pull and existing mutation/bootstrap/snapshot flows.
- `cargo fmt --all -- --check`: passed after applying rustfmt-only layout changes.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Single-instance desktop smoke: confirmed no existing app/Vite listener, launched one instrumented Tauri instance, verified the Inventory Management title and all four switcher labels, selected TE Lab Components before its C1 port, returned to TE, retained the empty local table state (**0 → 0 rows**), observed no runtime errors or `Illegal invocation`, then stopped the app and confirmed ports **5173/9222** and the desktop process were closed.
- Residual risk: the live desktop cache check used an empty local DB; the seeded-row path is covered by `inventory-shell-sync.test.tsx` (`keeps cached TE rows visible across placeholder switch`). Optional IM-011 live cadence soak A2 remains separate and non-blocking.

## Phase C1 verification — 2026-07-20

- Full frontend suite: **19 files passed, 1 skipped; 148 tests passed, 1 skipped**. The existing unrelated React `act(...)` warning in `entry-dialog.test.tsx` remains non-failing.
- `bun run lint`: passed.
- `bun run build`: passed (`tsc -b` plus Vite production build; JS **419.81 kB**, CSS **49.85 kB** before gzip).
- Rust library suite: **107/107 passed**, including Lab model/storage/query/mutations, schema-v1 sync isolation, dual-store filenames, module roots, independent gates/sessions, and the Lab workbook contract.
- `cargo test --test shared_sync_flow`: **40/40 passed**.
- `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- Automated shell/bridge coverage verifies TE→Lab→TE session isolation, hard deactivation, scoped events, cached Lab rows, correct titles, module IDs on every command, Lab CRUD/query/export contracts, and absence of calibration UI in Lab.
- Single-instance live desktop smoke: preflight found no unified/standalone writer and no listeners on **5173/9222**; Lab opened **Shared** with **1 inventory / 0 archive** and its known row; TE switched in **Shared** with **529 inventory / 13 archive**; ME Storage remained a placeholder; returning to Lab preserved its row and no runtime exception or `Illegal invocation` was observed. The process and both listeners were fully stopped afterward.
- The smoke was intentionally non-mutating on live shared data. Automated tests cover Lab create/update/delete/archive/verify; owner release QA should still add/edit one Lab entry and confirm persistence after restart.

## Next slices

1. ~~Connect new GitHub remote; initial push~~ done (`origin` / `main`)  
2. ~~Implement IM-011 adaptive sync against switcher + TE module~~ done and verified 2026-07-20 (optional residual: live DevTools cadence counts when owner soak-tests)  
3. **Capability map + roadmap:** [superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md](superpowers/plans/2026-07-20-desktop-capability-and-roadmap.md)  
4. ~~Phase A TE path: shared health + cutover~~ **done** — owner confirmed **Shared** on product module path; **IM-013** = long-term `modules\TE_Test_Equipment` (not default legacy pilot)  
5. ~~Phase B architecture extract (IM-012)~~ **done and verified 2026-07-20** — plan: [superpowers/plans/2026-07-20-platform-module-architecture-extract.md](superpowers/plans/2026-07-20-platform-module-architecture-extract.md); results in § IM-012 verification  
6. ~~Phase C1 TE Lab Components port~~ **done and verified 2026-07-20** — plan: [superpowers/plans/2026-07-20-te-lab-components-port.md](superpowers/plans/2026-07-20-te-lab-components-port.md).
7. ~~Phase C1~~ done. **Next: Phase D first team release** — plan: [superpowers/plans/2026-07-20-first-team-release.md](superpowers/plans/2026-07-20-first-team-release.md). Copy script: `scripts/release/copy-shared-to-product-modules.ps1`. Order: copy shared data → flip defaults → smoke → NSIS installer (updater optional fast-follow).
8. ME Storage + TE Storage Room remain deferred to a later post-release update; optional IM-011 live cadence soak A2 remains non-blocking.
