import { useEffect, useState } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { searchActivities } from "@/adapters";
import type { ActivityDetails } from "@/lib/types";
import { Button } from "@wealthfolio/ui/components/ui/button";
import type { CaptureFields } from "./types";

const schema = z.object({ originalId: z.string().min(1, "Choose the original payment") });

export default function RefundLinkForm({
  fields,
  busy,
  save,
}: {
  fields: CaptureFields;
  busy: boolean;
  save: (id: string) => Promise<void>;
}) {
  const [rows, setRows] = useState<ActivityDetails[]>([]);
  const [page, setPage] = useState(0);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: { originalId: "" },
  });
  useEffect(() => {
    let active = true;
    setLoading(true);
    searchActivities(
      page,
      25,
      { accountIds: [fields.accountId], activityTypes: ["WITHDRAWAL"], dateTo: fields.date },
      "",
    )
      .then((result) => {
        if (active) {
          setRows(result.data.filter((row) => row.currency === fields.currency));
          setTotal(result.meta.totalRowCount);
          setError("");
        }
      })
      .catch(() => {
        if (active) setError("Could not load original payments. Reload review to try again.");
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [page, fields.accountId, fields.currency, fields.date]);
  return (
    <form className="space-y-3" onSubmit={form.handleSubmit(({ originalId }) => save(originalId))}>
      <p>The refund remains a separate credit. Select its original debit to link their history.</p>
      <label>
        Original payment
        <select
          {...form.register("originalId")}
          disabled={loading || busy}
          className="border-input bg-background block w-full rounded-md border p-2"
        >
          <option value="">Choose payment</option>
          {rows.map((row) => (
            <option key={row.id} value={row.id}>
              {new Date(row.date).toLocaleDateString()} · {row.currency} {row.amount} ·{" "}
              {row.comment || row.accountName}
            </option>
          ))}
        </select>
      </label>
      {(error || form.formState.errors.originalId) && (
        <p role="alert" className="text-destructive">
          {error || form.formState.errors.originalId?.message}
        </p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button
          variant="outline"
          type="button"
          disabled={loading || page === 0}
          onClick={() => {
            form.reset();
            setPage(page - 1);
          }}
        >
          Previous payments
        </Button>
        <Button
          variant="outline"
          type="button"
          disabled={loading || (page + 1) * 25 >= total}
          onClick={() => {
            form.reset();
            setPage(page + 1);
          }}
        >
          More payments
        </Button>
        <Button type="submit" disabled={busy || loading || form.formState.isSubmitting}>
          Link refund
        </Button>
      </div>
    </form>
  );
}
