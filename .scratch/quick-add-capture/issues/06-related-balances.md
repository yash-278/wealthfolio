# Automatic loan and investment estimates

Type: grilling Label: wayfinder:grilling Status: resolved Resolution:
out-of-scope Assignee: Yash / Codex Blocked by: 01, 04 Parent:
[Quick Add transaction capture](../map.md)

## Question

Yash wants automatic loan principal reductions and investment changes. Which
evidence and configured relationships permit these updates, which remain
provisional, and how do confirmed statements replace estimates? Cover known EMI
schedules, prepayments/fees/arrears, card settlement, investment debits lacking
units/NAV, EPF withdrawals, and linked asset/cash double counting. Define
reversible multi-record updates and reconciliation rather than silently
converting estimates into confirmed holdings.

## Comments

## Answer

Closed as out of scope on 2026-09-11. Yash said this is not urgent and can be
manual, superseding the earlier request for automatic loan and investment
estimates. Quick Add records evidenced account transactions under the agreed
posting/review rules. Related loan principal, investment holdings and EPF
balance updates remain manual for this release. No automatic estimate engine or
estimate-correction workflow is required. This does not defer Quick Add itself,
categorization, duplicate reconciliation or evidenced account transfers.

Automatic related-balance updates may be considered in a future effort if Yash
promotes them; they are not a blocker or remaining fog in this map.

### Clarification, 2026-09-11

Yash clarified that related balance updates should be added to the review queue.
Automatic balance calculation remains out of scope, but manual follow-up is
required: save an independently clear payment under the normal rules and create
a linked review item for its unresolved related loan, investment or EPF balance
update. Do not recapture the payment when resolving that item, change the
related balance automatically, or require a separate manual reminder outside
Quick Add. If the payment itself is unresolved, retain it in review under the
ordinary posting contract.
