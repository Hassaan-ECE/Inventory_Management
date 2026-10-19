# Inventory Management

Windows desktop app that unifies multiple inventory systems in **one install**, with an in-app switcher:

- **TE Test Equipment** (implemented)
- **TE Lab Components** (implemented)
- **EVCS Storage** (implemented; internal id `te-storage`)

Stack: Tauri 2, React 19, TypeScript, Vite, Tailwind v4, Bun, Rust, FeOxDB.

EVCS Storage (formerly TE Storage) starts empty and supports adding, editing, deleting, searching and sorting items. Columns are PN #, PR #, PO #, Manufacturer, Model, Description, Qty, Location and Notes; PR # and Description are hidden by default. It keeps its records in its own namespace of the local database and syncs through the `modules/TE_Storage_Room` shared folder. Override that folder with `INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT`. Right-click any column header to show or hide columns. Team sync runs automatically; local edits queue while the share is unavailable. As with the other inventories, synchronization is not a backup.

TE Lab Components is a generalized electronic-parts catalog: flexible specifications, separate stock placements, area/desk → container → Excel-style bin locations, multiple locations per part, shared-bin warnings, physical counts/moves, reviewed legacy migration, Lab-only sync schema v2, and a six-sheet audit workbook. Live Lab migration remains an owner-coordinated cutover; see `docs/runbooks/te-lab-components-catalog-v2-migration.md`.

**Package version: `0.2.0`**.

## Product share (S:)

| Item | Path |
|------|------|
| Product root | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Current installer (when shipped) | place NSIS `.exe` at product root |
| Release files per version | `release-support\vX.Y.Z\` (installer, `.sig`, `latest.json`, checksums, notes) |
| Superseded installers | `archive\` |
| Import source workbooks | `source-data\` (calibration roster: `source-data\calibration\`) |
| TE Test Equipment module sync | `modules\TE_Test_Equipment\shared\inventory\` |
| TE Lab Components module | `modules\TE_Lab_Components\shared\catalog-v2\` (active catalog stream; the older `shared\inventory\` v1 stream is blocked after cutover) |
| ME Storage (reserved, not built) | `modules\ME_Storage\` |
| EVCS Storage module | `modules\TE_Storage_Room\shared\inventory\` |
| Legacy standalone pointers | `legacy-pointers\README.md` |

Share README: that folder’s `README.md`.

### Legacy standalone shares (still live for old apps)

Until cutover, standalone apps keep using:

- `...\InventoryApps\TE_Test_Equipment_Inventory`
- `...\InventoryApps\ME`
- `...\InventoryApps\TE`

Do **not** run old + new clients against the same live shared root at once.

## Identity

| Item | Value |
|------|--------|
| Display | Inventory Management |
| Tauri id | `com.inventory.management` |
| Local database (0.2+) | `%LOCALAPPDATA%\com.inventory.management\inventory-management.feox` (all inventories; 0.1.x files are merged in once and kept in `pre-0.2-databases\`) |
| TE shared-root env | `INVENTORY_MANAGEMENT_SHARED_ROOT` |
| Lab shared-root env | `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT` |
| Storage shared-root env | `INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT` |
| Env sync enable | `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` (`0`/`false`/`no`/`off` off; applies to all modules) |
| Sync signing env | `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` (optional, ≥16 bytes; set on every client or none) |
| Copied-data DB root env | `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` (absolute path; diagnostics/rehearsals only) |

Updater is **configured for this product** (public key + GitHub `latest.json` endpoint). Sign builds with the private key on the release machine — see `docs/engineering/UPDATER_AND_RELEASE.md`. Do not reuse TE/ME updater keys.

## Workspace

```text
C:\Projects\Active\Inventory_Management
```

Sibling standalone apps remain under `C:\Projects\Active\Inventory_Apps\` (TE / ME). This unified product is intentionally **not** nested under `Inventory_Apps`.

## Development

```powershell
cd C:\Projects\Active\Inventory_Management
bun install --frozen-lockfile
bun run lint
bun run test
bun run build
cargo test --manifest-path backend/Cargo.toml --no-fail-fast
bun run dev:share
bun run dev:desktop
```

`bun run dev:desktop` is a developer build (see below) and never touches real data. Copied-data testing of real databases must set the absolute `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` override, redirect the TE, Lab, and Storage shared roots, and disable shared sync; changing `LOCALAPPDATA` alone is not sufficient on Windows. See `docs/runbooks/im-014-im-015-combined-desktop-smoke.md`.

### Shared data (product modules)

Release builds use the product module shares:

- TE Test Equipment: `S:\...\Inventory_Management_App\modules\TE_Test_Equipment`
- TE Lab Components: `S:\...\Inventory_Management_App\modules\TE_Lab_Components`
- EVCS Storage: `S:\...\Inventory_Management_App\modules\TE_Storage_Room` (created automatically only if the product `modules\` folder is reachable)

**One writer per root** — do not run standalone TE/Lab apps against the same product module share.

Sync design and folder layout: `docs/engineering/FEOXDB_SYNC_MIGRATION_PLAN.md`. Where code lives: `docs/engineering/PROJECT_FOLDER_BREAKDOWN.md`.

## Developer builds

`bun run dev:desktop` and `cargo test` are developer builds. They keep databases in `.tmp/dev-data` and sync through `.tmp/dev-share` (same `modules\<folder>` layout as S:), never `%LOCALAPPDATA%` or the live share, unless one of the env overrides above is set. The footer shows **DEV build** in place of the author credit.

`bun run dev:share` fills `.tmp/dev-share` with fixture records for every inventory, written by three fake computers through the real sync engine (operations, snapshots, and manifests, including concurrent edits and deletes), and moves `.tmp/dev-data` aside to `.tmp/dev-data-previous` so the next developer build starts from the share. To build it elsewhere, for example an S: dev folder for two-PC testing, set `INVENTORY_MANAGEMENT_DEV_SHARE_TARGET` to an empty folder; the generated `DEV-SHARE.md` lists the env overrides that point a developer build at it. The live share is refused. Fixtures live in `backend/src/dev_share/`, and **every new field or feature adds fixtures in the same change** (a coverage test fails when a synced field is never set).

Removed features and ideas not yet built (ME Storage, HTML export, window-size restore, the old TE spreadsheet import): `docs/engineering/REMOVED_AND_FUTURE.md`.

## Cutover (ops sketch)

1. Keep standalone installers available under `InventoryApps\...`.
2. Ship Inventory Management installer to product share root.
3. Users install new app; stop using standalone writers for each cutover module.
4. Point each module’s shared root at `modules\<Name>\` only after data/ops migration or deliberate dual-root plan is complete.
