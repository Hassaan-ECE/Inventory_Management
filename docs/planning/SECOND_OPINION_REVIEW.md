# TE Test Equipment Inventory — Second-Opinion Review

**Review date:** 2026-07-13  
**Reviewed document:** [PROJECT_DISCUSSION.md](PROJECT_DISCUSSION.md)  
**Review type:** Read-only product, data-model, migration, and architecture review

**Current decision authority:** [DECISIONS.md](DECISIONS.md). This review is advisory and describes issues found in the proposal as it existed at review time; later accepted decisions supersede proposal wording quoted here.

## Executive summary

The project has a sound overall direction. Reusing the Tauri/React/FeOxDB inventory stack, keeping TE equipment separate from TE components, using a local working database with shared synchronization artifacts, and treating overdue/due-soon states as derived values are all reasonable decisions.

I would approve an initial scaffold and data-discovery phase. I would **not** freeze the domain model or build the complete UI until the following areas are resolved:

1. Calibration history and status semantics
2. Safe record identity and repeatable Excel migration
3. The Windows location of the machine-local database
4. Selection and validation of the actual source codebase
5. Shared-drive ownership, operational validation, backup, and restore

The most important conclusion is that this should not be modeled merely as the existing inventory entry plus five mutable calibration fields if calibration is the product's primary purpose.

## What is already strong

- The scope is clear: replace the Python equipment app without merging equipment and component inventories.
- The proposal distinguishes current production data from the stale local reference database.
- The local-database-plus-operation-log approach is much safer than opening one shared database file from multiple machines.
- Overdue and due-soon are correctly proposed as derived states rather than stored statuses.
- Archive is a useful addition for retaining scrapped, junk, and historical records without routine hard deletion.
- The proposal intentionally avoids a major UI redesign, reducing implementation and training risk.
- The Tauri identifier `com.te.test.equipment.inventory` is unique, descriptive, and worth keeping stable.

## Priority findings

### 1. Calibration needs clearer semantics and probably an event history

The proposed status values mix three different concepts:

- `reference_only`, `not_required`, and `unknown` describe whether calibration is required.
- `out_to_cal` describes a workflow state.
- `calibrated` describes a past action, but does not prove that an item is currently in calibration.

An item can be marked `calibrated` and still be overdue. More importantly, the sample data contains approximately 212 calibrated records but only approximately 140 due dates across the entire database. Therefore, at least roughly 72 calibrated records cannot be safely displayed as current based on a due date.

#### Recommended v1 representation

On the equipment record:

- `calibrationRequirement`: `required | reference_only | not_required | unknown`
- `outToCalibration`: workflow state, separate from requirement
- Optional scheduling policy such as `calibrationIntervalMonths`, if the lab uses a predictable interval

As derived display health:

- `missing_due`
- `overdue`
- `due_soon`
- `current`
- `not_applicable`
- `unknown`
- `out_to_cal`

If the app is expected to provide audit or compliance history, add a separate `CalibrationEvent` record with fields such as:

- event UUID and equipment UUID
- performed date and next due date
- provider/vendor
- certificate number, link, or managed attachment reference
- cost stored as integer minor units or a decimal value, not a binary floating-point value
- notes
- created/updated timestamps and, if accountability matters, the operator identity

Without calibration events, editing the next calibration will overwrite the previous vendor, cost, dates, and evidence. If the lab only needs a current reminder board and not history, that limitation should be stated explicitly as a v1 non-goal.

#### Recommended derived-status precedence

Evaluate date-only values using the workstation's agreed local date:

1. Archived or excluded lifecycle state → omit from active calibration counts
2. `reference_only` or `not_required` → Not applicable
3. `unknown` → Unknown
4. Out to calibration → Out to cal
5. Required with no due date → Missing due date
6. Due date before today → Overdue
7. Due date from today through today plus the configured window → Due soon
8. Otherwise → Current

This makes “due today” due soon, not overdue, and prevents a required item with no due date from appearing healthy.

### 2. Manufacturer and model must not be an automatic identity match

The migration plan currently proposes matching historical rows using asset number, serial number, or manufacturer plus model. Manufacturer plus model is not a safe identity key because a lab can own many identical instruments.

The existing Python importer is more conservative: it preserves duplicate-aware indexes, resolves only unique asset/serial matches, and reports conflicts when the identifiers are ambiguous or point to different records. The new importer should keep that behavior.

Recommended rules:

- The application UUID is the stable internal identity.
- Asset and serial numbers are normalized matching keys, but duplicates should produce warnings rather than destructive uniqueness assumptions.
- Manufacturer plus model may rank possible matches for review but must never trigger an unattended merge.
- An import rerun against the same inputs must be idempotent.
- Every run must report inserted, matched, conflicted, rejected, and ignored rows.
- No row or column should be silently discarded.

Migration provenance should not be fully deferred. At minimum, preserve an import batch ID, source filename, sheet, row number, and original source identifiers. This can be much smaller than the Python app's complete raw-cell system while still making reconciliation and rollback possible.

### 3. Profile a live Excel export before freezing the model

The latest production workbook should be obtained and profiled as early as practical. It does not need to be imported into the application immediately, but its headers, value formats, duplicates, missing identifiers, invalid dates, location variants, and calibration combinations should inform the schema.

Waiting until after the UI is complete creates a risk that the real export does not fit the frozen model. A representative export may contain sensitive inventory data, so it should remain outside version control and be handled according to lab policy.

The recommended importer shape is:

1. Read and map headers.
2. Normalize values without destroying the originals.
3. Produce a dry-run reconciliation report.
4. Require review of ambiguous matches and conflicts.
5. Commit a deterministic import batch.
6. Support a safe rerun or rollback during migration rehearsal.

Historical workbooks should supplement the live export only after the primary snapshot is reconciled.

### 4. Consider Local AppData for the machine-local FeOxDB

The proposal currently uses `%APPDATA%\<identifier>\inventory.feox`. On Windows, Tauri's `app_data_dir()` resolves through the roaming data directory, while `app_local_data_dir()` resolves through Local AppData. See the official [Tauri path documentation](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html) and [Tauri filesystem documentation](https://v2.tauri.app/plugin/file-system/).

Because this database is intended to be machine-local and the application already performs its own explicit shared synchronization, the better default is likely:

```text
%LOCALAPPDATA%\com.te.test.equipment.inventory\inventory.feox
```

This is a recommendation, not a statement that Roaming AppData will fail on every workstation. The final choice should be confirmed against the organization's Windows profile policy. Starting the new app in Local AppData is easier than moving a populated database later.

### 5. Select the base by behavior and commit, not only by product name

The proposal says to clone TE Component Inventory, while the locally available ME Tauri repository contains substantial sync, conflict, recovery, validation, and security hardening. The ME handoff also says that a real shared-drive multi-machine smoke is still pending, despite extensive automated coverage.

Before scaffolding:

1. Compare the current TE Components and ME branches.
2. Identify which contains the newest storage, sync, validation, packaging, and security fixes.
3. Start from that implementation or deliberately port the missing hardening.
4. Record the source repository and exact commit SHA in this project's documentation.

The current TE Component Inventory GitHub branch was not available to this reviewer, so its exact parity with the local ME implementation could not be confirmed. The base-code recommendation should remain provisional until that comparison is performed.

### 6. Shared sync should be integrated early but enabled only after operational proof

The inherited synchronization code should remain part of the model and storage work from the beginning. Adding calibration fields later without updating field-diff, merge, serialization, snapshot, export, and conflict tests would be risky.

However, production shared synchronization should remain configuration-gated until a real two-machine S-drive test verifies:

- create, update, archive, restore, and delete convergence
- simultaneous edits to different fields
- simultaneous edits to the same calibration field
- offline edits followed by reconnection
- stale update and tombstone behavior
- corrupt or incomplete operation files
- snapshot bootstrap on a clean machine
- clock skew visibility and conflict behavior

The shared path should be owned by a stable lab or department group rather than living under a person-specific folder. Windows/share ACLs remain the primary trust boundary. Optional HMAC protects shared artifacts when configured, but it does not replace access control, user attribution, backups, or operational key distribution.

### 7. Backup and restore need an explicit plan

Operation logs and synchronization snapshots are not automatically a complete backup strategy because a valid deletion or bad edit can synchronize successfully.

Before release, define:

- retention for known-good snapshots or backups
- who owns the shared folder and restore process
- how to restore one record versus the complete inventory
- how local-only pending changes are handled during restore
- a cutover backup of the live Excel export and Python data
- a rollback window during which the Python app is retained read-only

A restore drill should be a release gate, not merely documentation.

### 8. A few smaller fields need cross-machine semantics

- Replace or supplement the timeless `verified` boolean with `verifiedAt` and optionally `verifiedBy`; a true flag can become stale without indicating when the location was checked.
- Define whether `scrapped` automatically suggests archiving. Lifecycle describes the equipment, while archive controls normal visibility; they should remain separate concepts.
- Do not depend on ordinary absolute local picture paths in a multi-machine product. Use a managed attachment store, durable UNC path, or URL, or defer pictures for v1.
- Normalize locations through a reviewable alias map or controlled suggestions. Preserve the imported value so normalization remains auditable.
- If ownership is deferred, confirm whether `rental` should remain a lifecycle value; otherwise the model may preserve a rental state while removing its supporting fields.

## Answers to the current open questions

| Question | Second-opinion recommendation |
|---|---|
| Due-soon window | Use 30 days as the default. Implement the exact rule as `today <= dueDate <= today + 30 days`; make it configurable later if needed. |
| Add `not_required`? | Preserve the distinction, but split calibration requirement from workflow state rather than adding another value to the mixed status enum. |
| Shared S-drive path | Use a durable lab/department-owned folder and confirm writer ACLs, backup ownership, and final naming with IT/lab operations. |
| Tauri identifier | Keep `com.te.test.equipment.inventory`; shortening provides little benefit and stability matters more. |
| Import timing | Obtain and profile a representative live export now. Implement the final importer after the initial scaffold and model decision. |
| Archive | Keep it. Do not make archive synonymous with lifecycle status. |
| Single-PC or multi-PC first | Keep sync integrated and configuration-gated from the start; release shared mode only after real two-machine validation. |

## Recommended build sequence

1. **Data discovery** — obtain/profile a representative live export and document anomalies.
2. **Decision record** — lock calibration semantics, history requirement, date rules, attachment approach, local data path, and shared-root ownership.
3. **Base selection** — compare TE Components and ME, choose the better source, and record the exact commit.
4. **Scaffold/rebrand** — establish the new repository, identifier, application name, preference keys, update identity, and local database path.
5. **Domain/storage/sync** — add equipment and calibration structures across validation, persistence, field merge, snapshots, conflicts, and export.
6. **Importer** — build dry-run mapping, provenance, conflict reporting, deterministic commit, and reconciliation totals.
7. **UI** — add calibration editor/history, table fields, derived badges, filters, counts, missing-due handling, and archive behavior.
8. **Operational hardening** — backup/restore, one-machine sync, two-machine sync, offline recovery, packaging, updater, and security validation.
9. **Migration rehearsal** — import copies of the intended cutover files, reconcile totals and exceptions, and obtain owner approval.
10. **Cutover** — freeze Python writes, take final exports/backups, perform the final import, validate, and retain a documented rollback window.

## Suggested acceptance criteria

### Data and migration

- Every source row is accounted for as imported, matched, conflicted, rejected, or intentionally ignored.
- No automatic merge is based only on manufacturer and model.
- Running the same import twice does not create duplicates or silently overwrite reviewed decisions.
- Imported totals and calibration-status counts are reconciled against the source workbook.
- Original source references remain available for migrated records.

### Calibration

- Required equipment without a due date appears as missing setup, never current.
- Due today, due soon, and overdue boundary behavior has automated tests.
- Exempt, unknown, out-to-calibration, archived, and inactive records follow an explicit count/filter policy.
- Date validation rejects or flags impossible combinations such as a due date earlier than the calibration date.
- Calibration history is retained, or the lack of history is explicitly accepted as a v1 limitation.

### Storage, synchronization, and recovery

- Local data survives application upgrades and reinstall according to the documented policy.
- Shared mode passes real two-machine create/update/archive/restore/delete and conflict tests.
- A clean client can bootstrap and converge from verified shared artifacts.
- Shared-root outages do not block local viewing and permitted local work.
- A documented backup can be restored successfully in a rehearsal.

### Release and cutover

- The source base and commit are recorded.
- Package identifier, update keys, endpoints, and shared paths are unique to TE Test Equipment Inventory.
- The final production export is backed up before import.
- The Python application is no longer a parallel write source after cutover.
- The owner signs off on reconciliation exceptions and the rollback plan.

## Final recommendation

Proceed with the project, but treat the next phase as **data and domain validation plus scaffold selection**, not immediate full UI implementation. The stack and product boundary are sensible. Resolving calibration history, safe identity matching, Local AppData, operational sync proof, and backup/restore now will prevent the most expensive rework later.

## Review basis and limitations

This review used:

- `docs/PROJECT_DISCUSSION.md`
- the local Python TE Lab Equipment model, duplicate-aware matcher, export definitions, and related source
- the local ME Tauri v2 domain/storage/sync implementation and engineering handoff documentation
- current Tauri path documentation for AppData versus Local AppData behavior

No project files were changed as part of the original review, no application test suites were run, and the latest production Excel export was not available. The current TE Component Inventory GitHub branch could not be inspected, so base-code parity must still be verified.
