import { placementPosition } from "@/modules/te-lab-components/catalog/catalogUtils";
import type {
  Part,
  PartInput,
  PartStockSummary,
  StockPlacement,
  StorageContainer,
} from "@/modules/te-lab-components/types";

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

type ComponentType = keyof typeof COMPONENT_PROFILES;

const SIMPLE_LOCATION = /^[A-Z][1-9][0-9]*$/;
/** Existing grid labels may use multi-letter columns (e.g. AA1). */
const DISPLAY_LOCATION = /^[A-Z]+[1-9][0-9]*$/;

const RECOGNIZED_UNITS = Array.from(
  new Set(
    Object.values(COMPONENT_PROFILES).flatMap((profile) => [...profile.units]),
  ),
).sort((a, b) => b.length - a.length);

export const COMPONENT_TYPE_OPTIONS: readonly string[] = Object.keys(COMPONENT_PROFILES);

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

function isComponentType(value: string): value is ComponentType {
  return Object.prototype.hasOwnProperty.call(COMPONENT_PROFILES, value);
}

export function categoryForComponentType(componentType: string): string | null {
  if (!isComponentType(componentType)) {
    return null;
  }
  return COMPONENT_PROFILES[componentType].category;
}

export function engineeringUnitsFor(componentType: string): readonly string[] {
  if (!isComponentType(componentType)) {
    return [];
  }
  return COMPONENT_PROFILES[componentType].units;
}

export function formatComponentValue(
  _componentType: string,
  value: string,
  unit: string,
): string {
  if (unit.trim() !== "") {
    return `${value} ${unit}`;
  }
  return value;
}

export function readComponentValue(part: Part): { value: string; unit: string } {
  const attributeValue = part.attributes.value;
  if (attributeValue) {
    return { value: attributeValue.value, unit: attributeValue.unit };
  }

  const displayValue = part.displayValue.trim();
  for (const unit of RECOGNIZED_UNITS) {
    const suffix = ` ${unit}`;
    if (displayValue.endsWith(suffix)) {
      return {
        value: displayValue.slice(0, -suffix.length).trimEnd(),
        unit,
      };
    }
    if (displayValue.endsWith(unit) && displayValue.length > unit.length) {
      const withoutUnit = displayValue.slice(0, -unit.length).trimEnd();
      if (withoutUnit.length > 0 && /[\d.]$/.test(withoutUnit)) {
        return { value: withoutUnit, unit };
      }
    }
  }

  return { value: part.displayValue, unit: "" };
}

export function normalizeShelfLocation(value: string): string {
  return value.trim().toUpperCase();
}

export function shelfLocationError(value: string, quantity: number): string | null {
  const normalized = normalizeShelfLocation(value);
  if (normalized === "") {
    if (quantity > 0) {
      return "Location is required when quantity is greater than zero.";
    }
    return null;
  }
  if (!SIMPLE_LOCATION.test(normalized)) {
    return "Use a shelf code such as A1, B3, or M15.";
  }
  return null;
}

function isBlankPlacementPosition(
  placement: StockPlacement,
  container?: StorageContainer,
): boolean {
  if (
    container?.gridEnabled &&
    placement.rowIndex !== null &&
    placement.columnIndex !== null
  ) {
    return false;
  }
  return placement.freeformPosition.trim() === "";
}

function resolvePlacementLocation(
  placement: StockPlacement,
  container?: StorageContainer,
): string {
  if (
    container?.gridEnabled &&
    placement.rowIndex !== null &&
    placement.columnIndex !== null
  ) {
    return placementPosition(placement, container);
  }
  const freeform = placement.freeformPosition.trim();
  if (freeform === "") {
    return "";
  }
  return normalizeShelfLocation(freeform);
}

function isValidDisplayLocation(location: string): boolean {
  return SIMPLE_LOCATION.test(location) || DISPLAY_LOCATION.test(location);
}

function reviewProjection(
  placements: StockPlacement[],
  containersById: ReadonlyMap<string, StorageContainer>,
  reasons: SimpleReviewReason[],
): SimpleStockProjection {
  const locations = placements.map((item) => {
    const container = containersById.get(item.containerUuid);
    const location = resolvePlacementLocation(item, container);
    return location || placementPosition(item, container);
  });
  const uniqueLocations = Array.from(new Set(locations));
  const units = Array.from(new Set(placements.map((item) => item.unitOfMeasure)));
  const quantityLabel =
    units.length === 1
      ? `${placements.reduce((sum, item) => sum + item.quantity, 0)} ${units[0]}`
      : "Mixed units";

  return {
    kind: "review",
    locationLabel:
      uniqueLocations.length === 1 ? uniqueLocations[0]! : "Multiple locations",
    quantityLabel,
    reasons,
  };
}

export function projectSimpleStock(
  placements: StockPlacement[],
  containersById: ReadonlyMap<string, StorageContainer>,
): SimpleStockProjection {
  const active = placements.filter((item) => !item.archived);

  if (active.length === 0) {
    return {
      kind: "simple",
      location: "",
      placementUuid: null,
      quantity: 0,
    };
  }

  if (active.length === 1) {
    const only = active[0]!;
    const container = containersById.get(only.containerUuid);
    if (only.quantity === 0 && isBlankPlacementPosition(only, container)) {
      return {
        kind: "simple",
        location: "",
        placementUuid: null,
        quantity: 0,
      };
    }
  }

  if (active.length > 1) {
    const reasons: SimpleReviewReason[] = ["multiple_placements"];
    const units = new Set(active.map((item) => item.unitOfMeasure));
    if (units.size > 1) {
      reasons.push("mixed_units");
    }
    return reviewProjection(active, containersById, reasons);
  }

  const only = active[0]!;
  const container = containersById.get(only.containerUuid);
  const location = resolvePlacementLocation(only, container);

  if (only.unitOfMeasure !== "pcs") {
    return reviewProjection(active, containersById, ["non_piece_unit"]);
  }

  if (!isValidDisplayLocation(location)) {
    return reviewProjection(active, containersById, ["invalid_location"]);
  }

  return {
    kind: "simple",
    location,
    placementUuid: only.placementUuid,
    quantity: only.quantity,
  };
}

export function isOrderSelectable(summary: PartStockSummary | undefined): boolean {
  if (!summary) {
    return false;
  }
  if (summary.totals.length === 0) {
    return true;
  }
  return summary.totals.length === 1 && summary.totals[0]?.unitOfMeasure === "pcs";
}

export function applySimpleIdentity(
  input: PartInput,
  componentType: string,
  value: string,
  unit: string,
): PartInput {
  if (!isComponentType(componentType)) {
    return input;
  }

  const profile = COMPONENT_PROFILES[componentType];
  const next: PartInput = {
    ...input,
    category: profile.category,
    subcategory: componentType,
    displayValue: formatComponentValue(componentType, value, unit),
    attributes: { ...input.attributes },
  };

  if (profile.units.length > 0) {
    next.attributes = {
      ...next.attributes,
      value: { value, unit },
    };
  }

  return next;
}
