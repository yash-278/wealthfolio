import { MemoryRouter } from "react-router-dom";
import { fireEvent, render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import QuickAddPage from "./quick-add-page";

const api = vi.hoisted(() => ({ submitCapture: vi.fn(), getCapture: vi.fn() }));
vi.mock("@/adapters", () => api);

it("preserves unconfirmed text and distinguishes saved payments from unsaved review", async () => {
  api.submitCapture.mockRejectedValueOnce(new Error("Network unavailable")).mockResolvedValueOnce({
    id: "capture-1",
    status: "needs_review",
    version: 1,
    input: { text: "Paid for coffee and another unclear payment" },
    candidates: [
      {
        id: "one",
        status: "posted",
        activityId: "activity-1",
        fields: { amount: "100", currency: "INR", merchant: "Cafe" },
      },
      {
        id: "two",
        status: "needs_review",
        activityId: "",
        fields: { amount: "", currency: "INR", merchant: "Unknown" },
      },
    ],
    reviews: [{ id: "review-1", reason: "account_required", status: "open", activityId: null }],
  });
  render(
    <MemoryRouter>
      <QuickAddPage />
    </MemoryRouter>,
  );
  fireEvent.change(screen.getByLabelText("Transaction text"), {
    target: { value: "Paid for coffee and another unclear payment" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Capture" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Receipt unconfirmed");
  expect(screen.getByLabelText("Transaction text")).toHaveValue(
    "Paid for coffee and another unclear payment",
  );
  fireEvent.click(screen.getByRole("button", { name: "Capture" }));
  expect(await screen.findByText("Payment saved")).toBeInTheDocument();
  expect(screen.getByText("Not yet saved")).toBeInTheDocument();
  expect((api.submitCapture.mock.calls[1][0] as { clientRequestId: string }).clientRequestId).toBe(
    (api.submitCapture.mock.calls[0][0] as { clientRequestId: string }).clientRequestId,
  );
});
