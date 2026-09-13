import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { isWeb } from "@/adapters";
import { ownServerSync } from "./client";
export function ServerSyncRunner() {
  const queries = useQueryClient();
  const cursor = useRef<number | undefined>(undefined);
  useEffect(() => {
    if (isWeb) return;
    let active = true;
    let running = false;
    const run = async () => {
      if (!active || running || document.visibilityState === "hidden") return;
      running = true;
      try {
        const view = await ownServerSync("run");
        if (!active) return;
        queries.setQueryData(["own-server-sync"], view);
        if (view.connection && cursor.current !== view.connection.cursor) {
          cursor.current = view.connection.cursor;
          await queries.invalidateQueries({
            predicate: (q) => q.queryKey[0] !== "own-server-sync",
          });
        }
      } catch {
        /* Startup may finish after the frontend mounts. The next foreground cycle retries. */
      } finally {
        running = false;
      }
    };
    void run();
    const timer = window.setInterval(() => void run(), 30_000);
    const foreground = () => void run();
    window.addEventListener("online", foreground);
    window.addEventListener("focus", foreground);
    document.addEventListener("visibilitychange", foreground);
    return () => {
      active = false;
      window.clearInterval(timer);
      window.removeEventListener("online", foreground);
      window.removeEventListener("focus", foreground);
      document.removeEventListener("visibilitychange", foreground);
    };
  }, [queries]);
  return null;
}
