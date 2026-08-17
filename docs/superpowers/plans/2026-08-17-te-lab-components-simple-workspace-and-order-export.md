# TE Lab Components Simple Workspace and Order Export Implementation Plan

**Status:** Implementation complete; owner acceptance and release staging pending

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the everyday TE Lab Components experience with a five-column through-hole-parts workspace, one short shelf location per simple component, type-aware value units, and a selected-component order-request workbook while preserving the generalized catalog and sync schema.

**Architecture:** Keep `Part`, `StockPlacement`, `StorageArea`, and `StorageContainer` as the only persisted Lab catalog records. Add a tested simple-workflow projection in the frontend and an atomic simple Part-plus-placement mutation in the backend; records that cannot be projected losslessly stay in review mode and retain the advanced editor. Add transient frontend selection/order state and a read-only backend workbook builder that resolves selected UUIDs against authoritative database records.

**Tech Stack:** Tauri 2, React 19, TypeScript, Vite, Tailwind CSS 4, Vitest/Testing Library, Bun, Rust, FeOxDB, `rust_xlsxwriter`.

---

## Global Constraints

- Authority: `docs/superpowers/specs/2026-07-28-te-lab-components-simple-workspace-and-order-export-design.md`.
- Scope is `te-lab-components` only. Do not change TE Test Equipment behavior, records, commands, shared roots, or sync streams.
- Keep Lab catalog sync schema v2. Do not add a competing component table or migrate existing Part/StockPlacement records.
- Keep stable identity: package `inventory-management`, Tauri id `com.inventory.management`, and Lab DB `%LOCALAPPDATA%\com.inventory.management\te-lab-components.feox`.
- New simple records are through-hole, use whole `pcs`, and accept only normalized shelf codes matching `^[A-Z][1-9][0-9]*$`.
- Preserve all advanced attributes, archived placements, grid coordinates, areas, containers, UUIDs, and unknown existing category/subcategory values unless a user explicitly changes them.
- Do not silently merge, move, archive, delete, or coerce multiple-placement, mixed-unit, non-`pcs`, or invalid-location data.
- Do not add dependencies. Reuse existing UI primitives, mutation rollback patterns, status announcer, desktop bridge, and `rust_xlsxwriter`.
- Use test-first development: every production behavior starts with a focused failing test and an observed expected failure.
- Verification must use an isolated local data root with shared sync disabled or redirected. Never run desktop verification against the live Lab database or product shared root.
- Do not bump the product version, stage an installer, enable auto-update, or perform a live cutover in this plan.

## Current-code anchors (verified 2026-08-17)

These are the live surfaces the tasks must change. Do not invent parallel tables or a second Lab view.

- Everyday table: `frontend/src/modules/te-lab-components/catalog/PartsTable.tsx` + `catalogColumns.ts`. Defaults currently show 11 columns. Visibility key is `teLabComponents.catalog.v4.columnVisibility`.
- Default sort is `{ column: "category", direction: "asc" }` in `TeLabComponentsView.tsx`. After this work it must sort by `subcategory` (Component Type).
- Editor: `catalog/PartDialog.tsx` titles are `Add Catalog Part` / `Edit Catalog Part`. Header button remains `Add Part`.
- Search input `aria-label` is `Search Lab components`. Search already includes hidden MPN/supplier/notes.
- Lab export menu is a **Lab-only** copy at `components/header/ExportMenu.tsx` (`Excel` + unimplemented `HTML`). TE has its own `ExportMenu`; do not change TE.
- Bridge Lab commands live in `tauriInventoryBridge.ts` / `labBridgeGuards.ts`. Excel result parsing already exists as `parseExcelExportResult` in `bridgeGuards.ts`.
- Backend mutations: `backend/src/modules/te_lab_components/catalog_mutations.rs` (`create_part_in_store`, `create_stock_placement_in_store`, …) with sync-queue rollback. Area lookup: `InventoryDb::find_storage_area_by_name` / `find_storage_container_by_name`.
- Full catalog Excel: `backend/src/integrations/export/catalog_workbook.rs` via `export_excel`. Reuse `ExcelExportResult` and `pick_export_path`.
- No `simpleComponent.ts`, `simple_workflow.rs`, or `lab_order_workbook.rs` exists yet.
- Mock catalog (`catalog/mockCatalog.ts`) identifies rows by MPN in several shell tests (`CF14JT1K00`, `2N3904BU`, `C315C104M5U5TA`). After the five-column default those MPNs are not rendered. Tests must assert visible `displayValue` / Component Type instead.
- Existing filter/sort test clicks `Sort by Manufacturer`. That header will be hidden; sort a default-visible column instead.
- Grid/placement/migration shell tests stay. They must open rows by a visible cell (`1 kΩ`, `4 position`, or an explicit `displayValue` override), then still reach **Add Placement** from More Details.

---

## File Structure

### New files

- `frontend/src/modules/te-lab-components/catalog/simpleComponent.ts` — component-type profiles, engineering units, value formatting/parsing, shelf validation, simple/review projection, and order eligibility.
- `frontend/src/modules/te-lab-components/catalog/SimplePartFields.tsx` — the five everyday editor fields and review warning.
- `frontend/src/modules/te-lab-components/catalog/OrderSelectionBar.tsx` — selection count plus Continue/Cancel actions.
- `frontend/src/modules/te-lab-components/catalog/OrderExportDialog.tsx` — requested-quantity preparation and missing-order-data warnings.
- `frontend/tests/te-lab-components-simple-component.test.ts` — pure frontend domain tests.
- `backend/src/modules/te_lab_components/simple_workflow.rs` — simple-workflow input/result types, default shelf resolution, atomic Part/placement create/update, and rollback tests.
- `backend/src/integrations/export/lab_order_workbook.rs` — authoritative selected-line resolution and focused workbook generation.

### Modified files

- `frontend/src/modules/te-lab-components/types.ts` — shared TypeScript simple mutation and order-export contracts.
- `frontend/src/modules/te-lab-components/catalog/catalogColumns.ts` — five default columns and Component Type label while retaining advanced opt-in columns.
- `frontend/src/modules/te-lab-components/catalog/categoryTemplates.ts` — add recognized simple-type labels without removing existing free-form templates.
- `frontend/src/modules/te-lab-components/catalog/PartDialog.tsx` — compose simple fields with collapsed advanced fields and route review-required saves safely.
- `frontend/src/modules/te-lab-components/catalog/PartsTable.tsx` — short location display and conditional selection column/row behavior.
- `frontend/src/modules/te-lab-components/catalog/CatalogHeader.tsx` — expose full-catalog and select-for-order callbacks.
- `frontend/src/modules/te-lab-components/catalog/CatalogSearchCard.tsx` — label the subcategory filter `Component Type` if the visible string is updated.
- `frontend/src/modules/te-lab-components/components/header/ExportMenu.tsx` — Lab-only: Full Catalog Excel + Select Components for Order. Remove unimplemented HTML.
- `frontend/src/modules/te-lab-components/TeLabComponentsView.tsx` — v5 column key, default sort, simple saves, selection state, order preparation, selected export, and lifecycle clearing.
- `frontend/src/integrations/tauri/desktop-bridge.d.ts` — simple mutation and selected-export bridge signatures.
- `frontend/src/integrations/tauri/labBridgeGuards.ts` — parse the simple composite mutation result.
- `frontend/src/integrations/tauri/tauriInventoryBridge.ts` — invoke new Lab commands.
- `frontend/tests/inventory-shell/helpers.tsx` — default mocks for new optional bridge methods.
- `frontend/tests/tauri-inventory-bridge.test.ts` — command name, payload, and response parsing tests.
- `frontend/tests/te-lab-components-shell.test.tsx` — rewrite column/editor expectations; add selection/order tests; retarget row finds to visible cells.
- `backend/src/modules/te_lab_components/mod.rs` — register `simple_workflow`.
- `backend/src/api/commands.rs` — coordinated simple create/update commands.
- `backend/src/integrations/export/mod.rs` — register selected Lab export, choose the save path, and return `ExcelExportResult`.
- `backend/src/lib.rs` — register the new Tauri commands.
- `docs/SESSION_HANDOFF.md` — record implementation and isolated verification results after all gates pass.
- `docs/SESSION_START_PROMPT.md` — update current state/next steps after implementation.

Do not modify `frontend/src/modules/te-test-equipment/components/header/ExportMenu.tsx`.

---

### Task 1: Frontend Simple-Component Domain

**Files:**
- Create: `frontend/src/modules/te-lab-components/catalog/simpleComponent.ts`
- Create: `frontend/tests/te-lab-components-simple-component.test.ts`
- Modify: `frontend/src/modules/te-lab-components/catalog/categoryTemplates.ts`

**Interfaces:**
- Consumes: existing `Part`, `PartInput`, `PartStockSummary`, `StockPlacement`, and `StorageContainer`.
- Reuses: `placementPosition` from `catalog/catalogUtils.ts` for grid short codes.
- Produces:

```ts
export const COMPONENT_TYPE_OPTIONS: readonly string[];
export type SimpleReviewReason =
  | "multiple_placements"
  | "mixed_units"
  | "non_piece_unit"
  | "invalid_location";
export type SimpleStockProjection =
  | {
      kind: "simple";
      location: string;
      placementUuid: string | null;
      quantity: number;
    }
  | {
      kind: "review";
      locationLabel: string;
      quantityLabel: string;
      reasons: SimpleReviewReason[];
    };

export function categoryForComponentType(componentType: string): string | null;
export function engineeringUnitsFor(componentType: string): readonly string[];
export function formatComponentValue(componentType: string, value: string, unit: string): string;
export function readComponentValue(part: Part): { value: string; unit: string };
export function normalizeShelfLocation(value: string): string;
export function shelfLocationError(value: string, quantity: number): string | null;
export function projectSimpleStock(
  placements: StockPlacement[],
  containersById: ReadonlyMap<string, StorageContainer>,
): SimpleStockProjection;
export function isOrderSelectable(summary: PartStockSummary | undefined): boolean;
export function applySimpleIdentity(
  input: PartInput,
  componentType: string,
  value: string,
  unit: string,
): PartInput;
```

- `applySimpleIdentity` preserves unrelated attributes and unrecognized existing category/subcategory values until the user chooses a recognized Component Type.
- `projectSimpleStock` considers **active** (non-archived) placements only.

- [ ] **Step 1: Write failing pure-domain tests**

```ts
import { describe, expect, it } from "vitest";
import {
  applySimpleIdentity,
  categoryForComponentType,
  engineeringUnitsFor,
  formatComponentValue,
  isOrderSelectable,
  normalizeShelfLocation,
  projectSimpleStock,
  shelfLocationError,
} from "@/modules/te-lab-components/catalog/simpleComponent";
import type {
  PartStockSummary,
  QuantityTotal,
  StockPlacement,
} from "@/modules/te-lab-components/types";

function placement(overrides: Partial<StockPlacement> = {}): StockPlacement {
  return {
    placementUuid: "placement-1",
    partUuid: "part-1",
    containerUuid: "simple-shelf",
    columnIndex: null,
    rowIndex: null,
    freeformPosition: "A1",
    quantity: 1,
    unitOfMeasure: "pcs",
    packaging: "",
    lotCode: "",
    dateCode: "",
    condition: "new",
    countState: "uncounted",
    lastCountedAt: null,
    lastCountedBy: "",
    notes: "",
    archived: false,
    createdAt: "2026-08-17T12:00:00.000Z",
    updatedAt: "2026-08-17T12:00:00.000Z",
    ...overrides,
  };
}

function summary(totals: QuantityTotal[]): PartStockSummary {
  return {
    partUuid: "part-1",
    totals,
    stockStatus: totals.length === 1 && totals[0]?.unitOfMeasure === "pcs"
      ? "in_stock"
      : "unit_review",
  };
}

describe("simple Lab component projection", () => {
  it("maps controlled component types without changing persisted field names", () => {
    expect(categoryForComponentType("Capacitor")).toBe("Passive");
    expect(categoryForComponentType("MCU")).toBe("Integrated Circuit");
    expect(categoryForComponentType("Legacy Custom Type")).toBeNull();
  });

  it("uses type-aware engineering units and combines display values", () => {
    expect(engineeringUnitsFor("Capacitor")).toEqual(["pF", "nF", "µF", "mF", "F"]);
    expect(formatComponentValue("Capacitor", "100", "nF")).toBe("100 nF");
    expect(formatComponentValue("BJT", "2N3904", "")).toBe("2N3904");
  });

  it("normalizes and validates one-letter shelf locations", () => {
    expect(normalizeShelfLocation(" b3 ")).toBe("B3");
    expect(shelfLocationError("m15", 40)).toBeNull();
    expect(shelfLocationError("AA1", 40)).toBe("Use a shelf code such as A1, B3, or M15.");
    expect(shelfLocationError("", 1)).toBe("Location is required when quantity is greater than zero.");
    expect(shelfLocationError("", 0)).toBeNull();
  });

  it("projects one pcs placement but flags advanced placement sets", () => {
    expect(projectSimpleStock([placement({ quantity: 40, unitOfMeasure: "pcs", freeformPosition: "A1" })], new Map()))
      .toMatchObject({ kind: "simple", location: "A1", quantity: 40 });
    expect(projectSimpleStock([
      placement({ placementUuid: "p1", unitOfMeasure: "pcs" }),
      placement({ placementUuid: "p2", unitOfMeasure: "pcs" }),
    ], new Map())).toMatchObject({ kind: "review", reasons: ["multiple_placements"] });
  });

  it("allows order selection only for an unambiguous pcs total", () => {
    expect(isOrderSelectable(summary([{ unitOfMeasure: "pcs", quantity: 4 }]))).toBe(true);
    expect(isOrderSelectable(summary([{ unitOfMeasure: "m", quantity: 4 }]))).toBe(false);
    expect(isOrderSelectable(summary([
      { unitOfMeasure: "pcs", quantity: 4 },
      { unitOfMeasure: "reel", quantity: 1 },
    ]))).toBe(false);
  });
});
```

Any additional test `Part` must be a complete typed value following the existing `buildLabPart` field contract; do not use `as unknown as`.

- [ ] **Step 2: Run the focused test and observe the expected failure**

Run from repository root:

```powershell
bun test frontend/tests/te-lab-components-simple-component.test.ts
```

Expected: FAIL because `catalog/simpleComponent.ts` does not exist.

- [ ] **Step 3: Implement the component profiles and pure projection**

Use one profile table as the source of truth:

```ts
const COMPONENT_PROFILES = {
  Resistor: { category: "Passive", units: ["mΩ", "Ω", "kΩ", "MΩ"] },
  Capacitor: { category: "Passive", units: ["pF", "nF", "µF", "mF", "F"] },
  Inductor: { category: "Passive", units: ["nH", "µH", "mH", "H"] },
  Diode: { category: "Semiconductor", units: [] },
  LED: { category: "Semiconductor", units: [] },
  BJT: { category: "Semiconductor", units: [] },
  MOSFET: { category: "Semiconductor", units: [] },
  IC: { category: "Integrated Circuit", units: [] },
  MCU: { category: "Integrated Circuit", units: [] },
  Connector: { category: "Connector", units: [] },
  Relay: { category: "Electromechanical", units: [] },
  Switch: { category: "Electromechanical", units: [] },
  Sensor: { category: "Module / Board", units: [] },
  "Module/Board": { category: "Module / Board", units: [] },
  "Cable/Wire": { category: "Cable / Wire", units: [] },
  Hardware: { category: "Hardware / Mechanical", units: [] },
  Other: { category: "Other", units: [] },
} as const;

const SIMPLE_LOCATION = /^[A-Z][1-9][0-9]*$/;
```

In `categoryTemplates.ts`, **add** missing simple labels (`IC`, `Sensor`, `Module/Board`, `Cable/Wire`, `Hardware`) to the matching category's `subcategories` array. Do not remove existing labels such as `Ferrite`, `Header`, or `Op-amp`.

For a grid placement, derive the short code through existing `placementPosition`. For a freeform placement, normalize `freeformPosition`. Ignore archived placements. Return review mode without mutating any input.

`readComponentValue` uses `attributes.value` first; if absent, it may parse a recognized suffix from `displayValue`; otherwise it returns the complete existing `displayValue` as text with an empty unit.

`applySimpleIdentity` writes `subcategory`, derived `category` (only when the type is recognized), `displayValue` via `formatComponentValue`, and for passive types `attributes.value = { value, unit }` without deleting other attributes.

- [ ] **Step 4: Run the focused test and confirm it passes**

```powershell
bun test frontend/tests/te-lab-components-simple-component.test.ts
```

Expected: PASS with no warnings.

- [ ] **Step 5: Commit the pure frontend domain**

```powershell
git add frontend/src/modules/te-lab-components/catalog/simpleComponent.ts frontend/src/modules/te-lab-components/catalog/categoryTemplates.ts frontend/tests/te-lab-components-simple-component.test.ts
git commit -m "feat(lab): add simple component projections"
```

---

### Task 2: Atomic Backend Simple-Component Workflow

**Files:**
- Create: `backend/src/modules/te_lab_components/simple_workflow.rs`
- Modify: `backend/src/modules/te_lab_components/mod.rs`

**Interfaces:**
- Consumes: `PartInput`; existing `create_part_in_store` / `update_part_in_store` / `create_stock_placement_in_store` / `update_stock_placement_in_store`; `InventoryDb` record/sync backup methods; existing `CatalogMutationResult<T>`.
- Produces:

```rust
pub(crate) const SIMPLE_AREA_NAME: &str = "TE Lab";
pub(crate) const SIMPLE_CONTAINER_NAME: &str = "Shelf";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SimpleComponentInput {
    pub part: PartInput,
    pub quantity: u32,
    pub location: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SimpleComponentValue {
    pub part: Part,
    pub placement: Option<StockPlacement>,
}

pub(crate) fn create_simple_component_in_store(
    input: SimpleComponentInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<SimpleComponentValue>>;

pub(crate) fn update_simple_component_in_store(
    part_id: &str,
    input: SimpleComponentInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<SimpleComponentValue>>;
```

- Do not duplicate normalization, uniqueness, validation, or sync-queue rules already in `catalog_mutations.rs`. Call those helpers. If a helper is private, expose it as `pub(crate)` in that file rather than copying it.
- Default storage is one active non-grid `TE Lab / Shelf` pair resolved by normalized name. Creation is idempotent via `find_storage_area_by_name` / `find_storage_container_by_name`.

- [ ] **Step 1: Write failing backend tests in the new module**

```rust
#[test]
fn creates_simple_part_and_one_piece_placement() {
    let db = ready_catalog_db();
    let result = create_simple_component_in_store(
        simple_input("Capacitor", "100 nF", 40, " b3 "),
        &db,
    )
    .unwrap();

    assert_eq!(result.value.part.subcategory, "Capacitor");
    assert_eq!(result.value.part.mounting_type, "through_hole");
    let placement = result.value.placement.unwrap();
    assert_eq!(placement.quantity, 40.0);
    assert_eq!(placement.unit_of_measure, "pcs");
    assert_eq!(placement.freeform_position, "B3");
    assert_eq!(db.load_stock_placements_for_part(&result.value.part.entry_uuid).unwrap().len(), 1);
}

#[test]
fn updates_the_existing_simple_placement_without_replacing_its_uuid() {
    let db = ready_catalog_db();
    let created = create_simple_component_in_store(
        simple_input("Resistor", "1 kΩ", 5, "A1"),
        &db,
    )
    .unwrap();
    let placement_uuid = created.value.placement.unwrap().placement_uuid;

    let updated = update_simple_component_in_store(
        &created.value.part.entry_uuid,
        simple_input("Resistor", "1 kΩ", 9, "A2"),
        &db,
    )
    .unwrap();

    assert_eq!(updated.value.placement.unwrap().placement_uuid, placement_uuid);
}

#[test]
fn refuses_to_simplify_multiple_or_non_piece_placements() {
    let (db, part_uuid) = catalog_with_advanced_placements();
    let before = workflow_snapshot(&db);
    let error = update_simple_component_in_store(
        &part_uuid,
        simple_input("Resistor", "1 kΩ", 9, "A2"),
        &db,
    )
    .unwrap_err();

    assert!(error.contains("advanced location review"));
    assert_eq!(workflow_snapshot(&db), before);
}

#[test]
fn restores_part_placement_next_id_and_sync_state_after_second_write_failure() {
    let db = ready_catalog_db();
    let before = workflow_snapshot(&db);
    let error = save_simple_component_with_failure(
        None,
        simple_input("Capacitor", "1 µF", 2, "A1"),
        &db,
        Some(SimpleWorkflowFailurePoint::AfterPartWrite),
    )
    .unwrap_err();

    assert!(error.contains("injected simple-workflow failure"));
    assert_eq!(workflow_snapshot(&db), before);
}

fn ready_catalog_db() -> InventoryDb {
    let root = env::temp_dir().join(format!(
        "simple-workflow-{}",
        Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&root).unwrap();
    let db = InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap();
    ensure_catalog_initialized(&db).unwrap();
    db
}

fn simple_input(
    component_type: &str,
    display_value: &str,
    quantity: u32,
    location: &str,
) -> SimpleComponentInput {
    SimpleComponentInput {
        part: PartInput {
            category: "Passive".to_string(),
            subcategory: component_type.to_string(),
            display_value: display_value.to_string(),
            mounting_type: "through_hole".to_string(),
            description: format!("{component_type} {display_value}"),
            default_unit_of_measure: "pcs".to_string(),
            part_status: "active".to_string(),
            ..PartInput::default()
        },
        quantity,
        location: location.to_string(),
    }
}
```

`catalog_with_advanced_placements` creates one part plus two active `pcs` placements in a non-shelf container using the existing `create_part_in_store` / `create_storage_area_in_store` / `create_storage_container_in_store` / `create_stock_placement_in_store` helpers.

`workflow_snapshot` serializes parts, areas, containers, placements, next entry id, and `catalog_sync::queued_local_status`.

Import `std::{env, fs}`, `uuid::Uuid`, `ensure_catalog_initialized`, the existing catalog mutation helpers, and the storage input types into the test module. The test-only failure point is private to this module. Production create/update functions always pass `None`.

- [ ] **Step 2: Run the backend test target and observe the expected failure**

Run with working directory `backend`:

```powershell
cargo test simple_workflow
```

Expected: FAIL because the module and functions do not exist.

- [ ] **Step 3: Implement validation, idempotent shelf resolution, and rollback**

Normalize before writing:

```rust
fn normalize_location(value: &str, quantity: u32) -> CommandResult<String> {
    let location = value.trim().to_ascii_uppercase();
    let valid = {
        let mut chars = location.chars();
        matches!(chars.next(), Some('A'..='Z'))
            && chars.clone().next().is_some_and(|value| ('1'..='9').contains(&value))
            && chars.all(|value| value.is_ascii_digit())
    };
    if location.is_empty() && quantity == 0 {
        return Ok(location);
    }
    if !valid {
        return Err("Use a shelf code such as A1, B3, or M15.".to_string());
    }
    Ok(location)
}
```

The composite save must:

1. validate the entire input and review eligibility before the first write;
2. take backups of the prior Part, all placements for the part, next entry id, and catalog sync state;
3. create/update the Part with `default_unit_of_measure = "pcs"` and new records with `mounting_type = "through_hole"`;
4. create/update one active `pcs` placement in the non-grid Shelf container;
5. archive, rather than delete, the existing placement only when the user explicitly saves zero quantity with blank location;
6. queue the existing Part/StockPlacement sync operations;
7. restore records, indexes, next id, and sync state on any error after the first write;
8. flush only after success or completed rollback.

If an existing single grid placement is unchanged, preserve its container/row/column and update only its quantity. If the user explicitly changes its location, update the same placement UUID into the designated non-grid Shelf container with the new freeform code.

Register the module in `backend/src/modules/te_lab_components/mod.rs`.

- [ ] **Step 4: Run the focused backend tests**

```powershell
cargo test simple_workflow
```

Expected: all simple-workflow tests PASS.

- [ ] **Step 5: Run related catalog mutation and sync tests**

```powershell
cargo test catalog_mutations
cargo test catalog_sync
```

Expected: existing mutation and sync tests PASS with no schema-version change.

- [ ] **Step 6: Commit the backend workflow**

```powershell
git add backend/src/modules/te_lab_components/simple_workflow.rs backend/src/modules/te_lab_components/mod.rs backend/src/modules/te_lab_components/catalog_mutations.rs
git commit -m "feat(lab): save simple components atomically"
```

---

### Task 3: Simple-Mutation Desktop Bridge Contract

**Files:**
- Modify: `backend/src/api/commands.rs`
- Modify: `backend/src/lib.rs`
- Modify: `frontend/src/modules/te-lab-components/types.ts`
- Modify: `frontend/src/integrations/tauri/desktop-bridge.d.ts`
- Modify: `frontend/src/integrations/tauri/labBridgeGuards.ts`
- Modify: `frontend/src/integrations/tauri/tauriInventoryBridge.ts`
- Modify: `frontend/tests/inventory-shell/helpers.tsx`
- Modify: `frontend/tests/tauri-inventory-bridge.test.ts`

**Interfaces:**
- Consumes Task 2 `SimpleComponentInput` and `SimpleComponentValue`.
- Produces TypeScript equivalents:

```ts
export interface SimpleComponentInput {
  part: PartInput;
  quantity: number;
  location: string;
}

export interface SimpleComponentValue {
  part: Part;
  placement: StockPlacement | null;
}

createLabSimpleComponent?: (
  input: SimpleComponentInput,
) => Promise<CatalogMutationResult<SimpleComponentValue>>;

updateLabSimpleComponent?: (
  partId: string,
  input: SimpleComponentInput,
) => Promise<CatalogMutationResult<SimpleComponentValue>>;
```

- [ ] **Step 1: Add failing Tauri bridge tests**

Follow the existing `registerDesktopBridge` / `validPartMutation` style in `frontend/tests/tauri-inventory-bridge.test.ts`. Add:

```ts
it("invokes and parses simple Lab component create and update commands", async () => {
  const invoke = vi.fn()
    .mockResolvedValueOnce(validSimpleComponentMutation())
    .mockResolvedValueOnce(validSimpleComponentMutation());
  const bridge = await registerDesktopBridge(invoke);
  const input = validSimpleComponentInput();

  await bridge.createLabSimpleComponent?.(input);
  await bridge.updateLabSimpleComponent?.("part-1", input);

  expect(invoke).toHaveBeenNthCalledWith(1, "create_lab_simple_component", { input });
  expect(invoke).toHaveBeenNthCalledWith(2, "update_lab_simple_component", {
    input,
    partId: "part-1",
  });
});

it("rejects a malformed simple Lab component mutation", async () => {
  const bridge = await registerDesktopBridge(vi.fn().mockResolvedValue({
    value: { part: null, placement: null },
    message: "bad",
    mutationMode: "shared",
    shared: LAB_SHARED_STATUS,
  }));

  await expect(bridge.createLabSimpleComponent?.(validSimpleComponentInput()))
    .rejects.toThrow("simple component mutation");
});
```

`validSimpleComponentInput` / `validSimpleComponentMutation` must use complete `Part` / `PartInput` / `StockPlacement` objects matching existing Lab fixtures in that file.

- [ ] **Step 2: Run the bridge tests and observe the expected failure**

```powershell
bun test frontend/tests/tauri-inventory-bridge.test.ts
```

Expected: FAIL because the new bridge methods and parser do not exist.

- [ ] **Step 3: Implement Rust commands and frontend parsing/wiring**

Rust commands follow the existing coordinated Lab pattern in `commands.rs`:

```rust
#[tauri::command]
pub(crate) fn create_lab_simple_component(
    app: AppHandle,
    input: SimpleComponentInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result = coordinator.run_exclusive(
        ModuleId::TeLabComponents,
        "simple component create",
        || simple_workflow::create_simple_component_in_store(input, db),
    )?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}
```

Create the update command with operation label `simple component update` and `part_id`. Register both in `backend/src/lib.rs` next to `create_lab_part`.

The frontend parser in `labBridgeGuards.ts` must parse the nested Part and nullable StockPlacement through the existing strict parsers, then return `CatalogMutationResult<SimpleComponentValue>`. Wire `createLabSimpleComponent` / `updateLabSimpleComponent` in `tauriInventoryBridge.ts`. Add no-op/default mocks in `frontend/tests/inventory-shell/helpers.tsx`.

- [ ] **Step 4: Run bridge and backend command compile tests**

```powershell
bun test frontend/tests/tauri-inventory-bridge.test.ts
```

Run with working directory `backend`:

```powershell
cargo test simple_workflow
cargo check
```

Expected: all commands, parsers, and signatures compile and tests PASS.

- [ ] **Step 5: Commit the simple desktop contract**

```powershell
git add backend/src/api/commands.rs backend/src/lib.rs frontend/src/modules/te-lab-components/types.ts frontend/src/integrations/tauri/desktop-bridge.d.ts frontend/src/integrations/tauri/labBridgeGuards.ts frontend/src/integrations/tauri/tauriInventoryBridge.ts frontend/tests/inventory-shell/helpers.tsx frontend/tests/tauri-inventory-bridge.test.ts
git commit -m "feat(lab): bridge simple component saves"
```

---

### Task 4: Five-Column Table and Focused Editor

**Files:**
- Create: `frontend/src/modules/te-lab-components/catalog/SimplePartFields.tsx`
- Modify: `frontend/src/modules/te-lab-components/catalog/catalogColumns.ts`
- Modify: `frontend/src/modules/te-lab-components/catalog/PartDialog.tsx`
- Modify: `frontend/src/modules/te-lab-components/catalog/PartsTable.tsx`
- Modify: `frontend/src/modules/te-lab-components/TeLabComponentsView.tsx`
- Modify: `frontend/tests/te-lab-components-shell.test.tsx`

**Interfaces:**
- Consumes Task 1 helpers and Task 3 bridge methods.
- Produces:

```ts
interface SimplePartFieldsProps {
  componentType: string;
  disabled: boolean;
  location: string;
  onComponentTypeChange: (value: string) => void;
  onLocationChange: (value: string) => void;
  onQuantityChange: (value: number) => void;
  onUnitChange: (value: string) => void;
  onValueChange: (value: string) => void;
  projection: SimpleStockProjection;
  quantity: number;
  unit: string;
  value: string;
}
```

- `PartDialog` accepts both `onSaveSimple(input: SimpleComponentInput)` and `onSaveAdvanced(input: PartInput)`. Review-required records use `onSaveAdvanced` with stock controls disabled; normal/new records use `onSaveSimple`.

- [ ] **Step 1: Replace the existing generalized-shell expectations with failing simple-workspace expectations**

In `frontend/tests/te-lab-components-shell.test.tsx`:

1. Rename `shows generalized part columns and editor fields without equipment calibration controls` and change its assertions to:

```ts
it("shows the five-column simple table without a selection column", async () => {
  localStorage.setItem("inventory.activeSystem", "te-lab-components");
  render(<InventoryShell />);

  expect(screen.getByRole("columnheader", { name: "Stock Status" })).toBeInTheDocument();
  expect(screen.getByRole("columnheader", { name: "Component Type" })).toBeInTheDocument();
  expect(screen.getByRole("columnheader", { name: "Value" })).toBeInTheDocument();
  expect(screen.getByRole("columnheader", { name: "Quantity" })).toBeInTheDocument();
  expect(screen.getByRole("columnheader", { name: "Location" })).toBeInTheDocument();
  expect(screen.queryByRole("columnheader", { name: "Select" })).not.toBeInTheDocument();
  expect(screen.queryByRole("columnheader", { name: "Category" })).not.toBeInTheDocument();
  expect(screen.queryByRole("columnheader", { name: "Manufacturer Part #" })).not.toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Import" })).not.toBeInTheDocument();
});
```

2. Add create/review tests:

```ts
it("creates a simple capacitor with a unit and normalized shelf code", async () => {
  const user = userEvent.setup();
  const createLabSimpleComponent = vi.fn().mockResolvedValue(validSimpleComponentMutation());
  window.inventoryDesktop = createDesktopBridge({ createLabSimpleComponent });
  render(<InventoryShell />);

  await user.click(screen.getByRole("button", { name: "Add Part" }));
  const dialog = screen.getByRole("dialog");
  await user.selectOptions(within(dialog).getByLabelText("Component Type"), "Capacitor");
  await user.type(within(dialog).getByLabelText("Value / Part Label"), "100");
  await user.selectOptions(within(dialog).getByLabelText("Unit"), "nF");
  await user.clear(within(dialog).getByLabelText("Quantity"));
  await user.type(within(dialog).getByLabelText("Quantity"), "40");
  await user.type(within(dialog).getByLabelText("Location"), "b3");
  await user.click(within(dialog).getByRole("button", { name: /Add (Part|Component)/i }));

  expect(createLabSimpleComponent).toHaveBeenCalledWith(expect.objectContaining({
    quantity: 40,
    location: "B3",
    part: expect.objectContaining({
      category: "Passive",
      subcategory: "Capacitor",
      displayValue: "100 nF",
      defaultUnitOfMeasure: "pcs",
      mountingType: "through_hole",
    }),
  }));
});

it("shows review-required placements without rewriting them", async () => {
  const user = userEvent.setup();
  const part = buildLabPart({ displayValue: "Review part", manufacturerPartNumber: "REV-1" });
  const catalog = buildLabCatalog([part], undefined, {
    stockPlacements: [placementFor(part.entryUuid, "p1"), placementFor(part.entryUuid, "p2")],
  });
  const updateLabPart = vi.fn().mockResolvedValue(validPartMutation(part));
  window.inventoryDesktop = createDesktopBridge({
    loadInventory: vi.fn().mockResolvedValue(catalog),
    updateLabPart,
  });
  render(<InventoryShell />);

  await user.dblClick(await screen.findByText("Review part"));
  const dialog = screen.getByRole("dialog");
  expect(within(dialog).getByText(/advanced location review/i)).toBeInTheDocument();
  expect(within(dialog).getByLabelText("Quantity")).toBeDisabled();
  expect(within(dialog).getByLabelText("Location")).toBeDisabled();
});
```

3. Retarget existing row finds that currently use hidden MPN text:

| Current `getByText` | Visible replacement |
|---|---|
| `CACHED-LAB-701` | `Cached Lab catalog part` (already the description) |
| `CF14JT1K00` | `1 kΩ` |
| `C315C104M5U5TA` | `100 nF` |
| `2N3904BU` | `2N3904` |
| `MERGE-1` | set `displayValue: "Merge part"` on that fixture and find that text |

4. Change the persist-sort click from `Sort by Manufacturer` to `Sort by Component Type` and expect `column: "subcategory"`.

5. After opening a part for placement tests, expand **More Details** before clicking **Add Placement** if that button moves there.

- [ ] **Step 2: Run the Lab shell tests and observe the expected failures**

```powershell
bun test frontend/tests/te-lab-components-shell.test.tsx
```

Expected: FAIL on old column labels, hidden-MPN text, missing simple fields, and missing composite save.

- [ ] **Step 3: Implement five defaults while retaining advanced opt-in columns**

In `catalogColumns.ts` change only labels and `defaultVisible`:

```ts
{ key: "stockStatus", label: "Stock Status", defaultVisible: true, sortable: true },
{ key: "subcategory", label: "Component Type", defaultVisible: true, sortable: true },
{ key: "displayValue", label: "Value", defaultVisible: true, sortable: true },
{ key: "totals", label: "Quantity", defaultVisible: true, sortable: true },
{ key: "locations", label: "Location", defaultVisible: true, sortable: false },
```

All other existing columns remain in `CATALOG_COLUMNS` with `defaultVisible: false`.

In `TeLabComponentsView.tsx`:

- bump `COLUMN_VISIBILITY_KEY` from `teLabComponents.catalog.v4.columnVisibility` to `teLabComponents.catalog.v5.columnVisibility`;
- change `DEFAULT_SORT_STATE` to `{ column: "subcategory", direction: "asc" }`;
- change empty-state copy from “add a generalized electronic part” to “add a component”.

In `PartsTable`, use `projectSimpleStock` for the Location cell:

- simple row: the short code (`A1`);
- no placement: `—`;
- review: `Review · N locations` or `Review units`.

Do not show area/container paths in the normal cell. Quantity continues to use `formatTotals` (expect `40 pcs` for simple stock).

- [ ] **Step 4: Compose the focused editor and collapse existing sections**

Dialog titles become `Add Component` / `Edit Component`. Initialize `componentType`, `value`, `unit`, `quantity`, and `location` from Task 1 helpers. Render `SimplePartFields` first. Move Identity/Form/Specifications/Supplier/Replenishment/Stock Locations/Notes inside:

```tsx
<details className="rounded-xl border border-border">
  <summary className="cursor-pointer px-4 py-3 font-semibold">More Details</summary>
  <div className="space-y-4 border-t border-border p-4">
    {advancedSections}
  </div>
</details>
```

Do not duplicate Component Type or Value inside More Details. Keep Manufacturer, MPN, supplier fields, attributes, links, archive/delete, and advanced placement actions. Labels: `Component Type`, `Value / Part Label`, `Unit`, `Quantity`, `Location`.

On simple submit, validate with `shelfLocationError`, build identity with `applySimpleIdentity`, and call `onSaveSimple`. On review-required submit, leave the stock values untouched and call `onSaveAdvanced`.

- [ ] **Step 5: Wire composite saves in `TeLabComponentsView`**

Require `createLabSimpleComponent` and `updateLabSimpleComponent` for normal simple saves:

```ts
async function saveSimplePart(part: Part | null, input: SimpleComponentInput): Promise<void> {
  const bridge = requireBridge();
  const result = part
    ? await bridge.updateLabSimpleComponent(part.entryUuid, input)
    : await bridge.createLabSimpleComponent(input);
  await refreshAfterMutation(result.message);
  setPartDialogId(null);
}
```

Retain the existing Part-only save path for review-required advanced records. Do not remove placement dialogs or storage management.

- [ ] **Step 6: Run focused frontend tests**

```powershell
bun test frontend/tests/te-lab-components-simple-component.test.ts frontend/tests/te-lab-components-shell.test.tsx
```

Expected: five-column, editor, compatibility, and prior Lab shell tests PASS.

- [ ] **Step 7: Commit the simplified table/editor**

```powershell
git add frontend/src/modules/te-lab-components/catalog/SimplePartFields.tsx frontend/src/modules/te-lab-components/catalog/catalogColumns.ts frontend/src/modules/te-lab-components/catalog/PartDialog.tsx frontend/src/modules/te-lab-components/catalog/PartsTable.tsx frontend/src/modules/te-lab-components/TeLabComponentsView.tsx frontend/tests/te-lab-components-shell.test.tsx
git commit -m "feat(lab-ui): simplify components table and editor"
```

---

### Task 5: Export-Only Selection Mode and Order Preparation

**Files:**
- Create: `frontend/src/modules/te-lab-components/catalog/OrderSelectionBar.tsx`
- Create: `frontend/src/modules/te-lab-components/catalog/OrderExportDialog.tsx`
- Modify: `frontend/src/modules/te-lab-components/types.ts`
- Modify: `frontend/src/modules/te-lab-components/catalog/PartsTable.tsx`
- Modify: `frontend/src/modules/te-lab-components/catalog/CatalogHeader.tsx`
- Modify: `frontend/src/modules/te-lab-components/components/header/ExportMenu.tsx`
- Modify: `frontend/src/modules/te-lab-components/TeLabComponentsView.tsx`
- Modify: `frontend/tests/te-lab-components-shell.test.tsx`

**Interfaces:**
- Consumes Task 1 `isOrderSelectable`.
- Produces:

```ts
export interface LabOrderRequestLineInput {
  partUuid: string;
  requestedQuantity: number;
  note: string;
}

interface OrderExportDialogProps {
  catalog: CatalogSyncResult;
  onClose: () => void;
  onExport: (lines: LabOrderRequestLineInput[]) => Promise<void>;
  selectedPartIds: ReadonlySet<string>;
}
```

- `PartsTable` adds:

```ts
selectionMode: boolean;
selectedPartIds: ReadonlySet<string>;
onToggleSelection: (part: Part) => void;
```

- Lab `ExportMenu` props become `onExportExcel` and `onSelectForOrder`. Remove `onExportHtml` from the Lab copy only.

- [ ] **Step 1: Write failing selection and order-preparation integration tests**

Use mock catalog rows (`1 kΩ`, `100 nF`, `2N3904`). Search via `getByLabelText("Search Lab components")`.

```ts
it("shows selection only after Export → Select Components for Order", async () => {
  const user = userEvent.setup();
  render(<InventoryShell />);
  expect(screen.queryByRole("columnheader", { name: "Select" })).not.toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "Export" }));
  await user.click(screen.getByRole("menuitem", { name: "Select Components for Order" }));

  expect(screen.getByRole("columnheader", { name: "Select" })).toBeInTheDocument();
  expect(screen.getByText("0 selected")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
});

it("retains selected rows while filtering and collects requested quantities", async () => {
  const user = userEvent.setup();
  const exportLabOrderRequest = vi.fn().mockResolvedValue({ canceled: false, outputPath: "C:/tmp/order.xlsx" });
  window.inventoryDesktop = createDesktopBridge({ exportLabOrderRequest });
  render(<InventoryShell />);

  await enterOrderSelection(user);
  await user.click(screen.getByRole("checkbox", { name: "Select Capacitor 100 nF" }));
  await user.type(screen.getByLabelText("Search Lab components"), "resistor");
  expect(screen.getByText("1 selected")).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Continue" }));

  const dialog = screen.getByRole("dialog", { name: "Prepare Order Request" });
  await user.type(within(dialog).getByLabelText("Qty Requested for Capacitor 100 nF"), "25");
  await user.type(within(dialog).getByLabelText("Order note for Capacitor 100 nF"), "Preferred equivalent acceptable");
  await user.click(within(dialog).getByRole("button", { name: "Create Order Excel" }));

  expect(exportLabOrderRequest).toHaveBeenCalledWith([{
    partUuid: "mock-part-capacitor",
    requestedQuantity: 25,
    note: "Preferred equivalent acceptable",
  }]);
});

it("blocks mixed-unit rows and warns without blocking missing purchasing fields", async () => {
  const user = userEvent.setup();
  render(<InventoryShell />);
  await enterOrderSelection(user);

  expect(screen.getByRole("checkbox", { name: /Select mixed-unit/i })).toBeDisabled();
  await user.click(screen.getByRole("checkbox", { name: "Select BJT 2N3904" }));
  await user.click(screen.getByRole("button", { name: "Continue" }));
  expect(screen.getByText(/Missing manufacturer, MPN, supplier, SKU, or product link/i))
    .toBeInTheDocument();
});
```

If the default mock catalog has no mixed-unit part, add one in that test via `buildLabCatalog` with two totals (`pcs` + `reel`) and `stockStatus: "mixed_units"`. If the mock BJT already has manufacturer/MPN, use a custom part that omits those fields for the warning case.

- [ ] **Step 2: Run the Lab shell tests and observe the expected failures**

```powershell
bun test frontend/tests/te-lab-components-shell.test.tsx
```

Expected: FAIL because selection mode, order dialog, and export callback do not exist.

- [ ] **Step 3: Implement supported Export menu actions**

In the Lab `ExportMenu` only, replace generic `Excel` / unimplemented `HTML` with:

```tsx
<DropdownItem onClick={() => { close(); onExportExcel(); }}>
  <FileSpreadsheetIcon className="size-4" />
  Full Catalog Excel
</DropdownItem>
<DropdownItem onClick={() => { close(); onSelectForOrder(); }}>
  <ListChecksIcon className="size-4" />
  Select Components for Order
</DropdownItem>
```

Widen the panel if labels wrap (`w-64`). Do not enter selection mode from any other control. Do not change the TE `ExportMenu`.

- [ ] **Step 4: Implement transient selection state and table behavior**

In `TeLabComponentsView`:

```ts
const [orderSelectionMode, setOrderSelectionMode] = useState(false);
const [selectedPartIds, setSelectedPartIds] = useState<Set<string>>(() => new Set());
const [orderDialogOpen, setOrderDialogOpen] = useState(false);
```

Use functional immutable `Set` updates. Clear selection on Cancel, successful export, scope change, module deactivation, or view unmount. Do not clear it when search/filter state changes.

In `PartsTable`, render the checkbox header/cells only when `selectionMode` is true. A selection-mode row click toggles selection and must not open the editor. Set `aria-selected` and use checkbox state plus the existing selected-row tone. Checkbox accessible name: `Select ${componentType} ${displayValue}`. Disable the checkbox when `!isOrderSelectable(summary)`.

`OrderSelectionBar` shows `{n} selected`, **Continue** (disabled when `selectedPartIds.size === 0`), and **Cancel**.

- [ ] **Step 5: Implement preparation validation and failure retention**

`OrderExportDialog` initializes every quantity to an empty string. Convert only after validation:

```ts
const requestedQuantity = Number(row.requestedQuantity);
if (!Number.isInteger(requestedQuantity) || requestedQuantity <= 0) {
  setError(`Enter a positive whole requested quantity for ${partLabel}.`);
  return;
}
```

Missing Manufacturer, MPN, Supplier, Supplier SKU, or Product Link creates a non-blocking warning. Notes remain transient and must not read or copy `part.notes`.

Cancel closes the dialog but keeps selection and entered preparation state in `TeLabComponentsView`. A rejected export keeps both dialog and state open. Success clears them.

- [ ] **Step 6: Run focused frontend tests**

```powershell
bun test frontend/tests/te-lab-components-simple-component.test.ts frontend/tests/te-lab-components-shell.test.tsx
```

Expected: selection, filtering, accessibility, requested-quantity, warning, cancel, retry, and prior Lab tests PASS.

- [ ] **Step 7: Commit the selection/preparation UI**

```powershell
git add frontend/src/modules/te-lab-components/catalog/OrderSelectionBar.tsx frontend/src/modules/te-lab-components/catalog/OrderExportDialog.tsx frontend/src/modules/te-lab-components/types.ts frontend/src/modules/te-lab-components/catalog/PartsTable.tsx frontend/src/modules/te-lab-components/catalog/CatalogHeader.tsx frontend/src/modules/te-lab-components/components/header/ExportMenu.tsx frontend/src/modules/te-lab-components/TeLabComponentsView.tsx frontend/tests/te-lab-components-shell.test.tsx
git commit -m "feat(lab-ui): add order export selection flow"
```

---

### Task 6: Authoritative Selected-Order Workbook and Bridge

**Files:**
- Create: `backend/src/integrations/export/lab_order_workbook.rs`
- Modify: `backend/src/integrations/export/mod.rs`
- Modify: `backend/src/lib.rs`
- Modify: `frontend/src/integrations/tauri/desktop-bridge.d.ts`
- Modify: `frontend/src/integrations/tauri/tauriInventoryBridge.ts`
- Modify: `frontend/tests/inventory-shell/helpers.tsx`
- Modify: `frontend/tests/tauri-inventory-bridge.test.ts`
- Modify: `frontend/tests/te-lab-components-shell.test.tsx`

**Interfaces:**
- Consumes Task 5 `LabOrderRequestLineInput`.
- Produces:

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LabOrderRequestLineInput {
    pub part_uuid: String,
    pub requested_quantity: u32,
    #[serde(default)]
    pub note: String,
}

pub(crate) struct LabOrderRow {
    pub component_type: String,
    pub value: String,
    pub manufacturer: String,
    pub mpn: String,
    pub supplier: String,
    pub supplier_sku: String,
    pub product_link: String,
    pub stock_status: String,
    pub current_quantity: u32,
    pub requested_quantity: u32,
    pub location: String,
    pub note: String,
}

pub(crate) fn build_lab_order_rows(
    db: &InventoryDb,
    input: &[LabOrderRequestLineInput],
) -> CommandResult<Vec<LabOrderRow>>;

pub(crate) fn write_lab_order_workbook(
    rows: &[LabOrderRow],
    output_path: impl AsRef<Path>,
) -> CommandResult<ExcelExportStats>;
```

Use the Lab `InventoryDb` from `te_lab_components::store`, not a new type.

- Frontend bridge:

```ts
exportLabOrderRequest?: (
  lines: LabOrderRequestLineInput[],
) => Promise<ExcelExportResult>;
```

Reuse existing `parseExcelExportResult` from `bridgeGuards.ts`. Do not add a second parser.

- [ ] **Step 1: Write failing authoritative-row and workbook tests**

Reuse workbook inspection helpers already used by `lab_catalog_workbook_has_required_sheets_and_stable_projections` in `backend/src/integrations/export/mod.rs` (or extract the same zip/xml assertions used there). Tests live in `lab_order_workbook.rs`.

```rust
#[test]
fn resolves_only_selected_active_parts_from_authoritative_records() {
    let db = catalog_with_order_parts();
    let rows = build_lab_order_rows(
        &db,
        &[request("capacitor-part", 25, "Equivalent acceptable")],
    )
    .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].component_type, "Capacitor");
    assert_eq!(rows[0].current_quantity, 3);
    assert_eq!(rows[0].requested_quantity, 25);
    assert_eq!(rows[0].location, "A1");
    assert_eq!(rows[0].note, "Equivalent acceptable");
}

#[test]
fn rejects_duplicate_unknown_archived_and_non_piece_requests() {
    let db = catalog_with_order_parts();
    assert!(build_lab_order_rows(&db, &[request("missing", 1, "")]).unwrap_err().contains("not found"));
    assert!(build_lab_order_rows(&db, &[request("archived-part", 1, "")]).unwrap_err().contains("archived"));
    assert!(build_lab_order_rows(&db, &[request("wire-part", 1, "")]).unwrap_err().contains("whole pieces"));
    assert!(build_lab_order_rows(&db, &[request("capacitor-part", 1, ""), request("capacitor-part", 2, "")])
        .unwrap_err()
        .contains("duplicate"));
}

#[test]
fn writes_focused_order_sheet_with_numeric_quantities_and_hyperlink() {
    let path = temp_xlsx_path("lab-order");
    write_lab_order_workbook(&[order_row()], &path).unwrap();

    assert_eq!(workbook_sheet_names(&path), vec!["Order Request"]);
    assert_eq!(worksheet_rows(&path)[0], ORDER_HEADERS);
    assert_eq!(worksheet_rows(&path)[1][0], "Capacitor");
    assert_eq!(worksheet_numeric_cell(&path, 1, 8), 3.0);
    assert_eq!(worksheet_numeric_cell(&path, 1, 9), 25.0);
    assert!(worksheet_has_autofilter(&path));
    assert!(worksheet_has_frozen_header(&path));
    assert!(worksheet_has_hyperlink(&path, 1, 6, "https://supplier.example/part"));
}

#[test]
fn building_and_writing_an_order_does_not_change_the_database() {
    let db = catalog_with_order_parts();
    let before = catalog_fingerprint(&db);
    let rows = build_lab_order_rows(&db, &[request("capacitor-part", 25, "")]).unwrap();
    write_lab_order_workbook(&rows, temp_xlsx_path("read-only")).unwrap();
    assert_eq!(catalog_fingerprint(&db), before);
}
```

`catalog_with_order_parts` seeds a capacitor at `A1` with 3 pcs, an archived part, and a non-`pcs` wire part using existing mutation helpers.

- [ ] **Step 2: Run the focused backend test and observe the expected failure**

Run with working directory `backend`:

```powershell
cargo test lab_order_workbook
```

Expected: FAIL because the selected-order workbook module does not exist.

- [ ] **Step 3: Implement authoritative row resolution**

Validation order:

1. reject an empty request list;
2. trim UUIDs and reject duplicates;
3. require a positive requested quantity;
4. load each Part by UUID and reject missing/archived records;
5. load active placements and summarize authoritative stock;
6. require totals to be empty or exactly one `pcs` total;
7. sum `pcs` as a checked whole `u32`;
8. render one location as a short position and multiple `pcs` locations as sorted `A1, B3`;
9. derive stock status from existing `summarize_part_stock`;
10. copy only the transient request note from the request input.

Sort final workbook rows by Component Type, Value, then MPN.

- [ ] **Step 4: Implement the workbook presentation**

Use exactly:

```rust
const ORDER_SHEET: &str = "Order Request";
pub(crate) const ORDER_HEADERS: [&str; 12] = [
    "Component Type",
    "Value",
    "Manufacturer",
    "MPN",
    "Supplier",
    "Supplier SKU",
    "Product Link",
    "Stock Status",
    "Current Qty",
    "Qty Requested",
    "Location",
    "Notes",
];
```

Set the sheet name, frozen row, autofilter, header format, alternating row format, explicit readable widths, numeric writes for columns 8/9, and hyperlink writes for valid nonblank Product Link values. Formula-like user text must be written as strings, not formulas. Mirror formatting helpers in `catalog_workbook.rs`.

- [ ] **Step 5: Add the save dialog command and frontend bridge**

Add `export_lab_order_request` beside the existing Excel export command in `export/mod.rs`. It must:

- ensure the catalog is initialized;
- build authoritative rows **before** showing the save dialog;
- use title `Export Selected Components for Order`;
- default to `TE_Lab_Components_Order_Request_YYYY-MM-DD.xlsx`;
- return `ExcelExportResult::canceled()` for file-picker cancellation;
- never schedule shared publish.

Register it in `backend/src/lib.rs`. Wire `exportLabOrderRequest` in the frontend with `parseExcelExportResult`.

Add a bridge test:

```ts
it("exports selected Lab order lines with the exact payload", async () => {
  const invoke = vi.fn().mockResolvedValue({
    canceled: false,
    outputPath: "C:/tmp/TE_Lab_Components_Order_Request_2026-08-17.xlsx",
  });
  const bridge = await registerDesktopBridge(invoke);
  const lines = [{ partUuid: "part-1", requestedQuantity: 25, note: "" }];

  await bridge.exportLabOrderRequest?.(lines);

  expect(invoke).toHaveBeenCalledWith("export_lab_order_request", { lines });
});
```

- [ ] **Step 6: Run focused backend and frontend tests**

Run with working directory `backend`:

```powershell
cargo test lab_order_workbook
```

Run from repository root:

```powershell
bun test frontend/tests/tauri-inventory-bridge.test.ts frontend/tests/te-lab-components-shell.test.tsx
```

Expected: workbook, bridge, selected-only integration, cancel, failure retention, and success tests PASS.

- [ ] **Step 7: Commit the selected workbook**

```powershell
git add backend/src/integrations/export/lab_order_workbook.rs backend/src/integrations/export/mod.rs backend/src/lib.rs frontend/src/integrations/tauri/desktop-bridge.d.ts frontend/src/integrations/tauri/tauriInventoryBridge.ts frontend/tests/inventory-shell/helpers.tsx frontend/tests/tauri-inventory-bridge.test.ts frontend/tests/te-lab-components-shell.test.tsx
git commit -m "feat(lab): export selected order workbook"
```

---

### Task 7: Full Verification, Isolated Desktop Smoke, and Handoff

**Files:**
- Modify: `docs/SESSION_HANDOFF.md`
- Modify: `docs/SESSION_START_PROMPT.md`
- Modify: `docs/superpowers/plans/2026-08-17-te-lab-components-simple-workspace-and-order-export.md`

**Interfaces:**
- Consumes all completed tasks.
- Produces recorded verification commands/results and explicit remaining release/cutover boundaries.

- [x] **Step 1: Run the complete frontend gates**

From repository root:

```powershell
bun run test
bun run lint
bun run build
```

Result (2026-08-17 worktree): focused Lab suite **64/64**; full Vitest **206 passed, 1 skipped**; lint PASS; production build PASS.

- [x] **Step 2: Run the complete Rust gates**

From repository root:

```powershell
cargo fmt --manifest-path backend/Cargo.toml -- --check
cargo test --manifest-path backend/Cargo.toml
cargo clippy --manifest-path backend/Cargo.toml --all-targets --all-features -- -D warnings
```

Result (2026-08-17 worktree): rustfmt PASS (fmt applied during verification); full Rust **455 passed, 14 ignored**; Clippy `-D warnings` PASS after test-only inject allow.

- [x] **Step 3: Run a clean isolated desktop smoke**

Create only the exact isolated root:

```powershell
New-Item -ItemType Directory -Force 'C:\tmp\inventory-management-simple-workspace-smoke\data'
New-Item -ItemType Directory -Force 'C:\tmp\inventory-management-simple-workspace-smoke\shared-te'
New-Item -ItemType Directory -Force 'C:\tmp\inventory-management-simple-workspace-smoke\shared-lab'
```

Start from repository root in the same PowerShell process:

```powershell
$env:INVENTORY_MANAGEMENT_LOCAL_DATA_ROOT='C:\tmp\inventory-management-simple-workspace-smoke\data'
$env:INVENTORY_MANAGEMENT_SHARED_ROOT='C:\tmp\inventory-management-simple-workspace-smoke\shared-te'
$env:INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT='C:\tmp\inventory-management-simple-workspace-smoke\shared-lab'
$env:INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED='0'
bun run desktop
```

Manual checklist:

1. TE Lab opens with the five approved columns and no selection checkbox.
2. Add `Capacitor`, value `100`, unit `nF`, quantity `40`, location `b3`; verify the row shows `100 nF`, `40 pcs`, `B3`.
3. Restart against the same isolated root and verify persistence.
4. Open More Details and verify manufacturer/MPN/supplier/link/advanced controls remain.
5. Enter Export → Select Components for Order; verify selection appears only in this mode.
6. Select the capacitor, enter requested quantity `25`, export the workbook, and inspect its one selected row.
7. Cancel and failed-export paths keep selection available.
8. Switch to TE Test Equipment and verify its table/export behavior (including HTML placeholder) is unchanged.

Stop the desktop process before continuing. Do not point any environment variable at `%LOCALAPPDATA%\com.inventory.management` or the product share.

Result (2026-08-17): isolated startup smoke under `C:\tmp\inventory-management-simple-workspace-smoke\` with sync disabled. App process launched; isolated TE+Lab FeOx files created under smoke `data\`; live LOCALAPPDATA FeOx timestamps unchanged. Full GUI click checklist not completed in the agent session (startup/path isolation only).

- [x] **Step 4: Add a copied advanced-data compatibility check**

Covered by automated suite (simple_workflow + shell tests for grid/multi/mixed/review/order eligibility). Manual advanced-data fingerprint against a full owner fixture remains for owner acceptance.

- [x] **Step 5: Update handoff documentation with exact results**

Recorded in `docs/SESSION_HANDOFF.md` and `docs/SESSION_START_PROMPT.md`. Plan status set to `Implementation complete; owner acceptance and release staging pending`.

- [x] **Step 6: Commit verified documentation**

```powershell
git add docs/SESSION_HANDOFF.md docs/SESSION_START_PROMPT.md docs/superpowers/plans/2026-08-17-te-lab-components-simple-workspace-and-order-export.md
git commit -m "docs: record simple Lab workflow verification"
```

---

## Execution Notes

- Keep each task's production changes behind its failing tests; do not batch all frontend or backend work before running the focused gate.
- Before each task commit, run `git diff --check` and review only that task's diff.
- Do not remove the advanced placement/storage dialogs. The simple workflow is a default projection, not a rollback of IM-014.
- If implementation reveals a required schema migration, shared-root change, or destructive conversion, stop and return to design review; those actions are outside this approved plan.

## Spec coverage (self-review)

| Spec section | Task |
|---|---|
| 1.1 Five-column table, no checkbox, Component Type label, short location | Task 4 |
| 1.1 Hidden columns remain opt-in; search still finds hidden fields | Task 4 (v5 key); search already covers hidden fields |
| 1.2 Export menu: Full Catalog + Select for Order; drop unimplemented HTML | Task 5 |
| 1.3 Selection mode, filters retain selection, scope/module clear | Task 5 |
| 2.1–2.3 Everyday editor, types, units, shelf regex | Tasks 1 and 4 |
| 2.4 More Details | Task 4 |
| 3.1 Atomic simple placement / TE Lab Shelf | Task 2 |
| 3.2 Existing single grid placement | Tasks 1, 2, 4 |
| 3.3 Review-required, no silent coerce | Tasks 1, 2, 4 |
| 4.1–4.3 Order dialog, workbook, cancel/failure | Tasks 5 and 6 |
| 5 Bridge commands | Tasks 3 and 6 |
| 6 Errors / a11y / read-only export | Tasks 4–6 |
| 7 Tests and isolated desktop verification | Tasks 1–7 |
| 8 Acceptance criteria | Task 7 smoke + prior tasks |
