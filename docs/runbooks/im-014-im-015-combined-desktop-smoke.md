# IM-014 + IM-015 Combined Desktop Smoke

Use this runbook for the owner manual test of IM-014 and IM-015 together from the normal `main` checkout. It does not authorize either live cutover.

## Prepared workspace

| Item | Value |
|------|-------|
| Checkout | `C:\Projects\Active\Inventory_Management` |
| Branch | `main` |
| Launch command | `bun run desktop` |
| Calibration workbook | `data\import\calibration-tracking.xlsx` |

Normal desktop operation uses the product databases under `%LOCALAPPDATA%\com.inventory.management` and the configured product module shared roots.

## Safety boundary

- Normal `bun run desktop` uses the real product databases and normal shared-sync configuration.
- Do not run either standalone TE/Lab writer against the same product shared roots while Inventory Management is open.
- For a copied-data rehearsal, use an absolute `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` and disable or redirect shared sync; changing `LOCALAPPDATA` alone is insufficient on Windows.
- Run one Inventory Management desktop process at a time.

The startup proof on July 25, 2026 confirmed that the copied Lab database was the file held by the app while both live product databases remained readable and byte-unchanged.

## Start normally

Close any existing Inventory Management or Vite process first, then run:

```powershell
cd C:\Projects\Active\Inventory_Management
bun run desktop
```

This is the normal application runtime. Any committed edits affect the product database and may publish through the configured shared-sync roots.

## TE Test Equipment checklist

1. Open **TE Test Equipment → Equipment** and confirm the normal equipment list loads.
2. Confirm Equipment keeps **Calibration due** and **Calibration health** without the detailed calibration columns.
3. Switch to **Calibration** and confirm its search, filters, sorting, columns, and summary are independent from Equipment.
4. Edit one record in Calibration, return to Equipment, and confirm the same record reflects the shared change.
5. Add and remove calibration membership using the explicit actions, then verify the projections update correctly.
6. Open **Initialize from Calibration Workbook**, select `data\import\calibration-tracking.xlsx`, and review matched, create, conflict, duplicate, junk, and absent classifications.
7. Do not commit the live roster cutover until the owner backup, review, and single-writer gates in the IM-015 plan are complete.

## TE Lab Components checklist

1. Switch to **TE Lab Components** and confirm calibration controls are absent.
2. Review the catalog-v2 migration preview; do not commit the live migration until the backup and cutover gates in the Lab migration runbook are complete.
3. Confirm generalized part fields, flexible attributes, detailed filters, and persisted sorting.
4. Create or edit storage areas and containers, then use the Excel-style grid picker and keyboard navigation.
5. Confirm occupied/shared bins show warnings and that an indistinguishable placement merges quantity instead of duplicating stock.
6. Test multiple locations, move/count actions, derived totals/status by unit, and the six-sheet catalog export.

## Switching and persistence

1. Switch TE → Lab → TE and confirm each module retains its independent view state.
2. Close the desktop window and allow the development command to exit.
3. Run `bun run desktop` again.
4. Confirm approved edits and view preferences persist after restart.
5. Close the app and confirm no `inventory-management.exe` process and no listener on port `5173` remain.

## After testing

- Record any failing workflow with the module, exact action, expected result, and observed result.
- Do not perform either live cutover until owner acceptance, backups, and one-writer coordination are recorded.
