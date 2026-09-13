import { useId, useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  Button,
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  Icons,
} from "@wealthfolio/ui";

const schema = z.object({ category: z.string().min(1, "Choose a category") });
const categoryGroups = [
  { id: "spending_categories", label: "Expenses" },
  { id: "income_sources", label: "Income" },
  { id: "savings_categories", label: "Savings" },
];

export default function CategoryForm({
  categories,
  busy,
  save,
}: {
  categories: { taxonomyId: string; categoryId: string; name: string; path?: string }[];
  busy: boolean;
  save: (taxonomyId: string, categoryId: string) => Promise<void>;
}) {
  const id = useId();
  const [open, setOpen] = useState(false);
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: { category: "" },
  });
  const selectedKey = form.watch("category");
  const selected = categories.find((c) => `${c.taxonomyId}:${c.categoryId}` === selectedKey);
  const disabled = busy || form.formState.isSubmitting;
  const groups = [
    ...categoryGroups,
    ...Array.from(new Set(categories.map((c) => c.taxonomyId)))
      .filter((id) => !categoryGroups.some((group) => group.id === id))
      .map((id) => ({ id, label: "Other categories" })),
  ];

  return (
    <form
      className="space-y-3"
      onSubmit={form.handleSubmit(async () => {
        if (selected) await save(selected.taxonomyId, selected.categoryId);
      })}
    >
      <div className="space-y-2">
        <label htmlFor={id} className="text-sm font-medium">
          Category
        </label>
        <Button
          id={id}
          type="button"
          variant="outline"
          aria-haspopup="dialog"
          aria-expanded={open}
          disabled={disabled || !categories.length}
          onClick={() => setOpen(true)}
          className="h-auto min-h-11 w-full justify-between gap-3 whitespace-normal text-left"
        >
          <span className="min-w-0 break-words">
            {selected
              ? `${groups.find((g) => g.id === selected.taxonomyId)?.label} / ${selected.path || selected.name}`
              : "Choose category"}
          </span>
          <Icons.ChevronDown className="size-4 shrink-0" />
        </Button>
        {!categories.length && (
          <p className="text-muted-foreground text-sm">
            No categories available. Reload review to try again.
          </p>
        )}
      </div>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent
          onCloseAutoFocus={(event) => {
            event.preventDefault();
            document.getElementById(id)?.focus();
          }}
          className="flex max-h-[85dvh] flex-col overflow-hidden sm:max-w-lg"
          mobileClassName="flex h-[85dvh] max-h-[85dvh] flex-col overflow-hidden"
        >
          <DialogHeader>
            <DialogTitle>Choose category</DialogTitle>
            <DialogDescription>
              Browse expenses, income and savings, or search by category or parent.
            </DialogDescription>
          </DialogHeader>
          <Command className="min-h-0 flex-1">
            <CommandInput
              placeholder="Search categories…"
              aria-label="Search categories"
              className="text-base"
            />
            <CommandList className="max-h-[55dvh] min-h-0 overscroll-contain">
              <CommandEmpty>No matching categories. Try another search.</CommandEmpty>
              {groups.map((group) => {
                const items = categories
                  .filter((c) => c.taxonomyId === group.id)
                  .sort((a, b) => (a.path || a.name).localeCompare(b.path || b.name));
                return (
                  items.length > 0 && (
                    <CommandGroup key={group.id} heading={group.label}>
                      {items.map((category) => {
                        const key = `${category.taxonomyId}:${category.categoryId}`;
                        const path = category.path || category.name;
                        const parent = path.includes(" / ")
                          ? path.slice(0, path.lastIndexOf(" / "))
                          : "";
                        return (
                          <CommandItem
                            key={key}
                            value={key}
                            keywords={[path, group.label]}
                            disabled={disabled}
                            className="min-h-12 items-center gap-3 py-3"
                            onSelect={() => {
                              form.setValue("category", key, { shouldValidate: true });
                              setOpen(false);
                            }}
                          >
                            <span className="min-w-0 flex-1 whitespace-normal break-words">
                              <span className="block font-medium">{category.name}</span>
                              {parent && (
                                <span className="text-muted-foreground block text-xs">
                                  {parent}
                                </span>
                              )}
                            </span>
                            {key === selectedKey && <Icons.Check className="size-4 shrink-0" />}
                          </CommandItem>
                        );
                      })}
                    </CommandGroup>
                  )
                );
              })}
            </CommandList>
          </Command>
        </DialogContent>
      </Dialog>
      {form.formState.errors.category && (
        <p role="alert" className="text-destructive">
          {form.formState.errors.category.message}
        </p>
      )}
      <Button disabled={disabled || !selected} type="submit">
        Save category
      </Button>
    </form>
  );
}
