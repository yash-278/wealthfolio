# Quick Add transaction capture

Label: wayfinder:map Status: resolved

## Destination

An implementation-ready specification for a reusable, configurable Wealthfolio
Quick Add feature that turns pasted text into categorized account transactions,
with the same flow callable from an iPhone Shortcut. This map resolves product,
integration, safety, and delivery decisions before implementation.

## Notes

- Planning requested by Yash on 2026-09-11. No implementation, deployment,
  commits, pushes, or changes to live financial records are authorized by this
  map.
- Confirmed requirements: application Quick Add is the primary flow; iPhone
  Shortcut is an additional entry point. Accept bank alerts, credit-card alerts,
  and typed expense notes. Post automatically when account, amount, date, and
  duplicate checks pass; otherwise send to review. Provide reusable settings,
  account mappings, and Shortcut tokens, rather than hardcoding Yash's accounts.
- Explicit review requirement: anything not directly insertable goes to a
  durable review queue with source text, extracted candidates and a specific
  reason. For accepted submissions, ambiguity, unsupported inputs, duplicate
  uncertainty and write failures must not be silently dropped. A client that
  cannot reach or authenticate to the server must show delivery failure rather
  than falsely claiming an item is queued. Confirmed duplicates need a visible
  receipt rather than a second transaction. See the resolved lifecycle ticket
  for the posting contract.
- Related loan, investment or EPF balance updates require linked manual review
  items after clear payments are captured. Completing the review must not
  duplicate the original payment. Automatic related-balance calculation is
  deferred.
- GPT-5.6 Luna is the requested cost-sensitive model candidate. Model choice is
  configurable in the application; research establishes capability, costs, and
  existing provider integration.
- A block of pasted text may contain multiple events; batch behavior is recorded
  in Capture lifecycle and automatic posting rules.
- Consult Wayfinder, grilling, and domain-modeling for decision work; research
  for external facts. Read repository AGENTS.md and CONTEXT.md. Use current code
  and primary documentation as evidence.
- Local Markdown tracker convention: one child file per decision, Status and
  Assignee fields, blocking references, and answers appended on resolution.
  Open, unclaimed children whose blockers are resolved are the frontier. Titles
  are the human-facing references.
- The checkout contains unrelated, uncommitted Bedrock implementation work.
  Preserve it. Research branches must remain separate; no commits required
  without authorization. Cite working-tree evidence where it differs from HEAD.
- Further confirmed requirements: retain original submitted text for review,
  with Yash stating sensitivity is not a concern for his use; automatically
  learn from every category correction. Learning boundaries, retention controls
  and permissions are recorded in the resolved configuration ticket. Raw-message
  retention in the application does not authorize publishing messages or logging
  credentials.
- Configuration uses the existing installation-wide dataset. Separate user
  accounts and multitenancy are deferred.
- Existing bank statements and imported transactions are context, not a reusable
  test dataset. Use synthetic or explicitly redacted fixtures.

## Decisions so far

- [Existing Wealthfolio integration and ownership boundaries](issues/01-existing-integration.md):
  reuse core primitives; intake ownership, capture-scoped REST authentication,
  atomicity and cross-source reconciliation need explicit design.
- [iPhone Shortcut entry and delivery constraints](issues/02-iphone-entry.md):
  clipboard/typed-text JSON submission is supported; direct SMS sharing and
  stronger background guarantees require device validation.
- [GPT-5.6 Luna extraction and cost contract](issues/03-model-extraction.md):
  structured outputs are supported; a dedicated extraction operation and
  capability checks are required. Example cost is $0.56 per 1,000 captures at
  1,000 input and 300 output tokens each, excluding retries and hosting.

- [Capture lifecycle and automatic posting rules](issues/04-capture-lifecycle.md):
  independently clear entries post, typed notes can use configured defaults, and
  category-only uncertainty reviews an already-saved transaction.

- [Duplicate detection and statement reconciliation](issues/05-dedupe-reconcile.md):
  retries reuse their result, reliable statement matches enrich existing
  transactions, ambiguous matches require review, and transfers/refunds/pending
  completions preserve event history.

- [Configuration, correction learning and capture credentials](issues/07-settings-learning.md):
  installation-wide settings, independent model selection, scoped Shortcut
  tokens, editable future-only learning, and source retention that preserves
  unresolved review evidence.

- [Application Quick Add and Shortcut experience](issues/08-quick-add-experience.md):
  walkthrough flow accepted; all production web UI must follow existing
  Wealthfolio design tokens, shared components and themes.

- [Evaluation and implementation handoff](issues/09-handoff-gates.md):
  critical-error-free evaluation and supervised trial gate automatic posting; AI
  budget exhaustion preserves intake in review.
  [Implementation handoff](spec.md) is ready.

## Not yet specified

None for this planning destination. Implementation must verify the code, model
capability, resource limits and phone behavior against the handoff; those are
execution checks, not unresolved product decisions.

## Out of scope

- [Automatic loan and investment estimates](issues/06-related-balances.md):
  automatic calculation deferred by Yash as non-urgent; related balance updates
  must enter the review queue for manual completion.

- Implementing or deploying this feature during charting.
- Moving money, paying bills, placing trades, or fetching bank credentials.
- Replacing statement reconciliation with SMS alone.
- Publishing private bank messages, account identifiers, or credentials in
  planning artifacts.

## Specification publication

On 2026-09-12, the to-spec pass consolidated the handoff into the required
specification template with 47 user stories and explicitly confirmed testing
boundaries. [Quick Add transaction capture specification](spec.md) is the
canonical implementation input and is labelled `ready-for-agent`. The earlier
handoff is retained only as a superseded archive. No implementation or
deployment occurred.
