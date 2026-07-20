# Inventory Management — Decision Register

**Status:** Authoritative for this product  
**Last updated:** 2026-07-20

## Accepted decisions

| ID | Decision | Consequence |
|----|----------|-------------|
| IM-001 | Ship a **new** product **Inventory Management** rather than renaming TE Test Equipment Inventory in place. | New Tauri id, AppData, installer, updater, and product share. Standalone apps remain installable until cutover. |
| IM-002 | Tauri identifier `com.inventory.management`. | Keep stable after first team install; do not reuse TE/ME ids. |
| IM-003 | Product share root `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App`. | Installer at root when shipping; `release-support\`; per-module **separate** shared trees under `modules\`. |
| IM-010 | Code workspace lives at `C:\Projects\Active\Inventory_Management` (not under `Inventory_Apps`). | `Inventory_Apps` holds standalone sibling apps only. |
| IM-004 | Modules stay separate under the hood (own shared roots / data); UI is a switcher, not a merged inventory. | No mixed-row single table across TE/ME domains. |
| IM-005 | Initial module labels: TE Test Equipment, TE Lab Components, ME Storage, TE Storage Room. | Only TE Test Equipment implemented in v0 scaffold. |
| IM-006 | TE shared root: **pilot** default is live legacy `...\InventoryApps\TE_Test_Equipment_Inventory` so local `bun run desktop` has real data; **release** target remains product `...\Inventory_Management_App\modules\TE_Test_Equipment`. | Override anytime with `INVENTORY_MANAGEMENT_SHARED_ROOT`. Flip constant + copy data before team ship (IM-013). |
| IM-007 | Shared sync env prefix `INVENTORY_MANAGEMENT_*` (not TE_TEST_EQUIPMENT_*). | Distinct from standalone TE env vars. |
| IM-008 | Product uses **its own** Tauri updater keypair and GitHub Releases endpoint (`Inventory_Management` only). First team ship includes updater so later versions install via in-app Update. | Do not ship TE/ME pubkeys/endpoints. Private key lives outside git (`%USERPROFILE%\.tauri\inventory-management.key`). |
| IM-009 | Cutover is manual install of the new app; keep old installers available. Auto-install via old-app updater is optional later work, not required for scaffold. | |
| IM-011 | **Adaptive per-inventory sync lifecycle** (TE-first): selected inventory uses completion-aware polling (~2 s focused+active; ~60 s idle/unfocused/hidden), immediate sync on activate/focus/visibility/mutation/watcher, hard deactivation when deselected, opaque session tokens so stale work cannot rearm an inactive inventory. | **Implemented for TE 2026-07-20.** Behavioral authority: [../superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md](../superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md). Origin TE D-029. Future modules use independent DBs/roots/watchers and a `systemId → session` map. Shared formats unchanged; `syncIntervalMs` removed. Sync is not a backup. Residual optional: live DevTools call-rate smoke (see SESSION_HANDOFF). |
| IM-012 | Prefer a **whole redesign monorepo** (shared platform + domain modules) under this product, new GitHub repo, archive standalone repos after cutover—not perpetual triple-app maintenance. | **Logical extract implemented 2026-07-20** (shell/platform/modules; one package + one crate). Plan: [../superpowers/plans/2026-07-20-platform-module-architecture-extract.md](../superpowers/plans/2026-07-20-platform-module-architecture-extract.md). TE only implemented; no multi-crate; next is Phase C ports. |
| IM-013 | **TE shared-data strategy:** pre-release pilot uses the live InventoryApps TE share as app default for convenience; **team release** must use product `modules\TE_Test_Equipment` after copying latest shared data. | Do not leave the installer pointed at InventoryApps forever. Never dual-write unified + standalone to the same root. Future modules each get `modules\<Name>\`. |

## Supersedes (context only)

TE-only decisions in the TE Test Equipment Inventory repo (D-001 separate app, etc.) remain true **for that standalone product** until archived. This register governs the **unified** product only.
