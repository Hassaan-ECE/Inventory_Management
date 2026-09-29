# TE Storage Room

User-authorized scope: a working, initially empty inventory with local saving and team sync.

Columns, in order: PN #, PR #, PO #, Manufacturer, Model, Description, Qty, Location, Notes. PR # and Description are hidden by default and available in Columns. All fields remain editable. Quantity is optional, finite, nonnegative, at most 1,000,000; at least one identifying field is required. Repeated part numbers are allowed.

Use the existing shell/theme and compact table styling, search, sortable headers, column preferences, add/edit and confirmed deletion. Desktop data loading errors must not substitute demo data or allow a save before initialization. Browser preview starts empty and is read-only. No import or export is required for this slice.

Isolate Storage Room records in its own domain module and `te-storage-room.feox`, with module ID `te-storage`. Shared folder: product `modules/TE_Storage_Room`; override `INVENTORY_MANAGEMENT_TE_STORAGE_SHARED_ROOT`. Honor the existing sync-disable and copied-local-root settings. Adapt the established operation-log, outbox, recovery, snapshot and conflict handling to the new domain; leave existing TE/Lab streams unchanged. Use existing module-keyed sync gates, watcher sessions and adaptive polling. Field-aware edits must preserve unrelated concurrent changes.

Verify empty startup, exact default/optional columns, editing failures, local reopen persistence, two-client sync, offline recovery, stale edits and deletion. Tests use temporary local/shared roots. Do not stage a release or modify existing team data.
