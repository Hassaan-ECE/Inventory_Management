# Updater and release (Inventory Management)

**Product:** `com.inventory.management`  
**GitHub:** `https://github.com/Hassaan-ECE/Inventory_Management`  
**Do not** reuse TE Test Equipment or ME updater keys or release endpoints.

## Why v1 must ship with updater enabled

The in-app **Update** button only works for builds that already contain:

1. A **public** signing key in `backend/tauri.conf.json`
2. An **endpoint** that serves `latest.json` for this product
3. Release artifacts **signed** with the matching **private** key

If v1 ships with an empty pubkey / no endpoint, users on v1 cannot auto-update to v2. They would need a full reinstall of an updater-enabled build first.

**Owner decision:** first team ship **includes** updater config so later versions install via Update without redoing setup.

## Keys (generated on build PC)

| Item | Location |
|------|----------|
| Private key | `%USERPROFILE%\.tauri\inventory-management.key` (**never commit**) |
| Public key | `%USERPROFILE%\.tauri\inventory-management.key.pub` (also embedded in `tauri.conf.json`) |

If you lose the private key, **existing installs cannot receive signed updates** until users reinstall a build signed with a new keypair (and you rotate `pubkey` in config).

### Sign a release build (PowerShell)

```powershell
cd C:\Projects\Active\Inventory_Management
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "$env:USERPROFILE\.tauri\inventory-management.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""   # empty if key was generated with no password
bun run build:desktop
```

Outputs under `backend/target/release/bundle/nsis/` typically include:

- `Inventory Management_X.Y.Z_x64-setup.exe` — installer (and updater payload source)
- `Inventory Management_X.Y.Z_x64-setup.exe.sig` — signature

## GitHub Release layout

Each release tag (e.g. `v0.1.0`) should attach at least:

1. The NSIS setup `.exe`
2. The matching `.sig`
3. A **`latest.json`** static manifest (see Tauri updater static JSON format)

Example `latest.json` shape (fill version, url, signature text from `.sig` file contents):

```json
{
  "version": "0.1.0",
  "notes": "First team release: TE Test Equipment + TE Lab Components.",
  "pub_date": "2026-07-20T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "<paste full contents of the .sig file>",
      "url": "https://github.com/Hassaan-ECE/Inventory_Management/releases/download/v0.1.0/Inventory%20Management_0.1.0_x64-setup.exe"
    }
  }
}
```

Config endpoint (already set):

```text
https://github.com/Hassaan-ECE/Inventory_Management/releases/latest/download/latest.json
```

`latest` always points at the newest GitHub Release; that release’s assets must include `latest.json` with the newest version.

## Also place installer on S: (team download)

Copy the setup exe to:

```text
S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\
```

and archive under `release-support\vX.Y.Z\` for history. First install is still from S: or GitHub; later installs use **Update** when a newer GitHub Release + `latest.json` exist.

## Config source of truth

`backend/tauri.conf.json`:

- `bundle.createUpdaterArtifacts`: `true`
- `plugins.updater.pubkey`: public key string
- `plugins.updater.endpoints`: GitHub `latest.json` URL above
- `plugins.updater.windows.installMode`: `passive` (progress UI, little interaction)

Frontend already uses `@tauri-apps/plugin-updater` via `check` / download / install in `tauriInventoryBridge.ts`.

## Checklist for each new version

1. Bump version in `package.json`, `backend/Cargo.toml`, `backend/tauri.conf.json` (keep in sync).
2. Set `TAURI_SIGNING_PRIVATE_KEY` from the private key file.
3. `bun run build:desktop`
4. Create GitHub Release `vX.Y.Z` with setup exe, `.sig`, and updated `latest.json`.
5. Copy setup to product share root + `release-support\vX.Y.Z\`.
6. From a machine on the previous version: open app → Update should see the new version.
