# TE Test Equipment Inventory — Import Profile

**Status:** Live aggregate profile available; source corrections and cutover rehearsal remain blocking

**Last checked:** 2026-07-14

**Authority:** [DECISIONS.md](DECISIONS.md), especially D-004, D-007, D-008, D-025, and D-026

## Evidence boundary

The locally supplied `data/import/TE_Lab_Equipment_Export.xlsx` was read only for aggregate profiling and importer dry-run verification. The workbook is gitignored and must never be committed. This document records no row contents, identifier values, or source-file hash.

Visible workbook sheets:

| Sheet | Data rows | Import treatment |
|-------|----------:|------------------|
| `Inventory` | 573 | Selected as the inventory source. |
| `Import Issues` | 313 | Supporting sheet excluded by sheet selection. |
| `Export Summary` | 20 | Supporting sheet excluded by sheet selection. |

Supporting-sheet rows are not inventory rows and are not counted as ignored rows. The importer selects the single visible usable sheet named `Inventory`, case-insensitively, while preserving its actual sheet name in batch identity and provenance.

## Live aggregate observations

The selected `Inventory` sheet contains 573 rows. Aggregate calibration and verification observations are:

- calibration status: 211 `calibrated`, 314 `reference_only`, and 48 `unknown`;
- 209 nonblank last-calibration dates;
- 140 nonblank calibration due dates;
- 414 nonblank calibration vendors; and
- 115 true timeless verification flags.

Aggregate identity observations are:

- 145 blank asset numbers;
- 134 blank serial numbers;
- 41 rows blank in both asset and serial identity;
- four duplicate normalized asset keys across eight rows; and
- twelve duplicate normalized serial keys across 48 rows.

No row values or identifier keys are recorded here. Duplicate identity participation overlaps, so key-participation counts must not be added to infer the number of conflicted rows.

## Exact 22-column treatment

Header matching is case-insensitive, trims surrounding whitespace, and normalizes spaces/punctuation to underscores. Original headers and raw cell values remain available in the dry-run report. Every live column is classified below.

| Live header | Normalized target | Treatment |
|-------------|-------------------|-----------|
| `Asset Number` | `asset_number` | Mapped |
| `Serial Number` | `serial_number` | Mapped |
| `Manufacturer` | `manufacturer` | Mapped |
| `Model` | `model` | Mapped |
| `Description` | `description` | Mapped |
| `Location` | `location` | Mapped |
| `Assigned To` | `assigned_to` | Mapped |
| `Lifecycle` | `lifecycle_status` | Mapped |
| `Working` | `working_status` | Mapped |
| `Condition` | `condition` | Mapped |
| `Cal Status` | `calibration_status` | Mapped |
| `Last Cal Date` | `last_calibrated_at` | Mapped |
| `Cal Due Date` | `calibration_due_at` | Mapped |
| `Cal Vendor` | `calibration_vendor` | Mapped |
| `Cal Cost` | — | Intentionally ignored; calibration cost is deferred. |
| `Ownership` | — | Intentionally ignored; ownership/rental detail is deferred. |
| `Rental Vendor` | — | Intentionally ignored; ownership/rental detail is deferred. |
| `Rental Cost/Mo` | — | Intentionally ignored; ownership/rental detail is deferred. |
| `Verified` | `verified` | Mapped. A valid true flag without `verifiedAt` leaves `verified_at` empty and reports `Timeless verified flag ignored; re-verification required`. |
| `Blue Dot` | — | Intentionally ignored; the legacy marker is deferred. |
| `Est. Age (Yrs)` | — | Intentionally ignored; estimated age is deferred. |
| `Notes` | `notes` | Mapped |

The importer also recognizes the normalized live aliases `cal_due_date` and `cal_vendor`. Deferred age ignores are deliberately limited to `est_age_yrs`, `est_age_years`, `estimated_age_years`, `age`, and `age_years`. Genuinely unknown columns with nonblank values remain row-rejecting.

## Calibration and verification semantics

| Legacy value or combination | `calibrationRequirement` | `outToCalibration` | Required handling |
|-----------------------------|--------------------------|--------------------|-------------------|
| `calibrated` | `required` | `false` | Preserve explicit last/due dates. A missing due date remains valid input and derives `missing_due`, never `current`. |
| `out_to_cal` | `required` | `true` | Derived health is `out_to_cal` according to accepted precedence. |
| `reference_only` | `reference_only` | `false` | User-facing label is **Reference only**; derived health is `not_applicable`. |
| Explicit `not_required` | `not_required` | `false` | User-facing label is **Not required**; derived health is `not_applicable`. |
| `unknown`, blank, or missing status | `unknown` | `false` | Do not infer `required` from dates alone. |
| Separate requirement plus out-to-cal flag | Mapped requirement | Parsed flag | Keep the concepts separate; contradictory values reject the row. |

An explicit calibration due date remains authoritative. An interval may suggest a due date but cannot overwrite an imported explicit value. Current-state fields may be seeded, but the importer does not manufacture prior `CalibrationEvent` records.

A supplied valid `verifiedAt` must be RFC 3339. Invalid verification booleans and invalid supplied timestamps remain rejecting. A true timeless flag is non-blocking but does not invent a historical timestamp; it produces the exact re-verification issue shown in the mapping table.

## Verified live dry run

The mapping version is `te-test-equipment-v2`. Against a temporary empty FeOxDB, the selected `Inventory` sheet produced:

| Total | Inserted | Matched | Conflicted | Rejected | Ignored | Blocking |
|------:|---------:|--------:|-----------:|---------:|--------:|:--------:|
| 573 | 515 | 0 | 50 | 8 | 0 | `true` |

The reconciliation equation holds:

```text
573 = 515 + 0 + 50 + 8 + 0
```

The 50 identity-conflicted rows and eight invalid-date rows must be corrected in the source and dry-run again before any live commit. Under D-026, the v0.1 desktop path is full-batch-only: conflicts or rejections block the batch, and the desktop command rejects partial commit requests. Matched and intentionally ignored rows are durable no-ops.

Do not partial-load the 515 currently insertable rows into real Local AppData. Batch identity is derived from workbook content, selected sheet, and mapping version, so correcting the workbook creates a different batch identity. Rows that uniquely match on asset or serial may reconcile as no-ops, but the 41 rows with neither identity are especially vulnerable to being inserted again.

## Importer and synthetic verification boundary

The v0.1 importer accepts local `.csv`, `.xlsx`, and `.xls` paths. CSV and generated XLSX fixtures exercise parsing and mapping. The `.xls` path is routed through calamine and invalid binary input produces an explicit error, but a valid binary `.xls` fixture has not been generated.

Synthetic fixtures cover case-insensitive `Inventory` sheet preference, ambiguity without an `Inventory` sheet, the exact 22 live headers, spaced and snake-case calibration aliases, due/vendor preservation, calibrated-without-due behavior, valid timeless verification diagnostics, invalid verification inputs, unknown nonblank columns, duplicate identities, all five row classifications, commit provenance, idempotency, and durable matched/no-op behavior. Synthetic fixtures contain no live lab inventory data.

Batch identity is derived from source content, selected sheet, and mapping version. Commit requires explicit confirmation, re-parses the source, revalidates source and reconciliation state, and writes inserts through normal FeOxDB mutation/outbox paths. The importer engine retains an opt-in partial path for synthetic/internal tests, but the v0.1 desktop command rejects it and the Local AppData live-load helper requires a non-blocking preview plus a full commit.

## Release and operations boundary

Aggregate profile completion is not cutover completion. Do not commit the workbook, partial-load real Local AppData, enable production shared sync, publish, install on lab PCs, delete lab data, or retire the Python workflow. Before cutover, protect the corrected source, obtain a non-blocking dry run, rehearse the full-batch import and restore, prove backup retention and rollback, and retain the documented Python read-only window. Sync artifacts are not a backup.
