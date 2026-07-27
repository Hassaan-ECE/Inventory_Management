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
  { key: "category", label: "Category", defaultVisible: true, sortable: true },
  { key: "subcategory", label: "Subcategory", defaultVisible: true, sortable: true },
  { key: "manufacturerPartNumber", label: "Manufacturer Part #", defaultVisible: true, sortable: true },
  { key: "displayValue", label: "Value / Label", defaultVisible: true, sortable: true },
  { key: "packageType", label: "Package", defaultVisible: true, sortable: true },
  { key: "mountingType", label: "Mounting", defaultVisible: true, sortable: true },
  { key: "totals", label: "Total Quantity", defaultVisible: true, sortable: true },
  { key: "locations", label: "Locations", defaultVisible: true, sortable: false },
  { key: "manufacturer", label: "Manufacturer", defaultVisible: true, sortable: true },
  { key: "links", label: "Links", defaultVisible: true, sortable: false },
  { key: "internalPartNumber", label: "Internal Part #", defaultVisible: false, sortable: true },
  { key: "supplier", label: "Supplier", defaultVisible: false, sortable: true },
  { key: "supplierSku", label: "Supplier SKU", defaultVisible: false, sortable: true },
  { key: "partStatus", label: "Part Status", defaultVisible: false, sortable: true },
];
