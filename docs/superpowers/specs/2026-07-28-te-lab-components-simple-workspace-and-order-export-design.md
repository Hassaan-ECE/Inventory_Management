# TE Lab Components Simple Workspace and Order Export Design

**Date:** 2026-07-28  
**Status:** Approved in conversation; written specification awaiting owner review  
**Scope:** TE Lab Components only

## Context

The generalized Lab catalog released in Inventory Management 0.1.1 supports flexible catalog fields, multiple stock placements, storage areas, storage containers, and grid coordinates. The team currently needs a much simpler through-hole-parts workflow:

- one row per component type;
- one active shelf location per component;
- stock counted in pieces;
- short location codes such as `A1`, `B3`, and `M15`;
- a small, consistent component-type list;
- type-aware engineering units such as `nF` and `kΩ`;
- a focused selected-component Excel export for the ordering team.

The simplified experience must not discard the generalized catalog or change the Lab sync schema. The richer model may be needed again later.

## Goals

1. Make the normal Lab table visually consistent with the other Inventory Management tables.
2. Show only Stock Status, Component Type, Value, Quantity, and Location by default.
3. Provide one focused add/edit experience for the current through-hole workflow.
4. Store one active shelf location and one whole-piece quantity for new simple components.
5. Preserve all existing part, placement, area, container, migration, export, and sync records.
6. Add an explicit selection mode that produces a ready-to-send order-request workbook.
7. Retain the existing full catalog workbook for audit and engineering inventory sharing.

## Non-goals

- Removing the generalized catalog or storage structures.
- Changing Lab catalog sync schema v2.
- Combining Lab data with TE Test Equipment.
- Automatically merging or deleting multiple existing placements.
- Engineering-unit conversion, mixed-unit arithmetic, or numeric range search.
- Supporting surface-mount inventory in the simple workflow.
- Replacing the existing full catalog export.
- Changing live shared roots or performing a live-data cutover.

## 1. Everyday Table

### 1.1 Normal mode

The normal table has no selection column and defaults to these columns:

1. Stock Status
2. Component Type
3. Value
4. Quantity
5. Location

The table retains the established Inventory Management styling, including sortable headers, row colors, hover states, context actions, keyboard focus, and double-click-to-edit behavior.

`Component Type` is the user-facing label for the existing persisted `subcategory` field. The stored field name and sync payload do not change.

Quantity is displayed as a whole-piece count, for example `40 pcs`. Location is displayed as a short normalized code, for example `A1`; the table does not show an area/container path.

Existing detailed catalog columns remain available as opt-in View settings. They are hidden by default rather than deleted.

Search continues to include hidden manufacturer, MPN, supplier, link, notes, advanced attributes, and storage information so simplifying the visible columns does not reduce findability.

### 1.2 Export menu

The Export menu contains:

- **Full Catalog Excel** — the existing detailed multi-sheet workbook.
- **Select Components for Order** — enters order-export selection mode.
- Existing unrelated export entries may remain only if they are implemented and useful; an unimplemented HTML entry should not compete with the two supported Excel actions.

### 1.3 Selection mode

Selection controls are visible only in selection mode.

- A leading checkbox column appears.
- Clicking a row or its checkbox toggles that component.
- Selected rows receive the app's standard subtle selected-row highlight.
- A compact selection bar shows the selected count and **Continue** and **Cancel** actions.
- **Continue** is disabled until at least one active component is selected.
- Search and filters remain usable.
- Selections remain selected if a filter temporarily hides them.
- Exiting selection mode, switching module, or switching Inventory/Archive scope clears the selection.
- Normal edit/context-menu actions do not trigger from a selection-mode row click.
- Keyboard users can focus and toggle rows, continue, or cancel.

Only active Inventory components participate in the order workflow. Components with mixed or non-`pcs` active stock units cannot be selected until their units are reviewed, because the focused workbook requests whole pieces. Multiple active placements that all use `pcs` may be selected because their authoritative piece total is unambiguous. The existing full catalog export remains the way to share archived and complete catalog records.

## 2. Focused Component Editor

### 2.1 Everyday fields

The initially visible fields are:

- **Component Type**
- **Value / Part Label**
- **Unit**, when the chosen type has an engineering-value unit
- **Quantity**
- **Location**

New records default to:

- mounting type `through_hole`;
- stock unit `pcs`;
- quantity `0`;
- one active placement after a valid location is supplied.

Quantity must be a nonnegative whole number.

Location accepts lowercase input but normalizes it to uppercase. The accepted simple format is:

```text
^[A-Z][1-9][0-9]*$
```

Examples: `A1`, `B3`, `M15`. Invalid examples include blank location for positive stock, `A0`, `12A`, and `A-1`.

A zero-quantity component may retain a valid shelf location so users can record where replenished stock belongs. A new zero-quantity component may also be saved without a location; the table then shows `—`.

### 2.2 Component types and category mapping

The initial Component Type choices are:

- Resistor
- Capacitor
- Inductor
- Diode
- LED
- BJT
- MOSFET
- IC
- MCU
- Connector
- Relay
- Switch
- Sensor
- Module/Board
- Cable/Wire
- Hardware
- Other

The existing broader `category` value is derived when a recognized Component Type is chosen:

| Component Type | Persisted Category |
|---|---|
| Resistor, Capacitor, Inductor | Passive |
| Diode, LED, BJT, MOSFET | Semiconductor |
| IC, MCU | Integrated Circuit |
| Connector | Connector |
| Relay, Switch | Electromechanical |
| Sensor, Module/Board | Module / Board |
| Cable/Wire | Cable / Wire |
| Hardware | Hardware / Mechanical |
| Other | Other |

An existing unrecognized subcategory remains representable and is not silently replaced. Its current category/subcategory values remain unchanged unless the user explicitly selects a new Component Type.

### 2.3 Engineering-value units

For passive components, the editor separates the typed value from its unit:

| Component Type | Unit choices |
|---|---|
| Capacitor | pF, nF, µF, mF, F |
| Resistor | mΩ, Ω, kΩ, MΩ |
| Inductor | nH, µH, mH, H |

Common rated-value units available to advanced attributes include `mV`, `V`, `mA`, `A`, `mW`, and `W`.

The table and exports combine the value and unit into a display string such as `100 nF` or `1 kΩ`. Types such as BJT, MCU, and Connector use Value / Part Label as normal text, such as `2N3904` or `STM32F103C8`.

For recognized passive values:

- `attributes.value.value` stores the value text;
- `attributes.value.unit` stores the selected engineering unit;
- `displayValue` stores the combined user-facing form.

When loading older data, the editor first uses `attributes.value`. If it is absent, it may parse a recognized suffix from `displayValue`; otherwise it preserves the complete existing `displayValue` as text. Saving the simple form must not remove unrelated advanced attributes.

### 2.4 More Details

A collapsed **More Details** section retains access to:

- Internal Part Number
- Manufacturer
- Manufacturer Part Number
- Supplier
- Supplier SKU
- Supplier Packaging
- Product URL
- Datasheet URL
- Package
- Description
- Notes
- Reorder Point
- Target Quantity
- Part Status
- Picture
- Existing custom attributes
- Existing advanced placement information

The richer storage manager and placement operations remain in the codebase. They may be exposed from More Details when needed for compatibility review and can be promoted again in a future release without a schema migration.

## 3. Storage Compatibility

The simple workflow is a projection over the existing Part and StockPlacement records; it does not add a competing component table.

### 3.1 Simple placement

A designated non-grid storage area/container represents the single TE Lab shelf under the existing storage model. Its names are implementation constants and are not shown in the everyday table. The short shelf code is stored in the placement's `freeformPosition`.

Creating or updating a simple component:

- uses exactly one active placement;
- uses `pcs`;
- stores the normalized shelf code in `freeformPosition`;
- preserves the placement UUID on update;
- updates quantity rather than creating indistinguishable duplicates;
- does not delete archived placement history.

Part and simple-placement changes must be atomic from the user's perspective. If either write fails, the command restores the prior records and reports the failure; the UI does not show a partially saved component.

### 3.2 Existing grid placement

An existing part with exactly one active `pcs` placement remains usable in the simple table. A grid placement displays only its coordinate, while a non-grid placement displays its freeform position. The underlying area, container, row, and column are preserved.

### 3.3 Review-required data

The app must not coerce or lose data that does not fit the simple projection.

A component requires advanced location review when it has:

- more than one active placement;
- a non-`pcs` active placement;
- mixed stock units;
- an active placement with no valid short coordinate/freeform code.

For review-required components:

- the table preserves the authoritative total and shows a concise review indicator in Location;
- identity fields remain editable;
- simple Quantity and Location edits are disabled;
- More Details shows the preserved placements and the reason review is required;
- no placement is merged, archived, moved, or deleted automatically.

## 4. Focused Order Export

### 4.1 Order preparation

Choosing **Continue** in selection mode opens an order-preparation dialog.

Each selected component shows:

- Component Type
- Value
- Manufacturer and MPN when present
- Current Quantity
- Stock Status
- a required **Qty Requested** input
- an optional request note used only in the workbook

Qty Requested must be a positive whole number. It starts blank so the app does not guess procurement intent from reorder settings.

Missing Manufacturer, MPN, Supplier, Supplier SKU, or Product Link data produces a visible non-blocking warning. Users may still export components for which the ordering team will determine the source.

### 4.2 Workbook

The focused workbook contains one sheet named **Order Request** with these columns:

1. Component Type
2. Value
3. Manufacturer
4. MPN
5. Supplier
6. Supplier SKU
7. Product Link
8. Stock Status
9. Current Qty
10. Qty Requested
11. Location
12. Notes

Workbook presentation includes:

- a frozen header row;
- filters on all columns;
- readable column widths;
- hyperlink formatting for Product Link;
- numeric Current Qty and Qty Requested cells;
- only the selected active components;
- the user-entered request note, not private/internal catalog notes.

The default filename is:

```text
TE_Lab_Components_Order_Request_YYYY-MM-DD.xlsx
```

The backend loads authoritative catalog and placement data by selected part UUID. The frontend sends only the selected UUID, requested quantity, and optional request note for each line. Unknown, archived, or duplicate UUIDs are rejected with a clear error rather than exported ambiguously.

Export is read-only and never changes inventory records, reorder settings, or sync state.

### 4.3 Cancellation and failure

- Canceling order preparation returns to selection mode with selections and entered values intact.
- Canceling the file picker is reported as cancellation, not failure.
- A workbook-generation failure keeps the dialog and selection available for retry.
- A successful export closes preparation and exits selection mode.

## 5. Interfaces and Boundaries

The implementation should add focused frontend helpers for:

- component-type/category mapping;
- engineering-unit options and display formatting;
- simple-location normalization and validation;
- projecting a Part plus placements into normal, simple, or review-required editor state;
- export selection state and order-request lines.

The desktop bridge should add focused Lab-only commands for:

- atomically saving a simple Part plus its single placement;
- exporting selected Lab order-request lines.

Existing advanced Part, StockPlacement, StorageArea, StorageContainer, full-catalog export, migration, and sync commands remain supported. TE Test Equipment interfaces do not change.

No persisted user selection, requested quantity, or order note is required. These are session-only UI state.

## 6. Error Handling and Accessibility

- Validation messages identify the exact invalid field.
- Lowercase shelf codes normalize without an error.
- Positive stock without a location is blocked.
- Selection mode and the order dialog announce selected counts and export status through the existing status announcer.
- Checkboxes, selected rows, More Details, dialog controls, and unit selectors have accessible labels.
- Selection is conveyed by checkbox state and highlight, not color alone.
- Read-only/shared-sync restrictions continue to block inventory mutations but do not block read-only exports.

## 7. Testing and Verification

### 7.1 Frontend tests

Tests must cover:

- five default columns and the `Component Type` label;
- absence of selection controls in normal mode;
- Export → Select Components entering and canceling selection mode;
- row/checkbox selection, filtering with retained selection, and selection clearing boundaries;
- controlled Component Type choices and category derivation;
- type-aware unit choices and combined display values;
- lowercase-to-uppercase location normalization and invalid location rejection;
- whole-piece quantity validation;
- simple editor creation/update projection;
- preservation of unrelated attributes and unrecognized existing subcategories;
- review-required behavior for multiple placements and non-piece units;
- order preparation, positive requested quantities, warnings, cancellation, retry, and success.

### 7.2 Backend tests

Tests must cover:

- simple shelf area/container resolution without duplicates;
- atomic create and update of a Part plus one placement;
- preservation of UUIDs and advanced/archived records;
- rollback when either part or placement persistence fails;
- rejection of attempts to overwrite review-required placement sets;
- selected-only workbook row membership and ordering;
- authoritative field loading by UUID;
- rejection of unknown, archived, and duplicate UUIDs;
- required headers, sheet name, filters, frozen row, hyperlinks, numeric quantities, and filename;
- no database changes during successful, canceled, or failed export.

### 7.3 Release verification

Before completion:

- run focused frontend tests and backend unit tests;
- run the full frontend test suite, lint, and production build;
- run Rust formatting, the relevant filtered Rust suites, and strict Clippy;
- run the desktop app only against copied local data with shared sync disabled or redirected, following `docs/SESSION_HANDOFF.md`;
- verify normal/edit/selection/order-export flows manually;
- verify an existing multiple-placement record remains byte-for-byte represented after viewing and canceling;
- do not modify the live Lab database or shared root during verification.

## 8. Acceptance Criteria

The design is complete when:

1. Normal Lab use presents the five-column simple table with no persistent checkbox column.
2. Users can add or edit a through-hole component using Component Type, value/unit, whole-piece quantity, and one short shelf code.
3. Existing advanced data remains present and recoverable without a Lab sync-schema migration.
4. Multiple-placement or mixed-unit records are flagged rather than silently simplified.
5. Export selection appears only after the user chooses it from the Export menu.
6. Users can enter requested quantities and create a focused workbook containing only selected components.
7. The existing full catalog workbook remains available.
8. Exports do not mutate inventory.
9. Automated gates and copied-data desktop verification pass without touching live data.
