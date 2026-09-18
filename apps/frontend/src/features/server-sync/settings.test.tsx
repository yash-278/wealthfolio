import { fireEvent, render, screen, waitFor, cleanup } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { afterEach, expect, it, vi } from "vitest";
import ServerSyncSettings from "./settings";
import type { SyncView } from "./client";
const api = vi.hoisted(() => ({ call: vi.fn() }));
vi.mock("./client", () => ({ ownServerSync: api.call }));
vi.mock("@/adapters", () => ({ isWeb: false }));
vi.mock("@/pages/settings/settings-header", () => ({
  SettingsHeader: ({ heading }: { heading: string }) => <h1>{heading}</h1>,
}));
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});
const connected: SyncView = {
  connection: {
    endpoint: "https://example.com",
    serverId: "server",
    cursor: 1,
    paused: false,
    lastSync: null,
    pending: 2,
    conflicts: 0,
  },
  error: null,
  conflict: null,
};
function show() {
  return render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ServerSyncSettings />
    </QueryClientProvider>,
  );
}
it("connects with the entered address and clears the password after success", async () => {
  api.call.mockImplementation((command: string) =>
    Promise.resolve(
      command === "status" ? { connection: null, error: null, conflict: null } : connected,
    ),
  );
  show();
  fireEvent.change(await screen.findByLabelText("Server address"), {
    target: { value: "https://example.com" },
  });
  fireEvent.change(screen.getByLabelText("Server password"), { target: { value: "test-secret" } });
  fireEvent.click(screen.getByRole("button", { name: "Connect and download" }));
  await waitFor(() =>
    expect(api.call).toHaveBeenCalledWith("connect", {
      endpoint: "https://example.com",
      password: "test-secret",
    }),
  );
  await waitFor(() => expect(screen.getByLabelText("Server password")).toHaveValue(""));
});
it("pauses without removing the pending edits", async () => {
  api.call.mockImplementation((command: string) =>
    Promise.resolve(
      command === "pause"
        ? { ...connected, connection: { ...connected.connection, paused: true } }
        : connected,
    ),
  );
  show();
  fireEvent.click(await screen.findByRole("button", { name: "Pause" }));
  await waitFor(() => expect(api.call).toHaveBeenCalledWith("pause", { paused: true }));
  expect(await screen.findByText(/Sync paused · 2 pending edits/)).toBeInTheDocument();
});
it("requires an explicit choice before discarding conflicting local edits", async () => {
  api.call.mockResolvedValue({
    ...connected,
    conflict: { eventId: "local-event", serverEventId: "remote-event", label: "My goal" },
  });
  show();
  fireEvent.click(await screen.findByRole("button", { name: "Use server version" }));
  expect(api.call).not.toHaveBeenCalledWith("accept_server", expect.anything());
  fireEvent.click(screen.getByRole("button", { name: "Discard local edits" }));
  await waitFor(() =>
    expect(api.call).toHaveBeenCalledWith("accept_server", {
      eventId: "local-event",
      serverEventId: "remote-event",
    }),
  );
});
