import { useCallback, useEffect, useRef, useState } from "react";
import { AdaptiveSyncController } from "@/platform/sync/adaptiveSyncController";
import type { StorageBridge, StorageEntry, StorageInput, StorageSharedStatus } from "./types";

export function useStorageInventory(active: boolean, bridge: StorageBridge | null) {
  const [entries, setEntries] = useState<StorageEntry[]>([]);
  const [shared, setShared] = useState<StorageSharedStatus | null>(null);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState("");
  const controllerRef = useRef<AdaptiveSyncController | null>(null);
  const mutationEpoch = useRef(0);
  const mutationPending = useRef(false);

  useEffect(() => {
    if (!active || !bridge) return;
    let disposed = false;
    let token: string | null = null;
    let unsubscribe: (() => void) | undefined;
    const controller = new AdaptiveSyncController({
      focused: document.hasFocus(), visible: document.visibilityState !== "hidden",
      runSync: async () => {
        if (!token || mutationPending.current) return;
        const epoch = mutationEpoch.current;
        try {
          const result = await bridge.sync(token);
          if (!disposed && result && epoch === mutationEpoch.current) {
            setEntries(result.entries); setShared(result.shared); setError(""); setLoaded(true);
          }
        } catch (cause) {
          if (!disposed) {
            setError(`Sync unavailable: ${String(cause)}`);
            setShared(current => current ? { ...current, available: false, mutationMode: "local" } : current);
          }
        }
      },
    });
    controllerRef.current = controller;
    controller.start();
    const activity = () => controller.recordActivity();
    const focus = () => controller.setFocused(true);
    const blur = () => controller.setFocused(false);
    const visibility = () => controller.setVisible(document.visibilityState !== "hidden");
    window.addEventListener("focus", focus); window.addEventListener("blur", blur);
    document.addEventListener("visibilitychange", visibility);
    document.addEventListener("pointerdown", activity); document.addEventListener("keydown", activity);
    void (async () => {
      try {
        const epoch = mutationEpoch.current;
        const snapshot = await bridge.load();
        if (disposed) return;
        if (epoch === mutationEpoch.current) {
          setEntries(snapshot.entries); setShared(snapshot.shared); setLoaded(true); setError("");
        }
        token = await bridge.activate();
        if (disposed) { void bridge.deactivate(token).catch(() => undefined); return; }
        try {
          unsubscribe = await bridge.subscribe(() => { void controller.requestSync(); });
        } catch { /* Adaptive polling still works when event subscription is unavailable. */ }
        if (disposed) { unsubscribe?.(); return; }
        await controller.finishInitialization();
      } catch (cause) {
        if (!disposed) setError(`Could not initialize TE Storage: ${String(cause)}`);
      }
    })();
    return () => {
      disposed = true; controller.stop(); controllerRef.current = null; unsubscribe?.();
      if (token) void bridge.deactivate(token).catch(() => undefined);
      window.removeEventListener("focus", focus); window.removeEventListener("blur", blur);
      document.removeEventListener("visibilitychange", visibility);
      document.removeEventListener("pointerdown", activity); document.removeEventListener("keydown", activity);
    };
  }, [active, bridge]);

  const save = useCallback(async (input: StorageInput, original: StorageEntry | null) => {
    if (!bridge || !loaded || shared?.canModify === false) throw new Error("TE Storage is not ready to save.");
    mutationPending.current = true; mutationEpoch.current += 1;
    try {
      const result = await bridge.save(input, original);
      setEntries(current => [...current.filter(row => row.entryUuid !== result.entry.entryUuid), result.entry]);
      setShared(result.shared); setError("");
    } finally {
      mutationPending.current = false; mutationEpoch.current += 1;
      void controllerRef.current?.requestSync();
    }
  }, [bridge, loaded, shared?.canModify]);

  const remove = useCallback(async (entry: StorageEntry) => {
    if (!bridge || !loaded || shared?.canModify === false) throw new Error("TE Storage is not ready to delete.");
    mutationPending.current = true; mutationEpoch.current += 1;
    try {
      const result = await bridge.remove(entry);
      setEntries(current => current.filter(row => row.entryUuid !== entry.entryUuid));
      setShared(result.shared); setError("");
    } finally {
      mutationPending.current = false; mutationEpoch.current += 1;
      void controllerRef.current?.requestSync();
    }
  }, [bridge, loaded, shared?.canModify]);
  return { entries, shared, error, loaded, save, remove, refresh: () => controllerRef.current?.requestSync(), canModify: Boolean(bridge && loaded && shared?.canModify) };
}
