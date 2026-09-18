import { invoke } from "./core";
import type { CaptureInput, CaptureReceipt } from "@/features/quick-add/types";
export const submitCapture = (input: CaptureInput) =>
  invoke<CaptureReceipt>("submit_capture", { input });
export const getCapture = (id: string) => invoke<CaptureReceipt>("get_capture", { id });
export const getCaptureReviews = (page = 0, pageSize = 25, reason?: string) =>
  invoke<CaptureReceipt[]>("get_capture_reviews", { page, pageSize, reason });
export const resolveCaptureReview = (
  id: string,
  resolution: import("@/features/quick-add/types").CaptureResolution,
) => invoke<CaptureReceipt>("resolve_capture_review", { id, resolution });
export const dismissCaptureReview = (id: string, version: number) =>
  invoke<CaptureReceipt>("dismiss_capture_review", { id, version });
export const getCaptureSettings = () =>
  invoke<import("@/features/quick-add/types").CaptureSettings>("get_capture_settings");
export const updateCaptureSettings = (
  settings: import("@/features/quick-add/types").CaptureSettings,
) => invoke<void>("update_capture_settings", { settings });
export const listCaptureTokens = (): Promise<import("@/features/quick-add/types").CaptureToken[]> =>
  Promise.reject(new Error("Manage Shortcut access in the web application"));
export const createCaptureToken = (
  _name: string,
  _expiresAt: string | null,
): Promise<import("@/features/quick-add/types").CaptureToken & { token: string }> =>
  Promise.reject(new Error("Manage Shortcut access in the web application"));
export const revokeCaptureToken = (_id: string): Promise<void> =>
  Promise.reject(new Error("Manage Shortcut access in the web application"));

export const retryCapture = (id: string, version: number) =>
  invoke<CaptureReceipt>("retry_capture", { id, version });
export const clearCaptureSource = (id: string, version: number) =>
  invoke<CaptureReceipt>("clear_capture_source", { id, version });

export const getCaptureUsage = () =>
  invoke<import("@/features/quick-add/types").CaptureUsage>("get_capture_usage");
