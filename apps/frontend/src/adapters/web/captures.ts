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
export const listCaptureTokens = () =>
  invoke<import("@/features/quick-add/types").CaptureToken[]>("list_capture_tokens");
export const createCaptureToken = (name: string, expiresAt: string | null) =>
  invoke<import("@/features/quick-add/types").CaptureToken & { token: string }>(
    "create_capture_token",
    { name, expiresAt },
  );
export const revokeCaptureToken = (id: string) => invoke<void>("revoke_capture_token", { id });

export const retryCapture = (id: string, version: number) =>
  invoke<CaptureReceipt>("retry_capture", { id, version });
export const clearCaptureSource = (id: string, version: number) =>
  invoke<CaptureReceipt>("clear_capture_source", { id, version });

export const getCaptureUsage = () =>
  invoke<import("@/features/quick-add/types").CaptureUsage>("get_capture_usage");
