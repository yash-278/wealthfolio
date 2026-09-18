import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useId } from "react";
import { Button } from "@wealthfolio/ui/components/ui/button";
import { Input } from "@wealthfolio/ui/components/ui/input";
import type { CaptureFields } from "./types";
export interface CaptureAccount {
  id: string;
  name: string;
  currency: string;
  isActive: boolean;
  isArchived: boolean;
}
export default function CorrectionForm({
  fields,
  accounts,
  busy,
  save,
}: {
  fields?: CaptureFields;
  accounts: CaptureAccount[];
  busy: boolean;
  save: (fields: CaptureFields) => void;
}) {
  const prefix = useId();
  const form = useForm<CaptureFields>({
    resolver: zodResolver(
      z.object({
        accountId: z.string().min(1, "Choose an account"),
        amount: z
          .string()
          .regex(/^\d+(\.\d+)?$/, "Enter a positive amount")
          .refine((v) => Number(v) > 0, "Enter a positive amount"),
        currency: z.string().regex(/^[A-Z]{3}$/, "Use a three-letter currency"),
        date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, "Choose a date"),
        direction: z.string().refine((v) => ["debit", "credit"].includes(v)),
        kind: z
          .string()
          .refine((v) =>
            [
              "payment",
              "loan_payment",
              "investment_payment",
              "epf_payment",
              "transfer",
              "card_payment",
              "refund",
            ].includes(v),
          ),
        merchant: z.string().nullable(),
        reference: z.string().nullable(),
      }),
    ),
    defaultValues: fields ?? {
      accountId: "",
      amount: "",
      currency: "",
      date: "",
      direction: "debit",
      merchant: null,
      reference: null,
      kind: "payment",
    },
  });
  const value = form.watch();
  const setValue = (next: CaptureFields) => {
    for (const key of Object.keys(next) as (keyof CaptureFields)[])
      form.setValue(key, next[key], { shouldDirty: true });
  };
  const selectStyle =
    "border-input bg-background block min-h-11 w-full min-w-0 rounded-md border p-2 text-base";
  return (
    <form onSubmit={form.handleSubmit(save)}>
      <fieldset disabled={busy} className="grid gap-3 sm:grid-cols-2">
        <label htmlFor={`${prefix}-account`}>
          Account
          <select
            required
            id={`${prefix}-account`}
            value={value.accountId}
            className={selectStyle}
            onChange={(e) =>
              setValue({
                ...value,
                accountId: e.target.value,
                currency: accounts.find((a) => a.id === e.target.value)?.currency ?? value.currency,
              })
            }
          >
            <option value="">Choose account</option>
            {accounts
              .filter((a) => a.isActive && !a.isArchived)
              .map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                </option>
              ))}
          </select>
        </label>
        <label htmlFor={`${prefix}-amount`}>
          Amount
          <Input
            id={`${prefix}-amount`}
            required
            inputMode="decimal"
            value={value.amount}
            onChange={(e) => setValue({ ...value, amount: e.target.value })}
          />
        </label>
        <label htmlFor={`${prefix}-currency`}>
          Currency
          <Input
            id={`${prefix}-currency`}
            required
            value={value.currency}
            onChange={(e) => setValue({ ...value, currency: e.target.value.toUpperCase() })}
          />
        </label>
        <label htmlFor={`${prefix}-date`}>
          Date
          <Input
            id={`${prefix}-date`}
            required
            type="date"
            value={value.date}
            onChange={(e) => setValue({ ...value, date: e.target.value })}
          />
        </label>
        <label htmlFor={`${prefix}-direction`}>
          Direction
          <select
            id={`${prefix}-direction`}
            value={value.direction}
            className={selectStyle}
            onChange={(e) => setValue({ ...value, direction: e.target.value })}
          >
            <option value="debit">Money out</option>
            <option value="credit">Money in</option>
          </select>
        </label>
        <label htmlFor={`${prefix}-kind`}>
          Payment kind
          <select
            id={`${prefix}-kind`}
            value={value.kind}
            className={selectStyle}
            onChange={(e) => setValue({ ...value, kind: e.target.value })}
          >
            <option value="payment">Payment</option>
            <option value="transfer">Owned-account transfer</option>
            <option value="card_payment">Credit-card settlement</option>
            <option value="refund">Refund or reversal</option>
            <option value="loan_payment">Loan payment</option>
            <option value="investment_payment">Investment contribution</option>
            <option value="epf_payment">EPF contribution</option>
          </select>
        </label>
        <label htmlFor={`${prefix}-merchant`}>
          Merchant
          <Input
            id={`${prefix}-merchant`}
            value={value.merchant ?? ""}
            onChange={(e) => setValue({ ...value, merchant: e.target.value || null })}
          />
        </label>
        <label htmlFor={`${prefix}-reference`}>
          Bank reference
          <Input
            id={`${prefix}-reference`}
            value={value.reference ?? ""}
            onChange={(e) => setValue({ ...value, reference: e.target.value || null })}
          />
        </label>
        {Object.entries(form.formState.errors).map(([key, error]) => (
          <p key={key} role="alert" className="text-destructive">
            {error.message}
          </p>
        ))}
        <Button type="submit">Save payment</Button>
      </fieldset>
    </form>
  );
}
