# Inventory Management

Windows desktop app that unifies multiple inventory systems in **one install**, with an in-app switcher:

- **TE Test Equipment** (implemented)
- **TE Lab Components** (implemented)
- **ME Storage** (placeholder)
- **TE Storage** (implemented)

Stack: Tauri 2, React 19, TypeScript, Vite, Tailwind v4, Bun, Rust, FeOxDB.

TE Storage starts empty and supports adding, editing, deleting, searching and sorting items. Columns are PN #, PR #, PO #, Manufacturer, Model, Description, Qty, Location and Notes; PR # and Description are hidden by default. It uses its own `te-storage-room.feox` database and `modules/TE_Storage_Room` shared folder. Override that folder with `INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT`. Right-click any column header to show or hide columns. Team sync runs automatically; local edits queue while the share is unavailable. As with the other inventories, synchronization is not a backup.

TE Lab Components is a generalized electronic-parts catalog: flexible specifications, separate stock placements, area/desk → container → Excel-style bin locations, multiple locations per part, shared-bin warnings, physical counts/moves, reviewed legacy migration, Lab-only sync schema v2, and a six-sheet audit workbook. Live Lab migration remains an owner-coordinated cutover; see `docs/runbooks/te-lab-components-catalog-v2-migration.md`.

**Package version: `0.1.3`**.

## Product share (S:)

| Item | Path |
|------|------|
| Product root | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Current installer (when shipped) | place NSIS `.exe` at product root |
| Release archive | `release-support\vX.Y.Z\` |
| TE Test Equipment module sync | `modules\TE_Test_Equipment\shared\inventory\` |
| TE Lab Components module | `modules\TE_Lab_Components\shared\inventory\` |
| ME Storage module | `modules\ME_Storage\shared\inventory\` |
| TE Storage module | `modules\TE_Storage_Room\shared\inventory\` |
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
| TE Test Equipment DB | `%LOCALAPPDATA%\com.inventory.management\inventory.feox` |
| TE Lab Components DB | `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox` |
| TE shared-root env | `INVENTORY_MANAGEMENT_SHARED_ROOT` |
| Lab shared-root env | `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT` |
| Env sync enable | `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` (`0`/`false`/`no`/`off` off) |
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
bun run desktop
```

Normal `bun run desktop` uses the product's real Local AppData databases. Copied-data testing must set the absolute `INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT` override and disable shared sync; changing `LOCALAPPDATA` alone is not sufficient on Windows. See `docs/runbooks/im-014-im-015-combined-desktop-smoke.md`.

### Shared data (product modules)

`bun run desktop` uses product module shares by default:

- TE Test Equipment: `S:\...\Inventory_Management_App\modules\TE_Test_Equipment`
- TE Lab Components: `S:\...\Inventory_Management_App\modules\TE_Lab_Components`

**One writer per root** — do not run standalone TE/Lab apps against the same product module share.

## Cutover (ops sketch)

1. Keep standalone installers available under `InventoryApps\...`.
2. Ship Inventory Management installer to product share root.
3. Users install new app; stop using standalone writers for each cutover module.
4. Point each module’s shared root at `modules\<Name>\` only after data/ops migration or deliberate dual-root plan is complete.
