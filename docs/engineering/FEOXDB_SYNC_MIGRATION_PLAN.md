# FeOxDB Shared Sync Notes

Last updated: 2026-10-15 (rewritten for Inventory Management 0.1.3, updated for 0.1.4 snapshot merge; the previous version described the ME Inventory 1.0.4 standalone app)

Compact design note for how the unified product syncs. Constants quoted here are from code; re-check them before relying on them.

## Current Shape

- Each machine owns one local FeOxDB file per module under `%LOCALAPPDATA%\com.inventory.management\`: `inventory.feox` (TE Test Equipment), `te-lab-components.feox` (TE Lab Components), `te-storage-room.feox` (TE Storage).
- The S: drive is sync transport, not a database. Each module has its own shared root and stream; modules never read each other's streams.
- Shared sync is on by default. `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED=0|false|no|off` disables it for all modules in that process.
- Optional `INVENTORY_MANAGEMENT_SYNC_HMAC_KEY` (≥16 bytes) signs and verifies operations, snapshots, and manifests for all modules. Either every client sets the same key or none do.
- Old app-owned SQLite `.db` files are quarantined locally once (`integrations/deprecated_db_cleanup.rs`) and are not import sources.

## Shared Layout

Product share: `S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\`

```text
Inventory_Management_App\
  Inventory Management_X.Y.Z_x64-setup.exe   (+ .sig)
  release-support\vX.Y.Z\
  modules\
    TE_Test_Equipment\shared\inventory\       TE stream (sync schema v2)
    TE_Storage_Room\shared\inventory\         Storage stream (sync schema v1)
    TE_Lab_Components\shared\catalog-v2\      Lab catalog stream (catalog schema v2, active)
    TE_Lab_Components\shared\inventory\       Lab legacy v1 stream (blocked after catalog cutover)
```

Each `shared\inventory\` folder contains:

```text
manifest.json
ops\<client-id>\000000000001.op.json
snapshots\snapshot-*.snapshot.json
locks\snapshot.lock
backups\
```

`shared\catalog-v2\` has the same folders plus `cutover.json`, the marker that enables v2 sync for that root.

Override roots with `INVENTORY_MANAGEMENT_SHARED_ROOT` (TE), `INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT` (Lab), and `INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT` (Storage). Defaults live in `backend/src/platform/shared_root.rs`.

## Two Engines

1. **Inventory-entry engine**: `backend/src/sync/` (TE). Near-identical copies live in `modules/te_lab_components/sync/` (Lab v1) and `modules/te_storage/sync/` (Storage). Fixes must be applied to each copy.
2. **Lab catalog engine**: `modules/te_lab_components/catalog_sync.rs`. Separate code for parts, storage areas, containers, and stock placements.

## Sync Behavior (inventory-entry engine)

- Local edits write the entry and a durable outbox record as one unit under the module's sync gate; a background task then publishes pending operations to S:.
- A sync run pushes local ops, takes in the latest verified snapshot when there are no local-only pending changes, then applies operation files newer than the watermarks.
- Taking in a snapshot (0.1.4+): an empty store adopts it wholesale; a store with data **merges** it entry by entry (newest entry state wins; a local-only entry is dropped only if the snapshot covers the op that created it; watermarks become the per-client maximum). Before 0.1.4 a snapshot that did not cover every local watermark was skipped entirely, which stranded the publisher's compacted work (2026-10-15 incident).
- Snapshot failures never replace local data.
- A new snapshot is published after 1,000 operations, when the latest is older than 24 hours, or immediately when this client holds applied operations whose files are no longer on the share (only a snapshot from this client can carry them).
- A client never publishes unless its watermarks cover the current manifest's, and never publishes over a manifest written by a newer app version (it shows "Click Update" instead; ops still sync). Covered op files are compacted only after the snapshot and manifest are written and verified. The latest 3 snapshots are kept. A snapshot lock older than 10 minutes is treated as stale.
- Startup recovery rules are in `SYNC_RECOVERY_INVARIANTS.md`.

## Conflict Behavior (inventory-entry engine)

Concurrent edits to different fields merge when both started from the same base version. For example, if machine A changes `location` and machine B changes `notes`, both fields are kept and no duplicate row is created. Overlapping edits use newer-operation-wins and record a stale conflict.

Regression coverage: `backend/tests/sync_divergence_flow.rs` (TE) and `snapshot_missing_a_newer_op_is_merged_instead_of_stranding_its_publisher` in `modules/te_storage/tests.rs`. Owner recovery tool for a bad shared state: `backend/src/sync/snapshot/restore.rs` (ignored test; dry run by default).

## Lab Catalog Engine Differences

- Sync is refused until the shared root has been cut over (`cutover.json`); until then the app shows "Shared catalog cutover is required before v2 synchronization."
- A local database that still needs v1 → v2 migration does not sync at all; the UI shows the migration panel.
- Conflicts are resolved per **whole entity**, newest wins, ordered by mutation timestamp, then client id, local sequence, and op id. There is no per-field merge.
- A snapshot is republished after any run that pushed, pulled, or applied changes, but only when no local ops are pending.
- Orphan or conflicting remote ops are quarantined and reported in the status message.

## Validation Worth Running Before A Release Or When Investigating Sync Bugs

- Confirm both machines keep their own local `.feox` files.
- Confirm a clean profile hydrates from `manifest.json`, the snapshot, and newer ops.
- Confirm create/update/delete (and archive/verify for TE) converge both ways.
- Confirm different-field concurrent edits merge (TE, Storage) and same-field edits record a conflict and keep the newer value.
- Confirm the ops folder compacts after snapshot publication.
- Use `scripts/smoke-sync-one-machine.ps1` against an isolated root; never against the live product share.
