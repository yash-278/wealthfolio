# Evaluation and implementation handoff

Type: grilling Label: wayfinder:grilling Status: resolved Assignee: Yash / Codex
Blocked by: 05, 07, 08 Parent: [Quick Add transaction capture](../map.md)

## Question

What acceptance tests, synthetic/redacted message corpus, error limits, rollout
gates and implementation slices make this feature ready to build? Cover
account/amount/date correctness, no duplicates on retry or statement import,
manual related-balance review completion without duplicate payments, learning
conflicts, permission isolation, partial failures, provider failures, cost
limits and rollback. Identify user-owned iPhone testing. Produce the final spec
only after the map decisions are resolved.

## Comments

### Confirmed UI requirement

Yash accepted the walkthrough on 2026-09-12 on condition that all website UI
follows Wealthfolio's existing design tokens. Include shared-component reuse,
existing typography/spacing/color tokens, light/dark themes, responsive behavior
and accessibility in the handoff and verification. Do not port the prototype's
standalone hardcoded styles.

## Answer

Resolved with Yash on 2026-09-12. Yash confirmed both final policy choices:

- Automatic posting requires zero wrong accounts, amounts, directions, dates or
  duplicate postings in the evaluation set, followed by a small supervised
  trial. Uncertain cases may correctly enter review.
- Reaching the configurable AI spend limit pauses AI processing while continuing
  to accept captures into review until the budget resets or Yash changes it.

[Quick Add implementation handoff](../spec.md) records the API/service contract,
persistence and recovery invariants, shared reconciliation path, configuration,
source retention, evaluation matrix, implementation stages and rollout
boundaries. Clear supported fixtures must actually post as expected; putting
everything in review is not a valid pass. Critical gates cover saved-field
correctness, duplicate exclusion, recovery and preservation of manual-review
follow-ups.

Phone validation remains user-owned. Model availability, latency and actual
usage cost require verification during implementation; this planning task
performed no paid inference, device test, migration or deployment. The accepted
experience must use Wealthfolio design tokens and components. Automatic
related-balance calculations remain out of scope; linked manual review remains
required.

### Testing seams confirmed and spec published, 2026-09-12

Yash confirmed a primary real capture/review API seam with temporary SQLite and
controlled external AI responses, plus a small UI interaction suite, separate
real-Luna evaluation, and user-owned iPhone checks. Existing authenticated
router integration tests and component interaction tests provide prior art. The
[final specification](../spec.md) now follows the to-spec template and is
labelled `ready-for-agent`. This completes specification work; no tests or
feature implementation were performed by publication.
