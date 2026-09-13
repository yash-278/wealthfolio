import { Link } from "react-router-dom";
import { Page, PageContent, PageHeader } from "@wealthfolio/ui";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { useEffect, useRef, useState } from "react";
import { clearCaptureSource, getCapture, submitCapture } from "@/adapters";
import { Button } from "@wealthfolio/ui/components/ui/button";
import { Textarea } from "@wealthfolio/ui/components/ui/textarea";
import type { CaptureInputKind, CaptureReceipt } from "./types";

export default function QuickAddPage() {
  const form = useForm<{ text: string; kind: CaptureInputKind }>({
    resolver: zodResolver(
      z.object({
        text: z.string().trim().min(1).max(16000),
        kind: z.enum(["unknown", "bank_alert", "card_alert", "typed_note"]),
      }),
    ),
    defaultValues: { text: "", kind: "unknown" },
  });
  const { text, kind } = form.watch();
  const setText = (text: string) => form.setValue("text", text);
  const setKind = (kind: CaptureInputKind) => form.setValue("kind", kind);
  const [receipt, setReceipt] = useState<CaptureReceipt | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const pending = useRef<{ payload: string; id: string } | null>(null);
  useEffect(() => {
    if (receipt?.status !== "processing") return;
    let cancelled = false;
    const timer = setTimeout(() => {
      getCapture(receipt.id)
        .then((next) => {
          if (!cancelled) setReceipt(next);
        })
        .catch(() => {
          if (!cancelled)
            setError(
              "Capture accepted. Status is temporarily unavailable; open Review to check it.",
            );
        });
    }, 1000);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [receipt]);
  async function submit() {
    const payload = JSON.stringify({ text, kind });
    if (pending.current?.payload !== payload)
      pending.current = { payload, id: crypto.randomUUID() };
    setBusy(true);
    setError("");
    try {
      setReceipt(
        await submitCapture({ clientRequestId: pending.current.id, text, inputKind: kind }),
      );
    } catch {
      setError(
        "Receipt unconfirmed. Your text is still here. Retry to check or submit the same capture.",
      );
    } finally {
      setBusy(false);
    }
  }
  return (
    <Page>
      <PageHeader
        heading="Quick Add"
        dragRegion={false}
        actions={
          <nav aria-label="Quick Add" className="flex items-center gap-1">
            <Button asChild variant="ghost" className="min-h-11 px-3">
              <Link to="/quick-add/review" aria-label="Open review queue">
                Review
              </Link>
            </Button>
            <Button asChild variant="ghost" className="min-h-11 px-3">
              <Link to="/settings/quick-add" aria-label="Quick Add settings">
                Settings
              </Link>
            </Button>
          </nav>
        }
      />
      <PageContent className="px-4 md:px-6">
        <div className="mx-auto max-w-3xl space-y-6">
          <div className="space-y-2">
            <h2 className="text-xl font-semibold tracking-tight md:text-2xl">Add a transaction</h2>
            <p className="text-muted-foreground max-w-prose text-sm leading-relaxed">
              Paste a bank alert or card message, or describe a payment. Anything unresolved stays
              in review.
            </p>
          </div>
          <form
            onSubmit={form.handleSubmit(submit)}
            className="bg-card space-y-4 rounded-xl border p-4 sm:space-y-5 sm:p-6"
          >
            <div className="space-y-2">
              <label className="text-sm font-medium" htmlFor="capture-kind">
                Input type
              </label>
              <select
                id="capture-kind"
                disabled={busy}
                value={kind}
                onChange={(e) => setKind(e.target.value as CaptureInputKind)}
                className="border-input bg-background focus-visible:ring-ring block min-h-11 w-full min-w-0 rounded-md border px-3 py-2 text-base focus-visible:outline-none focus-visible:ring-2 sm:max-w-xs"
              >
                <option value="unknown">Pasted text</option>
                <option value="bank_alert">Bank alert</option>
                <option value="card_alert">Card alert</option>
                <option value="typed_note">Typed expense note</option>
              </select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium" htmlFor="capture-text">
                Transaction text
              </label>
              <Textarea
                id="capture-text"
                value={text}
                onChange={(e) => setText(e.target.value)}
                maxLength={16000}
                rows={4}
                className="min-h-32 resize-y text-base leading-relaxed md:min-h-44 md:text-base"
                placeholder="e.g. Paid 250 INR for lunch today from my spending account"
                aria-describedby="capture-help"
                disabled={busy}
                required
              />
            </div>
            <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
              <p id="capture-help" className="text-muted-foreground max-w-sm text-sm">
                Include the amount, account and date when you know them.
              </p>
              <Button
                className="min-h-11 w-full sm:w-auto sm:min-w-32"
                type="submit"
                disabled={busy || !text.trim()}
              >
                {busy ? "Capturing…" : "Capture"}
              </Button>
            </div>
          </form>
          {error && (
            <p role="alert" className="text-destructive">
              {error}
            </p>
          )}
          {receipt && (
            <section
              aria-label="Capture receipt"
              aria-live="polite"
              className="space-y-4 rounded-xl border p-4 sm:p-6"
            >
              <h2 className="text-lg font-medium">
                {receipt.status === "processing" ? "Accepted. Processing…" : "Capture received"}
              </h2>
              {receipt.candidates.map((candidate) => (
                <article
                  key={candidate.id}
                  className="bg-card text-card-foreground space-y-2 rounded-lg border p-4"
                >
                  <h3 className="font-medium">
                    {candidate.status === "posted"
                      ? "Payment saved"
                      : candidate.status === "already_recorded"
                        ? "Already recorded"
                        : "Not yet saved"}
                  </h3>
                  <p>
                    {candidate.fields.merchant || "Transaction"} {candidate.fields.currency}{" "}
                    {candidate.fields.amount}
                  </p>
                </article>
              ))}
              {receipt.status === "complete" && receipt.input.text && (
                <Button
                  variant="outline"
                  disabled={busy}
                  onClick={async () => {
                    setBusy(true);
                    try {
                      setReceipt(await clearCaptureSource(receipt.id, receipt.version));
                      setText("");
                    } catch {
                      setError("Could not clear source text. Reload the receipt and try again.");
                    } finally {
                      setBusy(false);
                    }
                  }}
                >
                  Delete retained source text
                </Button>
              )}
              <p>
                {receipt.reviews.filter((review) => review.status === "open").length} items need
                review
              </p>
            </section>
          )}
        </div>
      </PageContent>
    </Page>
  );
}
