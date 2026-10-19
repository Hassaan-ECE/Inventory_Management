import type { Part } from "@/integrations/tauri/generated/te-lab-components/Part";

// Bridge payloads are generated from the Rust types; `cargo test` regenerates them.
export type { Part };
export type { CatalogCounts } from "@/integrations/tauri/generated/te-lab-components/CatalogCounts";
export type { CatalogDeleteResult } from "@/integrations/tauri/generated/te-lab-components/CatalogDeleteResult";
export type { CatalogMutationResult } from "@/integrations/tauri/generated/te-lab-components/CatalogMutationResult";
export type { CatalogSyncResult } from "@/integrations/tauri/generated/te-lab-components/CatalogSyncResult";
export type { ComponentAttributeValue } from "@/integrations/tauri/generated/te-lab-components/ComponentAttributeValue";
export type { LabOrderRequestLineInput } from "@/integrations/tauri/generated/te-lab-components/LabOrderRequestLineInput";
export type { PartInput } from "@/integrations/tauri/generated/te-lab-components/PartInput";
export type { PartStockSummary } from "@/integrations/tauri/generated/te-lab-components/PartStockSummary";
export type { QuantityTotal } from "@/integrations/tauri/generated/te-lab-components/QuantityTotal";
export type { SimpleComponentInput } from "@/integrations/tauri/generated/te-lab-components/SimpleComponentInput";
export type { SimpleComponentValue } from "@/integrations/tauri/generated/te-lab-components/SimpleComponentValue";
export type { StockCountInput } from "@/integrations/tauri/generated/te-lab-components/StockCountInput";
export type { StockMoveInput } from "@/integrations/tauri/generated/te-lab-components/StockMoveInput";
export type { StockPlacement } from "@/integrations/tauri/generated/te-lab-components/StockPlacement";
export type { StockPlacementInput } from "@/integrations/tauri/generated/te-lab-components/StockPlacementInput";
export type { StockStatus } from "@/integrations/tauri/generated/te-lab-components/StockStatus";
export type { StorageArea } from "@/integrations/tauri/generated/te-lab-components/StorageArea";
export type { StorageAreaInput } from "@/integrations/tauri/generated/te-lab-components/StorageAreaInput";
export type { StorageContainer } from "@/integrations/tauri/generated/te-lab-components/StorageContainer";
export type { StorageContainerInput } from "@/integrations/tauri/generated/te-lab-components/StorageContainerInput";
export type { InventorySharedStatus } from "@/integrations/tauri/generated/platform/InventorySharedStatus";

export type ComponentAttributes = Part["attributes"];
export type CatalogScope = "inventory" | "archive";
export type SortDirection = "asc" | "desc";
