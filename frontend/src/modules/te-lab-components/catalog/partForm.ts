import { normalizeAttributeKey } from "@/modules/te-lab-components/catalog/categoryTemplates";
import type { ComponentAttributes, Part, PartInput } from "@/modules/te-lab-components/types";

/** One editable row of the part dialog's specifications list. */
export interface AttributeRow {
  rowId: string;
  key: string;
  value: string;
  unit: string;
}

export function inputFromPart(part: Part | null): PartInput {
  return part
    ? {
        internalPartNumber: part.internalPartNumber,
        category: part.category,
        subcategory: part.subcategory,
        manufacturer: part.manufacturer,
        manufacturerPartNumber: part.manufacturerPartNumber,
        displayValue: part.displayValue,
        mountingType: part.mountingType,
        packageType: part.packageType,
        description: part.description,
        attributes: part.attributes,
        supplier: part.supplier,
        supplierSku: part.supplierSku,
        supplierPackaging: part.supplierPackaging,
        productUrl: part.productUrl,
        datasheetUrl: part.datasheetUrl,
        defaultUnitOfMeasure: part.defaultUnitOfMeasure,
        reorderPoint: part.reorderPoint,
        targetQuantity: part.targetQuantity,
        partStatus: part.partStatus,
        picturePath: part.picturePath,
        notes: part.notes,
        archived: part.archived,
      }
    : {
        internalPartNumber: "",
        category: "Passive",
        subcategory: "",
        manufacturer: "",
        manufacturerPartNumber: "",
        displayValue: "",
        mountingType: "through_hole",
        packageType: "",
        description: "",
        attributes: {},
        supplier: "",
        supplierSku: "",
        supplierPackaging: "",
        productUrl: "",
        datasheetUrl: "",
        defaultUnitOfMeasure: "pcs",
        reorderPoint: null,
        targetQuantity: null,
        partStatus: "active",
        picturePath: "",
        notes: "",
        archived: false,
      };
}

export function attributeRows(attributes: ComponentAttributes): AttributeRow[] {
  return Object.entries(attributes).map(([key, value], index) => ({
    rowId: `${key}-${index}`,
    key,
    value: value.value,
    unit: value.unit ?? "",
  }));
}

/** The rows as stored attributes; blank rows are dropped, and a repeated key is an error. */
export function attributesFromRows(rows: AttributeRow[]): { attributes: ComponentAttributes } | { error: string } {
  const attributes: ComponentAttributes = {};
  for (const row of rows) {
    const key = normalizeAttributeKey(row.key);
    if (!key || !row.value.trim()) {
      continue;
    }
    if (attributes[key]) {
      return { error: `Attribute key “${key}” is duplicated.` };
    }
    attributes[key] = { value: row.value.trim(), unit: row.unit.trim() };
  }
  return { attributes };
}

export function numberOrNull(value: string): number | null {
  if (!value.trim()) {
    return null;
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

/** Field-by-field equality; attributes are compared separately through their rows. */
export function partFormEquals(a: PartInput, b: PartInput): boolean {
  const keys = Object.keys(a) as Array<keyof PartInput>;
  return keys.every((key) => key === "attributes" || a[key] === b[key]);
}

export function attributeRowsEqual(a: AttributeRow[], b: AttributeRow[]): boolean {
  if (a.length !== b.length) {
    return false;
  }
  return a.every((row, index) => {
    const other = b[index];
    return row.key === other.key && row.value === other.value && row.unit === other.unit;
  });
}
