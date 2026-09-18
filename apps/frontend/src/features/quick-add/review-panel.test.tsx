import { fireEvent, render, screen } from "@testing-library/react";
import { beforeAll, afterAll, expect, it, vi } from "vitest";
import ReviewPanel from "./review-panel";
const api = vi.hoisted(() => ({
  getCaptureReviews: vi.fn(),
  resolveCaptureReview: vi.fn(),
  dismissCaptureReview: vi.fn(),
  retryCapture: vi.fn(),
  getAccounts: vi.fn(),
  getSpendCategories: vi.fn(),
}));
vi.mock("@/adapters", () => api);
it("completes balance follow-up on an already saved payment", async () => {
  api.getAccounts.mockResolvedValue([]);
  api.getSpendCategories.mockResolvedValue([]);
  const capture = {
    id: "c",
    version: 3,
    input: { text: "Loan installment" },
    candidates: [],
    reviews: [{ id: "r", status: "open", reason: "balance_update_required", activityId: "a" }],
  };
  api.getCaptureReviews.mockResolvedValueOnce([capture]).mockResolvedValue([]);
  api.resolveCaptureReview.mockResolvedValue({ ...capture, reviews: [] });
  render(<ReviewPanel />);
  expect(await screen.findByText("Payment already saved")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Save payment" })).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Mark balance update complete" }));
  expect(await screen.findByText("No items need review")).toBeInTheDocument();
});

it("lets an unresolved entry be corrected before saving", async () => {
  api.getAccounts.mockResolvedValue([
    { id: "bank", name: "Test bank", currency: "INR", isActive: true, isArchived: false },
  ]);
  api.getSpendCategories.mockResolvedValue([]);
  const capture = {
    id: "c2",
    version: 1,
    input: { text: "Unclear debit" },
    candidates: [],
    reviews: [{ id: "r2", status: "open", reason: "account_required", activityId: null }],
  };
  api.getCaptureReviews.mockResolvedValueOnce([capture]).mockResolvedValue([]);
  api.resolveCaptureReview.mockResolvedValue({
    ...capture,
    version: 3,
    reviews: [{ id: "category", status: "open", reason: "category_required", activityId: "saved" }],
  });
  render(<ReviewPanel />);
  fireEvent.change(await screen.findByLabelText("Account"), { target: { value: "bank" } });
  fireEvent.change(screen.getByLabelText("Amount"), { target: { value: "120" } });
  fireEvent.change(screen.getByLabelText("Date"), { target: { value: "2026-09-10" } });
  fireEvent.click(screen.getByRole("button", { name: "Save payment" }));
  expect(await screen.findByText("Payment already saved")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Save payment" })).not.toBeInTheDocument();
});

it("categorizes the existing payment through category review", async () => {
  api.getAccounts.mockResolvedValue([]);
  api.getSpendCategories.mockResolvedValue([
    { taxonomyId: "spending_categories", categoryId: "coffee", name: "Coffee Shops" },
  ]);
  const capture = {
    id: "c3",
    version: 3,
    input: { text: "Coffee" },
    candidates: [],
    reviews: [{ id: "r3", status: "open", reason: "category_required", activityId: "a3" }],
  };
  api.getCaptureReviews.mockResolvedValueOnce([capture]).mockResolvedValue([]);
  api.resolveCaptureReview.mockResolvedValue({ ...capture, reviews: [] });
  render(<ReviewPanel />);
  fireEvent.click(await screen.findByLabelText("Category"));
  fireEvent.click(await screen.findByRole("option", { name: "Coffee Shops" }));
  fireEvent.click(screen.getByRole("button", { name: "Save category" }));
  expect(await screen.findByText("No items need review")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Save payment" })).not.toBeInTheDocument();
});

it("retries an extraction without submitting a new capture", async () => {
  api.getAccounts.mockResolvedValue([]);
  api.getSpendCategories.mockResolvedValue([]);
  const capture = {
    id: "retry",
    version: 5,
    status: "needs_review",
    input: { text: "Original text" },
    candidates: [],
    reviews: [{ id: "failed", status: "open", reason: "extraction_failed", activityId: null }],
  };
  api.getCaptureReviews.mockResolvedValueOnce([capture]).mockResolvedValue([]);
  api.retryCapture.mockResolvedValue({ ...capture, status: "processing", reviews: [] });
  render(<ReviewPanel />);
  fireEvent.click(await screen.findByRole("button", { name: "Retry extraction" }));
  expect(await screen.findByText("No items need review")).toBeInTheDocument();
  expect(api.retryCapture).toHaveBeenCalledWith("retry", 5);
});

// jsdom does not provide the layout APIs used by the shared command picker.
beforeAll(() => {
  vi.stubGlobal(
    "ResizeObserver",
    class {
      observe() {}
      unobserve() {}
      disconnect() {}
    },
  );
  Element.prototype.scrollIntoView = vi.fn();
});
afterAll(() => vi.unstubAllGlobals());
