import { useCallback, useEffect, useRef, useState } from "react";

import { MOCK_INVENTORY } from "@/modules/te-test-equipment/data/mockInventory";
import type { InventoryEntry, InventorySharedStatus } from "@/modules/te-test-equipment/types";
import { AdaptiveSyncController } from "@/platform/sync/adaptiveSyncController";
import type { InventorySyncResult } from "@/integrations/tauri/desktop-bridge";

import {
  DESKTOP_SHARED_PENDING_STATUS,
  MOCK_SHARED_STATUS,
  hasDesktopBridge,
  normalizeSharedStatus,
  sharedStatusesMatch,
} from "./helpers";

const LOCAL_MUTATION_SYNC_DELAY_MS = 75;

interface UseDesktopInventoryOptions {
  announceStatus: (message: string) => void;
  teActive: boolean;
}

interface RefreshDesktopEntriesOptions {
  generation?: number;
  keepLoading?: boolean;
  preserveEntriesOnError?: boolean;
  showLoading?: boolean;
}

export function useDesktopInventory({ announceStatus, teActive }: UseDesktopInventoryOptions) {
  const [entries, setEntries] = useState<InventoryEntry[]>(() => (hasDesktopBridge() ? [] : MOCK_INVENTORY));
  const [dataSource, setDataSource] = useState<"desktop" | "mock">(() => (hasDesktopBridge() ? "desktop" : "mock"));
  const [isLoading, setIsLoading] = useState<boolean>(() => hasDesktopBridge() && teActive);
  const [sharedStatus, setSharedStatus] = useState<InventorySharedStatus>(() =>
    hasDesktopBridge() ? DESKTOP_SHARED_PENDING_STATUS : MOCK_SHARED_STATUS,
  );
  const controllerRef = useRef<AdaptiveSyncController | null>(null);
  const delayedSyncTimeoutRef = useRef<number | null>(null);
  const hasLoadedRef = useRef(!hasDesktopBridge());
  const lifecycleGenerationRef = useRef(0);
  const mountedRef = useRef(false);
  const queryRequestRef = useRef(0);
  const sessionIdRef = useRef<string | null>(null);

  const isCurrentGeneration = useCallback(
    (generation: number): boolean => mountedRef.current && lifecycleGenerationRef.current === generation,
    [],
  );

  const markSharedUnavailable = useCallback(
    (
      message = "Shared workspace unavailable. Saving changes locally.",
      disableSync = false,
    ): void => {
      setSharedStatus((current) => ({
        ...current,
        available: false,
        canModify: true,
        enabled: disableSync ? false : current.enabled,
        hasLocalOnlyChanges: current.hasLocalOnlyChanges,
        message,
        mutationMode: "local",
      }));
    },
    [],
  );

  const refreshDesktopEntries = useCallback(
    async ({
      generation = lifecycleGenerationRef.current,
      keepLoading = false,
      preserveEntriesOnError = false,
      showLoading = false,
    }: RefreshDesktopEntriesOptions = {}): Promise<InventorySyncResult | null> => {
      const desktopBridge = window.inventoryDesktop;
      if (!desktopBridge?.loadInventory || !isCurrentGeneration(generation)) {
        return null;
      }

      const requestId = queryRequestRef.current + 1;
      queryRequestRef.current = requestId;
      if (showLoading && isCurrentGeneration(generation)) {
        setIsLoading(true);
      }
      try {
        const payload = await desktopBridge.loadInventory();
        if (!isCurrentGeneration(generation) || requestId !== queryRequestRef.current) {
          return null;
        }
        const shared = normalizeSharedStatus(payload.shared);
        hasLoadedRef.current = true;
        setEntries(payload.entries);
        setDataSource("desktop");
        setSharedStatus((current) => (sharedStatusesMatch(current, shared) ? current : shared));
        return shared === payload.shared ? payload : { ...payload, shared };
      } catch {
        if (isCurrentGeneration(generation) && requestId === queryRequestRef.current) {
          if (!preserveEntriesOnError || !hasLoadedRef.current) {
            setEntries([]);
          }
          setDataSource("desktop");
          markSharedUnavailable(
            "Inventory database unavailable. Restart the app or check app data permissions.",
            true,
          );
          announceStatus("Could not load the TE Test Equipment Inventory database.");
        }
        return null;
      } finally {
        if (!keepLoading && isCurrentGeneration(generation) && requestId === queryRequestRef.current) {
          setIsLoading(false);
        }
      }
    },
    [announceStatus, isCurrentGeneration, markSharedUnavailable],
  );

  const syncEntriesFromDesktop = useCallback(
    async (sessionId: string, generation: number): Promise<void> => {
      const desktopBridge = window.inventoryDesktop;
      if (
        !desktopBridge?.syncInventory ||
        !isCurrentGeneration(generation) ||
        sessionIdRef.current !== sessionId
      ) {
        return;
      }

      const startingRequestId = queryRequestRef.current;
      try {
        const payload = await desktopBridge.syncInventory(sessionId);
        if (
          payload === null ||
          !isCurrentGeneration(generation) ||
          sessionIdRef.current !== sessionId
        ) {
          return;
        }
        const shared = normalizeSharedStatus(payload.shared);
        setSharedStatus((current) => (sharedStatusesMatch(current, shared) ? current : shared));
        if (payload.entriesChanged === true && startingRequestId === queryRequestRef.current) {
          setEntries(payload.entries);
          setDataSource("desktop");
        }
      } catch {
        if (isCurrentGeneration(generation) && sessionIdRef.current === sessionId) {
          markSharedUnavailable();
        }
      }
    },
    [isCurrentGeneration, markSharedUnavailable],
  );

  const scheduleDesktopSync = useCallback((): void => {
    const controller = controllerRef.current;
    const sessionId = sessionIdRef.current;
    if (!controller || !sessionId) {
      return;
    }
    if (delayedSyncTimeoutRef.current !== null) {
      window.clearTimeout(delayedSyncTimeoutRef.current);
    }
    const generation = lifecycleGenerationRef.current;
    delayedSyncTimeoutRef.current = window.setTimeout(() => {
      delayedSyncTimeoutRef.current = null;
      if (
        isCurrentGeneration(generation) &&
        controllerRef.current === controller &&
        sessionIdRef.current === sessionId
      ) {
        void controller.requestSync();
      }
    }, LOCAL_MUTATION_SYNC_DELAY_MS);
  }, [isCurrentGeneration]);

  useEffect(() => {
    mountedRef.current = true;

    return () => {
      mountedRef.current = false;
      lifecycleGenerationRef.current += 1;
      queryRequestRef.current += 1;
    };
  }, []);

  useEffect(() => {
    const desktopBridge = window.inventoryDesktop;
    if (
      !teActive ||
      !desktopBridge?.activateInventorySync ||
      !desktopBridge.deactivateInventorySync ||
      !desktopBridge.loadInventory ||
      !desktopBridge.syncInventory
    ) {
      return undefined;
    }

    const activateInventorySync = desktopBridge.activateInventorySync;
    const deactivateInventorySync = desktopBridge.deactivateInventorySync;
    const onSharedInventoryChanged = desktopBridge.onSharedInventoryChanged;
    const generation = lifecycleGenerationRef.current + 1;
    lifecycleGenerationRef.current = generation;
    let controller: AdaptiveSyncController | null = null;
    let disposed = false;
    let sessionId: string | null = null;
    let unsubscribeSharedChanges: (() => void) | undefined;
    let listenersAttached = false;

    const isCurrent = (): boolean => !disposed && isCurrentGeneration(generation);
    const handleActivity = (): void => controller?.recordActivity();
    const handleBlur = (): void => controller?.setFocused(false);
    const handleFocus = (): void => controller?.setFocused(true);
    const handleVisibilityChange = (): void => {
      controller?.setVisible(document.visibilityState === "visible");
    };

    const releaseFrontendLifecycle = (): void => {
      controller?.stop();
      unsubscribeSharedChanges?.();
      unsubscribeSharedChanges = undefined;
      if (listenersAttached) {
        window.removeEventListener("blur", handleBlur);
        window.removeEventListener("focus", handleFocus);
        window.removeEventListener("keydown", handleActivity);
        window.removeEventListener("pointerdown", handleActivity);
        window.removeEventListener("touchstart", handleActivity);
        window.removeEventListener("wheel", handleActivity);
        document.removeEventListener("visibilitychange", handleVisibilityChange);
        listenersAttached = false;
      }
    };

    const clearSessionReferences = (): void => {
      if (controllerRef.current === controller) {
        controllerRef.current = null;
      }
      if (sessionIdRef.current === sessionId) {
        sessionIdRef.current = null;
      }
    };

    const deactivateSession = (token: string): void => {
      void deactivateInventorySync(token).catch(() => undefined);
    };

    async function activateDesktopInventory(): Promise<void> {
      setIsLoading(!hasLoadedRef.current);
      let token: string;
      try {
        token = await activateInventorySync();
      } catch {
        if (isCurrent()) {
          await refreshDesktopEntries({
            generation,
            preserveEntriesOnError: true,
            showLoading: !hasLoadedRef.current,
          });
          markSharedUnavailable("Shared synchronization could not be activated.", true);
          setIsLoading(false);
        }
        return;
      }

      if (!isCurrent()) {
        deactivateSession(token);
        return;
      }

      sessionId = token;
      sessionIdRef.current = token;
      controller = new AdaptiveSyncController({
        focused: typeof document.hasFocus === "function" ? document.hasFocus() : true,
        runSync: () => syncEntriesFromDesktop(token, generation),
        visible: document.visibilityState === "visible",
      });
      controllerRef.current = controller;
      controller.start();
      unsubscribeSharedChanges = onSharedInventoryChanged?.(() => {
        if (isCurrent()) {
          void controller?.requestSync();
        }
      });

      window.addEventListener("blur", handleBlur);
      window.addEventListener("focus", handleFocus);
      window.addEventListener("keydown", handleActivity);
      window.addEventListener("pointerdown", handleActivity);
      window.addEventListener("touchstart", handleActivity);
      window.addEventListener("wheel", handleActivity);
      document.addEventListener("visibilitychange", handleVisibilityChange);
      listenersAttached = true;

      const payload = await refreshDesktopEntries({
        generation,
        keepLoading: true,
        preserveEntriesOnError: true,
        showLoading: !hasLoadedRef.current,
      });
      if (!isCurrent() || sessionIdRef.current !== token) {
        return;
      }

      if (!payload || !payload.shared.enabled) {
        releaseFrontendLifecycle();
        clearSessionReferences();
        deactivateSession(token);
        sessionId = null;
        setIsLoading(false);
        return;
      }

      await controller.finishInitialization();
      if (isCurrent() && sessionIdRef.current === token) {
        setIsLoading(false);
      }
    }

    void activateDesktopInventory();

    return () => {
      disposed = true;
      if (lifecycleGenerationRef.current === generation) {
        lifecycleGenerationRef.current += 1;
      }
      queryRequestRef.current += 1;
      if (delayedSyncTimeoutRef.current !== null) {
        window.clearTimeout(delayedSyncTimeoutRef.current);
        delayedSyncTimeoutRef.current = null;
      }
      releaseFrontendLifecycle();
      clearSessionReferences();
      if (sessionId) {
        deactivateSession(sessionId);
      }
    };
  }, [
    isCurrentGeneration,
    markSharedUnavailable,
    refreshDesktopEntries,
    syncEntriesFromDesktop,
    teActive,
  ]);

  return {
    dataSource,
    entries,
    isLoading,
    refreshDesktopEntries,
    scheduleDesktopSync,
    setEntries,
    setSharedStatus,
    sharedStatus,
  };
}
