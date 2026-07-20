# Temporary pilot: run Inventory Management against the *legacy* TE Test Equipment share.
# Long-term product default remains modules\TE_Test_Equipment (IM-013).
# Before team release: stop this override, copy latest shared data into the product module path,
# and ship the new app as the writer for that module only.
#
# Usage (from repo root):
#   powershell -File scripts\desktop-legacy-te-share.ps1
#
# Hard rules:
# - Only ONE writer on this shared root at a time.
# - Do NOT run standalone TE Test Equipment Inventory against the same share while this runs.
# - Unset / stop using this script when validating the empty product path or preparing cutover copy.

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$legacyTeRoot = "S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE_Test_Equipment_Inventory"

if (-not (Test-Path -LiteralPath $legacyTeRoot)) {
    Write-Error "Legacy TE share not found: $legacyTeRoot"
}

$sharedInventory = Join-Path $legacyTeRoot "shared\inventory"
if (-not (Test-Path -LiteralPath $sharedInventory)) {
    Write-Error "Legacy TE shared inventory folder missing: $sharedInventory"
}

$env:INVENTORY_MANAGEMENT_SHARED_ROOT = $legacyTeRoot
# Ensure shared sync is on for this pilot session.
Remove-Item Env:INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "Inventory Management — TEMPORARY legacy TE shared root" -ForegroundColor Yellow
Write-Host "  INVENTORY_MANAGEMENT_SHARED_ROOT=$legacyTeRoot"
Write-Host "  Shared inventory: $sharedInventory"
Write-Host ""
Write-Host "One writer only: close standalone TE if it uses this share." -ForegroundColor Yellow
Write-Host "Product default (without this script) stays:" -ForegroundColor DarkGray
Write-Host "  ...\Inventory_Management_App\modules\TE_Test_Equipment" -ForegroundColor DarkGray
Write-Host ""

bun run desktop
