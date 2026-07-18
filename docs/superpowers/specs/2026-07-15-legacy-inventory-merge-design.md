# Legacy Inventory One-Time Merge Design

**Date:** 2026-07-15

**Goal:** Produce a new gitignored cutover workbook that keeps the current live export authoritative and appends only legacy equipment that is demonstrably absent from it.

## Sources and output

Primary source:

- `data/import/TE_Lab_Equipment_Export.xlsx`
- Its `Inventory` sheet and all 573 existing rows remain unchanged.

Legacy source:

- `S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\Calibration Data\2026 New Eng Lab Calibration List\2026 Newest Eng Calibration List..xlsx`
- This July 10 workbook supersedes the July 8 near-duplicate for the merge. Five of six worksheet cell-content hashes are identical; the July 10 workbook has one additional meaningful row in the remaining worksheet.

Output:

- `data/import/TE_Test_Equipment_Merged_Cutover.xlsx`
- The output is local and gitignored. Neither source workbook is overwritten or modified.
- Expected `Inventory` row count: 594, consisting of 573 primary rows plus 21 consolidated legacy additions.

No workbook contents, equipment identifiers, or row-level review data may be committed to Git.

## Identity policy

Normalize asset and serial identities exactly as the importer does: trim surrounding whitespace and compare case-insensitively. Do not match on manufacturer plus model.

The July 10 legacy workbook contains 268 physical rows with at least one asset or serial identity. Compared with the current primary export:

- 228 physical legacy rows match current equipment and are excluded;
- three physical legacy rows conflict with current identities and are excluded;
- 37 physical rows do not match the current export;
- four additional nonblank rows have neither asset nor serial identity and are excluded.

The 37 unmatched physical rows form 23 connected identity components when rows sharing a nonblank normalized asset or serial key are grouped. Two components contain inconsistent asset/serial relationships and are excluded. The remaining 21 components are the only automatic additions:

- 18 have both asset and serial identity;
- one has asset identity only; and
- two have serial identity only.

Repeated rows inside one accepted component consolidate into one output row. The merge must re-check that the 21 output identities are mutually non-conflicting and absent from the primary export before writing.

## Field mapping and precedence

The output uses the existing exact 22-column `Inventory` contract. Existing primary rows are copied without transformation.

For each accepted legacy component:

| Legacy concept | Output column | Rule |
|----------------|---------------|------|
| Asset Number / Asset Numbers | `Asset Number` | Preserve the accepted component's nonblank value. |
| Serial Number | `Serial Number` | Preserve the accepted component's nonblank value. |
| Manufacturer | `Manufacturer` | Use the newest nonblank consistent value. |
| Model | `Model` | Use the newest nonblank consistent value. |
| Description | `Description` | Use the newest nonblank value. |
| Location | `Location` | Use the newest nonblank value. |
| Assigned to | `Assigned To` | Use the newest nonblank value. |
| Condition / Condition & Notes | `Condition` | Use the newest nonblank condition value. |
| Comments or remaining condition notes | `Notes` | Preserve as notes without overwriting another nonblank note. |
| Last Calibration Date / New Cal Date | `Last Cal Date` | Use the newest valid explicit date. |
| Calibration Due Date / Cal Due Date | `Cal Due Date` | Use the newest valid explicit due date. |
| Last Calibrated by | `Cal Vendor` | Preserve the newest nonblank value as the legacy calibration provider/attribution text. |
| Recognized explicit status | `Cal Status` | Preserve only importer-recognized values. Otherwise use `unknown`. |

Defaults for new rows are `Lifecycle=active`, `Working=unknown`, blank verification, and blank deferred ownership/cost/rental/blue-dot/estimated-age columns. Do not infer `calibrated`, `reference_only`, or `not_required` solely from a worksheet name or the presence of dates.

When a component appears more than once, prefer the occurrence with the newest valid last-calibration or due date, then fill its blank text fields from other consistent occurrences. Conflicting non-identity text does not block the equipment addition; the selected precedence and source occurrence are recorded in the local merge summary.

Invalid or ambiguous legacy dates are not written into mapped date columns. The item remains addable with that date blank, and the source value is retained only in the local review sheet so it is not silently lost.

## Workbook structure

The output starts as a copy of the current primary workbook so its formatting and supporting evidence remain available. It contains:

- `Inventory`: the original 573 rows followed by the 21 accepted legacy rows under the exact live 22-column header;
- the existing `Import Issues` and `Export Summary` sheets unchanged;
- `Legacy Merge Summary`: aggregate source hashes, source sheet counts, included/excluded counts, and verification results; and
- `Legacy Merge Review`: excluded identity groups and any accepted candidate fields that could not be mapped safely.

Supporting-sheet rows are excluded by the importer's `Inventory` sheet selection and do not count as ignored inventory rows.

## Safety and verification

The merge operation must:

1. Hash both source files before and after generation and prove they did not change.
2. Refuse to overwrite an existing output file unless its prior hash is explicitly recorded and replacement is requested.
3. Reconcile the 21 appended rows to 21 accepted identity components.
4. Confirm that every appended row has at least one asset or serial identity.
5. Confirm no appended identity matches or conflicts with a primary identity or another appended identity.
6. Dry-run the generated workbook through the real importer against a temporary empty FeOxDB.
7. Assert the selected sheet, all 22 column treatments, and the five-way row equation.

If the 21 legacy rows introduce no new blocking issue, the expected dry-run snapshot is:

```text
594 total / 536 inserted / 0 matched / 50 conflicted / 8 rejected / 0 ignored / blocking=true
```

The existing 50 primary identity conflicts and eight primary invalid-date rows remain blocking. This merge does not correct them, commit a batch, write the app database, enable sync, or claim cutover completion.

## Scope boundary

This is a one-time cutover-data preparation operation. It does not change the entry schema, UUID matching, production importer semantics, UI, sync behavior, Tauri identity, or either source workbook. No frontend work is required.
