# Duplicate detection and statement reconciliation

Type: grilling
Label: wayfinder:grilling
Status: resolved
Assignee: Yash / Codex
Blocked by: 01, 04
Parent: [Quick Add transaction capture](../map.md)

## Question

How are transport retries, repeated SMS messages, both sides of a transfer, reversals/refunds and later statement imports reconciled without double counting? Separate request identity from financial-event identity. Decide reference normalization, ambiguous same-amount purchases, pending-to-posted transitions, correction history, concurrency-safe writes, and handling of imported transactions that lack reliable references.

## Comments

### Confirmed first round, 2026-09-11

Yash confirmed all three defaults with "Yes, that works for me."

- Later statement evidence attaches to the existing transaction when account, reliable reference, amount and currency match. Preserve user category corrections. Conflicting amounts or uncertain matches go to review.
- Same-day, same-amount payments without reliable references are possible duplicates for review. Amount/date/merchant similarity alone must not merge two real purchases. Retrying the same submission returns its existing result without another write.
- Transfers between owned accounts have two linked account entries and are excluded from income and spending. If only one alert arrives, record only the evidenced side and mark its counterpart awaiting confirmation. This refines the lifecycle ticket's grouping rule: a transfer may be partially evidenced, but must not be represented as complete and the missing side must not be fabricated.

The remaining branches were confirmed in the second round below.

### Confirmed second round, 2026-09-11

Yash confirmed both remaining defaults with "Yes".

- Refunds and reversals preserve the original payment and add a linked credit, including partial refunds. Unclear matches go to review.
- Pending alerts remain in review without changing balances. A reliably matching completion alert posts the transaction once and resolves the pending item. Conflicting details remain in review.

## Answer

Resolved with Yash on 2026-09-11 through the two rounds above. Those confirmations are the canonical reconciliation policy.

Request identity and financial-event identity are separate: the same submission retry returns its existing outcome, while independent submissions and statement imports are compared using transaction evidence. Reliable references must be scoped to the account and compared alongside amount and currency; similarity alone is not proof of a duplicate. Preserve source evidence and user category corrections when attaching a statement to an existing transaction. Uncertain matches require review rather than destructive merging.

The later-confirmed one-sided transfer policy refines the earlier lifecycle grouping rule. An evidenced side can be posted with its counterpart awaiting confirmation, but the transfer cannot be shown as complete or a missing balance movement invented.

Implementation must enforce duplicate exclusion across concurrent writes and retry responses; a preliminary lookup alone is insufficient. Exact bank-reference normalization, candidate-date windows, reversal categorization, correction audit storage, and transaction boundaries must be specified and tested in the implementation handoff under this policy. These details must not weaken the confirmed review rule or treat amount/date/merchant similarity as sufficient for an automatic merge.
