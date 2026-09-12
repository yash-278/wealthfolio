import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { Button } from "@wealthfolio/ui/components/ui/button";

const schema = z.object({ category: z.string().min(1, "Choose a category") });

export default function CategoryForm({
  categories,
  busy,
  save,
}: {
  categories: { taxonomyId: string; categoryId: string; name: string; path?: string }[];
  busy: boolean;
  save: (taxonomyId: string, categoryId: string) => Promise<void>;
}) {
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: { category: "" },
  });
  return (
    <form
      className="space-y-3"
      onSubmit={form.handleSubmit(async ({ category }) => {
        const selected = categories.find((c) => `${c.taxonomyId}:${c.categoryId}` === category);
        if (selected) await save(selected.taxonomyId, selected.categoryId);
      })}
    >
      <label>
        Category
        <select
          {...form.register("category")}
          className="border-input bg-background block w-full rounded-md border p-2"
        >
          <option value="">Choose category</option>
          {categories.map((c) => (
            <option
              key={`${c.taxonomyId}:${c.categoryId}`}
              value={`${c.taxonomyId}:${c.categoryId}`}
            >
              {c.path || c.name}
            </option>
          ))}
        </select>
      </label>
      {form.formState.errors.category && (
        <p role="alert" className="text-destructive">
          {form.formState.errors.category.message}
        </p>
      )}
      <Button disabled={busy || form.formState.isSubmitting} type="submit">
        Save category
      </Button>
    </form>
  );
}
