# Inventory Management — Decision Register

**Status:** Authoritative for this product
**Last updated:** 2026-07-25

## Accepted decisions

| ID | Decision | Consequence |
|----|----------|-------------|
| IM-001 | Ship a **new** product **Inventory Management** rather than renaming TE Test Equipment Inventory in place. | New Tauri id, AppData, installer, updater, and product share. Standalone apps remain installable until cutover. |
| IM-002 | Tauri identifier `com.inventory.management`. | Keep stable after first team install; do not reuse TE/ME ids. |
| IM-003 | Product share root `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App`. | Installer at root when shipping; `release-support\`; per-module **separate** shared trees under `modules\`. |
| IM-010 | Code workspace lives at `C:\Projects\Active\Inventory_Management` (not under `Inventory_Apps`). | `Inventory_Apps` holds standalone sibling apps only. |
| IM-004 | Modules stay separate under the hood (own shared roots / data); UI is a switcher, not a merged inventory. | No mixed-row single table across TE/ME domains. |
| IM-005 | Initial module labels: TE Test Equipment, TE Lab Components, ME Storage, TE Storage Room. | TE Test Equipment and TE Lab Components are implemented; ME Storage and TE Storage Room remain placeholders. |
| IM-006 | TE shared root default is product `...\Inventory_Management_App\modules\TE_Test_Equipment`. Lab default is product `...\modules\TE_Lab_Components`. | Override with `INVENTORY_MANAGEMENT_SHARED_ROOT` / `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT`. InventoryApps paths are legacy pilots only. |
| IM-007 | Shared sync env prefix `INVENTORY_MANAGEMENT_*` (not `TE_TEST_EQUIPMENT_*`). | Distinct from standalone TE environment variables. |
| IM-008 | Product uses **its own** Tauri updater keypair and GitHub Releases endpoint (`Inventory_Management` only). First team ship includes updater so later versions install via in-app Update. | Do not ship TE/ME pubkeys/endpoints. Private key lives outside git (`%USERPROFILE%\.tauri\inventory-management.key`). |
| IM-009 | Cutover is manual install of the new app; keep old installers available. Auto-install via old-app updater is optional later work, not required for scaffold. | Standalone apps remain available until owner-controlled retirement. |
| IM-011 | **Adaptive per-inventory sync lifecycle** (TE-first): selected inventory uses completion-aware polling (~2 s focused+active; ~60 s idle/unfocused/hidden), immediate sync on activate/focus/visibility/mutation/watcher, hard deactivation when deselected, and opaque session tokens so stale work cannot rearm an inactive inventory. | **Implemented for TE 2026-07-20 and reused by Lab.** Behavioral authority: [2026-07-18-adaptive-per-inventory-sync-lifecycle.md](../superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md). |
| IM-012 | Prefer a **whole redesign monorepo** (shared platform + domain modules) under this product, new GitHub repo, archive standalone repos after cutover—not perpetual triple-app maintenance. | **Logical extract implemented 2026-07-20** (shell/platform/modules; one package + one crate). TE and Lab are real isolated modules; ME Storage and TE Storage Room remain placeholders. Plan: [2026-07-20-platform-module-architecture-extract.md](../superpowers/plans/2026-07-20-platform-module-architecture-extract.md). |
| IM-013 | **Shared-data strategy:** team defaults use product `modules\TE_*` after copy from InventoryApps (done 2026-07-20 for TE + Lab). | Never dual-write unified + standalone to the same root. Future modules each get `modules\<Name>\`. |
| IM-014 | **TE Lab Components is a generalized electronic-parts catalog:** flexible category attributes; Part separate from StockPlacement; area/desk → container → Excel-style bin; per-placement quantities; shared bins allowed with warnings; totals/status derived by unit; lossless reviewed migration; Lab-only sync schema v2; six-sheet export. | Implementation completed 2026-07-25. Owner copied-data desktop rehearsal and coordinated live Lab root cutover remain mandatory. Authority: [2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md](../superpowers/plans/2026-07-24-te-lab-components-generalized-catalog-and-grid-storage.md) and [migration runbook](../runbooks/te-lab-components-catalog-v2-migration.md). |
| IM-015 | Add a dedicated **Calibration** workspace inside TE Test Equipment as a second projection of the existing equipment records, plus a calibration-specific one-time workbook roster preview/review/commit flow. The workbook seeds initial membership; Inventory Management is the source of truth afterward. | No second equipment table, database, shared root, or sync stream. Equipment keeps due/health only; detailed calibration tracking, explicit membership, and roster review live in Calibration. Implementation and pre-cutover verification completed 2026-07-25; backed-up single-writer live cutover remains pending. Authority: [2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md](../superpowers/plans/2026-07-24-te-test-equipment-calibration-workspace-and-roster-cutover.md). |
| IM-016 | Copied-data desktop rehearsals use the absolute `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` override. A process-local `LOCALAPPDATA` change alone is not an accepted isolation boundary on Windows. | TE, Lab, and deprecated-database cleanup resolve through one explicit test root; the normal production Local AppData path remains unchanged when the override is absent. |

## Supersedes (context only)

TE-only decisions in the TE Test Equipment Inventory repo remain true **for that standalone product** until archived. This register governs the **unified** product only.
