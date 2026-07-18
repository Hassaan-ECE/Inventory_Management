# TE Test Equipment Inventory — Engineering Suggestions

**Author:** working notes from implementation planning (Grok / build session)  
**Date:** 2026-07-13  
**Audience:** owner + future implementers  

This file is advisory and is **not** a decision record. It captures practical recommendations on process, domain, stack, and optional UI tools (TanStack Table, Reicon).

**Decision authority:** [DECISIONS.md](DECISIONS.md)  
**Review input:** [SECOND_OPINION_REVIEW.md](SECOND_OPINION_REVIEW.md)  
**Product discussion:** [PROJECT_DISCUSSION.md](PROJECT_DISCUSSION.md)

---

## 1. Overall stance

Agree with the second-opinion verdict:

- **Proceed** with the project.
- Next phase is **data/domain validation + scaffold selection**, not full UI or a frozen schema.
- Largest risks are **calibration semantics**, **import identity**, and **multi-machine ops** — not Tauri/React.

Stack and product boundary (equipment ≠ components, reuse inventory shell UX) remain sound.

---

## 2. Process recommendations

### Recommended sequence (aligned with the review)

1. Obtain and **profile** a live Python Excel export (keep out of git; use `data/import/` locally only if needed).  
2. Resolve blocking **open decisions** in `DECISIONS.md` (history, intervals, attachments, operator attribution).  
3. **Compare bases** TE Component Inventory vs local ME Tauri v2; pick winner; record **repo + commit SHA**.  
4. Scaffold / rebrand only after base selection.  
5. Domain + storage + sync (including calibration structures in merge/snapshot/export).  
6. Importer (dry-run → conflict report → commit).  
7. UI (derived cal health, filters, simple event add/list).  
8. Ops: backup/restore, one- then two-machine shared sync.  
9. Migration rehearsal → cutover with Python read-only rollback window.

### Do not jump the queue for

- Full calibration UI polish  
- Table rewrites (TanStack)  
- Icon library migrations (Reicon)  
- Production shared-sync enablement before two-machine proof  

---

## 3. Domain recommendations

### Calibration model

Follow accepted decision D-005 and split calibration semantics rather than using five flat mutable fields:

| Layer | Examples |
|--------|----------|
| **Requirement** (on equipment) | `required \| reference_only \| not_required \| unknown` |
| **Workflow** (on equipment) | `outToCalibration` (separate from requirement) |
| **History** | `CalibrationEvent` rows (performed date, due, vendor, certificate, cost, notes…) |
| **Derived health** | `missing_due`, `overdue`, `due_soon`, `current`, `not_applicable`, `unknown`, `out_to_cal` |

**Recommendation for open decision O-001 (not yet accepted)**

- **Storage:** include `CalibrationEvent` from day one (even if import only creates 0–1 events per asset).  
- **UI v1:** table shows derived health + last/due; dialog = current summary + “add calibration” (append event); history list can be minimal.  
- **Explicit non-goal v1** unless requested: full compliance vault, multi-year audit reports, heavy cal-job workflow.

**Due-soon rule (agree with review):**  
`today <= dueDate <= today + 30 days` (local workstation date). Make window configurable later if needed.

**Required without due date → `missing_due`**, never “current.” Sample data (~212 calibrated vs ~140 due dates) makes this mandatory.

### Import identity

- Internal identity = app UUID.  
- Asset / serial = normalized match keys; **duplicates → conflicts**, not silent unique merge.  
- **Never unattended merge on manufacturer + model** (rank candidates for review only).  
- Idempotent import batches; report inserted / matched / conflicted / rejected / ignored.  
- Keep lightweight provenance: batch id, source file, sheet, row.

### Other field notes

- Prefer **`verifiedAt`** (and optional `verifiedBy`) over a timeless `verified` boolean when easy.  
- Keep **archive** separate from lifecycle (`scrapped` ≠ archived).  
- **Pictures:** managed shared media, UNC/URL, or **defer v1** — do not rely on absolute local paths multi-machine.  
- **Money:** avoid binary float for cal cost (minor units or decimal string). Don’t block the project on accounting precision.  
- **Locations:** reviewable alias/normalization; preserve imported raw value for audit.

---

## 4. Storage and identity recommendations

| Item | Suggestion |
|------|------------|
| Tauri identifier | Keep `com.te.test.equipment.inventory` (stable forever) |
| Local DB | Prefer **`%LOCALAPPDATA%\com.te.test.equipment.inventory\inventory.feox`** (Local, not Roaming) |
| Shared root | Lab/dept-owned path long-term; person-path OK for pilot if ACLs/backup are understood |
| Sync | Integrated in model early; **config-gated** until real two-machine S: validation |
| Backup | Sync ≠ backup; define retention, restore drill, cutover Excel/Python backup, rollback window |

---

## 5. Base codebase selection

Do **not** clone “by product name” only.

1. Diff TE Component Inventory (GitHub) vs local `ME_Inventory` (`C:\Projects\Active\Inventory_Apps\ME\ME_Inventory`) for: storage, sync, recovery, validation, packaging, security.  
2. Choose the better base **or** port missing hardening intentionally.  
3. Record in docs: **repository URL + commit SHA**.

UI branding can still match TE Components even if ME is the stronger sync base.

---

## 6. UI toolkit notes (optional later)

### TanStack Table — [tanstack.com/table](https://tanstack.com/table/latest)

**What it is:** Headless table state (columns, sort, filter, visibility, row models). Often paired with TanStack Virtual for large lists.

**What you already have:** Custom inventory table + fixed-row-height virtualization in ME/TE Components style (`InventoryTable`, `getVisibleRange`).

| Phase | Recommendation |
|--------|----------------|
| Domain / import / scaffold | **Do not adopt yet** — avoid UI rewrite noise |
| First equipment UI | **Port existing InventoryTable** patterns so UX matches sibling apps |
| Later | **Optional** if column/filter state becomes hard to maintain; keep the same visual shell |

**Opinion:** Worth learning; not a v1 pillar. Inventory size (hundreds–low thousands) does not force a rewrite. If adopted later, treat as **internals**, not a redesign.

### Reicon — local clone `D:\Projects\Reicon`

**What it is:** Free open-source SVG icon set (Outline/Filled), React packages (`reicon-react`), demo and docs in your Projects tree. Site: [reicon.dev](https://reicon.dev/).

**What you already have:** Sibling inventory apps use **lucide-react**.

| Choice | When |
|--------|------|
| Keep lucide for TE Equipment | Forking TE Components/ME — consistency across lab inventory apps |
| Switch to Reicon | Deliberate family-wide icon migration, or a greenfield UI not tied to lucide |
| Avoid | Mixing lucide + Reicon in one app without a rule |

**Opinion:** Fine library; **last 2% polish**. Does not affect calibration, import, or sync. Prefer matching TE Components icons unless you intentionally standardize on Reicon later.

### Priority vs domain work

```text
High  →  cal semantics, import identity, live Excel profile
      →  base commit, LocalAppData, sync gates, backup
      →  scaffold, domain, importer, existing-shell UI
Low   →  TanStack Table refactor
      →  Reicon icon swap
```

---

## 7. Decision-status snapshot

The authoritative and complete list is [DECISIONS.md](DECISIONS.md).

**Accepted:**

- Requirement + separate `outToCalibration` workflow + derived health model  
- Due-soon window = 30 days, inclusive of today  
- Local FeOx under LocalAppData  
- Tauri id `com.te.test.equipment.inventory`  
- Dry-run/idempotent import, no manufacturer+model auto-merge, provenance retained  
- Shared sync integrated early and disabled in production until two-machine proof  

**Still open:**

- Calibration history in v1  
- Explicit versus interval-derived due dates  
- Base repository + exact commit SHA  
- Picture/certificate handling  
- Operator attribution  
- Final shared-folder ownership, ACLs, backup owner, and naming  

---

## 8. One-line stakeholder summary

> Build TE Test Equipment Inventory on the proven desktop inventory stack, but lock calibration rules, import identity, and multi-machine ops from real export data before freezing the schema or shipping a full UI.

---

## 9. Related paths

| Path | Role |
|------|------|
| `C:\Projects\Active\Inventory_Apps\TE\TE_Test_Equipment_Inventory` | This project |
| `D:\coding\TE_Lab_Equipment` | Python reference (behavior/schema shape; not latest data) |
| `C:\Projects\Active\Inventory_Apps\ME\ME_Inventory` | Candidate modern base (local) |
| TE Component Inventory (GitHub) | Candidate modern base + UI lineage |
| `D:\Projects\Reicon` | Local Reicon library / demo (optional later) |

---

*Edit freely. When a decision changes, update DECISIONS.md first and reconcile this file in the same change.*
