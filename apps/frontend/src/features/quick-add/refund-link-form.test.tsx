import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi, it, expect } from "vitest";
import { searchActivities } from "@/adapters";
import RefundLinkForm from "./refund-link-form";

vi.mock("@/adapters", () => ({ searchActivities: vi.fn() }));

it("requires the user to select an original debit before linking a saved refund", async () => {
  vi.mocked(searchActivities).mockResolvedValue({
    data: [
      {
        id: "original",
        date: new Date("2026-09-09"),
        amount: "120",
        currency: "INR",
        comment: "Test shop",
        accountName: "Test bank",
      },
    ],
    meta: { totalRowCount: 1 },
  } as Awaited<ReturnType<typeof searchActivities>>);
  const save = vi.fn().mockResolvedValue(undefined);
  render(
    <RefundLinkForm
      fields={{
        accountId: "bank",
        amount: "40",
        currency: "INR",
        date: "2026-09-10",
        direction: "credit",
        merchant: "Test shop",
        reference: null,
        kind: "refund",
      }}
      busy={false}
      save={save}
    />,
  );
  const user = userEvent.setup();
  await screen.findByRole("option", { name: /Test shop/ });
  await user.click(screen.getByRole("button", { name: "Link refund" }));
  expect(save).not.toHaveBeenCalled();
  await user.selectOptions(screen.getByRole("combobox", { name: "Original payment" }), "original");
  await user.click(screen.getByRole("button", { name: "Link refund" }));
  expect(save).toHaveBeenCalledExactlyOnceWith("original");
  expect(searchActivities).toHaveBeenCalledWith(
    0,
    25,
    { accountIds: ["bank"], activityTypes: ["WITHDRAWAL"], dateTo: "2026-09-10" },
    "",
  );
});
