# Session handoff — Inventory Management

## Published release 0.1.3 - 2026-09-29

- User authorized the next release including current Storage work. Display name is now **TE Storage**; its manual sync button is removed. Automatic sync/status and the right-click column picker remain.
- Version is 0.1.3 in package.json, Cargo.toml/Cargo.lock, Tauri config/window title. Stable module ID `te-storage`, `te-storage-room.feox`, shared `modules/TE_Storage_Room`, signing key and updater endpoint are unchanged. Location values remain as imported.
- Signed installer built at `backend/target/release/bundle/nsis/Inventory Management_0.1.3_x64-setup.exe`; SHA-256 `c0ec0f20cae9dc380d3649d76cd3c0607d06771c84d980f97ad70d45d1ca3182`. Installer signature independently verified against the embedded updater public key. Release files and checksums prepared in `.tmp/release-0.1.3/`.
- Live S: is accessible in the normal user session (sandbox drive mappings differ). The 71 imported rows have reached its operation stream. A fresh release executable against an isolated copy loaded 71 rows: 62 TE Storage Room, 6 Eng Lab, 3 Eng Lab/E8. Verified exact default headers, hidden PR/Description, right-click picker and absent manual sync button. The live app/database were not stopped or edited.
- Frontend lint and all 216 tests passed (1 skipped); production frontend/NSIS build passed. Rust library tests: 161 passed/5 ignored. Seven integration test executables freshly compiled during this release check: 310 passed/9 ignored. They were run directly because the default cargo test invocation tried replacing the running development executable and hit Windows file locking. The binary target has zero tests and passed separately. Clippy passed with all targets/features and warnings denied. Existing Vite chunk-size advisory remains.
- S: installer and signature staged at the product root, with all five release-support files checksum-verified under `release-support/v0.1.3/`. Team download README updated; previous releases preserved. GitHub release published: https://github.com/Hassaan-ECE/Inventory_Management/releases/tag/v0.1.3 (source tag `v0.1.3`, commit `cad5800`). All five downloaded GitHub assets match the local files and S: archive. The unauthenticated public latest.json endpoint returns 0.1.3; its installer URL downloads the verified SHA-256 and its signature matches. Notes: `docs/releases/0.1.3.md`. No installed-machine upgrade has been performed during verification.
- GitHub sanitizes spaces in asset filenames to dots. The updater URL uses `Inventory.Management_0.1.3_x64-setup.exe`; the S: root installer retains spaces. Archive/checksum filenames match GitHub. Isolated smoke process stopped.
- The unrelated untracked July 28 Lab planning document is preserved outside this release commit.

## Working checkout update — 2026-09-28

TE Storage Room is now implemented in source, ahead of the published 0.1.2 installer. On 2026-09-28 the local database was seeded from `Shin_EVCS_Inventory\EVCS_Inventory.xlsx` (sheet Spare Parts(2023)). No, Remarks, and the end-of-line marker were not imported. Repeated rows were then combined: Warning Light qty 2, EVMA253-S `230071240403000990` qty 4, and EVMA253-S `230071232403000000` qty 2. Every location `10031` is now `TE Storage Room`. Eng Lab and Eng Lab/E8 were left as written. 71 rows remain, still queued for team sync and not yet published to the shared folder. Columns: PN #, PR # (hidden), PO #, Manufacturer, Model, Description (hidden), Qty, Location, Notes. Includes add/edit/confirmed delete, search, sorting, persisted column visibility and adaptive team sync.

Column visibility is available by right-clicking any Storage Room table header, matching the other inventories. Keyboard users can focus a header and press Shift+F10. The separate Columns toolbar button has been removed.

- Module ID `te-storage`; independent local DB `te-storage-room.feox` under the existing app data root.
- Shared default: product `modules/TE_Storage_Room`; override `INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT`. Creates only the module directory beneath an already available product modules directory. Missing roots remain offline; local edits stay queued.
- Shares the module coordinator/watcher lifecycle. Module-local domain and operation engine are adapted from the prior inventory implementation, with per-field versions, merged-entry recovery and coordinated row/outbox rollback. Existing TE/Lab streams are unchanged.
- Rehearsals must also redirect the Storage Room shared root, as well as TE/Lab roots, and disable shared sync.
- Design: `docs/superpowers/specs/2026-09-28-te-storage-room-design.md`; execution/verification: `docs/superpowers/plans/2026-09-28-te-storage-room.md`.
- The EVCS spare-part import above is local until the next Storage Room sync. No version bump, installer staging, or live shared-data cutover performed. Coworkers will need an updated installer to use the new module.

The release and earlier worktree notes below are historical context.

**Last updated:** 2026-08-17  
**State:** Product source and signed team installer are **0.1.2**. Simple Lab workspace + bin picker + order export are on `main` and staged on S:. Live IM-014 Lab catalog cutover and IM-015 calibration roster cutover remain owner ops. In-app updater still needs a GitHub Release of 0.1.2.

**New chat:** paste [SESSION_START_PROMPT.md](SESSION_START_PROMPT.md).

## 0.1.2 team staging (2026-08-17)

- Version triple: `package.json` / `backend/Cargo.toml` / `backend/tauri.conf.json` = **0.1.2**
- Signed installer: `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\Inventory Management_0.1.2_x64-setup.exe`
- Signature: same folder, `.sig`
- Support: `release-support\v0.1.2\` (installer copy, `.sig`, `latest.json`, `SHA256SUMS.txt`, `RELEASE_NOTES.md`)
- SHA-256: `2583cd3d20627538cc7bcc14cdb706f8752f05bc8e6b7e862a034d37813fda15`
- GitHub Release **published:** https://github.com/Hassaan-ECE/Inventory_Management/releases/tag/v0.1.2 (`latest.json` + installer + `.sig`). In-app Update can now see 0.1.2.

## Workspaces and branches

| Purpose | Path / branch |
|---------|---------------|
| Active app checkout (main product) | `C:\Projects\Active\Inventory_Management` / `main` |
| This feature worktree | `C:\Users\Syed.h.Shah\.grok\worktrees\active-inventory-management\subagent-01a01092-c1e3-7292-9e15-0f9ec316cc30` / `feature/lab-simple-workspace` |
| Clippy/fmt verification fix | `808a38b5b65069d225bb9d98f8901d097d4d93ad` |
| Docs verification commit | recorded immediately after this handoff update (see `git log -1`) |

## Stable identity

| Item | Value |
|------|-------|
| Display | Inventory Management |
| Package | `inventory-management` `0.1.2` |
| Tauri id | `com.inventory.management` |
| TE Test Equipment DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| TE Lab Components DB | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| Product share | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Default TE shared root | `...\Inventory_Management_App\modules\TE_Test_Equipment` |
| Default Lab shared root | `...\Inventory_Management_App\modules\TE_Lab_Components` |

## Implemented product shape

- One desktop product with isolated TE Test Equipment and TE Lab Components modules; ME Storage and TE Storage Room remain placeholders.
- Product shell/module switching and IM-011 adaptive sync remain module-scoped.
- TE uses the existing `inventory.feox` and sync schema v2.
- Lab uses the separate `te-lab-components.feox` and IM-014 Lab-only catalog sync schema v2.
- Shared roots, watcher sessions, sync gates, tokens, statuses, and events remain keyed by module.
- Team installer **0.1.2** is on S: root: `Inventory Management_0.1.2_x64-setup.exe` (+ `.sig`). 0.1.1 moved to `archive\`. Notes: `release-support\v0.1.2\`.

## Shipped on `feature/lab-simple-workspace` (2026-08-17)

Everyday **TE Lab Components** workflow (catalog + sync schema unchanged):

- **Five-column** default Lab table: Component Type, Value, Quantity, Location, Stock Status (advanced columns remain opt-in; visibility key bumped).
- **Simple editor** for through-hole `pcs` parts: type-aware value units, one short shelf location (`^[A-Z][1-9][0-9]*$`), atomic Part+placement create/update with rollback.
- **Review mode** when a row cannot be projected losslessly (multi-placement, mixed/non-`pcs` units, invalid location); advanced editor retained.
- **Order export:** select rows → requested quantities → focused “Order Request” workbook from authoritative DB records (read-only export path).
- Full-catalog Excel export retained; unimplemented HTML export removed from Lab-only menu. TE export path unchanged.

Authority:

- Spec: [simple workspace design](superpowers/specs/2026-07-28-te-lab-components-simple-workspace-and-order-export-design.md)
- Plan: [2026-08-17 implementation plan](superpowers/plans/2026-08-17-te-lab-components-simple-workspace-and-order-export.md) — status: **Implementation complete; owner acceptance and release staging pending**

## Verification (worktree, 2026-08-17)

### Focused frontend

```text
bun run test -- frontend/tests/te-lab-components-simple-component.test.ts frontend/tests/te-lab-components-shell.test.tsx frontend/tests/tauri-inventory-bridge.test.ts
```

- **64 passed** (16 + 13 + 35), 3 files, ~30s

### Full frontend

```text
bun run test
```

- **206 passed | 1 skipped** (207 total), 22 files passed | 1 skipped, ~29s  
- Lint: `bun run lint` **PASS**  
- Build: `bun run build` **PASS** (chunk-size advisory only)

### Rust

```text
cargo fmt --manifest-path backend/Cargo.toml -- --check   # after cargo fmt
cargo test --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
```

- rustfmt: **PASS** (fmt applied during verification for simple workflow / order workbook / commands)
- Full suite: **455 passed; 14 ignored; 0 failed** (lib + integration binaries; ignored include intentional live-repair/live-audit helpers)
- Clippy `-D warnings`: **PASS** after allowing test-only `AfterPartWrite` inject (constructed under `cfg(test)` only)
- Focused recheck: simple_workflow **9/9**, lab_order_workbook **4/4**

### Isolated desktop smoke

Root (only):

```text
C:\tmp\inventory-management-simple-workspace-smoke\
  data\          → inventory.feox + te-lab-components.feox created at startup
  shared-te\     → empty (sync off)
  shared-lab\    → empty (sync off)
```

Env in the same process:

- `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT=C:\tmp\inventory-management-simple-workspace-smoke\data`
- `INVENTORY_MANAGEMENT_SHARED_ROOT=C:\tmp\inventory-management-simple-workspace-smoke\shared-te`
- `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT=C:\tmp\inventory-management-simple-workspace-smoke\shared-lab`
- `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED=0`

**Verified:**

- Vite frontend ready at `http://127.0.0.1:5173/`
- `inventory-management.exe` launched from this worktree `backend\target\debug\`
- Isolated TE + Lab FeOx files written under the smoke `data` root only
- Redirected shared roots stayed empty
- Live `%LOCALAPPDATA%\com.inventory.management\inventory.feox` and `te-lab-components.feox` LastWriteTime stayed **2026-08-17 10:52:28** (before smoke ~12:46)
- Process stopped after smoke

**Not verified (no GUI clicks in this agent session):**

- Five-column table appearance, Add Capacitor flow, restart persistence of that row
- More Details advanced fields, order selection mode, workbook export content
- TE module switch / TE export HTML placeholder unchanged in UI

### Live boundaries for this session

- **Live Lab DB and product shared root were not modified** (no writes under product `S:\...\Inventory_Management_App` modules; live FeOx timestamps unchanged).
- No version bump, no installer staging, no S: release copy.

## Prior IM-014 / IM-015 (still on main product)

- IM-014 generalized Lab catalog + grid storage remains the persistence model under the simple projection.
- IM-015 TE Equipment/Calibration workspace remains separate; not changed by this branch’s Lab-only scope.
- Live IM-014 catalog cutover and IM-015 roster cutover remain pending owner operations.

## Live-data boundaries

- Sync is **not** a backup.
- Normal `bun run desktop` uses the product shared roots. Copied-data rehearsals must disable or redirect them.
- Run copied-data desktop testing with an absolute `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT`, redirected TE/Lab shared roots, and `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED=0`.
- Do not rely on changing `LOCALAPPDATA` alone; Windows app-path resolution can still select the real product databases.
- IM-014 live Lab migration and IM-015 live calibration roster commit are separate owner operations with separate backups and review steps.
- Do not run standalone writers against the corresponding shared roots while either live cutover is active.

## Next steps

1. Owner acceptance on `feature/lab-simple-workspace`: manual Lab simple workspace + order export checklist (five columns, add/edit, selection export, advanced review cases).
2. Merge/integrate to the active product checkout only after owner sign-off.
3. **Not done here:** product version bump, signed installer staging to S:, GitHub release/updater, live Lab cutover.
4. Continue separate IM-014 / IM-015 live cutovers only under their runbooks after backups and one-writer rules.

## Future release backlog (do not forget)

- **IM-F01 — Settings: due-soon days** — Owner wants a later in-app **Settings** control for the calibration “due soon” window (currently hard-coded **30** local days). Recorded in [DECISIONS.md](planning/DECISIONS.md) under *Future release backlog*. Touch points: `calibrationHealth.ts` / backend `derive_calibration_health`, counts strip, health filter, export labels.
