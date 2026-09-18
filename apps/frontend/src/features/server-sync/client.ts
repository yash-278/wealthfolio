import { invoke } from "@tauri-apps/api/core";
import { isWeb } from "@/adapters";
export interface SyncView {
  connection: {
    endpoint: string;
    serverId: string;
    cursor: number;
    paused: boolean;
    lastSync: string | null;
    pending: number;
    conflicts: number;
  } | null;
  error: string | null;
  conflict: { eventId: string; serverEventId: string; label: string } | null;
}
export const ownServerSync = (
  command: string,
  args?: Record<string, unknown>,
): Promise<SyncView> => {
  if (isWeb)
    return Promise.reject(
      new Error("Server connection settings are available in the installed app"),
    );
  return invoke<SyncView>(`own_server_sync_${command}`, args);
};
