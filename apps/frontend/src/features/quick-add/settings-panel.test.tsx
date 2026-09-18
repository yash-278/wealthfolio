import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import SettingsPanel from "./settings-panel";
vi.mock("./learned-rules", () => ({ default: () => null }));
const api = vi.hoisted(() => ({
  getCaptureUsage: vi.fn().mockResolvedValue({
    month: "2026-09",
    reservedOrUsedMicros: 1250,
    monthlyBudgetMicros: 1000000,
  }),
  getCaptureSettings: vi.fn(),
  updateCaptureSettings: vi.fn(),
  getAccounts: vi.fn(),
}));
vi.mock("@/adapters", () => ({ ...api, isWeb: false }));
it("configures a separate capture model and spending limit", async () => {
  api.getAccounts.mockResolvedValue([]);
  api.getCaptureSettings.mockResolvedValue({
    extractionVersion: "bedrock-luna-capture-v3",
    provider: "",
    model: "",
    monthlyBudgetMicros: null,
    timezone: "Asia/Kolkata",
    mappings: [],
    typedNoteAccountId: null,
    typedNoteToday: false,
    automaticPosting: false,
  });
  api.updateCaptureSettings.mockResolvedValue(null);
  render(<SettingsPanel />);
  fireEvent.change(await screen.findByLabelText("Monthly AI budget in USD"), {
    target: { value: "1" },
  });
  fireEvent.change(screen.getByLabelText("Source retention in days"), { target: { value: "30" } });
  fireEvent.click(screen.getByRole("button", { name: "Save settings" }));
  expect(await screen.findByText("Settings saved")).toBeInTheDocument();
  expect(api.updateCaptureSettings.mock.calls[0][0]).toMatchObject({
    extractionVersion: "bedrock-luna-capture-v3",
    sourceRetentionDays: 30,
    model: "openai.gpt-5.6-luna",
    monthlyBudgetMicros: 1000000,
    automaticPosting: false,
  });
});

it("requires evaluation and trial confirmation before enabling automatic posting", async () => {
  api.getAccounts.mockResolvedValue([]);
  api.getCaptureSettings.mockResolvedValue({
    provider: "bedrock",
    model: "openai.gpt-5.6-luna",
    monthlyBudgetMicros: 1000000,
    timezone: "Asia/Kolkata",
    mappings: [],
    typedNoteAccountId: null,
    typedNoteToday: false,
    automaticPosting: false,
    evaluationPassed: false,
    supervisedTrialCompleted: false,
  });
  render(<SettingsPanel />);
  const automatic = await screen.findByLabelText("Automatically post eligible transactions");
  expect(automatic).toBeDisabled();
  fireEvent.click(screen.getByLabelText("Evaluation completed with zero critical errors"));
  fireEvent.click(screen.getByLabelText("Supervised trial completed"));
  expect(automatic).toBeEnabled();
});
