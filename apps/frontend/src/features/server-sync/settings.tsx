import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { isWeb } from "@/adapters";
import { Button } from "@wealthfolio/ui/components/ui/button";
import { Input } from "@wealthfolio/ui/components/ui/input";
import { SettingsHeader } from "@/pages/settings/settings-header";
import { ownServerSync } from "./client";

export default function ServerSyncSettings() {
  const queries = useQueryClient();
  const status = useQuery({
    queryKey: ["own-server-sync"],
    queryFn: () => ownServerSync("status"),
    enabled: !isWeb,
    refetchInterval: 5000,
  });
  const [endpoint, setEndpoint] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [confirmServer, setConfirmServer] = useState(false);
  const view = status.data;
  const connection = view?.connection;
  const act = async (command: string, args?: Record<string, unknown>) => {
    setBusy(true);
    setError("");
    try {
      const result = await ownServerSync(command, args);
      queries.setQueryData(["own-server-sync"], result);
      if (command === "connect" || command === "accept_server" || command === "run") {
        await queries.invalidateQueries({ predicate: (q) => q.queryKey[0] !== "own-server-sync" });
      }
      if (command === "connect") setPassword("");
      setConfirmServer(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="space-y-6">
      <SettingsHeader
        heading="Your server"
        text="Keep this device in sync with your own Wealthfolio server."
      />
      {isWeb ? (
        <p>Open these settings in the installed iOS or desktop app to connect a device.</p>
      ) : (
        <>
          <div className="space-y-4 rounded-lg border p-4">
            {connection && (
              <div className="space-y-2">
                <p className="break-all font-medium">{connection.endpoint}</p>
                <p>
                  {connection.paused ? "Sync paused" : "Sync enabled"} · {connection.pending}{" "}
                  pending edits
                </p>
                <p className="text-muted-foreground text-sm">
                  {connection.lastSync
                    ? `Last sync: ${new Date(connection.lastSync).toLocaleString()}`
                    : "Initial download complete"}
                </p>
                <div className="flex flex-wrap gap-2">
                  <Button disabled={busy || connection.paused} onClick={() => void act("run")}>
                    Sync now
                  </Button>
                  <Button
                    variant="outline"
                    disabled={busy}
                    onClick={() => void act("pause", { paused: !connection.paused })}
                  >
                    {connection.paused ? "Resume" : "Pause"}
                  </Button>
                </div>
              </div>
            )}
            <form
              className="space-y-3"
              onSubmit={(e) => {
                e.preventDefault();
                void act("connect", { endpoint: connection?.endpoint ?? endpoint, password });
              }}
            >
              {!connection && (
                <>
                  <label className="block space-y-1">
                    <span>Server address</span>
                    <Input
                      type="url"
                      autoCapitalize="none"
                      autoCorrect="off"
                      placeholder="https://wealthfolio.example.com"
                      value={endpoint}
                      onChange={(e) => setEndpoint(e.target.value)}
                      required
                    />
                  </label>
                  <p className="text-muted-foreground text-sm">
                    First connection downloads your server portfolio. Existing local records will
                    never be replaced. Use a fresh installation to connect.
                  </p>
                </>
              )}
              <label className="block space-y-1">
                <span>Server password</span>
                <Input
                  type="password"
                  autoComplete="current-password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                />
              </label>
              <Button type="submit" disabled={busy || !password || (!connection && !endpoint)}>
                {busy ? "Working…" : connection ? "Sign in again" : "Connect and download"}
              </Button>
            </form>
            <p className="text-muted-foreground text-sm">
              Edits stay on this device while offline. Sync runs while the app is open and when you
              return to it. Pausing keeps your data and pending edits.
            </p>
            <p className="text-muted-foreground text-sm">
              Quick Add review items and some advanced records are not synchronized in this first
              version.
            </p>
          </div>
          {view?.conflict && (
            <div className="space-y-3 rounded-lg border p-4">
              <h2 className="font-medium">An edit needs review</h2>
              <p>{view.conflict.label}</p>
              <p>
                This record changed on both this device and the server. Your local edits are
                preserved.
              </p>
              {confirmServer ? (
                <>
                  <p>Discard pending edits to this record and use the server version?</p>
                  <div className="flex gap-2">
                    <Button
                      disabled={busy}
                      onClick={() =>
                        void act("accept_server", {
                          eventId: view.conflict!.eventId,
                          serverEventId: view.conflict!.serverEventId,
                        })
                      }
                    >
                      Discard local edits
                    </Button>
                    <Button variant="outline" onClick={() => setConfirmServer(false)}>
                      Cancel
                    </Button>
                  </div>
                </>
              ) : (
                <Button variant="outline" onClick={() => setConfirmServer(true)}>
                  Use server version
                </Button>
              )}
            </div>
          )}
          {(error || view?.error || status.error) && (
            <p role="alert" className="text-destructive">
              {error || view?.error || String(status.error)}
            </p>
          )}
        </>
      )}
    </div>
  );
}
