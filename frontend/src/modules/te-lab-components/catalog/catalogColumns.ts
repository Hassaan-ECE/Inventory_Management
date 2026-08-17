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
  | "links"
  | "internalPartNumber"
  | "subcategory"
  | "supplier"
  | "supplierSku"
  | "partStatus";

export interface CatalogColumnDefinition {
  key: CatalogColumnKey;
  label: string;
  defaultVisible: boolean;
  sortable: boolean;
}

export const CATALOG_COLUMNS: CatalogColumnDefinition[] = [
  { key: "stockStatus", label: "Stock Status", defaultVisible: true, sortable: true },
  { key: "subcategory", label: "Component Type", defaultVisible: true, sortable: true },
  { key: "displayValue", label: "Value", defaultVisible: true, sortable: true },
  { key: "totals", label: "Quantity", defaultVisible: true, sortable: true },
  { key: "locations", label: "Location", defaultVisible: true, sortable: false },
  { key: "category", label: "Category", defaultVisible: false, sortable: true },
  { key: "manufacturerPartNumber", label: "Manufacturer Part #", defaultVisible: false, sortable: true },
  { key: "packageType", label: "Package", defaultVisible: false, sortable: true },
  { key: "mountingType", label: "Mounting", defaultVisible: false, sortable: true },
  { key: "manufacturer", label: "Manufacturer", defaultVisible: false, sortable: true },
  { key: "links", label: "Links", defaultVisible: false, sortable: false },
  { key: "internalPartNumber", label: "Internal Part #", defaultVisible: false, sortable: true },
  { key: "supplier", label: "Supplier", defaultVisible: false, sortable: true },
  { key: "supplierSku", label: "Supplier SKU", defaultVisible: false, sortable: true },
  { key: "partStatus", label: "Part Status", defaultVisible: false, sortable: true },
];
