import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  ACTIVE_SYNC_DELAY_MS,
  IDLE_AFTER_MS,
  SLOW_SYNC_DELAY_MS,
  AdaptiveSyncController,
} from "@/features/inventory/sync/adaptiveSyncController";
import { createDeferred } from "./inventory-shell/helpers";

describe("AdaptiveSyncController", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-20T12:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("uses default timers without Illegal invocation when no timer injectors are provided", async () => {
    vi.useRealTimers();
    const runSync = vi.fn().mockResolvedValue(undefined);
    const controller = new AdaptiveSyncController({
      focused: true,
      runSync,
      visible: true,
    });

    expect(() => controller.start()).not.toThrow();
    controller.stop();
  });

  it("schedules active polling only after the previous sync completes", async () => {
    const firstSync = createDeferred<void>();
    const runSync = vi.fn()
      .mockReturnValueOnce(firstSync.promise)
      .mockResolvedValue(undefined);
    const controller = createController(runSync);

    controller.start();
    const initialization = controller.finishInitialization();

    expect(runSync).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(ACTIVE_SYNC_DELAY_MS * 3);
    expect(runSync).toHaveBeenCalledTimes(1);

    firstSync.resolve();
    await initialization;
    await flushPromises();
    await vi.advanceTimersByTimeAsync(ACTIVE_SYNC_DELAY_MS - 1);
    expect(runSync).toHaveBeenCalledTimes(1);

    await vi.advanceTimersByTimeAsync(1);
    expect(runSync).toHaveBeenCalledTimes(2);
  });

  it("switches to the slow cadence after two minutes without activity", async () => {
    const firstSync = createDeferred<void>();
    const runSync = vi.fn()
      .mockReturnValueOnce(firstSync.promise)
      .mockResolvedValue(undefined);
    const controller = createController(runSync);

    controller.start();
    const initialization = controller.finishInitialization();
    await vi.advanceTimersByTimeAsync(IDLE_AFTER_MS);
    firstSync.resolve();
    await initialization;
    await flushPromises();

    await vi.advanceTimersByTimeAsync(SLOW_SYNC_DELAY_MS - 1);
    expect(runSync).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(1);
    expect(runSync).toHaveBeenCalledTimes(2);
  });

  it.each([
    ["blur", (controller: AdaptiveSyncController) => controller.setFocused(false)],
    ["hidden visibility", (controller: AdaptiveSyncController) => controller.setVisible(false)],
  ])("switches immediately to the slow cadence on %s", async (_label, makeInactive) => {
    const runSync = vi.fn().mockResolvedValue(undefined);
    const controller = createController(runSync);

    controller.start();
    await controller.finishInitialization();
    await flushPromises();
    makeInactive(controller);

    await vi.advanceTimersByTimeAsync(ACTIVE_SYNC_DELAY_MS);
    expect(runSync).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(SLOW_SYNC_DELAY_MS - ACTIVE_SYNC_DELAY_MS);
    expect(runSync).toHaveBeenCalledTimes(2);
  });

  it.each([
    ["focus", (controller: AdaptiveSyncController) => controller.setFocused(true)],
    ["visibility", (controller: AdaptiveSyncController) => controller.setVisible(true)],
  ])("syncs immediately when %s is restored", async (_label, restore) => {
    const runSync = vi.fn().mockResolvedValue(undefined);
    const controller = createController(runSync);

    controller.start();
    await controller.finishInitialization();
    await flushPromises();
    controller.setFocused(false);
    controller.setVisible(false);
    runSync.mockClear();

    restore(controller);

    expect(runSync).toHaveBeenCalledTimes(1);
  });

  it("syncs immediately when meaningful activity wakes an idle controller", async () => {
    const firstSync = createDeferred<void>();
    const runSync = vi.fn()
      .mockReturnValueOnce(firstSync.promise)
      .mockResolvedValue(undefined);
    const controller = createController(runSync);

    controller.start();
    const initialization = controller.finishInitialization();
    await vi.advanceTimersByTimeAsync(IDLE_AFTER_MS);
    firstSync.resolve();
    await initialization;
    await flushPromises();
    runSync.mockClear();

    controller.recordActivity();

    expect(runSync).toHaveBeenCalledTimes(1);
  });

  it("services watcher-style requests immediately while idle", async () => {
    const firstSync = createDeferred<void>();
    const runSync = vi.fn()
      .mockReturnValueOnce(firstSync.promise)
      .mockResolvedValue(undefined);
    const controller = createController(runSync);

    controller.start();
    const initialization = controller.finishInitialization();
    await vi.advanceTimersByTimeAsync(IDLE_AFTER_MS);
    firstSync.resolve();
    await initialization;
    await flushPromises();
    runSync.mockClear();

    controller.requestSync();

    expect(runSync).toHaveBeenCalledTimes(1);
  });

  it("coalesces initialization and in-flight triggers into one follow-up", async () => {
    const firstSync = createDeferred<void>();
    const runSync = vi.fn()
      .mockReturnValueOnce(firstSync.promise)
      .mockResolvedValue(undefined);
    const controller = createController(runSync);

    controller.start();
    controller.requestSync();
    controller.requestSync();
    expect(runSync).not.toHaveBeenCalled();

    const initialization = controller.finishInitialization();
    controller.requestSync();
    controller.requestSync();
    firstSync.resolve();
    await initialization;
    await flushPromises();

    expect(runSync).toHaveBeenCalledTimes(2);
  });

  it("does not rearm after stop while a sync is in flight", async () => {
    const firstSync = createDeferred<void>();
    const runSync = vi.fn().mockReturnValue(firstSync.promise);
    const controller = createController(runSync);

    controller.start();
    const initialization = controller.finishInitialization();
    controller.stop();
    firstSync.resolve();
    await initialization;
    await flushPromises();
    await vi.advanceTimersByTimeAsync(SLOW_SYNC_DELAY_MS * 2);

    expect(runSync).toHaveBeenCalledTimes(1);
    expect(vi.getTimerCount()).toBe(0);
  });
});

function createController(runSync: () => Promise<void>): AdaptiveSyncController {
  return new AdaptiveSyncController({
    focused: true,
    now: () => Date.now(),
    runSync,
    visible: true,
  });
}

async function flushPromises(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}
