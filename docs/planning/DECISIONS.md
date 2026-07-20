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
| IM-006 | Default shared root for the implemented TE surface: `...\Inventory_Management_App\modules\TE_Test_Equipment`. | Override with `INVENTORY_MANAGEMENT_SHARED_ROOT` only for temporary diagnostics. Legacy TE share remains under InventoryApps until a deliberate migration into the product module path (see IM-013). |
| IM-007 | Shared sync env prefix `INVENTORY_MANAGEMENT_*` (not TE_TEST_EQUIPMENT_*). | Distinct from standalone TE env vars. |
| IM-008 | Updater deferred until product-specific signing key + GitHub Releases exist. | Do not ship TE’s pubkey/endpoints on this product. |
| IM-009 | Cutover is manual install of the new app; keep old installers available. Auto-install via old-app updater is optional later work, not required for scaffold. | |
| IM-011 | **Adaptive per-inventory sync lifecycle** (TE-first): selected inventory uses completion-aware polling (~2 s focused+active; ~60 s idle/unfocused/hidden), immediate sync on activate/focus/visibility/mutation/watcher, hard deactivation when deselected, opaque session tokens so stale work cannot rearm an inactive inventory. | **Implemented for TE 2026-07-20.** Behavioral authority: [../superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md](../superpowers/plans/2026-07-18-adaptive-per-inventory-sync-lifecycle.md). Origin TE D-029. Future modules use independent DBs/roots/watchers and a `systemId → session` map. Shared formats unchanged; `syncIntervalMs` removed. Sync is not a backup. Residual optional: live DevTools call-rate smoke (see SESSION_HANDOFF). |
| IM-012 | Prefer a **whole redesign monorepo** (shared platform + domain modules) under this product, new GitHub repo, archive standalone repos after cutover—not perpetual triple-app maintenance. | TE redesign quality is the UX/ops baseline; do not force one row schema for all modules. GitHub `origin` connected 2026-07-18; monorepo extract still open. |
| IM-013 | **TE shared-data strategy (long-term):** build and pilot against the **new** product module path `...\Inventory_Management_App\modules\TE_Test_Equipment`, not by defaulting the unified app at the legacy InventoryApps TE share. Prefer modularity over temporary dual-path pilot. | Standalone TE apps keep using legacy `InventoryApps\...` until a planned migration/import into the product module tree. Do **not** dual-write unified + standalone to the same root. Env override to legacy is emergency/diagnostic only, not the product default. Future modules each get their own `modules\<Name>\` tree the same way. |

## Supersedes (context only)

TE-only decisions in the TE Test Equipment Inventory repo (D-001 separate app, etc.) remain true **for that standalone product** until archived. This register governs the **unified** product only.
