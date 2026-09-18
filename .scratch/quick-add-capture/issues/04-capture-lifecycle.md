# Capture lifecycle and automatic posting rules

Type: grilling
Label: wayfinder:grilling
Status: resolved
Assignee: Yash / Codex
Blocked by: 01, 03
Parent: [Quick Add transaction capture](../map.md)

## Question

What is a capture, candidate transaction, review item, posted event, estimate and correction? Decide multi-event text parsing, required evidence for account/amount/currency/date, missing dates and default account policy for typed notes, partial success, rejected/OTP/failed/pending alerts, and create versus review gates. Do not use model confidence alone as proof of correctness. Define user-visible status and explainability. Yash explicitly requires anything not directly insertable to enter a review queue; include unparseable/non-transaction text and provider/write failures, and distinguish receipts for confirmed duplicates from unresolved duplicate candidates. Decide whether one failed candidate blocks a whole text block. Accepted captures must persist before processing; clients must distinguish delivery failure or unknown receipt from a confirmed review item. The precise persistence/API contract remains part of this decision.

## Comments

## Answer

Resolved with Yash on 2026-09-11. Yash confirmed all three proposed defaults with "Yes sounds good". These refine the previously confirmed automatic-posting and review requirements.

### Posting contract

- A capture retains the submitted text and may contain multiple candidate transactions. Acceptance means the server has durably received it, not that a transaction has been posted. A delivery/authentication failure must not be reported as a queued review item.
- For independent candidates, save those that pass checks and queue only the unresolved candidates. A mixed capture can therefore have both posted transactions and outstanding review items. Financially related entries must not be presented as complete when incomplete. The later resolution in [Duplicate detection and statement reconciliation](05-dedupe-reconcile.md) permits an evidenced transfer side to post while its counterpart awaits confirmation; see that ticket for the canonical rule.
- Automatic posting requires a resolved permitted account, amount, direction, currency, transaction date and no unresolved duplicate concern. Required facts must come from the text, reliable source context or an explicitly configured default. Model confidence alone is insufficient.
- Typed notes may use a configured default account and today's date in the configured timezone. Explicit text takes precedence. Without a configured account or another unambiguous match, review is required. Bank alerts with missing details require review unless reliable source information resolves them; submission time alone does not establish the date of an old pasted alert.
- When only the category is uncertain, save the transaction as uncategorized and create a category-review item linked to that existing transaction. Resolving this review updates categorization rather than creating another activity.
- Uncertain account/amount/date, conflicting facts, unsupported events, unresolved duplicate candidates, and unsuccessful processing/writes remain reviewable with a specific reason. Pending, failed, rejected and non-transaction messages such as OTPs do not establish a completed financial event; retain them for review without changing balances.
- A confirmed duplicate produces a visible reference to the existing record, without another transaction. Exact duplicate and pending-to-posted matching belongs to Duplicate detection and statement reconciliation.
- Results distinguish accepted/processing, posted, already recorded and needs review. A block reports per-candidate outcomes and counts, including posted transactions still needing category review. No blanket success message for a partially unresolved capture.

### Review and estimates

A review item must make clear whether nothing has been posted, an existing transaction needs a category, or a related estimate remains unresolved. It carries the original text, extracted fields, reason and any existing transaction reference. Successful correction must not replay a previously successful write. Detailed editing, dismissal and retry interactions belong to Application Quick Add and Shortcut experience.

Automatic loan/investment estimates remain an explicit requirement. Their evidence, coupled-write boundaries and reconciliation are owned by Automatic loan and investment estimates; this resolution does not invent units, principal allocation or an estimate policy.

### Downstream ownership

- Duplicate detection and statement reconciliation: reference matching, retries, reversals and imported statements.
- Automatic loan and investment estimates: evidence and grouping for related balance effects.
- Configuration, correction learning and capture credentials: exact default controls, mappings and learning precedence.
- Application Quick Add and Shortcut experience: review actions, receipts and mixed-capture UI.
- Evaluation and implementation handoff: persistence mechanics, partial-write recovery and acceptance checks.

### Scope update, 2026-09-11

The estimate requirements above are superseded by [Automatic loan and investment estimates](06-related-balances.md), closed as out of scope at Yash's direction. Related balances remain manual. Yash clarified that these updates must enter the review queue linked to the captured payment. Only automatic estimation is deferred; manual balance-review follow-up remains required.
