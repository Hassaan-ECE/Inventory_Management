import { useCallback, useEffect, useRef, useState } from "react";

import { MOCK_CATALOG } from "@/modules/te-lab-components/catalog/mockCatalog";
import { TE_LAB_COMPONENTS_MODULE_ID } from "@/modules/te-lab-components/moduleId";
import type { CatalogSyncResult, InventorySharedStatus } from "@/modules/te-lab-components/types";
import { AdaptiveSyncController } from "@/platform/sync/adaptiveSyncController";

const LOCAL_MUTATION_SYNC_DELAY_MS = 75;

const PENDING_SHARED_STATUS: InventorySharedStatus = {
  available: false,
  canModify: false,
  enabled: false,
  message: "Checking Lab catalog and shared workspace...",
  mutationMode: "local",
};

const EMPTY_CATALOG: CatalogSyncResult = {
  dbPath: "",
  parts: [],
  storageAreas: [],
  storageContainers: [],
  stockPlacements: [],
  summaries: [],
  counts: {
    activeParts: 0,
    archivedParts: 0,
    totalParts: 0,
    noStock: 0,
    lowStock: 0,
    unitReview: 0,
  },
  migration: {
    schemaVersion: null,
    required: false,
    legacyEntryCount: 0,
    catalogInitialized: false,
    message: "Loading Lab Components catalog...",
  },
  shared: PENDING_SHARED_STATUS,
};

interface UseLabCatalogOptions {
  active: boolean;
  announceStatus: (message: string) => void;
}

interface RefreshOptions {
  preserveOnError?: boolean;
  showLoading?: boolean;
}

function hasDesktopBridge(): boolean {
  return typeof window !== "undefined" && Boolean(window.inventoryDesktop?.isDesktop);
}

export function useLabCatalog({ active, announceStatus }: UseLabCatalogOptions) {
  const desktop = hasDesktopBridge();
  const [catalog, setCatalog] = useState<CatalogSyncResult>(() => (desktop ? EMPTY_CATALOG : MOCK_CATALOG));
  const [isLoading, setIsLoading] = useState(desktop && active);
  const [lastError, setLastError] = useState<string | null>(null);
  const controllerRef = useRef<AdaptiveSyncController | null>(null);
  const sessionIdRef = useRef<string | null>(null);
  const delayedSyncRef = useRef<number | null>(null);
  const lifecycleGenerationRef = useRef(0);
  const mountedRef = useRef(false);
  const hasLoadedRef = useRef(!desktop);

  const isCurrentGeneration = useCallback(
    (generation: number) => mountedRef.current && lifecycleGenerationRef.current === generation,
    [],
  );

  const refreshCatalog = useCallback(
    async ({ preserveOnError = true, showLoading = false }: RefreshOptions = {}): Promise<CatalogSyncResult | null> => {
      const bridge = window.inventoryDesktop;
      const generation = lifecycleGenerationRef.current;
      if (!bridge?.loadInventory || !isCurrentGeneration(generation)) {
        return null;
      }
      if (showLoading) {
        setIsLoading(true);
      }
      try {
        const payload = await bridge.loadInventory(TE_LAB_COMPONENTS_MODULE_ID);
        if (!isCurrentGeneration(generation)) {
          return null;
        }
        hasLoadedRef.current = true;
        setCatalog(payload);
        setLastError(null);
        return payload;
      } catch (error) {
        if (isCurrentGeneration(generation)) {
          const message = error instanceof Error ? error.message : "Could not load the Lab Components catalog.";
          if (!preserveOnError || !hasLoadedRef.current) {
            setCatalog((current) => ({
              ...EMPTY_CATALOG,
              shared: {
                ...current.shared,
                available: false,
                canModify: true,
                enabled: false,
                message,
                mutationMode: "local",
              },
            }));
          }
          setLastError(message);
          announceStatus(message);
        }
        return null;
      } finally {
        if (showLoading && isCurrentGeneration(generation)) {
          setIsLoading(false);
        }
      }
    },
    [announceStatus, isCurrentGeneration],
  );

  const syncCatalog = useCallback(
    async (sessionId: string, generation: number): Promise<void> => {
      const bridge = window.inventoryDesktop;
      if (!bridge?.syncInventory || !isCurrentGeneration(generation) || sessionIdRef.current !== sessionId) {
        return;
      }
      try {
        const payload = await bridge.syncInventory(TE_LAB_COMPONENTS_MODULE_ID, sessionId);
        if (!payload || !isCurrentGeneration(generation) || sessionIdRef.current !== sessionId) {
          return;
        }
        setCatalog((current) =>
          payload.entriesChanged === true
            ? payload
            : {
                ...current,
                migration: payload.migration,
                shared: payload.shared,
              },
        );
        setLastError(null);
      } catch (error) {
        if (isCurrentGeneration(generation) && sessionIdRef.current === sessionId) {
          const message = error instanceof Error ? error.message : "Shared Lab catalog synchronization failed.";
          setCatalog((current) => ({
            ...current,
            shared: {
              ...current.shared,
              available: false,
              canModify: true,
              message,
              mutationMode: "local",
            },
          }));
          setLastError(message);
        }
      }
    },
    [isCurrentGeneration],
  );

  const scheduleDesktopSync = useCallback(() => {
    const controller = controllerRef.current;
    const sessionId = sessionIdRef.current;
    if (!controller || !sessionId) {
      return;
    }
    if (delayedSyncRef.current !== null) {
      window.clearTimeout(delayedSyncRef.current);
    }
    const generation = lifecycleGenerationRef.current;
    delayedSyncRef.current = window.setTimeout(() => {
      delayedSyncRef.current = null;
      if (isCurrentGeneration(generation) && controllerRef.current === controller) {
        void controller.requestSync();
      }
    }, LOCAL_MUTATION_SYNC_DELAY_MS);
  }, [isCurrentGeneration]);

  const refreshAfterMutation = useCallback(
    async (message?: string) => {
      const payload = await refreshCatalog({ preserveOnError: true });
      scheduleDesktopSync();
      if (message) {
        announceStatus(message);
      }
      return payload;
    },
    [announceStatus, refreshCatalog, scheduleDesktopSync],
  );

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      lifecycleGenerationRef.current += 1;
    };
  }, []);

  useEffect(() => {
    const bridge = window.inventoryDesktop;
    if (
      !active ||
      !bridge?.activateInventorySync ||
      !bridge.deactivateInventorySync ||
      !bridge.loadInventory ||
      !bridge.syncInventory
    ) {
      return undefined;
    }
    const desktopBridge = bridge;

    const generation = lifecycleGenerationRef.current + 1;
    lifecycleGenerationRef.current = generation;
    let disposed = false;
    let sessionId: string | null = null;
    let controller: AdaptiveSyncController | null = null;
    let unsubscribeSharedChanges: (() => void) | undefined;

    const current = () => !disposed && isCurrentGeneration(generation);
    const recordActivity = () => controller?.recordActivity();
    const setFocused = () => controller?.setFocused(document.hasFocus());
    const setVisible = () => controller?.setVisible(document.visibilityState === "visible");

    async function activate(): Promise<void> {
      setIsLoading(!hasLoadedRef.current);
      try {
        const token = await desktopBridge.activateInventorySync(TE_LAB_COMPONENTS_MODULE_ID);
        if (!current()) {
          void desktopBridge.deactivateInventorySync(TE_LAB_COMPONENTS_MODULE_ID, token);
          return;
        }
        sessionId = token;
        sessionIdRef.current = token;
        controller = new AdaptiveSyncController({
          focused: document.hasFocus(),
          runSync: () => syncCatalog(token, generation),
          visible: document.visibilityState === "visible",
        });
        controllerRef.current = controller;
        controller.start();
        unsubscribeSharedChanges = desktopBridge.onSharedInventoryChanged?.((payload) => {
          if (payload.systemId === TE_LAB_COMPONENTS_MODULE_ID && current()) {
            void controller?.requestSync();
          }
        });
        window.addEventListener("blur", setFocused);
        window.addEventListener("focus", setFocused);
        window.addEventListener("keydown", recordActivity);
        window.addEventListener("pointerdown", recordActivity);
        document.addEventListener("visibilitychange", setVisible);

        const payload = await refreshCatalog({ preserveOnError: true, showLoading: !hasLoadedRef.current });
        if (!current() || sessionIdRef.current !== token) {
          return;
        }
        if (!payload?.shared.enabled) {
          controller.stop();
          controllerRef.current = null;
          sessionIdRef.current = null;
          sessionId = null;
          void desktopBridge.deactivateInventorySync(TE_LAB_COMPONENTS_MODULE_ID, token);
          return;
        }
        await controller.finishInitialization();
      } catch (error) {
        if (current()) {
          const message = error instanceof Error ? error.message : "Could not activate Lab catalog synchronization.";
          setLastError(message);
          await refreshCatalog({ preserveOnError: true, showLoading: !hasLoadedRef.current });
        }
      } finally {
        if (current()) {
          setIsLoading(false);
        }
      }
    }

    void activate();

    return () => {
      disposed = true;
      if (lifecycleGenerationRef.current === generation) {
        lifecycleGenerationRef.current += 1;
      }
      if (delayedSyncRef.current !== null) {
        window.clearTimeout(delayedSyncRef.current);
        delayedSyncRef.current = null;
      }
      controller?.stop();
      unsubscribeSharedChanges?.();
      window.removeEventListener("blur", setFocused);
      window.removeEventListener("focus", setFocused);
      window.removeEventListener("keydown", recordActivity);
      window.removeEventListener("pointerdown", recordActivity);
      document.removeEventListener("visibilitychange", setVisible);
      controllerRef.current = null;
      sessionIdRef.current = null;
      if (sessionId) {
        void desktopBridge.deactivateInventorySync(TE_LAB_COMPONENTS_MODULE_ID, sessionId);
      }
    };
  }, [active, isCurrentGeneration, refreshCatalog, syncCatalog]);

  return {
    catalog,
    dataSource: desktop ? ("desktop" as const) : ("mock" as const),
    isLoading,
    lastError,
    refreshAfterMutation,
    refreshCatalog,
    scheduleDesktopSync,
    setCatalog,
  };
}
