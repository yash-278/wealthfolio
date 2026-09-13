import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeAll, afterAll, expect, it, vi } from "vitest";
import CategoryForm from "./category-form";

const categories = [
  {
    taxonomyId: "income_sources",
    categoryId: "interest",
    name: "Interest",
    path: "Investments / Interest",
  },
  {
    taxonomyId: "spending_categories",
    categoryId: "coffee",
    name: "Coffee",
    path: "Food / Coffee",
  },
  {
    taxonomyId: "savings_categories",
    categoryId: "fund",
    name: "Emergency fund",
    path: "Reserves / Emergency fund",
  },
];
it("groups categories, searches parent paths, and saves the selected IDs only on confirmation", async () => {
  const save = vi.fn().mockResolvedValue(undefined);
  render(<CategoryForm categories={categories} busy={false} save={save} />);
  fireEvent.click(screen.getByLabelText("Category"));
  const dialog = await screen.findByRole("dialog");
  for (const label of ["Expenses", "Income", "Savings"])
    expect(within(dialog).getByText(label, { exact: true })).toBeVisible();
  fireEvent.change(within(dialog).getByRole("combobox"), { target: { value: "Food" } });
  expect(within(dialog).getByRole("option", { name: /Coffee/ })).toBeVisible();
  expect(within(dialog).queryByRole("option", { name: /Interest/ })).not.toBeInTheDocument();
  fireEvent.click(within(dialog).getByRole("option", { name: /Coffee/ }));
  expect(save).not.toHaveBeenCalled();
  expect(screen.getByLabelText("Category")).toHaveTextContent("Expenses / Food / Coffee");
  fireEvent.click(screen.getByRole("button", { name: "Save category" }));
  await vi.waitFor(() => expect(save).toHaveBeenCalledWith("spending_categories", "coffee"));
});
it("keeps selection unchanged when search is empty and the picker is dismissed", async () => {
  render(<CategoryForm categories={categories} busy={false} save={vi.fn()} />);
  fireEvent.click(screen.getByLabelText("Category"));
  fireEvent.change(await screen.findByRole("combobox"), { target: { value: "NoSuchCategory" } });
  expect(await screen.findByText("No matching categories. Try another search.")).toBeVisible();
  fireEvent.keyDown(screen.getByRole("combobox"), { key: "Escape" });
  expect(screen.getByRole("button", { name: "Save category" })).toBeDisabled();
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
