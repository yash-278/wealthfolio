import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import ShortcutTokens from "./shortcut-tokens";
const api = vi.hoisted(() => ({
  listCaptureTokens: vi.fn(),
  createCaptureToken: vi.fn(),
  revokeCaptureToken: vi.fn(),
}));
vi.mock("@/adapters", () => api);
it("shows a newly generated token once and allows revocation", async () => {
  api.listCaptureTokens.mockResolvedValue([]);
  api.createCaptureToken.mockResolvedValue({
    id: "t",
    name: "Phone",
    token: "synthetic-token",
    createdAt: "2026-09-12",
    lastUsedAt: null,
  });
  api.revokeCaptureToken.mockResolvedValue(null);
  render(<ShortcutTokens />);
  await screen.findByText("No Shortcut tokens");
  fireEvent.change(screen.getByLabelText("Shortcut token name"), { target: { value: "Phone" } });
  fireEvent.click(screen.getByRole("button", { name: "Create token" }));
  expect(await screen.findByLabelText("New Shortcut token")).toHaveValue("synthetic-token");
  fireEvent.click(screen.getByRole("button", { name: "Revoke Phone" }));
  expect(await screen.findByText("No Shortcut tokens")).toBeInTheDocument();
  expect(screen.queryByLabelText("New Shortcut token")).not.toBeInTheDocument();
});
