export const ACTIVE_SYNC_DELAY_MS = 2_000;
export const SLOW_SYNC_DELAY_MS = 60_000;
export const IDLE_AFTER_MS = 120_000;

type TimerHandle = ReturnType<typeof setTimeout>;

interface AdaptiveSyncControllerOptions {
  activeDelayMs?: number;
  clearTimeout?: (timer: TimerHandle) => void;
  focused: boolean;
  idleAfterMs?: number;
  now?: () => number;
  runSync: () => Promise<void>;
  setTimeout?: (callback: () => void, delayMs: number) => TimerHandle;
  slowDelayMs?: number;
  visible: boolean;
}

export class AdaptiveSyncController {
  private readonly activeDelayMs: number;
  private readonly clearScheduledTimeout: (timer: TimerHandle) => void;
  private readonly idleAfterMs: number;
  private readonly now: () => number;
  private readonly runSync: () => Promise<void>;
  private readonly scheduleTimeout: (callback: () => void, delayMs: number) => TimerHandle;
  private readonly slowDelayMs: number;
  private active = false;
  private focused: boolean;
  private followUpRequested = false;
  private generation = 0;
  private idleTimeout: TimerHandle | null = null;
  private inFlightGeneration: number | null = null;
  private initialized = false;
  private lastActivityAt = 0;
  private pollTimeout: TimerHandle | null = null;
  private visible: boolean;

  constructor(options: AdaptiveSyncControllerOptions) {
    this.activeDelayMs = options.activeDelayMs ?? ACTIVE_SYNC_DELAY_MS;
    // Wrap defaults so calling as a method never loses the global timer receiver
    // (bare `setTimeout` / `clearTimeout` throw "Illegal invocation" in the WebView).
    this.clearScheduledTimeout =
      options.clearTimeout ?? ((timer) => globalThis.clearTimeout(timer));
    this.focused = options.focused;
    this.idleAfterMs = options.idleAfterMs ?? IDLE_AFTER_MS;
    this.now = options.now ?? Date.now;
    this.runSync = options.runSync;
    this.scheduleTimeout =
      options.setTimeout ?? ((callback, delayMs) => globalThis.setTimeout(callback, delayMs));
    this.slowDelayMs = options.slowDelayMs ?? SLOW_SYNC_DELAY_MS;
    this.visible = options.visible;
  }

  start(): void {
    this.stop();
    this.active = true;
    this.initialized = false;
    this.lastActivityAt = this.now();
    this.scheduleIdleTransition(this.generation);
  }

  finishInitialization(): Promise<void> {
    if (!this.active) {
      return Promise.resolve();
    }

    this.initialized = true;
    return this.requestSync();
  }

  requestSync(): Promise<void> {
    if (!this.active) {
      return Promise.resolve();
    }

    this.clearPollTimeout();
    if (!this.initialized) {
      return Promise.resolve();
    }

    const generation = this.generation;
    if (this.inFlightGeneration === generation) {
      this.followUpRequested = true;
      return Promise.resolve();
    }

    return this.executeSync(generation);
  }

  recordActivity(): void {
    if (!this.active) {
      return;
    }

    const wasIdle = this.isIdle();
    this.lastActivityAt = this.now();
    this.scheduleIdleTransition(this.generation);
    if (wasIdle && this.focused && this.visible) {
      void this.requestSync();
    }
  }

  setFocused(focused: boolean): void {
    if (this.focused === focused) {
      return;
    }

    this.focused = focused;
    this.handleAttentionChange(focused);
  }

  setVisible(visible: boolean): void {
    if (this.visible === visible) {
      return;
    }

    this.visible = visible;
    this.handleAttentionChange(visible);
  }

  stop(): void {
    this.generation += 1;
    this.active = false;
    this.initialized = false;
    this.followUpRequested = false;
    this.inFlightGeneration = null;
    this.clearPollTimeout();
    this.clearIdleTimeout();
  }

  private executeSync(generation: number): Promise<void> {
    this.inFlightGeneration = generation;
    let syncPromise: Promise<void>;
    try {
      syncPromise = this.runSync();
    } catch {
      syncPromise = Promise.resolve();
    }

    return syncPromise
      .catch(() => undefined)
      .then(() => {
        if (!this.active || this.generation !== generation || this.inFlightGeneration !== generation) {
          return;
        }

        this.inFlightGeneration = null;
        if (this.followUpRequested) {
          this.followUpRequested = false;
          void this.executeSync(generation);
          return;
        }

        this.schedulePoll(generation);
      });
  }

  private handleAttentionChange(restored: boolean): void {
    if (!this.active) {
      return;
    }

    if (restored) {
      this.lastActivityAt = this.now();
      this.scheduleIdleTransition(this.generation);
      void this.requestSync();
      return;
    }

    this.reschedulePoll(this.generation);
  }

  private isIdle(): boolean {
    return this.now() - this.lastActivityAt >= this.idleAfterMs;
  }

  private usesActiveCadence(): boolean {
    return this.focused && this.visible && !this.isIdle();
  }

  private schedulePoll(generation: number): void {
    if (!this.active || !this.initialized || this.generation !== generation) {
      return;
    }

    this.clearPollTimeout();
    const delayMs = this.usesActiveCadence() ? this.activeDelayMs : this.slowDelayMs;
    this.pollTimeout = this.scheduleTimeout(() => {
      if (!this.active || this.generation !== generation) {
        return;
      }
      this.pollTimeout = null;
      void this.requestSync();
    }, delayMs);
  }

  private reschedulePoll(generation: number): void {
    if (
      !this.active ||
      !this.initialized ||
      this.generation !== generation ||
      this.inFlightGeneration === generation
    ) {
      return;
    }

    this.schedulePoll(generation);
  }

  private scheduleIdleTransition(generation: number): void {
    this.clearIdleTimeout();
    const delayMs = Math.max(0, this.lastActivityAt + this.idleAfterMs - this.now());
    this.idleTimeout = this.scheduleTimeout(() => {
      if (!this.active || this.generation !== generation) {
        return;
      }
      this.idleTimeout = null;
      this.reschedulePoll(generation);
    }, delayMs);
  }

  private clearPollTimeout(): void {
    if (this.pollTimeout === null) {
      return;
    }
    this.clearScheduledTimeout(this.pollTimeout);
    this.pollTimeout = null;
  }

  private clearIdleTimeout(): void {
    if (this.idleTimeout === null) {
      return;
    }
    this.clearScheduledTimeout(this.idleTimeout);
    this.idleTimeout = null;
  }
}
