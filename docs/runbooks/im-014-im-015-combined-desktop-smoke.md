# IM-014 + IM-015 Combined Desktop Smoke

Use this runbook for the owner manual test of the combined integration branch only. It does not authorize either live cutover.

## Prepared workspace

| Item | Value |
|------|-------|
| Worktree | `C:\Projects\Active\Inventory_Management_IM014_IM015` |
| Branch | `feature/im-014-im-015-integration` |
| Reset helper | `.tmp\reset-combined-desktop.ps1` |
| Launch helper | `.tmp\run-combined-desktop.ps1` |
| Calibration workbook | `.tmp\desktop-combined\calibration-tracking.xlsx` |

The ignored `.tmp\desktop-combined\seed` directory contains hash-verified source copies for TE, Lab, and the calibration workbook. The working databases are under `.tmp\desktop-combined\localappdata\com.inventory.management`.

## Safety boundary

- The launcher sets the absolute `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` to the copied database directory.
- Shared sync is disabled with `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED=0`.
- TE and Lab shared roots are redirected to empty `.tmp` directories.
- Changing `LOCALAPPDATA` alone is not sufficient on Windows and must not be used as the database isolation mechanism.
- Run one Inventory Management desktop process at a time.

The startup proof on July 25, 2026 confirmed that the copied Lab database was the file held by the app while both live product databases remained readable and byte-unchanged.

## Start from pristine copies

Close Inventory Management and any existing Vite process first, then run:

```powershell
cd C:\Projects\Active\Inventory_Management_IM014_IM015
powershell -ExecutionPolicy Bypass -File .\.tmp\reset-combined-desktop.ps1
powershell -ExecutionPolicy Bypass -File .\.tmp\run-combined-desktop.ps1
```

Do not run the reset helper between the first close and the restart-persistence check. Run it only when you intentionally want to discard all sandbox changes.

## TE Test Equipment checklist

1. Open **TE Test Equipment → Equipment** and confirm the normal equipment list loads.
2. Confirm Equipment keeps **Calibration due** and **Calibration health** without the detailed calibration columns.
3. Switch to **Calibration** and confirm its search, filters, sorting, columns, and summary are independent from Equipment.
4. Edit one record in Calibration, return to Equipment, and confirm the same record reflects the shared change.
5. Add and remove calibration membership using the explicit actions, then verify the projections update correctly.
6. Open **Initialize from Calibration Workbook**, select `.tmp\desktop-combined\calibration-tracking.xlsx`, and review matched, create, conflict, duplicate, junk, and absent classifications.
7. Resolve any intentionally tested conflict before commit. A sandbox commit is allowed; it must never target the live database.

## TE Lab Components checklist

1. Switch to **TE Lab Components** and confirm calibration controls are absent.
2. Review the catalog-v2 migration preview on the copied legacy Lab database; commit only inside this sandbox when the preview is non-blocking and its fingerprint remains current.
3. Confirm generalized part fields, flexible attributes, detailed filters, and persisted sorting.
4. Create or edit storage areas and containers, then use the Excel-style grid picker and keyboard navigation.
5. Confirm occupied/shared bins show warnings and that an indistinguishable placement merges quantity instead of duplicating stock.
6. Test multiple locations, move/count actions, derived totals/status by unit, and the six-sheet catalog export.

## Switching and persistence

1. Switch TE → Lab → TE and confirm each module retains its independent view state.
2. Close the desktop window and allow the development command to exit.
3. Run only `.tmp\run-combined-desktop.ps1` again without resetting.
4. Confirm sandbox edits and view preferences persist after restart.
5. Close the app and confirm no `inventory-management.exe` process and no listener on port `5173` remain.

## After testing

- Record any failing workflow with the module, exact action, expected result, and observed result.
- Do not copy sandbox databases or generated sync artifacts into the live product paths.
- Do not merge to `main` or perform either live cutover until owner acceptance is recorded.
