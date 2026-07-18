# TE Test Equipment Inventory — Project Discussion

**Status:** Direction green-lit — next phase is **data/domain validation + scaffold selection**, not full UI  
**Last updated:** 2026-07-13  
**Working name:** TE Test Equipment Inventory  
**Local workspace:** `C:\Projects\Active\Inventory_Apps\TE\TE_Test_Equipment_Inventory`

This document consolidates the product direction, data sources, domain model recommendations, stack choices, and open questions discussed so far.

**Decision authority:** [DECISIONS.md](DECISIONS.md) — accepted, open, and deferred items.  
**Review input:** [SECOND_OPINION_REVIEW.md](SECOND_OPINION_REVIEW.md) — independent findings and acceptance criteria. This §0 summarizes the review, but the decision register wins if planning documents conflict.

---

## 0. Review index

**Verdict:** Proceed. Stack and product boundary are sound. Do **not** freeze the domain model or build complete UI until calibration, import identity, local path, base selection, and shared-drive ops are resolved.

### Do now vs later

| Do now | Do not freeze yet |
|--------|-------------------|
| Data discovery (profile live Excel export) | Full domain model freeze |
| Resolve open items in `DECISIONS.md` | Complete calibration/equipment UI |
| Base selection (TE Components vs ME by commit, not name) | Production shared-sync enablement |
| Scaffold/rebrand after base is chosen | Cutover while Python remains a write source |

### Priority findings (detail in review)

| # | Finding | One-line takeaway |
|---|---------|-------------------|
| 1 | Calibration semantics + history | Split **requirement**, **workflow** (`outToCalibration`), and **derived health**; prefer `CalibrationEvent` history over five overwriteable fields |
| 2 | Import identity | Never auto-merge on manufacturer+model; UUID internal; asset/serial match with conflicts; idempotent batches + provenance |
| 3 | Profile live export first | Headers/duplicates/dates/cal combos must inform schema before UI freeze |
| 4 | Local AppData for FeOx | Prefer `%LOCALAPPDATA%\…\inventory.feox` over Roaming `%APPDATA%` for machine-local DB |
| 5 | Base codebase selection | Compare TE Components vs local ME Tauri; pick by newest hardening; record **repo + commit SHA** |
| 6 | Shared sync | Wire into model early; **config-gate** production until real two-machine S: proof |
| 7 | Backup / restore | Ops logs ≠ backup; define retention, restore drill, cutover backup, Python read-only rollback window |
| 8 | Smaller field semantics | `verifiedAt`/`verifiedBy`; archive ≠ scrapped; pictures multi-machine-safe or defer; location aliases; rental lifecycle if ownership deferred |

### Calibration model (accepted baseline; history still open)

**On equipment:**

- `calibrationRequirement`: `required | reference_only | not_required | unknown`
- `outToCalibration`: separate workflow flag
- Optional `calibrationIntervalMonths`

**Derived health (display/filters/counts):**  
`missing_due | overdue | due_soon | current | not_applicable | unknown | out_to_cal`

**Precedence (local date):** archived/excluded → N/A (exempt) → unknown → out to cal → missing due → overdue → due soon → current.  
**Due soon rule:** `today <= dueDate <= today + 30 days` (default window 30).

**Open history decision:** `CalibrationEvent` would contain event UUID, equipment UUID, performed/next due, vendor, certificate, cost as minor units/decimal rather than float, notes, timestamps, and optional operator. If history is skipped, record the current-state-only limitation in `DECISIONS.md` as an explicit v1 non-goal.

### Import rules (accepted)

- Stable identity = app UUID  
- Asset/serial = normalized match keys; duplicates → warnings/conflicts, not silent unique constraints  
- Manufacturer+model = rank candidates for review only — **never unattended merge**  
- Idempotent reruns; report inserted / matched / conflicted / rejected / ignored  
- No silent discard; keep import batch + source file/sheet/row provenance  

### Accepted build sequence

1. Data discovery  
2. Resolve open decisions  
3. Base selection (+ commit SHA)  
4. Scaffold / rebrand  
5. Domain / storage / sync  
6. Importer (dry-run → commit)  
7. UI  
8. Operational hardening  
9. Migration rehearsal  
10. Cutover  

Full acceptance criteria: [SECOND_OPINION_REVIEW.md](SECOND_OPINION_REVIEW.md) § Suggested acceptance criteria. Decision statuses: [DECISIONS.md](DECISIONS.md).

---

## 1. Problem and goal

### What exists today

| App | Role | Stack | Notes |
|-----|------|--------|--------|
| **TE Lab Equipment (Python)** | Original lab **equipment** inventory | PySide6 + SQLite | Still in use by the lab. Local reference copy: `D:\coding\TE_Lab_Equipment` (not necessarily latest production data on this machine). |
| **TE Component Inventory** | Lab **components/parts** inventory | Tauri 2 + React + TypeScript + Bun + Rust + FeOxDB | Newer stack; UI/UX refinements we want to reuse. Repo: https://github.com/Hassaan-ECE/TE_Component_Inventory |
| **ME Inventory (Tauri v2)** | ME lab inventory on the same modern stack | Same family as TE Components | Local reference: `C:\Projects\Active\Inventory_Apps\ME\ME_Inventory` |

### What we need

A new desktop app that **replaces the Python TE equipment inventory** with the **same modern stack and UI patterns** as TE Components / ME, with the product focus shifting to:

1. **Calibration status** of equipment (primary)
2. **Usual equipment identity** (asset, serial, maker, model, description, condition, lifecycle)
3. **Where it is** (location, and assigned user if still useful)

### What this is not

- Not a redesign of TE Components UI/UX
- Not a fork of the Python GUI
- Not merging TE equipment and TE components into one app (separate identity and data)

---

## 2. Decision summary

The authoritative statuses and rationale live in [DECISIONS.md](DECISIONS.md). This table is a convenience summary.

| Topic | Status | Current position |
|-------|--------|------------------|
| App identity | **Accepted** | **TE Test Equipment Inventory**, separate repository/releases, Tauri id `com.te.test.equipment.inventory` |
| UI / UX | **Accepted** | Keep TE Components / ME patterns; no major redesign |
| Data cutover | **Accepted** | Live Python Excel export first; reviewed historical workbooks second; profile before model freeze |
| Python reference | **Accepted** | `D:\coding\TE_Lab_Equipment` is behavior/schema reference only, not production truth |
| Local DB path | **Accepted** | `%LOCALAPPDATA%\com.te.test.equipment.inventory\inventory.feox` |
| Calibration semantics | **Accepted** | Requirement + separate workflow + derived health; required without due date is `missing_due` |
| Calibration history | **Open** | Choose `CalibrationEvent` history or explicitly accept a current-state-only v1 |
| Import identity | **Accepted** | UUID internal identity; unique asset/serial auto-match only; never manufacturer+model auto-merge |
| Base codebase | **Open** | Compare TE Components vs local ME Tauri and record winning repository + commit SHA |
| Shared multi-machine | **Accepted gate / open path** | Integrate early; enable only after two-machine proof; final department-owned root and ACLs remain open |
| Archive | **Accepted** | Keep archive separate from lifecycle |

---

## 3. Related codebases

### 3.1 Python TE Lab Equipment (reference)

- **Path:** `D:\coding\TE_Lab_Equipment`
- **Runtime DB (dev):** `lab_equipment.db` (or `%LOCALAPPDATA%\TE_Lab_Equipment\lab_equipment.db` for packaged builds)
- **Import sources in repo:** `Data/Master List of Eng.Equipment - All - 2020.RO.xls`, `Data/Survey oF Equip In Eng Lab.xlsx`
- **Capabilities today:** search/filter, table columns, add/edit, quick-edit, re-import Excel, export Excel, HTML report, lifecycle row colors, calibration fields

**Snapshot of a local DB instance inspected during discussion (may be stale vs lab production):**

| Metric | Value |
|--------|--------|
| Rows | 574 |
| Calibration status mix | reference_only ~314, calibrated ~212, unknown ~48 |
| Due dates filled | ~140 |
| Last cal filled | ~209 |
| Lifecycle | mostly active; some scrapped / repair / missing |
| Ownership | all `owned` in that snapshot |
| Top locations | Room 204 Hardware Lab, TE Lab, Room 210 Mech Lab, etc. |

Treat this as **shape of the data**, not the final cutover dataset.

### 3.2 TE Component Inventory (candidate base)

- **Repo:** https://github.com/Hassaan-ECE/TE_Component_Inventory
- **Stack:** Tauri 2, React 19, TypeScript, Vite, Tailwind v4, Bun, Rust, FeOxDB
- **Layout:** `frontend/`, `backend/`, `docs/`, `scripts/`
- **Patterns to reuse:** inventory/archive views, search, column filters, virtualized table, entry dialog, context menu, Excel export, local FeOxDB, shared op-log sync, signed updater, NSIS current-user install

### 3.3 ME Inventory Tauri v2 (candidate base)

- **Local path:** `C:\Projects\Active\Inventory_Apps\ME\ME_Inventory`
- **Why compare it:** the local repository contains substantial sync, conflict, recovery, validation, packaging, and security hardening.
- **Known validation gap:** its handoff still requires a real shared-drive multi-machine smoke, so automated coverage does not remove the TE release gate.

### 3.4 Inventory template / evolution path

- Public template lineage: https://github.com/Hassaan-ECE/Inventory_Management_System---Template  
- Python template already had `show_calibration_section`, equipment-oriented table fields, and TE dual-workbook import profile — useful for field history, not for the new shell.

---

## 4. Domain model — recommended v1

### 4.1 Core equipment + location (keep)

Aligned with TE Components field shape where practical:

- asset number, serial number  
- manufacturer, model, description  
- location, assigned to  
- lifecycle status, working status, condition  
- notes and links  
- `verifiedAt`, optional `verifiedBy`, archived flag  
- certificate/picture reference only after the open attachment decision; do not assume an ordinary local path in multi-machine use  
- entry UUID, created/updated timestamps  

### 4.2 Calibration block (primary addition vs TE Components)

> **Accepted semantic baseline:** [DECISIONS.md](DECISIONS.md) D-005 and D-006.  
> Do **not** implement a mixed `calibrationStatus` enum. Calibration-event history remains an open schema decision.

**Equipment-held:** `calibrationRequirement`, `outToCalibration`, optional interval.  
**History:** Open decision O-001: `CalibrationEvent` rows or an explicitly current-state-only v1.  
**Derived health:** missing due, overdue, due soon, current, N/A, unknown, out to cal — with documented precedence.  
**Python export mapping:** requirement/workflow flags + at most **one seed event** from last cal/vendor/cost/due; do not invent missing history.

Sample-data warning from review: ~212 “calibrated” vs ~140 due dates → many rows cannot be shown as “current” without a due date (`missing_due`, not healthy).

### 4.3 Defer from old Python unless needed later

| Field / concept | Why defer for v1 |
|-----------------|------------------|
| ownership / rental vendor / rental cost | Snapshot was 100% owned |
| manufacturer_raw | Import cleanup only |
| blue_dot_ref, complete legacy source_refs/raw-cell system | Keep lightweight cutover provenance: import batch, file, sheet, row, and original identifiers |
| estimated age / age basis / acquired date | Useful later; not core for cal + location |
| qty | Rarely meaningful for unique instruments |
| project name | Optional later |
| Master/survey dual-import pipeline | Replace with Excel export import for cutover |

The importer must report every source column. Approved mappings may preserve useful values in structured fields or notes; intentionally ignored columns must be named and counted in the dry-run report. Do not silently discard data merely to make migration succeed.

### 4.4 Default table columns (proposal)

`Verified At | Asset | Manufacturer | Model | Description | Location | Cal Health | Cal Due | Lifecycle`

### 4.5 Filters / status strip (proposal)

- Filters: asset, manufacturer, model, description, location, **calibration requirement**, **derived health**, and due window  
- Status strip counts: total, overdue, due soon, missing due, current, and not applicable/reference-only

---

## 5. Calibration requirement, workflow, and derived health

### Stored requirement

| Requirement | Meaning |
|-------------|---------|
| `required` | Equipment participates in the calibration program and requires a due-date assessment. |
| `reference_only` | Used as a reference and not enrolled in the active calibration program. |
| `not_required` | Explicitly determined not to require calibration. |
| `unknown` | Not yet classified. |

### Stored workflow

`outToCalibration` is stored separately from requirement. It indicates that equipment is currently away for calibration; it does not redefine whether calibration is required.

### Legacy Python mapping

| Python status | New requirement | `outToCalibration` |
|---------------|-----------------|--------------------|
| `calibrated` | `required` | `false` |
| `out_to_cal` | `required` | `true` |
| `reference_only` | `reference_only` | `false` |
| `unknown` or missing | `unknown` | `false` |

This is a semantic migration, not a 1:1 copy into another mixed status field.

### Derived display health

Evaluate date-only values using the agreed workstation local date and this precedence:

| Precedence | Health | Rule |
|------------|--------|------|
| 1 | Excluded | Archived or explicitly excluded lifecycle states do not contribute to active calibration counts. |
| 2 | `not_applicable` | Requirement is `reference_only` or `not_required`. |
| 3 | `unknown` | Requirement is `unknown`. |
| 4 | `out_to_cal` | `outToCalibration` is true. |
| 5 | `missing_due` | Requirement is `required` and no valid due date exists. |
| 6 | `overdue` | Due date is before today. |
| 7 | `due_soon` | `today <= dueDate <= today + 30 days`. |
| 8 | `current` | Required equipment has a later due date. |

The 30-day window is the accepted default. It may become configurable later without changing stored calibration meaning.

### Remaining calibration choices

- O-001: calibration-event history in v1 versus an explicit current-state-only limitation  
- O-006: explicit due dates only versus optional interval-based calculation  
- O-007: final user-facing language for `reference_only` and `not_required`

---

## 6. Data migration strategy

### Reality

- Lab is still on the Python app.  
- This machine may not have the latest SQLite.  
- Python DB can contain junk mixed with real equipment.  
- Owner has: **Excel export capability from Python** + **older Excel files** with rows never fully entered.

### Recommended cutover

1. **Primary import:** Excel export from the live Python app (full snapshot of what they use day to day).  
2. **Secondary review/import:** older Excel workbooks for missing rows. Automatically match only a unique normalized asset or serial number. Use manufacturer+model only to rank human-review candidates; never auto-merge on it.  
3. **No ongoing dependency** on Python SQLite after cutover.  
4. Normalize locations through a reviewable alias map while preserving the imported value and source provenance.  
5. Map legacy calibration values through the semantic table in §5; do not recreate the mixed Python status enum.  
6. Account for junk rows explicitly as imported, rejected, conflicted, or intentionally ignored; retain valid junk records for archive/filter cleanup when appropriate.

### Import product shape (v1)

- Prefer a testable **Import Excel** path or one-shot cutover tool with dry-run mapping, conflict reporting, reconciliation totals, provenance, and deterministic/idempotent batch commit.  
- Full dual-workbook master/survey pipeline from Python can wait.

### When ready for migration

Place files under something like:

```text
TE_Lab_Equipment_Inventory/
  data/
    import/
      # Python export workbook(s)
      # leftover historical Excel files
```

---

## 7. Stack and architecture

### 7.1 Implementation base

Compare the current **TE Component Inventory** repository with local **ME Inventory Tauri v2** for storage, sync, recovery, validation, packaging, and security hardening. Choose the stronger base or deliberately port the missing hardening, then record the selected repository and exact commit SHA in [DECISIONS.md](DECISIONS.md) before scaffolding.

Target layout (same spirit as TE Components):

```text
TE_Lab_Equipment_Inventory/
  frontend/     # React/Vite UI
  backend/      # Tauri/Rust, storage, sync, export
  docs/         # engineering notes (this file lives here)
  scripts/      # smoke / automation
  data/         # optional import sources (not live DB)
```

### 7.2 Why a Tauri identifier like `com.te.test.equipment.inventory`?

This is **not** the user-facing name. Users see **TE Test Equipment Inventory**.

The Tauri `identifier` is reverse-DNS style app identity used for:

1. **Uniqueness** — must differ from TE Components (`com.te.lab.components.inventory`) and ME (`com.me.inventory`) so apps don’t share appdata/updater state.  
2. **App-specific path suffix** — combined with the selected Tauri base directory; this project uses `%LOCALAPPDATA%\com.te.test.equipment.inventory\`  
3. **Installer / updater** — stable product identity across releases  

Shorter ids (e.g. `com.te.equipment`) are fine if **stable forever** and **unique**. Matching the Components naming pattern is for consistency only.

**Proposed:** `com.te.test.equipment.inventory`  
**Display name:** TE Test Equipment Inventory  

### 7.3 Why local FeOxDB (and which AppData)

**What it means**

- Each machine has its own working DB file.  
- App opens/reads/writes that file for normal use.  
- Shared S-drive holds **operation logs, snapshots, manifest** — not one multi-user FeOx file everyone opens.

**Roaming vs Local decision**

| Path | Windows folder | Project decision |
|------|----------------|------------------|
| Tauri `app_data_dir()` | often **Roaming** `%APPDATA%\…` | Do not inherit for this database without a new recorded decision. |
| Tauri `app_local_data_dir()` | **Local** `%LOCALAPPDATA%\…` | **Accepted default** for TE Test Equipment Inventory. |

**Accepted default:**

```text
%LOCALAPPDATA%\com.te.test.equipment.inventory\inventory.feox
```

Validate the choice against organizational Windows profile policy before release. Starting Local is easier than moving a populated database later.

**Why local DB + explicit sync fits**

| Concern | Local DB + S: sync artifacts | Single DB only on S: | DB next to repo / install dir |
|--------|------------------------------|----------------------|--------------------------------|
| S: / VPN down | UI still works offline | Breaks or locks | Works if path is local |
| Multi-machine edits | Op-log merge | File locks / corruption risk | No real multi-user |
| Upgrades / reinstall | Data survives under LocalAppData | Depends on path | Easy to wipe with install folder |

**Dev override:** env var for DB path (same idea as Python’s `TE_LAB_EQUIPMENT_DB_PATH`).

### 7.4 Shared root (proposal)

Keep TE equipment **separate** from TE components shared data.

Open decision O-004 must select a **durable lab/department-owned** folder with explicit ACL, backup, and naming ownership. The person-path example remains a staging sketch only:

```text
S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE\Equipment\
  # installer at root (optional)
  shared\
    inventory\
      manifest.json
      ops\
      snapshots\
      ...
  release-support\
    ...
```

**Production shared sync:** config-gated until two-machine create/update/archive/delete, conflict, offline, snapshot bootstrap, and recovery tests pass. Backup/restore plan required before release (ops logs alone are not backup).

Env override pattern (mirror Components):

- Shared root: e.g. `TE_LAB_EQUIPMENT_SHARED_ROOT`  
- Optional HMAC: e.g. `TE_LAB_EQUIPMENT_SYNC_HMAC_KEY`  

Do **not** point equipment clients at the components shared root.

### 7.5 Packaging / updates

- Current-user NSIS install (like TE Components)  
- Own GitHub Releases + signed Tauri updater + own updater keypair  
- Own preference keys (e.g. `teLabEquipmentInventory.*`) so themes/columns don’t clash with Components  

---

## 8. UI / UX scope

### Keep (from TE Components / ME)

- Single main window, inventory + archive scopes  
- Global search, column filters, sort, column visibility  
- Color rows, theme persistence  
- Virtualized table  
- Add / edit dialog, context menu (open, link, online search, archive/restore, delete)  
- Excel export  
- Status strip  
- Shared sync status messaging patterns  

### Add / adapt (equipment-specific)

- Calibration requirement/workflow section in the entry dialog  
- Calibration health + due date columns default visible  
- Derived missing-due / overdue / due-soon / current badges and filters  
- Status-strip counts based on derived health rather than a stored `calibrated` value  
- Minimal calibration-event add/list UI only if O-001 selects history for v1  
- `verifiedAt` display/edit behavior; `verifiedBy` only if O-005 adopts operator attribution  
- Equipment-oriented labels/copy (“equipment” vs “entry/parts” where it matters)  
- Export columns include requirement, workflow, derived health, dates, and event history when adopted  

### Avoid for v1

- Major visual redesign  
- Dashboard-heavy reporting (unless a lightweight due-list view is easy later)  
- Full Python import pipeline and raw-cell search UI  

---

## 9. High-level build plan

1. **Data discovery** — obtain/profile a representative live Excel export outside git; document headers, duplicates, invalid dates, location aliases, and calibration combinations.  
2. **Resolve open domain decisions** — history, interval behavior, attachments, and operator attribution in `DECISIONS.md`.  
3. **Base selection** — compare TE Components vs ME Tauri; choose and record repository + commit SHA.  
4. **Scaffold/rebrand** — establish app identity, preference keys, updater identity, and LocalAppData path.  
5. **Domain/storage/sync** — implement equipment and calibration across validation, persistence, merge, snapshots, conflicts, export, and tests.  
6. **Importer** — dry run, mappings, provenance, conflict workflow, reconciliation totals, deterministic batch commit.  
7. **UI** — existing shell plus equipment/calibration editing, derived health, filters, counts, verification, and archive.  
8. **Operational hardening** — backup/restore, one-machine sync, real two-machine sync, packaging, updater, and security checks.  
9. **Migration rehearsal** — import copies of cutover files, reconcile exceptions, and obtain owner approval.  
10. **Cutover** — freeze Python writes, take final backups/exports, import, validate, and retain the documented rollback window.  

---

## 10. Remaining open decisions and required inputs

The complete authoritative list is in [DECISIONS.md](DECISIONS.md). The items blocking later phases are:

| Item | Status | Needed before |
|------|--------|---------------|
| Representative live export | Required input | Model freeze and importer implementation |
| Calibration-event history | Open product decision | Domain/schema freeze |
| Explicit versus interval-derived due dates | Open product decision | Calibration workflow implementation |
| Certificate/picture handling | Open product/IT decision | Domain/schema freeze |
| `verifiedBy` and calibration operator attribution | Open product decision | Domain/schema freeze |
| TE Components vs ME base + exact commit | Open engineering decision | Scaffold/rebrand |
| Final shared root, ACLs, naming, and backup owner | Open IT/operations decision | Production shared-sync enablement |
| Real two-machine shared-drive proof | Required validation | Production shared-sync enablement |

---

## 11. Explicit non-goals (for now)

- Merging equipment + components inventories  
- Keeping Python as a long-term parallel write path  
- Shared single SQLite/FeOx file on the network as the live multi-writer store  
- Perfect cleaning of all historical junk data in v1 (import + archive/filter is enough)  
- Full UI implementation before data discovery + resolution of blocking open decisions  

---

## 12. Suggested next steps

1. **Data discovery** — obtain/profile representative live Excel export (out of git); document anomalies.  
2. **Resolve blocking open decisions** — history, interval behavior, attachments, and operator attribution; record outcomes in `DECISIONS.md`.  
3. **Base selection** — compare TE Components vs ME Tauri; choose; document **repo + commit SHA**.  
4. **Scaffold/rebrand** — new repo identity, preference keys, updater, local DB path.  
5. **Domain/storage/sync** — equipment + calibration across validation, merge, snapshots, export, tests.  
6. **Importer** — dry-run, provenance, conflicts, deterministic batch.  
7. **UI** — cal summary/history, derived badges, filters, counts, archive.  
8. **Operational hardening** — backup/restore, one- then two-machine sync, packaging.  
9. **Migration rehearsal** — reconcile totals; owner approval.  
10. **Cutover** — freeze Python writes; final backup/import; rollback window.  

Acceptance criteria: see [SECOND_OPINION_REVIEW.md](SECOND_OPINION_REVIEW.md). Authoritative statuses: [DECISIONS.md](DECISIONS.md).

---

## 13. Quick reference — identity sketch

| Item | Current value/status |
|------|----------------------|
| Display name | TE Test Equipment Inventory |
| Folder | `C:\Projects\Active\Inventory_Apps\TE\TE_Test_Equipment_Inventory` |
| Tauri identifier | `com.te.test.equipment.inventory` |
| Local DB | `%LOCALAPPDATA%\com.te.test.equipment.inventory\inventory.feox` |
| Shared root | Lab/dept-owned path TBD; person-path only as interim sketch |
| Base fork | **TBD** after TE Components vs ME comparison (+ commit SHA) |
| Decision authority | [DECISIONS.md](DECISIONS.md) |
| Second opinion | [SECOND_OPINION_REVIEW.md](SECOND_OPINION_REVIEW.md) |
| Python behavior reference | `D:\coding\TE_Lab_Equipment` |
| Data cutover | Live Excel export + leftover workbooks (profile first) |

---

*Decision authority lives in DECISIONS.md. This discussion supplies context and must be reconciled whenever a decision changes.*
