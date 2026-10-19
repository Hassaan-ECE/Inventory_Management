# Session handoff — Inventory Management

## 0.2.0 built and verified, not yet published — 2026-10-19

**Branch.** `feature/0.2-cleanup` holds every 0.2 phase. `main` (and `origin/main`) has Phases 1, 3, and 4 merged (`7f97329`); this branch adds Phase 2 (`b040b18`), Phase 5 (`48c7cb2`, `25cc0da`, `74b2b37`, `cf70219`), and the 0.2.0 release commit. Nothing from today is pushed yet.

**What 0.2.0 is.** One local database per PC (one-time merge of the 0.1.x files, originals kept in `pre-0.2-databases\`); one storage layer; one entry sync engine for TE and EVCS Storage; the Lab catalog on the same file helpers; footer warnings for skipped files, newer team versions, a nearly full database, and failed saves; bridge types generated from Rust (`ts-rs`); `bun run dev:share` fixture share; about 24,000 fewer lines than 0.1.4. Shared file formats are unchanged. Notes: `docs/releases/0.2.0.md`. Plan with full verification record: `docs/superpowers/plans/2026-10-16-v0.2-cleanup-and-consolidation.md` (Phase 6).

**Verified.**
- rustfmt, Clippy `-D warnings`, 251 Rust tests (4 ignored), frontend lint, 178 tests (1 skipped), production build.
- Cross-version both ways against the real `v0.1.4` source: 0.1.4 reads a 0.2.0 share and shows "Click Update" without publishing TE/EVCS snapshots; 0.2.0 takes in 0.1.4's creates and edits, no warnings.
- Signed release build on copies of this PC's databases and the live S: module folders: merge succeeded (TE 575, EVCS 71, Lab 2 parts), sync clean, 6% of the database used, no write failures. Lab quarantines 4 orphan or superseded operations already on the share, exactly as 0.1.4 does on the same copies.

**Release files** (local only): `.tmp/release-0.2.0/` — `Inventory.Management_0.2.0_x64-setup.exe` (SHA-256 `2053b5da23d1608979caba264ed55197cdd0e96f805f1ff3c70f8ebfb02bb8ce`), `.sig` (verified against the embedded updater key), `latest.json`, `RELEASE_NOTES.md`, `SHA256SUMS.txt`. Source installer: `backend/target/release/bundle/nsis/Inventory Management_0.2.0_x64-setup.exe`.

**Waiting for the owner (outward-facing, not done):**
1. Push `feature/0.2-cleanup`, merge it into `main`, tag `v0.2.0`.
2. S: staging per the runbook: 0.2.0 installer at the product root, the 0.1.4 installer to `archive\`, the five files in `release-support\v0.2.0\`, root `README.md` updated.
3. GitHub release `v0.2.0` (draft → verify assets → publish). Publishing `latest.json` is what makes every installed copy offer the update.

**Know before publishing.**
- If the one-time merge fails on a PC, the old files are untouched but the app stops at startup instead of opening (no message dialog). Safe, but a user would just see the app not open. Worth a native error dialog in a follow-up.
- After the first 0.2.0 PC syncs, 0.1.4 PCs show "Click Update" and stop publishing TE/EVCS snapshots (by design); their edits still sync.
- The release smoke run used this PC's real WebView profile (`%LOCALAPPDATA%\com.inventory.management\EBWebView`), so UI preferences such as the last active view may have changed; no inventory data was touched.
- Still deferred on purpose: pruning `sync:applied` / `sync:seq` markers (Phase 4 note). Still pending, separate owner operations: live IM-014 Lab cutover and IM-015 roster cutover.

## TE divergence incident and 0.1.4 - 2026-10-15

**What happened.** Randy's PC (client `4deef17d`, on 0.1.1 until recently) stopped receiving one of the owner PC's ops (`5c2023b5` #202, written 2026-07-27 by a dev build). Watermarks only advance by +1, so Randy's watermark for `5c2023b5` froze at 201. Every snapshot Randy published therefore "did not cover" the other clients' watermarks, and the old `manifest_covers_local_watermarks` rule made every other client skip them, while Randy's publish compacted his op files. From 2026-08-03 his 403 changes (ops 245-812) never reached anyone, silently. On 2026-10-15 15:46 a 0.1.3 client `c7e1327b` (owner unknown; never authored an op) published its stale state over the manifest. Not a database-location problem: DB paths have not changed since 0.1.0.

**Data restore (done, verified).** Backup of the TE shared folder and Randy's export: `.tmp/te-share-backup-2026-10-15/`. Published `snapshot-3bc82f95b0da43149410e8cab61ba9fb` (574 entries) built from Randy's 2026-10-13 snapshot plus his op #813, with watermarks raised so 0.1.3 clients accept it. Matches `C:\Randy_Inventory\TE_Test_Equipment_Inventory_Export.xlsx` on every field for all 574 rows. Rehearsed first on a copy of the share and a copy of the owner DB using the unchanged 0.1.3 sync loop (569 → 574). Net effect for the team: +6 records, 28 newer edits, -1 test record ("abcd" / 123456789, created 2026-08-17). Previous manifest saved under `shared\inventory\backups\manifest-before-restore-*.json`. Tool: `backend/src/sync/snapshot/restore.rs` (ignored tests, dry run unless `RESTORE_TE_CONFIRM=publish`).

**Fix (0.1.4).** Snapshot merge instead of skip, immediate republish when a client holds work missing from the share, no publishing below the current manifest's watermarks, version gate with "Click Update" message. Applied to TE (`backend/src/sync`) and EVCS Storage (`modules/te_storage/sync`); Lab v1 copy left unchanged (legacy stream is blocked; Lab catalog v2 never compacts). Regression tests: `backend/tests/sync_divergence_flow.rs` (4) and the Storage test; all fail on the old code and pass on the new. TE Storage display name is now **EVCS Storage** (id, DB file, and S: folder unchanged).

**0.1.4 release state.** Commit `d1af176` on branch `release/0.1.4` (not merged to `main`). Signed NSIS installer built; signature verified against the embedded updater public key; SHA-256 `f086c5e9deec7be2a9d435cf98e1a316b18b341b1ecbfd7208c9582364e3e7d6`. Staged on S: (root installer, `release-support\v0.1.4\` with all five files hash-checked, 0.1.3 installer moved to `archive\`, root README updated). Release files: `.tmp/release-0.1.4/`. GitHub Release published 2026-10-16: https://github.com/Hassaan-ECE/Inventory_Management/releases/tag/v0.1.4 (tag `v0.1.4` on `d1af176`, five assets). The public latest.json endpoint returns 0.1.4 and its installer URL downloads the verified SHA-256, so in-app Update on 0.1.3 now offers 0.1.4.

**Follow-ups.** Everyone should update to 0.1.4. Clients that had unsent edits at restore time, or that edit before updating, may still need 0.1.4's merge to converge. Identify who `c7e1327b` is.

## Docs reconciled to code - 2026-10-15

- No behavior changes. Docs-only pass plus five comment corrections (`sync/shared_paths.rs`, `platform/shared_root.rs`, `shell/InventoryShell.tsx`, `shell/ShellHeader.tsx`, `platform/modules/types.ts`).
- Rewritten: `engineering/PROJECT_FOLDER_BREAKDOWN.md` (was the pre-module `features/inventory` layout), `engineering/FEOXDB_SYNC_MIGRATION_PLAN.md` (was the ME 1.0.4 note), `SESSION_START_PROMPT.md` (was 0.1.2, Storage placeholder, dead `.grok` worktree).
- Updated: `engineering/AGENT_RUNBOOK.md` (correct signing key `inventory-management.key`, product share, `.tmp/release-X.Y.Z`, Bun shim resolved, `cargo-audit` not installed), `planning/DECISIONS.md` IM-005/006/011/012 (TE Storage implemented), `README.md`, `AGENTS.md`, `docs/README.md`.
- Recorded facts: sync engine is copied per module; Lab catalog v2 uses whole-entity newest-wins; TE Import UI, HTML export, and window-geometry restore are intentionally off; about 3,250 lines of old Lab v1 frontend UI are no longer loaded (deletion candidate).
- Sections below the 0.1.3 release entry are historical and still say "TE Storage Room remain placeholders"; that was true at the time.

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
