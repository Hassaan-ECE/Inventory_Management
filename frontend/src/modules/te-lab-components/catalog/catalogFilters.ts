import type { StockStatus } from "@/modules/te-lab-components/types";

export interface CatalogFilters {
  areaUuid: string;
  category: string;
  containerUuid: string;
  coordinate: string;
  displayValue: string;
  manufacturer: string;
  manufacturerPartNumber: string;
  mountingType: string;
  packageType: string;
  partStatus: string;
  stockStatus: "" | StockStatus;
  subcategory: string;
}

export const EMPTY_CATALOG_FILTERS: CatalogFilters = {
  areaUuid: "",
  category: "",
  containerUuid: "",
  coordinate: "",
  displayValue: "",
  manufacturer: "",
  manufacturerPartNumber: "",
  mountingType: "",
  packageType: "",
  partStatus: "",
  stockStatus: "",
  subcategory: "",
};
