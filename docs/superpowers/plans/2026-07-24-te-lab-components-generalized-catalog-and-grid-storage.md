# TE Lab Components Generalized Catalog + Grid Storage — Design and Implementation Plan

> **For agentic workers:** This is the authority for the Lab Components redesign. Follow the checklist in order, preserve existing data, and update this document as tasks are completed. Do not substitute a resistor-specific schema or continue extending the current equipment-shaped row without revisiting IM-014.

**Status:** Implementation complete on `feature/im-014-lab-components`; copied-owner-data desktop rehearsal and live cutover pending

**Date:** 2026-07-24

**Implemented:** 2026-07-25

**Decision:** IM-014

**Module:** `te-lab-components` only

**Related:** [SESSION_HANDOFF.md](../../SESSION_HANDOFF.md), [DECISIONS.md](../../planning/DECISIONS.md), [TE Lab Components port](./2026-07-20-te-lab-components-port.md)

**Goal:** Evolve TE Lab Components from a generic asset-style row into a generalized electronic-parts catalog with flexible specifications, structured storage areas and containers, Excel-style grid-bin selection, and quantity tracked separately at every physical placement.

**Architecture:** Keep TE Lab Components isolated from TE Test Equipment. Separate a component's engineering/catalog identity from its physical stock placements. A part may exist in multiple bins; a bin may contain multiple parts. Total quantity and stock status are derived from active placements rather than stored as independent editable totals.

---

## 0. Locked product decisions

These choices are accepted and must not drift during implementation:

1. The catalog is for **general electronic and PCB-related parts**, including resistors, capacitors, inductors, diodes, transistors, ICs, connectors, electromechanical parts, modules, cables, hardware, and future custom categories.
2. Category-specific specifications use flexible attributes and UI templates. The backend does **not** get a separate row schema for every component category.
3. Catalog identity and physical stock are separate concepts:
   - one part record describes what the component is;
   - one or more stock placements describe where quantities are stored.
4. Storage paths are hierarchical and unambiguous: **area or desk → container → bin coordinate**.
5. Grid containers use Excel-style coordinates: lettered columns on the X axis and numbered rows on the Y axis.
6. Numeric row/column indexes are persisted; labels such as `C7` are derived for display.
7. The same part may be stored in multiple locations, with a separate quantity at each location.
8. Different parts may share a bin. The UI warns and shows existing contents but does not prohibit the placement.
9. Total quantity is computed from placements. Users do not edit a second independent total that can disagree with bin quantities.
10. Existing Lab data, IDs, shared-root isolation, and one-writer sync rules must be preserved through a deliberate migration.

---

## 1. Scope

### In scope

- Generalized part identity and common component fields.
- Flexible key/value specifications assisted by category templates.
- Structured areas, including the main lab and individual desks.
- Configurable grid containers such as cabinets, drawer organizers, and desk-bin organizers.
- Clickable and keyboard-accessible grid-bin picker.
- Multiple stock placements and quantities per part.
- Bin-contents view and occupied-bin warnings.
- Current-state stock adjustment, transfer, and physical-count workflows.
- Search, filters, sorting, table columns, and Excel export for the new model.
- Lossless migration from the existing Lab `InventoryEntry` projection.
- Lab sync schema/version update without touching TE Test Equipment data or sync.

### Explicit non-goals for this slice

- Merging TE Lab Components with TE Test Equipment or any other module.
- A resistor-only, capacitor-only, or other category-specific database schema.
- Automatic parsing of DigiKey, Mouser, or manufacturer webpages.
- Purchasing, purchase orders, vendor pricing history, or accounting.
- Reservations against projects or people.
- A durable stock-movement audit ledger. This slice provides safe current-state adjustments and transfers; transaction history requires a later approved design.
- Barcode/QR printing or scanning.
- Automatic deduplication or unattended merging of existing records.
- Treating shared sync as backup.

---

## 2. User outcomes

The completed design must support these workflows:

1. Search for a component by manufacturer, manufacturer part number, internal part number, value, category, package, mounting type, flexible attribute, supplier SKU, or location.
2. Open one part and see its total quantity plus every physical location and per-location quantity.
3. Add a new cabinet or desk organizer by naming it and specifying its row and column count.
4. Select a bin visually from a grid whose columns are `A`, `B`, `C`… and rows are `1`, `2`, `3`….
5. Distinguish `Main Lab / Cabinet 1 / C7` from `Desk — Alex / Organizer 1 / C7`.
6. Store one part in several locations without duplicating its engineering specifications.
7. Store several part types in one bin after seeing a non-blocking occupied-bin warning.
8. Move some or all quantity from one placement to another without corrupting the total.
9. Perform a physical count for one placement and record who counted it and when.
10. Export parts, attributes, storage layout, and stock placements in a workbook that can be audited outside the app.

---

## 3. Domain model

Names below are the target concepts. Implementors may preserve current serialized names where that reduces migration risk, but the API and UI must expose the concepts consistently.

### 3.1 Part

One `Part` describes a component type, not a particular bag, bin, or desk allocation.

| Field | Type | Rule / purpose |
|---|---|---|
| `id`, `databaseId`, `entryUuid` | existing identity fields | Retain current UUID identity during migration; do not generate replacement identities unnecessarily. |
| `internalPartNumber` | string | Lab-defined identifier; unique when nonblank. Replaces the asset-oriented meaning of `assetNumber` in the new UI. |
| `category` | string | Preset-assisted but extensible; custom categories are allowed. |
| `subcategory` | string | Preset-assisted and extensible. |
| `manufacturer` | string | Manufacturer or brand. |
| `manufacturerPartNumber` | string | Explicit MPN; this is the primary manufacturer identity, not a supplier SKU. |
| `displayValue` | string | Human-readable key value or label such as `1 kΩ`, `100 nF`, `2N3904`, or `LM358`. |
| `mountingType` | string | Suggested values include through-hole, surface-mount, panel-mount, chassis, wire, module, other, and unknown. |
| `packageType` | string | Examples: axial, radial, TO-92, DIP-8, SOIC-8, terminal block, or custom text. |
| `description` | string | Plain-language description. |
| `attributes` | ordered map | Flexible normalized keys whose values contain display value and optional unit. Canonical serialization sorts keys. |
| `supplier` | string | Preferred supplier name for the initial slice. |
| `supplierSku` | string | Preferred supplier's catalog identifier. |
| `supplierPackaging` | string | Cut tape, reel, tube, tray, bag, bulk, or custom. |
| `productUrl` | string | Supplier or product page URL. |
| `datasheetUrl` | string | Separate manufacturer datasheet URL. |
| `defaultUnitOfMeasure` | string | Unit used for the normal stock total and replenishment thresholds; `unknown` is allowed until reviewed. |
| `reorderPoint` | optional number | Low-stock threshold evaluated against active quantity in the default unit. |
| `targetQuantity` | optional number | Desired replenished total in the default unit; must be greater than or equal to the reorder point when both exist. |
| `partStatus` | string | Suggested values: active, NRND, obsolete, discontinued, unknown. Separate from archive. |
| `picturePath` | string | Existing picture reference behavior. |
| `notes` | string | Free-form information not represented structurally. |
| `archived` | boolean | Removes the part from active inventory without deleting it. |
| `createdAt`, `updatedAt` | timestamps | Existing timestamp behavior, updated for new field changes. |

### 3.2 Flexible attributes

Conceptual shape:

```ts
interface ComponentAttributeValue {
  value: string;
  unit?: string;
}

type ComponentAttributes = Record<string, ComponentAttributeValue>;
```

Rules:

- Attribute keys are stable normalized slugs such as `tolerance`, `ratedVoltage`, or `pinCount`.
- Category templates provide labels and suggested units but do not reject unknown attributes.
- Empty values are removed during normalization.
- Attribute keys are unique per part and sorted before canonical checksums, snapshots, or exports.
- The first implementation does not attempt engineering-unit conversion or numeric range queries across mixed units.

### 3.3 Storage area

A `StorageArea` identifies the broad physical context so identical bin labels do not collide.

| Field | Type | Rule / purpose |
|---|---|---|
| `areaUuid` | UUID | Stable identity. |
| `name` | string | Examples: `Main Lab`, `Desk — Hassaan`, `Desk — Alex`. |
| `areaType` | string | Suggested values: lab, room, desk, mobile, other. |
| `owner` | string | Optional person/team for a desk or controlled area. |
| `description` | string | Optional physical description. |
| `archived` | boolean | Hides retired areas while preserving placements for history/migration. |
| `createdAt`, `updatedAt` | timestamps | Audit metadata. |

Area names must be unique among active areas after case/whitespace normalization. Renaming an area changes display paths but not references.

### 3.4 Storage container

A `StorageContainer` belongs to one area and may provide an Excel-style grid.

| Field | Type | Rule / purpose |
|---|---|---|
| `containerUuid` | UUID | Stable identity. |
| `areaUuid` | UUID | Required reference to an active or archived area. |
| `name` | string | Examples: `Component Cabinet 1`, `Organizer 2`, `Drawer Bank A`. |
| `containerType` | string | Cabinet, organizer, drawer bank, shelf, box, other. |
| `gridEnabled` | boolean | When true, placements require row and column indexes. |
| `rowCount` | optional positive integer | Required for grid containers. |
| `columnCount` | optional positive integer | Required for grid containers. |
| `rowStart` | positive integer | Defaults to `1`. |
| `origin` | string | Initial version is fixed to `top_left`; reserve the field for future layouts. |
| `description` | string | Optional identifying details. |
| `archived` | boolean | Prevents new placements while retaining existing references. |
| `createdAt`, `updatedAt` | timestamps | Audit metadata. |

Container names must be unique within an active area after normalization. Shrinking a grid is blocked if active placements would fall outside the new bounds.

### 3.5 Stock placement

A `StockPlacement` connects one part to one physical position and owns the quantity at that position.

| Field | Type | Rule / purpose |
|---|---|---|
| `placementUuid` | UUID | Stable identity. |
| `partUuid` | UUID | Required part reference. |
| `containerUuid` | UUID | Required container reference. |
| `columnIndex` | optional zero-based integer | Required and bounds-checked for grid containers. |
| `rowIndex` | optional zero-based integer | Required and bounds-checked for grid containers. |
| `freeformPosition` | string | Used only for non-grid or migrated unstructured locations. |
| `quantity` | nonnegative number | Quantity at this placement only. |
| `unitOfMeasure` | string | Required; presets include unknown, pcs, m, ft, g, kg, reel, roll, tube, tray, bag, and other. Defaults from the part for new placements. |
| `packaging` | string | Optional local packaging description. |
| `lotCode` | string | Optional lot identity. |
| `dateCode` | string | Optional manufacturer/date code. |
| `condition` | string | Suggested values: new, used, salvaged, damaged, quarantined, unknown. |
| `countState` | string | `uncounted`, `legacy_verified`, or `counted`. |
| `lastCountedAt` | optional timestamp | Required when `countState` is `counted`. |
| `lastCountedBy` | string | Optional counter identity; requested during a physical count. |
| `notes` | string | Placement-specific notes. |
| `archived` | boolean | Excludes the placement from active totals without deleting its record. |
| `createdAt`, `updatedAt` | timestamps | Audit metadata. |

### 3.6 Placement invariants

- Grid coordinates are valid only when both indexes are present and within the referenced container's bounds.
- Grid labels are derived: index `0` → `A`, `25` → `Z`, `26` → `AA`; displayed row is `rowStart + rowIndex`.
- The initial orientation is columns left-to-right and rows top-to-bottom.
- Non-grid containers reject row/column indexes and use `freeformPosition`.
- `pcs` quantities must be whole numbers. Other units may use decimals.
- Active total quantity is the sum of all non-archived placements for the part, grouped by unit. The UI must not add incompatible units into one number.
- Reorder status is computed only from the part's non-`unknown` default unit. Unknown or mixed units require review instead of a misleading low-stock calculation.
- A part may have no placements and therefore zero stock.
- Different parts may share one bin.
- For the same part, container, coordinate/freeform position, lot/date code, condition, and unit, the UI offers to update the existing placement rather than create an indistinguishable duplicate.
- Archiving a placement does not archive its part. Archiving a part blocks new stock changes until restored but does not silently archive placements.

---

## 4. Category templates

Templates are frontend assistance, not separate persisted schemas. Users may add, remove, or rename flexible attributes.

| Category | Example subcategories | Suggested attributes |
|---|---|---|
| Passive | Resistor, Capacitor, Inductor, Ferrite, Potentiometer | resistance/capacitance/inductance, tolerance, power, voltage, dielectric, polarity, temperature coefficient |
| Semiconductor | Diode, BJT, MOSFET, JFET, LED, Rectifier | polarity/type, voltage rating, current rating, power, gain, wavelength |
| Integrated Circuit | Op-amp, Logic, Regulator, MCU, Interface, Driver | function, supply voltage, channel count, output current, protocol, temperature range |
| Connector | Header, Terminal Block, Housing, Contact, Socket | positions, pitch, rows, gender, current rating, wire gauge |
| Electromechanical | Relay, Switch, Fan, Motor, Buzzer | coil voltage, contact rating, actuator, dimensions |
| Module / Board | Development Board, Sensor Module, Converter | function, interface, supply voltage, firmware/reference |
| Cable / Wire | Hook-up Wire, Ribbon Cable, Harness | conductor count, gauge, length, color, insulation |
| Hardware / Mechanical | Standoff, Screw, Heat Sink, Enclosure | thread, dimensions, material |
| Other | User-defined | User-defined |

The initial preset list is editable in code without a database migration because persisted category/subcategory values remain strings.

---

## 5. User interface contract

### 5.1 Main parts table

Default-visible columns:

1. Stock status
2. Category
3. Manufacturer part number
4. Value / label
5. Package
6. Mounting
7. Total quantity with unit summary
8. Locations
9. Manufacturer
10. Datasheet/product actions

Optional columns include internal part number, subcategory, supplier, supplier SKU, reorder point, target quantity, part status, updated date, and archived state.

The Locations cell shows compact placement chips such as:

```text
Cabinet 1 · C7 (100 pcs)   Desk — Alex · B4 (20 pcs)
```

When space is limited, show the first placement plus `+N more`; the full list remains available by tooltip, keyboard focus, or the part detail view.

### 5.2 Part editor

Organize fields into sections:

1. **Identity:** internal part number, category, subcategory, manufacturer, MPN, value/label.
2. **Form:** mounting type, package type, description, part status.
3. **Specifications:** category-suggested flexible attributes plus custom attributes.
4. **Supplier and documentation:** supplier, supplier SKU, supplier packaging, product URL, datasheet URL, picture.
5. **Replenishment:** default unit of measure, reorder point, and target quantity.
6. **Stock locations:** placement list, add placement, move quantity, adjust quantity, count stock.
7. **Notes and archive:** notes and archived state.

Creating a catalog part does not require positive stock. A user may save a part first and add placements later.

### 5.3 Location manager

Provide a Lab-only manager that can:

- add, edit, archive, and restore areas;
- add, edit, archive, and restore containers;
- configure a grid's rows and columns with a live preview;
- block destructive grid resizing when occupied cells would become invalid;
- open a container and inspect its cells and contents;
- search areas, containers, coordinates, MPNs, and part descriptions;
- distinguish archived locations and prevent new stock placement into them.

### 5.4 Grid-bin picker

Required behavior:

- Area selector followed by container selector.
- Column headers use Excel letters; row headers use numbers.
- Every cell exposes a label containing the full location path and occupancy summary.
- Empty, occupied, selected, and unavailable cells are visually distinct without relying on color alone.
- Hover/focus reveals contents; click or Enter selects the cell.
- Arrow keys navigate cells; Home/End and Page Up/Page Down are optional enhancements.
- Selecting an occupied cell is allowed after showing its contents and a non-blocking warning.
- Large grids remain usable with bounded cell sizing and scrolling; do not render unreadably tiny cells to force everything onscreen.

### 5.5 Stock actions

- **Add placement:** creates quantity at a selected location.
- **Adjust quantity:** replaces the current placement quantity after confirmation; zero may archive the placement or leave a visible zero based on user choice.
- **Move stock:** atomically subtracts from a source placement and adds to or creates a destination placement; cannot move more than available.
- **Count stock:** records counted quantity, `lastCountedAt`, `lastCountedBy`, and `countState = counted`.
- **View bin contents:** lists every part and quantity in one cell.

This slice is current-state inventory. Adjustment reasons may be captured in notes, but a durable transaction ledger is deferred explicitly rather than implied.

---

## 6. Search, filters, and derived status

Global search includes:

- internal part number;
- manufacturer and MPN;
- category/subcategory;
- value, mounting, and package;
- description and flexible attribute values;
- supplier and supplier SKU;
- area, container, coordinate, and freeform position;
- notes.

Column filters include category, subcategory, manufacturer, MPN, value, mounting, package, area, container, coordinate, stock status, part status, and archived scope.

Derived stock status, evaluated per compatible unit group:

1. `no_stock` when active quantity is zero.
2. `unit_review` when the part's default unit is `unknown` or no active placement uses the configured default unit.
3. `low_stock` when a reorder point exists and quantity in the default unit is less than or equal to it.
4. `in_stock` when quantity in the default unit is positive and above the reorder point, or no reorder point exists.
5. `mixed_units` when active placements use additional incompatible units; show grouped totals and require cleanup rather than producing a misleading combined number.
6. Archived parts are excluded from active stock counts.

---

## 7. Legacy data migration

Migration must be idempotent, tested with fixtures, and reversible from a pre-migration backup/export.

### 7.1 Field mapping

| Existing Lab field | Target |
|---|---|
| `entryUuid`, `id`, `databaseId` | Retain as part identity fields. |
| `assetNumber` | `internalPartNumber`. |
| `manufacturer` | `manufacturer`. |
| `model` | `manufacturerPartNumber`. |
| `description` | `description`. |
| `links` | `productUrl`; do not guess that it is a datasheet. |
| `picturePath` | `picturePath`. |
| `notes` | `notes`. |
| `qty` + `location` | Create zero or one initial stock placement using the exact quantity and original free-text location; set the unit to `unknown` rather than assuming pieces. |
| `verifiedInSurvey = true` | `countState = legacy_verified`; do not invent `lastCountedAt` from `updatedAt`. |
| `verifiedInSurvey = false` | `countState = uncounted`. |
| `archived` | Part archived state. |
| `createdAt`, `updatedAt` | Retain. |

Existing `serialNumber`, `projectName`, `assignedTo`, `lifecycleStatus`, `workingStatus`, and `condition` must not be silently discarded or semantically reinterpreted. Preserve them as compatibility fields through migration and include them in an explicit legacy export section until the owner completes a data audit and approves removal or remapping.

### 7.2 Unstructured locations

- Create a migration-owned area such as `Legacy / Unstructured` and a non-grid container such as `Imported Locations`.
- Store the original location text in `freeformPosition` exactly after safe trimming.
- Blank legacy locations with positive quantity become an `Unassigned` placement so quantity is not lost.
- Legacy quantities use `unitOfMeasure = unknown` until a user reviews them; migration must not infer `pcs` from the module name or description.
- Do not parse strings such as `Cabinet A / Bin C7` automatically during the initial migration. Provide a later review workflow to move them into configured structured containers.

### 7.3 Duplicate review

- Do not merge records automatically based on manufacturer plus MPN.
- Produce a migration report grouping likely duplicates by normalized manufacturer + MPN and by nonblank internal part number.
- Duplicate internal part numbers block commit until resolved; the migration cannot override the target uniqueness invariant.
- Likely MPN duplicates are review warnings because records may represent variants, incomplete labels, or intentionally separate catalog entries.

### 7.4 Cutover

1. Confirm no standalone/unified Lab writer is active.
2. Export the existing 19-column workbook and copy the local/shared Lab data to a dated backup location.
3. Run a dry-run migration report with counts, invalid rows, likely duplicates, and proposed placements.
4. Require explicit confirmation before committing.
5. Commit local migration and write a version marker only after every entity validates.
6. Produce the first new-schema snapshot from one designated upgraded client.
7. Block old Lab clients from writing to the upgraded shared root with a clear schema-version message.
8. Smoke on a copied/test root before upgrading the live product root.
9. Document rollback instructions and retain the pre-migration artifacts until the team validates the new release.

---

## 8. Persistence and shared sync

The Lab module remains in `te-lab-components.feox` and retains its own shared root and runtime session. TE Test Equipment is untouched.

Recommended FeOx key families:

```text
part:{part_uuid}
storage-area:{area_uuid}
storage-container:{container_uuid}
stock-placement:{placement_uuid}
```

Requirements:

- Bump the **Lab-only** shared schema from v1 to v2 for the new entity projection.
- Sync operations identify entity type and entity UUID; tombstones are entity-specific.
- Snapshots include parts, areas, containers, and placements in deterministic order.
- Apply dependencies in a safe order: areas → containers → parts → placements; deletes reverse dependency order or reject orphan creation.
- A placement referencing a missing part or container is rejected/quarantined rather than silently dropped.
- Every new field participates in normalization, validation, serialization, field-change tracking, canonical checksum, snapshotting, conflict handling, export, and tests.
- Numeric totals and stock status are derived and excluded from persisted sync payloads.
- Quantity updates use normal conflict detection; they are not automatically added together. The one-writer rule remains mandatory until a durable movement ledger provides commutative operations.
- Old Lab clients must not write v1 operations after the root is upgraded to v2.
- TE schema v2 and Lab schema v2 are unrelated module-local versions; do not share model or operation types merely because the numbers match.

---

## 9. Excel export contract

Replace the single flat Lab workbook as the canonical new-schema export with these sheets:

1. **Parts** — active catalog fields, derived totals, and stock status.
2. **Archived Parts** — archived catalog records.
3. **Stock Placements** — part identifiers, full location path, coordinate, quantity, unit, packaging, lot/date code, condition, and count metadata.
4. **Storage Layout** — areas, containers, grid dimensions, and archive status.
5. **Attributes** — one row per part/attribute key with value and unit.
6. **Legacy Fields** — preserved compatibility fields until their migration audit is complete.

Workbook requirements:

- Stable headers covered by tests.
- Part UUID and placement UUID included for traceability.
- Full location path and separate area/container/coordinate columns included.
- URLs remain clickable or plain valid text without merging datasheet and product URLs.
- Totals are formulas or values derived from the same placement aggregation used by the app.
- Keep a documented legacy export path only if team workflow still requires it; do not make the legacy flat sheet the source of truth after cutover.

---

## 10. Implementation tasks

### Task 1: Freeze fixtures and migration acceptance data

- [x] Add synthetic legacy Lab entries covering blank locations, grid-like text, decimal quantities, verified/unverified rows, archived rows, duplicate MPN warnings, and every legacy compatibility field.
- [x] Add expected migrated parts and placements as fixtures.
- [x] Add a migration dry-run report contract before implementation.
- [x] Record backup/restore commands for local and copied shared test roots.

### Task 2: Add domain entities and validation

Likely backend scope: `backend/src/modules/te_lab_components/model.rs`, new focused model files, storage codecs/keys, and tests.

- [x] Add Part, StorageArea, StorageContainer, StockPlacement, and flexible attribute types.
- [x] Add normalization and validation for identities, URLs, default units, thresholds, placement units, grids, coordinates, and count metadata.
- [x] Add Excel column-label conversion and round-trip tests through at least `AA`/`AB`.
- [x] Add placement invariants and total/status derivation tests.
- [x] Preserve legacy field decoding until migration/cutover is complete.

### Task 3: Add FeOx storage and query layer

- [x] Add isolated key prefixes and CRUD for all four entities.
- [x] Add indexes for part→placements, container→placements, and internal part number. Migration-only manufacturer+MPN duplicate review uses a deterministic scan because duplicate MPN groups are intentionally allowed.
- [x] Add atomic or rollback-safe stock move behavior.
- [x] Add pagination/batched scans where current limits require it.
- [x] Add orphan/reference-integrity tests.

### Task 4: Implement dry-run and commit migration

- [x] Implement idempotent migration version detection.
- [x] Map existing rows exactly as specified in §7.
- [x] Preserve unstructured location text and all legacy compatibility fields.
- [x] Produce counts, warnings, conflicts, and proposed placements without mutation.
- [x] Require explicit commit confirmation tied to the dry-run fingerprint.
- [x] Roll back all writes if any entity or version-marker step fails.
- [x] Add no-data-loss and rerun-idempotence tests.

### Task 5: Upgrade Lab shared sync to schema v2

- [x] Add entity-aware operations, snapshots, tombstones, checksums, and field changes.
- [x] Add dependency-safe apply ordering and orphan quarantine/error reporting.
- [x] Reject old schema writers after cutover.
- [x] Prove TE Test Equipment shared sync is unchanged.
- [x] Add two-database Lab v2 create/update/delete tests for every entity type.
- [x] Add conflict tests for simultaneous placement quantity edits.

### Task 6: Extend Tauri bridge contracts

- [x] Add Lab commands/results for parts, storage areas, containers, placements, move, adjust, count, bin contents, and migration dry-run/commit.
- [x] Add frontend runtime guards for every new payload.
- [x] Keep all commands module-scoped and reject cross-module entity use.
- [x] Update bridge mocks and shell tests.

### Task 7: Build generalized part UI

Likely frontend scope: `frontend/src/modules/te-lab-components/` only, plus shared primitive reuse where genuinely generic.

- [x] Replace asset-oriented labels with the approved part vocabulary.
- [x] Add category/subcategory presets with custom values.
- [x] Add common identity, form, supplier/documentation, replenishment, and status fields.
- [x] Add category-assisted flexible attribute editor.
- [x] Add the approved default and optional table columns.
- [x] Add total quantity, grouped-unit, and derived stock-status presentation.

### Task 8: Build location manager and grid picker

- [x] Add area and container management UI.
- [x] Add grid dimension preview and safe resize validation.
- [x] Add Excel-style column labels and numbered rows.
- [x] Add mouse and keyboard cell selection with accessible labels.
- [x] Add occupied-cell contents and non-blocking warnings.
- [x] Add scrolling/responsive behavior for large grids.
- [x] Add archived-area/container behavior.

### Task 9: Build stock placement workflows

- [x] Add placement list to part detail/edit UI.
- [x] Add, adjust, move, archive/restore, and count workflows.
- [x] Show quantity by placement and computed total.
- [x] Prevent over-move, invalid coordinates, negative quantity, and incompatible-unit totals.
- [x] Offer update/merge when creating an indistinguishable placement for the same part.
- [x] Add bin-contents view covering multiple part types in one cell.

### Task 10: Search, filters, preferences, and export

- [x] Extend global search, detailed filters, and sorting as specified in §6.
- [x] Add location-path and coordinate searching.
- [x] Version Lab column-visibility/filter preference keys so old settings do not hide the new required columns.
- [x] Implement the multi-sheet workbook in §9.
- [x] Add stable workbook-header and row-projection tests.

### Task 11: Verification and release cutover

- [x] Run targeted frontend tests during each UI slice.
- [x] Run targeted Rust model/storage/migration/sync/export tests during each backend slice.
- [x] Run full frontend test, lint, and production build. Lint/build pass; the raw full suite has one unrelated current-date-sensitive TE seeded-count assertion, while every other frontend test passes.
- [x] Run Rust format, library/integration tests, and clippy with warnings denied.
- [x] Run fixture-backed migration on temporary Lab data and compare source, migrated, and export projections.
- [ ] Repeat migration/export rehearsal on an owner-provided copy of the current Lab database and shared root.
- [ ] Run a single-instance desktop smoke covering part creation, two placements, occupied-bin warning, move, count, restart persistence, TE↔Lab switching, and Shared status.
- [ ] Upgrade the live Lab root only with all other writers stopped and rollback artifacts present.
- [x] Update SESSION_HANDOFF, SESSION_START_PROMPT, DECISIONS, roadmap status, and this checklist with measured results.

---

## 11. Required automated coverage

### Backend

- Legacy decode and no-data-loss migration.
- Migration dry-run fingerprint, explicit confirmation, rollback, and idempotence.
- Attribute normalization and deterministic ordering.
- Area/container uniqueness and archive behavior.
- Excel column conversion and grid-bound validation.
- Placement uniqueness guidance, quantity/unit validation, grouped totals, stock status, count state, and safe moves.
- Reference integrity and snapshot/apply ordering.
- Lab schema-v2 local and two-database shared flows.
- Old-schema rejection and TE module isolation.
- Multi-sheet export headers and values.

### Frontend

- Category templates permit custom categories and attributes.
- Part form validation and field mapping.
- Table columns, totals, location chips, search, filters, and preferences.
- Location manager create/edit/archive and destructive-resize blocking.
- Grid mouse selection, keyboard selection, accessible names, `Z`→`AA` labels, occupied warnings, and scrolling.
- Multiple placements, grouped units, adjust, move, count, and bin contents.
- Migration dry-run/confirm UI and clear old-client/schema errors.
- TE Test Equipment regression coverage and module switching/cache isolation.

---

## 12. Risk register

| Risk | Mitigation |
|---|---|
| Existing Lab data is lost or reinterpreted | Dry-run + fingerprinted commit, preserve IDs/legacy fields, backup/export, copied-root rehearsal, no unattended merge. |
| `C7` is ambiguous | Full area/container/coordinate path; references use UUIDs, not display strings. |
| Grid resizing or archive creates orphan stock | Validate references and bounds; block destructive resize; retain archived locations. |
| Total quantity disagrees with bin quantities | Never persist/edit an independent total; derive from placements. |
| Same bin contains mixed parts | Allow intentionally, display contents, warn before adding. |
| Same part is duplicated within one bin | Detect indistinguishable placement key and offer update/merge. |
| Flexible attributes become inconsistent | Template-assisted normalized keys, custom values allowed, searchable/exportable attribute rows. |
| Quantity edits are lost during sync conflicts | One-writer rule, base-version conflict detection, no additive merge without a movement ledger. |
| Old clients corrupt upgraded shared data | Lab schema-v2 gate and coordinated cutover; block v1 writers. |
| Wide table becomes unusable | Focused defaults, optional columns, compact location chips, detail sections. |
| Large grids become inaccessible | Scrollable bounded cells, keyboard navigation, text labels independent of color. |
| Scope expands into procurement/ERP | Keep purchasing history, pricing APIs, reservations, barcodes, and durable movement ledger explicit non-goals. |

---

## 13. Acceptance criteria

| Area | Acceptance |
|---|---|
| A — General catalog | Resistor, capacitor, transistor, IC, connector, and custom-category examples fit without schema changes. |
| B — Part identity | MPN, supplier SKU, value, mounting, package, datasheet, and flexible attributes remain distinct and searchable. |
| C — Storage hierarchy | Two different containers may both have `C7`; full paths and UUID references remain unambiguous. |
| D — Grid picker | User can configure dimensions and select cells with mouse or keyboard; letters continue after `Z`. |
| E — Multi-location stock | One part can have quantities in multiple bins; total equals placement aggregation; mixed units are never falsely summed. |
| F — Shared bins | One bin can contain several parts; contents are visible and adding another part gives a warning, not a hard failure. |
| G — Migration | Every existing row and legacy field is accounted for; counts and IDs survive; no timestamp certainty is invented. |
| H — Sync | Lab v2 entities synchronize between two test databases; old Lab writers are blocked; TE remains unchanged. |
| I — Export | Parts, archived parts, placements, storage layout, attributes, and legacy fields export with stable tested headers. |
| J — Verification | Full automated gates and a copied-data desktop smoke pass before live-root cutover. |

---

## 14. Definition of done

1. The Lab module is a generalized parts catalog, not an equipment-shaped or resistor-specific table.
2. Parts and stock placements are separate persisted entities with stable identities.
3. Areas and containers make every displayed bin path unambiguous.
4. Grid containers provide the approved Excel-style picker.
5. Multiple locations and shared-bin contents work with per-placement quantities.
6. Existing Lab data migrates without silent loss, invented facts, or automatic merges.
7. Lab shared sync upgrades safely and remains isolated from TE Test Equipment.
8. Search, filters, preferences, and export cover the new model.
9. Automated verification and copied-data desktop smoke are recorded in SESSION_HANDOFF.
10. This plan's task boxes and status are updated to reflect the implementation actually shipped.

---

## 15. Implementation configuration to collect

These are editable setup values, not unresolved architecture decisions:

- Initial area names and optional desk owners.
- Container names, types, row counts, and column counts.
- Preferred category/subcategory presets.
- Preferred units of measure.
- Whether the first rollout starts with only the main cabinet or includes desk organizers immediately.

The app must allow these values to evolve without a database-schema change.
