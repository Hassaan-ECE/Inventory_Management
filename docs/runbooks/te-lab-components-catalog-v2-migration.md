# TE Lab Components Catalog v2 Migration Runbook

Use this runbook only for an owner-approved cutover. Shared sync is not a backup. Stop every standalone and unified Lab Components writer before copying or changing any Lab data.

## Safety rules

- Never rehearse against the live local database or live shared root.
- Keep the pre-migration workbook, local database copy, and shared-root copy together under one dated backup folder.
- Run the dry-run on copied data, review every blocking duplicate and warning, then commit using the exact dry-run fingerprint.
- Keep all rollback artifacts until the owner validates migrated counts, workbook exports, restart persistence, and two-client sync.

## PowerShell backup template

Replace the example paths before running. The destination must not already be used by the app.

```powershell
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$backupRoot = "C:\InventoryBackups\TE-Lab-Components-$stamp"
$localSource = "$env:LOCALAPPDATA\com.inventory.management\te-lab-components.feox"
$sharedSource = "S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules\TE_Lab_Components"

New-Item -ItemType Directory -Path $backupRoot -Force | Out-Null
Copy-Item -LiteralPath $localSource -Destination "$backupRoot\te-lab-components.feox" -Force
Copy-Item -LiteralPath $sharedSource -Destination "$backupRoot\shared-root" -Recurse -Force
Get-FileHash "$backupRoot\te-lab-components.feox" -Algorithm SHA256 |
  Format-List | Out-File "$backupRoot\local-db-sha256.txt"
```

Export the existing 19-column Lab workbook from the pre-cutover app into the same backup folder before migration.

## Copied-data rehearsal

```powershell
$rehearsalLocalAppData = "$backupRoot\local-appdata-rehearsal"
$rehearsalDbDir = "$rehearsalLocalAppData\com.inventory.management"
$rehearsalSharedRoot = "$backupRoot\shared-root-rehearsal"

New-Item -ItemType Directory -Path $rehearsalDbDir -Force | Out-Null
Copy-Item -LiteralPath "$backupRoot\te-lab-components.feox" -Destination "$rehearsalDbDir\te-lab-components.feox" -Force
Copy-Item -LiteralPath "$backupRoot\shared-root" -Destination $rehearsalSharedRoot -Recurse -Force

$env:LOCALAPPDATA = $rehearsalLocalAppData
$env:INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT = $rehearsalDbDir
$env:INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT = "$backupRoot\shared-root-rehearsal"
$env:INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED = '0'
```

`INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` is the required database isolation boundary. Do not rely on changing `LOCALAPPDATA` alone because Windows app-path resolution may still select the real product directory.

Start the IM-014 build from the same PowerShell process, run the migration preview, save the report, and commit only when `blocking` is false and the fingerprint still matches. Export the six-sheet catalog workbook and compare source rows, migrated parts, placements, and legacy-field rows before any shared cutover rehearsal.

## Restore template

Stop every Lab Components writer before restoring.

```powershell
Copy-Item -LiteralPath "$backupRoot\te-lab-components.feox" -Destination $localSource -Force
Remove-Item -LiteralPath $sharedSource -Recurse -Force
Copy-Item -LiteralPath "$backupRoot\shared-root" -Destination $sharedSource -Recurse -Force
```

Restart one client with shared sync disabled, verify the legacy row count and workbook, then coordinate any shared-root retry from the beginning.
