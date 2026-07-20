# First Team Release — Inventory Management (Phase D)

> **For agentic workers / owner ops:** Checkbox plan. Prefer thin slices. Do not reuse TE/ME updater keys.

**Goal:** Ship a team-installable **Inventory Management** NSIS build that runs **TE Test Equipment** + **TE Lab Components** against **product module shared roots** on S:, with a clear cutover from InventoryApps pilots. ME Storage and TE Storage Room stay placeholders.

**Architecture:** One installer (`com.inventory.management`). Shared data remains **per module**. Pilot defaults currently point at InventoryApps; release flips to `Inventory_Management_App\modules\...` after a deliberate data copy. Auto-updater is **optional for the first ship** (manual install is enough per IM-009); keys/endpoints are required only if enabling in-app update.

**Tech / identity (do not change):**
- Package `inventory-management` (version below)
- Tauri id `com.inventory.management`
- Product share: `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App`
- GitHub: `https://github.com/Hassaan-ECE/Inventory_Management`

**Status:** Planning + cutover tooling started 2026-07-20. Not released yet.

**Recommended first version:** keep **`0.1.0`** for the first team installer (or bump to `0.2.0` if you want a clear “team pilot” tag). Do not jump to 1.0.0 unless you want that signal.

---

## 0. Non-goals (this release)

- Port ME Storage / TE Storage Room
- Merge inventories or shared ops streams
- Reuse TE Test Equipment / ME updater private keys or GitHub release endpoints
- Force dual-write unified + standalone to one root
- Automatic migration of old-app installers via TE/ME updaters

---

## 1. Release decision matrix (owner)

| Decision | Options | Recommendation |
|----------|---------|----------------|
| Version string | `0.1.0` / `0.2.0` / `1.0.0` | **`0.1.0`** unless you already distributed 0.1.0 internally |
| Auto-updater in v1 | On / **Off** | **Off for first ship** (installer only); enable keys+Releases in a fast follow |
| When to flip shared defaults | After copy + QA / At same time as installer | **After copy + smoke on product roots** |
| Standalone writers | Stop on cutover day / Parallel temporarily | **Stop writers** on cutover for TE + Lab shares being migrated |

Confirm before code flip of defaults.

---

## 2. Preflight (owner + machine)

- [ ] **P1** Code on `main` is green (lint/test/build + recent C1 smoke).
- [ ] **P2** Close **standalone** TE Test Equipment + TE Lab Components if they write to InventoryApps roots.
- [ ] **P3** Close unified `bun run desktop` during shared **copy** (or accept that a concurrent writer can race the copy).
- [ ] **P4** S: reachable; product folders exist:
  - `...\Inventory_Management_App\modules\TE_Test_Equipment`
  - `...\Inventory_Management_App\modules\TE_Lab_Components`
  - `...\Inventory_Management_App\release-support`
- [ ] **P5** Note current pilot data health in Shared mode for both modules.

---

## 3. Slice D-A — Copy live shared data into product modules

**Authority script:** `scripts/release/copy-shared-to-product-modules.ps1`

Source → destination:

| Module | From (pilot) | To (product) |
|--------|--------------|--------------|
| TE Test Equipment | `InventoryApps\TE_Test_Equipment_Inventory\shared\inventory` | `Inventory_Management_App\modules\TE_Test_Equipment\shared\inventory` |
| TE Lab Components | `InventoryApps\TE\shared\inventory` | `Inventory_Management_App\modules\TE_Lab_Components\shared\inventory` |

- [ ] **A1** Dry-run the script (`-WhatIf` / report mode).
- [ ] **A2** Run the real copy (robocopy `/E`, no `/MIR` by default — do not delete unexpected files on first pass unless owner confirms).
- [ ] **A3** Spot-check: `manifest.json`, `ops\`, `snapshots\` present; snapshot/op counts look sane vs source.
- [ ] **A4** Record copy timestamp in `docs/SESSION_HANDOFF.md`.

**Do not** delete InventoryApps trees yet.

---

## 4. Slice D-B — Flip app defaults to product roots

**Code:** `backend/src/platform/shared_root.rs`

- [ ] **B1** Set `DEFAULT_SHARED_ROOT` → product `...\modules\TE_Test_Equipment`
- [ ] **B2** Set `DEFAULT_LAB_COMPONENTS_SHARED_ROOT` → product `...\modules\TE_Lab_Components`
- [ ] **B3** Keep `PRODUCT_*` constants consistent (or remove dead pilot constants).
- [ ] **B4** Update unit tests for new default path suffixes.
- [ ] **B5** Update AGENTS / SESSION_HANDOFF / README / DECISIONS (IM-006 / IM-013) to “release defaults = product modules”.
- [ ] **B6** `cargo test --lib` for platform shared_root; full frontend test if docs-only skip.

Env overrides remain for diagnostics:

- `INVENTORY_MANAGEMENT_SHARED_ROOT`
- `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT`

---

## 5. Slice D-C — Smoke on product roots

- [ ] **C1** `bun run desktop` with **no** override env.
- [ ] **C2** TE: Shared, row counts roughly match pre-cutover; cal columns still work.
- [ ] **C3** Lab: Shared, no cal UI; verified pill; create/edit one **test** row if safe (or use a throwaway asset).
- [ ] **C4** Switch TE ↔ Lab ↔ ME placeholder; no Illegal invocation; one writer only.
- [ ] **C5** Optional: point env back to InventoryApps and confirm override still works (diagnostic).

---

## 6. Slice D-D — Build NSIS installer (updater optional)

### D-D1 Installer only (recommended first ship)

- [ ] Align versions in `package.json`, `backend/Cargo.toml`, `backend/tauri.conf.json` (and window title if hardcoded).
- [ ] Confirm `createUpdaterArtifacts: false` **or** leave true only after keys exist.
- [ ] `bun run build:desktop` (NSIS).
- [ ] Copy setup exe to:
  - `S:\...\Inventory_Management_App\Inventory Management_<ver>_x64-setup.exe` (or Tauri’s produced name)
  - `S:\...\Inventory_Management_App\release-support\vX.Y.Z\` archive of build outputs + notes
- [ ] Install on a clean/user profile; launch; both modules Shared on product roots.

### D-D2 Auto-updater (can be same release or fast follow)

- [ ] Generate **new** Tauri signer keypair for this product only (`tauri signer generate` / current Tauri 2 docs).
- [ ] Store **private** key outside the repo (password manager / secure store). **Never commit.**
- [ ] Put **public** key in `tauri.conf.json` → `plugins.updater.pubkey`.
- [ ] Set `createUpdaterArtifacts: true`.
- [ ] Set `endpoints` to GitHub latest JSON for **this** repo only, e.g.  
  `https://github.com/Hassaan-ECE/Inventory_Management/releases/latest/download/latest.json`  
  (exact path must match Tauri 2 updater plugin docs for your CLI version).
- [ ] Build signed artifacts; create GitHub Release `vX.Y.Z` with installer + `.sig` / updater files.
- [ ] Verify in-app Update check against the release (one machine).

**Never** copy TE’s pubkey or release URL into this product.

---

## 7. Slice D-E — Team cutover playbook

Publish a short note (product share README or email) covering:

1. Install Inventory Management from product share root.
2. For TE Test Equipment: stop using standalone TE writer; unified app owns product `modules\TE_Test_Equipment`.
3. For Lab Components: stop using standalone Lab writer; unified owns `modules\TE_Lab_Components`.
4. Old InventoryApps installers remain available as **rollback** until stable.
5. Sync is not a backup.
6. Placeholders: ME Storage / TE Storage Room not ready.

- [ ] **E1** Update `S:\...\Inventory_Management_App\README.md` with install + cutover text and date.
- [ ] **E2** Update repo `docs/SESSION_HANDOFF.md` release section with version, paths, and residual risks.
- [ ] **E3** Do **not** archive standalone repos until team is stable on unified writers.

---

## 8. Definition of done (first team release)

1. Product module shares hold copied TE + Lab shared data.
2. App defaults point at product modules (no InventoryApps default).
3. NSIS installer on product share root; team can install without this dev PC.
4. Smoke: TE + Lab Shared on product roots after clean install.
5. Standalone writers stopped for those two modules (or explicitly temporary dual-path documented — prefer stopped).
6. Updater either **documented deferred** or fully wired with **new** keys + GitHub Release.
7. Handoff docs updated.

---

## 9. Suggested execution order (this week)

1. Owner confirms version + updater on/off for v1.  
2. Run **D-A** copy script (writers closed).  
3. Implement **D-B** default flip + tests + docs.  
4. **D-C** smoke.  
5. **D-D1** build + place installer.  
6. **D-E** cutover note.  
7. Optional **D-D2** updater as same-day or next PR.

---

## 10. Implementor paste (when code slices start)

```text
You are releasing Inventory Management (Phase D).

Workspace: C:\Projects\Active\Inventory_Management
Plan: docs/superpowers/plans/2026-07-20-first-team-release.md

Identity: inventory-management / com.inventory.management — do not change.
Do not reuse TE/ME updater keys.
Do not port ME/Storage Room.

Current pilot defaults still point at InventoryApps until D-B.
Copy script: scripts/release/copy-shared-to-product-modules.ps1

Order: complete D-A (owner) → D-B code flip → D-C smoke → D-D1 installer.
Updater D-D2 only if owner approved keys + GitHub Release for this repo.

First reply: confirm git status, version target, and whether updater is in or out of this ship.
```
