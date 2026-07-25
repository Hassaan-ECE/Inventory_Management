export type CatalogColumnKey =
  | "stockStatus"
  | "category"
  | "manufacturerPartNumber"
  | "displayValue"
  | "packageType"
  | "mountingType"
  | "totals"
  | "locations"
  | "manufacturer"
  | "documents"
  | "internalPartNumber"
  | "subcategory"
  | "supplier"
  | "supplierSku"
  | "reorderPoint"
  | "targetQuantity"
  | "partStatus"
  | "updatedAt"
  | "archived";

export interface CatalogColumnDefinition {
  key: CatalogColumnKey;
  label: string;
  defaultVisible: boolean;
}

export const CATALOG_COLUMNS: CatalogColumnDefinition[] = [
  { key: "stockStatus", label: "Stock Status", defaultVisible: true },
  { key: "category", label: "Category", defaultVisible: true },
  { key: "manufacturerPartNumber", label: "Manufacturer Part #", defaultVisible: true },
  { key: "displayValue", label: "Value / Label", defaultVisible: true },
  { key: "packageType", label: "Package", defaultVisible: true },
  { key: "mountingType", label: "Mounting", defaultVisible: true },
  { key: "totals", label: "Total Quantity", defaultVisible: true },
  { key: "locations", label: "Locations", defaultVisible: true },
  { key: "manufacturer", label: "Manufacturer", defaultVisible: true },
  { key: "documents", label: "Documents", defaultVisible: true },
  { key: "internalPartNumber", label: "Internal Part #", defaultVisible: false },
  { key: "subcategory", label: "Subcategory", defaultVisible: false },
  { key: "supplier", label: "Supplier", defaultVisible: false },
  { key: "supplierSku", label: "Supplier SKU", defaultVisible: false },
  { key: "reorderPoint", label: "Reorder Point", defaultVisible: false },
  { key: "targetQuantity", label: "Target Quantity", defaultVisible: false },
  { key: "partStatus", label: "Part Status", defaultVisible: false },
  { key: "updatedAt", label: "Updated", defaultVisible: false },
  { key: "archived", label: "Archived", defaultVisible: false },
];
