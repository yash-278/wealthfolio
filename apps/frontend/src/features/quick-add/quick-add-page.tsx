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
    <main className="mx-auto flex w-full max-w-3xl flex-col gap-6 p-4 md:p-6">
      <header className="space-y-2">
        <h1 className="text-2xl font-semibold">Quick Add</h1>
        <p className="text-muted-foreground">
          Paste bank alerts, card messages, or a note. Anything unresolved stays in review.
        </p>
      </header>
      <a href="/settings/quick-add" className="text-primary underline">
        Quick Add settings
      </a>
      <a href="/quick-add/review" className="text-primary underline">
        Open review queue
      </a>
      <form onSubmit={form.handleSubmit(submit)} className="space-y-4">
        <div className="space-y-2">
          <label htmlFor="capture-kind">Input type</label>
          <select
            id="capture-kind"
            value={kind}
            onChange={(e) => setKind(e.target.value as CaptureInputKind)}
            className="border-input bg-background block w-full rounded-md border p-2"
          >
            <option value="unknown">Pasted text</option>
            <option value="bank_alert">Bank alert</option>
            <option value="card_alert">Card alert</option>
            <option value="typed_note">Typed expense note</option>
          </select>
        </div>
        <div className="space-y-2">
          <label htmlFor="capture-text">Transaction text</label>
          <Textarea
            id="capture-text"
            value={text}
            onChange={(e) => setText(e.target.value)}
            maxLength={16000}
            rows={7}
            required
          />
        </div>
        <Button type="submit" disabled={busy || !text.trim()}>
          {busy ? "Capturing…" : "Capture"}
        </Button>
      </form>
      {error && (
        <p role="alert" className="text-destructive">
          {error}
        </p>
      )}
      {receipt && (
        <section aria-label="Capture receipt" aria-live="polite" className="space-y-4">
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
            {receipt.reviews.filter((review) => review.status === "open").length} items need review
          </p>
        </section>
      )}
    </main>
  );
}
