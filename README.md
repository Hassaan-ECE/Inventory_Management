# Inventory Management

Windows desktop app that unifies multiple inventory systems in **one install**, with an in-app switcher:

- **TE Test Equipment** (implemented)
- **TE Lab Components** (implemented)
- **ME Storage** (placeholder)
- **TE Storage Room** (placeholder)

Stack: Tauri 2, React 19, TypeScript, Vite, Tailwind v4, Bun, Rust, FeOxDB.

**Package version: `0.1.0`** (scaffold / pre-team ship).

## Product share (S:)

| Item | Path |
|------|------|
| Product root | `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App` |
| Current installer (when shipped) | place NSIS `.exe` at product root |
| Release archive | `release-support\vX.Y.Z\` |
| TE Test Equipment module sync | `modules\TE_Test_Equipment\shared\inventory\` |
| TE Lab Components module | `modules\TE_Lab_Components\shared\inventory\` |
| ME Storage module | `modules\ME_Storage\shared\inventory\` |
| TE Storage Room module | `modules\TE_Storage_Room\shared\inventory\` |
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

Updater is **not** configured yet (new product needs its own signing key + GitHub repo). Do not reuse TE/ME updater keys.

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

### Shared data (pilot)

`bun run desktop` uses both live pilot shares by default:

- TE Test Equipment: `S:\...\InventoryApps\TE_Test_Equipment_Inventory`
- TE Lab Components: `S:\...\InventoryApps\TE`

**One writer per root** — do not also run the corresponding standalone against either share.
Before release: copy the latest shared data into `Inventory_Management_App\modules\TE_Test_Equipment` and `Inventory_Management_App\modules\TE_Lab_Components`, then deliberately flip both defaults.

## Cutover (ops sketch)

1. Keep standalone installers available under `InventoryApps\...`.
2. Ship Inventory Management installer to product share root.
3. Users install new app; stop using standalone writers for each cutover module.
4. Point each module’s shared root at `modules\<Name>\` only after data/ops migration or deliberate dual-root plan is complete.
