import RefundLinkForm from "./refund-link-form";
import CategoryForm from "./category-form";
import { useEffect, useState } from "react";
import {
  getCaptureReviews,
  resolveCaptureReview,
  dismissCaptureReview,
  retryCapture,
  getAccounts,
  getSpendCategories,
} from "@/adapters";
import { Button } from "@wealthfolio/ui/components/ui/button";
import CorrectionForm, { type CaptureAccount } from "./correction-form";
import type { CaptureReceipt, CaptureResolution } from "./types";
export default function ReviewPanel() {
  const [page, setPage] = useState(0);
  const [reason, setReason] = useState("");
  const [captures, setCaptures] = useState<CaptureReceipt[]>([]);
  const [categories, setCategories] = useState<
    { taxonomyId: string; categoryId: string; name: string; path?: string }[]
  >([]);
  const [accounts, setAccounts] = useState<CaptureAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let active = true;
    setLoading(true);
    getSpendCategories()
      .then((data) => {
        if (active) setCategories(data);
      })
      .catch(() => {
        if (active) setError("Could not load categories.");
      });
    getAccounts()
      .then((data) => {
        if (active) setAccounts(data);
      })
      .catch(() => {
        if (active) setError("Could not load accounts.");
      });
    getCaptureReviews(page, 25, reason || undefined)
      .then((data) => {
        if (active) setCaptures(data);
      })
      .catch(() => {
        if (active) setError("Could not load review. Try again.");
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [page, reason]);
  async function complete(
    capture: CaptureReceipt,
    id: string,
    resolution: Omit<CaptureResolution, "version"> = { action: "complete" },
  ) {
    setBusy(true);
    setError("");
    try {
      const next = await resolveCaptureReview(id, { version: capture.version, ...resolution });
      if (!next.reviews.some((r) => r.status === "open" && (!reason || r.reason === reason))) {
        setCaptures(await getCaptureReviews(page, 25, reason || undefined));
      } else {
        setCaptures((all) => all.map((c) => (c.id === next.id ? next : c)));
      }
    } catch {
      try {
        setCaptures(await getCaptureReviews(page, 25, reason || undefined));
      } catch {
        /* Keep the last receipt available for manual reload. */
      }
      setError("Review changed or could not be saved. The latest available review is shown.");
    } finally {
      setBusy(false);
    }
  }
  async function act(capture: CaptureReceipt, id: string, action: "retry" | "dismiss") {
    setBusy(true);
    setError("");
    try {
      const next =
        action === "retry"
          ? await retryCapture(capture.id, capture.version)
          : await dismissCaptureReview(id, capture.version);
      if (!next.reviews.some((r) => r.status === "open" && (!reason || r.reason === reason))) {
        setCaptures(await getCaptureReviews(page, 25, reason || undefined));
      } else {
        setCaptures((all) => all.map((c) => (c.id === next.id ? next : c)));
      }
    } catch {
      try {
        setCaptures(await getCaptureReviews(page, 25, reason || undefined));
      } catch {
        /* Keep the last receipt available for manual reload. */
      }
      setError("Review changed or could not be saved. The latest available review is shown.");
    } finally {
      setBusy(false);
    }
  }
  async function reload() {
    setLoading(true);
    try {
      setCaptures(await getCaptureReviews(page, 25, reason || undefined));
      setError("");
    } catch {
      setError("Could not reload review.");
    } finally {
      setLoading(false);
    }
  }
  const open = captures.flatMap((capture) =>
    capture.reviews
      .filter((r) => r.status === "open" && (!reason || r.reason === reason))
      .map((review) => ({ capture, review })),
  );
  return (
    <section aria-label="Review" className="space-y-4">
      <label>
        Review type
        <select
          className="border-input bg-background block w-full rounded-md border p-2"
          value={reason}
          disabled={busy || loading}
          onChange={(e) => {
            setReason(e.target.value);
            setPage(0);
          }}
        >
          <option value="">All unresolved items</option>
          {[
            "account_required",
            "date_required",
            "supervised_review",
            "category_required",
            "counterpart_required",
            "original_payment_required",
            "balance_update_required",
            "possible_duplicate",
            "reference_conflict",
            "unconfirmed_event",
            "extraction_failed",
            "configuration_required",
            "budget_exhausted",
          ].map((reason) => (
            <option key={reason} value={reason}>
              {reason.replaceAll("_", " ")}
            </option>
          ))}
        </select>
      </label>
      <div className="flex items-center gap-3">
        <Button
          variant="outline"
          disabled={busy || loading || page === 0}
          onClick={() => setPage(page - 1)}
        >
          Previous page
        </Button>
        <span>Page {page + 1}</span>
        <Button
          variant="outline"
          disabled={busy || loading || captures.length < 25}
          onClick={() => setPage(page + 1)}
        >
          Next page
        </Button>
      </div>
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-medium">Review</h2>
        <Button variant="outline" disabled={busy || loading} onClick={reload}>
          Reload review
        </Button>
      </div>
      {loading && <p>Loading review…</p>}
      {error && (
        <p role="alert" className="text-destructive">
          {error}
        </p>
      )}
      {!loading && !open.length && !error && <p>No items need review</p>}
      {open.map(({ capture, review }) => (
        <article
          key={review.id}
          className="bg-card text-card-foreground space-y-3 rounded-lg border p-4"
        >
          <h3 className="font-medium">
            {review.activityId ? "Payment already saved" : "Not yet saved"}
          </h3>
          <p className="whitespace-pre-wrap break-words">{capture.input.text}</p>
          <p className="text-muted-foreground">{review.reason.replaceAll("_", " ")}</p>
          {Object.entries(
            capture.candidates.find((c) => c.id === review.id)?.sourceDates ?? {},
          ).map(([kind, date]) => (
            <p className="text-muted-foreground text-sm" key={kind}>
              {(
                {
                  transactionDate: "Transaction date",
                  bookingDate: "Booking date",
                  valueDate: "Value date",
                } as Record<string, string>
              )[kind] ?? kind}
              : {date}
            </p>
          ))}
          {Object.entries(capture.candidates.find((c) => c.id === review.id)?.fieldOrigins ?? {})
            .filter(([, origin]) => origin === "configured_default")
            .map(([field]) => (
              <p className="text-muted-foreground text-sm" key={field}>
                Using your configured {field === "accountId" ? "account" : "local date"} default
              </p>
            ))}
          {!review.activityId && (
            <CorrectionForm
              fields={capture.candidates.find((c) => c.id === review.id)?.fields}
              accounts={accounts}
              busy={busy || capture.status === "processing"}
              save={(fields) => complete(capture, review.id, { fields })}
            />
          )}
          {review.reason === "category_required" && (
            <CategoryForm
              categories={categories}
              busy={busy}
              save={(taxonomyId, categoryId) =>
                complete(capture, review.id, { action: "categorize", taxonomyId, categoryId })
              }
            />
          )}
          {review.reason === "counterpart_required" && (
            <>
              <p>
                Only the evidenced side is saved. The other account must have its own confirmed
                transaction before linking.
              </p>
              <a className="text-primary underline" href="/activities">
                Open transactions to link a counterpart
              </a>
              <Button disabled={busy} onClick={() => complete(capture, review.id)}>
                Check transfer link
              </Button>
            </>
          )}
          {review.reason === "original_payment_required" &&
            (() => {
              const candidate = capture.candidates.find((c) => c.activityId === review.activityId);
              return candidate ? (
                <RefundLinkForm
                  fields={candidate.fields}
                  busy={busy}
                  save={(relatedActivityId) =>
                    complete(capture, review.id, { action: "link_refund", relatedActivityId })
                  }
                />
              ) : (
                <p>Reload review to load the saved refund.</p>
              );
            })()}
          {review.reason === "balance_update_required" && (
            <>
              <p>
                Update the related loan or investment balance manually, then complete this
                follow-up.
              </p>
              <Button disabled={busy} onClick={() => complete(capture, review.id)}>
                Mark balance update complete
              </Button>
            </>
          )}
          <div className="flex gap-2">
            {!capture.candidates.length &&
              ["configuration_required", "budget_exhausted", "extraction_failed"].includes(
                review.reason,
              ) && (
                <Button
                  variant="outline"
                  disabled={busy || capture.status === "processing"}
                  onClick={() => act(capture, review.id, "retry")}
                >
                  Retry extraction
                </Button>
              )}
            <Button
              variant="ghost"
              disabled={busy || capture.status === "processing"}
              onClick={() => act(capture, review.id, "dismiss")}
            >
              Dismiss review
            </Button>
          </div>
        </article>
      ))}
    </section>
  );
}
