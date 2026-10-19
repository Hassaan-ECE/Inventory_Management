import { APP_VERSION } from "@/app/branding";

type UpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "not-available"
  | "downloading"
  | "ready"
  | "installing"
  | "error";

export interface UpdateState {
  available: boolean;
  currentVersion: string;
  downloadPhase?: "copying" | "verifying" | "ready";
  downloadProgress?: number;
  downloadedInstallerPath?: string;
  error?: string;
  installLogPath?: string;
  installerPid?: number;
  latestVersion?: string;
  notes?: string;
  publishedAt?: string;
  status: UpdateStatus;
}

export const UPDATE_CHECK_INTERVAL_MS = 5 * 60_000;

export function buildIdleUpdateState(): UpdateState {
  return {
    available: false,
    currentVersion: APP_VERSION,
    status: "idle",
  };
}

export function chooseFreshUpdateState(current: UpdateState, next: UpdateState): UpdateState {
  if (current.latestVersion && current.latestVersion === next.latestVersion) {
    return getUpdateStatusRank(current.status) > getUpdateStatusRank(next.status) ? current : next;
  }

  return next;
}

function getUpdateStatusRank(status: UpdateState["status"]): number {
  switch (status) {
    case "idle":
      return 0;
    case "checking":
      return 1;
    case "not-available":
      return 2;
    case "available":
      return 3;
    case "downloading":
      return 4;
    case "ready":
      return 5;
    case "installing":
      return 6;
    case "error":
      return 7;
    default:
      return 0;
  }
}
