# Copy TE + Lab shared inventory trees from InventoryApps pilots into product module folders.
# Does NOT flip app code defaults — do that after QA (shared_root.rs).
#
# Usage (repo root):
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release\copy-shared-to-product-modules.ps1 -WhatIf
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts\release\copy-shared-to-product-modules.ps1
#
# Rules:
# - Close standalone TE / Lab writers and prefer closing unified desktop during copy.
# - First pass uses robocopy /E (copy subdirs, including empty). No /MIR (no delete on dest).
# - Re-run is safe for refresh; still not a substitute for one-writer discipline after cutover.

[CmdletBinding(SupportsShouldProcess = $true)]
param(
  [switch]$WhatIf
)

$ErrorActionPreference = "Stop"

function Copy-SharedInventory {
  param(
    [string]$Name,
    [string]$SourceInventory,
    [string]$DestInventory
  )

  if (-not (Test-Path -LiteralPath $SourceInventory)) {
    throw "Source missing for ${Name}: $SourceInventory"
  }

  $destParent = Split-Path -Parent $DestInventory
  if (-not (Test-Path -LiteralPath $destParent)) {
    if ($WhatIf -or $PSCmdlet.ShouldProcess($destParent, "Create directory")) {
      if (-not $WhatIf) {
        New-Item -ItemType Directory -Path $destParent -Force | Out-Null
      } else {
        Write-Host "[WhatIf] New-Item $destParent"
      }
    }
  }

  Write-Host ""
  Write-Host "=== $Name ===" -ForegroundColor Cyan
  Write-Host "From: $SourceInventory"
  Write-Host "To:   $DestInventory"

  # /E copy subdirs including empty; /NFL /NDL quieter; /R:2 /W:2 retries
  $args = @(
    $SourceInventory,
    $DestInventory,
    "/E",
    "/COPY:DAT",
    "/R:2",
    "/W:2",
    "/NFL",
    "/NDL",
    "/NP"
  )
  if ($WhatIf) {
    $args += "/L"  # list only
  }

  if ($WhatIf -or $PSCmdlet.ShouldProcess($DestInventory, "Robocopy shared inventory")) {
    & robocopy @args
    $code = $LASTEXITCODE
    # robocopy: 0-7 often success-ish; >=8 failure
    if ($code -ge 8) {
      throw "robocopy failed for $Name with exit code $code"
    }
    Write-Host "robocopy exit code: $code (0-7 = OK for robocopy)" -ForegroundColor Green
  }
}

$legacyTe = "S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE_Test_Equipment_Inventory\shared\inventory"
$legacyLab = "S:\Engineering\Public\Syed_Hassaan_Shah\InventoryApps\TE\shared\inventory"
$productTe = "S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules\TE_Test_Equipment\shared\inventory"
$productLab = "S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules\TE_Lab_Components\shared\inventory"

Write-Host "Inventory Management - shared data copy to product modules" -ForegroundColor Yellow
if ($WhatIf) {
  Write-Host "MODE: WhatIf / list only (no write)" -ForegroundColor Yellow
} else {
  Write-Host "MODE: LIVE copy (/E, no /MIR)" -ForegroundColor Yellow
  Write-Host "Ensure no dual writers on these shares." -ForegroundColor Yellow
}

Copy-SharedInventory -Name "TE Test Equipment" -SourceInventory $legacyTe -DestInventory $productTe
Copy-SharedInventory -Name "TE Lab Components" -SourceInventory $legacyLab -DestInventory $productLab

Write-Host ""
Write-Host "Done. Next: smoke product paths (env override or after code default flip)." -ForegroundColor Cyan
Write-Host "  TE product:  $productTe"
Write-Host "  Lab product: $productLab"
