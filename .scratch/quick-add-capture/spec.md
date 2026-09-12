# Quick Add transaction capture

Status: ready-for-agent Label: ready-for-agent Updated: 2026-09-12 Parent:
[Quick Add transaction capture map](map.md)

## Problem Statement

Recording payments from bank messages or memory requires manually choosing an
account, entering the amount and date, and assigning a category. Capturing the
same payment from a message and a later statement risks duplicates. Unclear
details and related balance updates need a visible place for follow-up rather
than being forgotten or guessed.

## Solution

Add Quick Add to Wealthfolio for pasted bank/card alerts and typed notes, with
an iPhone Shortcut calling the same capture flow. Save eligible transactions
automatically, return clear per-item receipts and retain unresolved work in a
review queue. Keep settings, model choice, account mappings, correction learning
and client credentials in the application. Use the existing design system.
Related loan, investment and EPF balance adjustments remain manual review tasks.

## User Stories

1. As a Wealthfolio user, I want to paste a bank alert into Quick Add, so that
   the payment is recorded without retyping its fields.
2. As a Wealthfolio user, I want to paste a credit-card alert, so that the event
   reaches the correct card account.
3. As a Wealthfolio user, I want to type a plain-language expense note, so that
   I can record spending without a bank alert.
4. As a Wealthfolio user, I want to submit a block containing several events, so
   that I can capture multiple transactions in one action.
5. As a Wealthfolio user, I want to have clear independent entries saved
   automatically, so that one ambiguous entry does not block the rest.
6. As a Wealthfolio user, I want to see per-entry results and outstanding review
   counts, so that I know what is saved and what remains unfinished.
7. As a Wealthfolio user, I want to have unclear or unsupported accepted input
   retained for review, so that my submission does not disappear.
8. As a Wealthfolio user, I want to see extracted fields and the reason for
   review alongside original text, so that I can resolve uncertainty accurately.
9. As a Wealthfolio user, I want to correct an unposted candidate and save it,
   so that its account balance changes only after required checks pass.
10. As a Wealthfolio user, I want to save a clear transaction even when its
    category is uncertain, so that financial records stay current while
    categorization awaits review.
11. As a Wealthfolio user, I want to resolve category review on the existing
    transaction, so that categorization does not add a second payment.
12. As a Wealthfolio user, I want to configure account suffixes and aliases, so
    that alerts map to my existing accounts.
13. As a Wealthfolio user, I want to send ambiguous account matches to review,
    so that similar suffixes do not send payments to the wrong account.
14. As a Wealthfolio user, I want to configure a default account and local date
    for typed notes, so that short notes require less detail.
15. As a Wealthfolio user, I want to keep bank alerts from silently inheriting
    typed-note defaults, so that old or incomplete alerts do not receive
    invented details.
16. As a Wealthfolio user, I want to have paid amounts distinguished from
    balances and limits, so that only the financial event affects my accounts.
17. As a Wealthfolio user, I want to retry a submission after an uncertain
    response, so that I can recover without creating another transaction.
18. As a Wealthfolio user, I want to see a reference to an already recorded
    event, so that duplicate messages have a clear outcome.
19. As a Wealthfolio user, I want to reconcile statement rows with captured
    transactions in either import order, so that SMS capture and statement
    import do not double-count payments.
20. As a Wealthfolio user, I want to retain category corrections when statement
    evidence is attached, so that later imports preserve my decisions.
21. As a Wealthfolio user, I want to review similar payments without reliable
    references, so that two genuine purchases are not silently merged.
22. As a Wealthfolio user, I want to link both sides of a transfer between my
    accounts, so that moving my own money is not counted as income and spending.
23. As a Wealthfolio user, I want to record an evidenced transfer side while its
    counterpart awaits confirmation, so that missing alerts do not fabricate
    balance movements.
24. As a Wealthfolio user, I want to retain the original payment when a refund
    or reversal arrives, so that full and partial credits preserve the event
    history.
25. As a Wealthfolio user, I want to keep pending alerts out of balances until
    completion is confirmed, so that an authorization does not become a
    completed payment prematurely.
26. As a Wealthfolio user, I want to receive linked manual review for related
    loan, investment or EPF updates, so that follow-up balance work is visible
    without automatic estimates.
27. As a Wealthfolio user, I want to complete manual balance review without
    reposting the payment, so that the account transaction remains recorded
    once.
28. As a Wealthfolio user, I want to have merchant corrections inform future
    captures automatically, so that repeated merchants need less manual
    categorization.
29. As a Wealthfolio user, I want to inspect, edit and reset learned rules, so
    that I can correct unwanted learning.
30. As a Wealthfolio user, I want to give explicit rules priority and review
    ambiguous conflicts, so that learning does not override intentional
    settings.
31. As a Wealthfolio user, I want to keep rule changes from silently
    recategorizing history, so that past corrections remain stable.
32. As a Wealthfolio user, I want to select a Quick Add model independently of
    chat, so that I can use a suitable inexpensive extraction model.
33. As a installation owner, I want to reuse configured AI-provider credentials,
    so that I do not manage another copy of the same secret.
34. As a installation owner, I want to configure an AI spending limit, so that
    capture costs remain visible and controlled.
35. As a Wealthfolio user, I want to retain new captures in review when AI
    processing pauses, so that budget exhaustion does not lose my input.
36. As a Wealthfolio user, I want to retain original text until deletion with
    optional cleanup, so that I can review mistakes under my chosen retention
    policy.
37. As a Wealthfolio user, I want to preserve transactions and unresolved
    evidence during cleanup, so that removing old source text does not damage
    financial records.
38. As a iPhone user, I want to submit copied or typed text through a Shortcut,
    so that I can capture transactions on the go.
39. As a iPhone user, I want to receive a clear receipt and application link, so
    that I can distinguish saved work, duplicates and review.
40. As a iPhone user, I want to see delivery failures without a false queued
    message, so that I know when my text has not been confirmed received.
41. As a installation owner, I want to create named revocable capture-only
    Shortcut tokens, so that each client has limited access that I can withdraw.
42. As a installation owner, I want to prevent one token from reading another
    token’s capture results or account history, so that capture access does not
    expose unrelated financial data.
43. As a Wealthfolio user, I want to use Quick Add and review with existing
    design tokens and shared components, so that the feature feels consistent
    with the application.
44. As a Wealthfolio user, I want to use the flow on mobile, with a keyboard and
    in either theme, so that capture and review remain accessible.
45. As a Wealthfolio user, I want to recover accepted captures after provider
    failures or server restarts, so that unfinished processing does not lose
    work or duplicate writes.
46. As a Wealthfolio user, I want to explicitly dismiss non-actionable review
    items, so that I can clear irrelevant text without deleting legitimate
    transactions.
47. As a installation owner, I want to qualify model changes and posting
    behavior before enabling automation, so that automatic capture meets the
    agreed correctness standard.

## Implementation Decisions

### Application structure

Follow the existing React adapter → Axum/Tauri → Rust service → SQLite
architecture. Use a shared capture service for in-app and Shortcut submission,
validation, review and outcomes. Keep HTTP handlers thin. In-app desktop entry
can reuse shared services; remote Shortcut access requires the reachable web
server, not an assumed always-on desktop listener.

Reuse core activity validation, category assignment/rules, account lookup,
provider configuration, secret storage, domain events and idempotency
primitives. Do not insert activities with raw SQL or route unattended capture
through a chat commit adapter that strips provenance. Current MCP credentials do
not authenticate ordinary REST endpoints. Current activity and category writes
are separate transactions, so their composition must be made safe explicitly.

### Capture records and processing

Introduce the minimum persistent records needed to preserve accepted text,
financial candidates and retry outcomes:

| Record             | Required responsibilities                                                                                                                                                                                       |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Capture            | ID, originating principal/token, client request ID, immutable payload hash, source type, original text, submitted time, supplied message metadata, configured timezone, processing state and processing version |
| Candidate          | Stable identity within a capture, source span, extracted fields with evidence/default origin, rule/model versions, validation reasons, existing/new activity IDs and duplicate or transfer relationships        |
| Review item        | Kind, reason, capture/candidate reference, optional existing activity reference, unresolved/resolved/dismissed status and resolution history                                                                    |
| Source evidence    | Relationship from capture or statement evidence to a financial event, preserving reliable source references and any normalization version                                                                       |
| Resolution receipt | Idempotent result of a review action or submission; it cannot replay a previously successful transaction write                                                                                                  |

These may share tables where the invariants remain clear. Do not create a second
financial ledger: existing activity records remain authoritative for account
transactions. Retaining text in capture records does not imply including it in
application logs or model telemetry.

Processing sequence:

1. Authenticate, check input size and the request identity. The same
   principal/request ID and payload returns its existing capture. Reusing the ID
   with a different payload returns a conflict without another write.
2. Persist the accepted capture before invoking the model. Return its ID
   promptly. A failed persistence attempt is not acceptance.
3. A durable worker claims the capture with a lease. Persisted unprocessed work
   is rediscovered after restart. Bound attempts; exhausted/model failures
   become reviewable rather than retrying indefinitely.
4. Extract candidates in one bounded, tool-free structured model operation.
   Handle refusals, truncation, invalid output and unsupported provider
   capability as review outcomes. Preserve unprocessed text in a review item if
   no usable candidates result.
5. Resolve accounts, dates, currency, categories and financial-event matches in
   application code against approved configuration. The model proposes fields;
   it cannot create accounts, change settings or grant itself permission.
6. Post independent eligible candidates. Save an eligible but uncategorized
   transaction with a linked category-review item. Save a clear payment with a
   linked manual balance-review item when the related asset/liability needs
   updating.
7. Return per-candidate receipts and capture-level counts. Counts may include
   saved transactions with outstanding review. Never use a single success flag
   to imply every item is finished.

For write safety, commit each candidate's activity, source links and necessary
review state atomically where the storage boundary supports it. If an existing
domain operation cannot be composed atomically, persist its activity ID before
retrying subsequent work and resume only the missing step. The implementation
must demonstrate crash recovery at each boundary; lookups followed by
independent inserts are not sufficient protection against concurrent duplicates.
Keep expensive model requests outside database transactions.

### HTTP contract

Resolve final route names against existing routing conventions during
implementation; these describe the intended semantics.

| Endpoint                                    | Behavior                                                                                                                            |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `POST /api/v1/captures`                     | Accept text and metadata; return durable capture ID and current processing/result state. Requires request idempotency key.          |
| `GET /api/v1/captures/{id}`                 | Retrieve allowed capture state, candidate outcomes and unresolved work.                                                             |
| `GET /api/v1/capture-reviews`               | Session-authenticated application queue with filters and pagination.                                                                |
| `POST /api/v1/capture-reviews/{id}/resolve` | Validate a versioned correction, classification, duplicate decision or manual-follow-up completion; return an idempotent receipt.   |
| `POST /api/v1/capture-reviews/{id}/dismiss` | Explicitly dismiss a non-actionable item while retaining the appropriate decision history. Never silently delete a posted activity. |
| `POST /api/v1/captures/{id}/retry`          | Reprocess only eligible failed/unresolved work. Never replay completed writes.                                                      |

Example submission fields: `text`, `inputKind` (`bank_alert`, `card_alert`,
`typed_note`, or `unknown`), `clientRequestId`, optional original sender/message
time, and submitted time. Use server installation timezone/defaults; validate
any client hints rather than accepting them as authority. The Shortcut cannot
fabricate an original message timestamp from the time it ran. Unknown source
types do not get typed-note defaults automatically.

Responses identify `processing`, `complete`, or `needs_review`, plus
per-candidate outcomes such as `posted`, `already_recorded`, `needs_review`, and
existing activity references. A posted candidate may also have linked review
IDs. Invalid authentication, rejected request shape/size and unconfirmed
delivery must be distinguishable from accepted captures. Rejected input should
remain available in the client for correction or retry.

Only the application session may edit/review settings and transactions. Shortcut
tokens can submit captures and read their own results; possession of an
arbitrary capture ID is not sufficient. Reuse token hashing, expiry and
revocation primitives with a capture-specific scope and ownership check, not
broad MCP financial permissions. Provide named tokens, one-time display,
last-use status and revoke/replace actions. No model API key in the Shortcut.

### Parsing and account resolution

- Use decimal strings for amounts and explicit currency. Do not confuse
  available balance, credit limit, fees or a quoted installment with the paid
  amount.
- Account matching uses configured bank/card suffixes and aliases; collisions
  remain reviewable. A missing account is never created automatically.
- Typed notes can use the configured default account and today's local date.
  Explicit facts override defaults. Bank alerts require source-supported details
  or review.
- Preserve separate transaction/value/booking dates when supplied. Do not move
  an event's date solely to force a match.
- Use existing cash-flow categories and transaction semantics. A card purchase
  affects its card account; a card settlement or owned-account transfer is not
  ordinary spending twice.
- Keep submitted text as data. Instructions embedded in a message must not
  affect the extraction policy, credentials, tools, account visibility or write
  permissions.

### Reconciliation implementation constraints

Maintain request deduplication separately from financial-event matching. Use
existing database uniqueness for explicit stable keys and add the minimum
persisted capture keys necessary for response replay. Never use the model to
pronounce an uncertain duplicate definitively matched.

Normalize bank references conservatively: trim known wrappers and whitespace
while preserving leading zeros and meaningful characters. Keep the raw reference
alongside the normalized value and its source. Add bank-specific transforms only
with fixtures demonstrating equivalence. Unknown, missing or colliding
references use review; a new generic normalization rule must not merge unrelated
events.

An automatic financial match needs a unique compatible existing event with the
same resolved account, direction, amount, currency and reliable reference.
Contradictory date or event evidence requires review. Statement and capture
ingestion must both call the matching path so import order does not matter.
Preserve explicit user categories and source history; do not blindly overwrite
manually corrected values with a later import.

Same-amount/day/merchant similarity is a candidate-retrieval signal only. Two
genuine payments must remain distinct unless evidence or review establishes
otherwise. A posted credit from a refund/reversal remains a separate linked
event; partial refunds do not erase the original debit. Link to the original
only on reliable evidence. Pending messages create no balance change until
matched completion.

One evidenced side of an owned-account transfer can be recorded with an
awaiting-counterpart review state. Do not fabricate the other side. When the
counterpart arrives, attach it to the transfer rather than adding either side
twice. Manual loan, investment and EPF balance review is separate from the
account payment and does not automatically infer principal, units or returns.

### User experience and configuration

Implement the accepted interaction walkthrough using the current application
design system. Its standalone HTML/CSS is a behavior reference only. Use
existing design tokens in the existing global theme tokens and shared UI
component library, established dialogs/forms, typography, spacing and light/dark
theme behavior.

- Quick Add accepts one message or a block; show processing and per-item
  outcomes, with the original input recoverable.
- Review explicitly distinguishes "Not yet saved", "Payment already saved" and
  manual balance follow-up. Editing a category updates an existing transaction.
  Manual follow-up completion does not add another payment.
- Provide correction, retry and explicit dismissal actions appropriate to the
  review kind. Stale concurrent review versions return a conflict and reload;
  they do not overwrite a newer resolution.
- Settings group model/provider, account mappings, typed-note defaults,
  automatic posting, learned rules, source retention and Shortcut tokens.
- Learned categories apply to future same-merchant/same-scope captures. Explicit
  user rules win. New corrections replace older learned rules at that scope;
  ambiguous conflicts go to review. Resetting learned rules does not undo
  historical corrections.
- Source retention defaults to keeping text until deletion. Cleanup does not
  remove saved financial records or evidence for unresolved review.
  Duplicate-prevention identifiers must remain available after text cleanup.

Shortcut onboarding uses copied/pasted or typed text and a reachable HTTPS
endpoint. A compatible app may supply share-sheet text; do not promise a direct
Messages share action. The template contains no real credentials. Provide a
bounded receipt check and an application link; an accepted capture continues
server-side after the Shortcut closes. Network failures say receipt is
unconfirmed. Reusing a request ID on retries and server event matching protect
against a lost response. Optional message automation is not a first-release
guarantee and requires separate phone testing.

### Model and cost controls

Use AWS Bedrock Mantle `openai.gpt-5.6-luna` for strict structured extraction
with reasoning disabled, following the user decision on 2026-09-12. Use the
verified `https://bedrock-mantle.us-east-1.api.aws/openai/v1` endpoint by
default. Quick Add model selection is independent of chat, while
credential/configuration resolution is shared. Other models/providers require
equivalent schema/error handling and evaluation before automatic posting. Do not
send the AWS key to OpenAI endpoints.

Record token usage, selected model/version, schema version and validation
outcomes without logging raw messages. Bound input size, candidates per block,
output tokens, model duration, retries and concurrency. Oversized requests are
rejected before acceptance with a clear recoverable client error; accepted
captures that cannot finish remain reviewable. Initial engineering limits are
16,000 characters per submission, 20 candidate events, 4,000 output tokens, a
45-second model-call timeout, at most two attempts for transient model transport
failures, and two concurrent extraction calls per installation. Validate these
limits against the corpus before release. Candidate overflow or truncation must
preserve the whole accepted capture in review, without posting a silently
incomplete extraction. Invalid model content is not retried indefinitely. Keep
these limits centrally configurable; they are resource controls, not model
correctness guarantees.

The spend limit is configurable per installation. Reserve a conservative
per-call allowance before dispatch and reconcile it with returned usage so
simultaneous calls cannot knowingly exceed the remaining configured budget.
Treat missing usage and in-flight calls conservatively. Costs remain estimates
subject to provider billing; do not claim an absolute financial ceiling from an
application-side estimate.

Confirmed on 2026-09-12: require zero wrong accounts, amounts, directions, dates
or duplicate postings in the evaluation set, with uncertain cases correctly sent
to review. Enable automatic posting only after a small supervised trial.
Reaching the configured AI spending limit pauses model processing while
continuing to accept captures into review. Processing can resume when the limit
resets or the user changes it. Budget-blocked captures stay visible and are
retried without duplicating existing writes. Capture-specific model
configuration and spending limit must be set during setup; until then, accepted
input remains in review with a configuration reason.

As of 2026-09-12, Bedrock in-region rates are $0.22 per million input tokens and
$1.32 per million output tokens. An illustrative 1,000 input plus 300 output
tokens costs $0.000616, excluding retries and hosting. These are not measured
production costs.
[AWS model documentation](https://docs.aws.amazon.com/bedrock/latest/userguide/model-card-openai-gpt-56-luna.html).

### Delivery slices

1. **Persistent intake and review foundations.** Shared service, schema
   migration, request receipts, source provenance, worker recovery and
   synthetic-only processor. Verify same-request concurrency and crash recovery
   before model integration.
2. **Extraction and posting.** Model selector/capability handling,
   account/default resolution, deterministic validation, per-candidate activity
   writes, category and manual follow-up review. Reuse core services and
   preserve domain recalculation events.
3. **Financial reconciliation and learning.** Shared SMS/statement matching,
   transfer/refund/pending handling, rule precedence and corrected-history
   preservation. Exercise both import orders.
4. **Application experience.** Quick Add, mixed receipts, review actions and
   configuration using existing design tokens and components. Add scope-limited
   Shortcut credential management.
5. **Shortcut and end-to-end qualification.** Blank configurable template, setup
   instructions, synthetic API test and user-owned phone acceptance. Document
   network failure and revoked-token behavior.
6. **Supervised rollout and handoff.** Run the evaluation gates, verify data
   backup/recovery, and perform a supervised trial covering clear capture,
   ambiguity, category correction, duplicate resubmission, mixed input and
   manual balance follow-up. Compare every proposed/saved field and balance
   effect against its input before broader activation, then enable the confirmed
   automatic-posting mode. Keep a feature control that stops posting without
   destroying accepted review work.

Use repository-prescribed validation commands, with targeted Rust
service/storage and frontend coverage during development. Complete relevant
lint/type checks and web/desktop compilation for shared code before handoff. Do
not add tests solely to raise test counts or use production transactions to
satisfy regression fixtures.

### Rollout and recovery

Migrate with a recoverable database backup and compatibility plan. Exercise
upgrade/restart in a disposable environment first. Disable capture/automatic
posting if invariants fail, but keep accepted items and receipts available for
review. Do not delete captures or reverse legitimate payments as a deployment
rollback strategy. Fix incorrect posting through an auditable, deliberate
correction path.

No deployment or iPhone UAT is performed by this planning task. The user owns
phone testing unless separately delegated. Production release requires evidence
for the acceptance matrix, actual model-account availability and the configured
provider, plus explicit deployment authorization.

## Testing Decisions

### Primary seam: capture and review API

Confirmed with Yash on 2026-09-12: exercise the real application router and
shared capture service with a temporary SQLite database and controlled responses
at the external AI boundary. Assert observable API results, saved activity
values/counts, review state, source reconciliation and account balance effects.
Do not mock away storage, account resolution, financial writes or duplicate
matching. A controlled AI transport should also exercise the request schema and
refusal/truncation handling; deterministic application tests do not establish
real-model accuracy.

Use this seam for intake, resolution, retries, token ownership, simultaneous
submissions, statement import in either order, transfer/refund/pending handling,
retention and budget exhaustion. Restart against the same temporary database to
verify recovery. Add only minimal fault injection at the actual external I/O or
write boundary needed to expose partial failures.

Prior art: the existing agent-access integration tests start the real
authenticated router with an isolated database and exercise token scopes over
HTTP; income-summary API tests build application state against a temporary
database and assert router responses. Extend these established patterns rather
than introducing a parallel application harness. Existing activity service and
SQLite agent repository tests can supply fixture and persistence patterns,
without duplicating the entire scenario suite at every layer.

### Small UI seam

Use the existing Vitest and Testing Library interaction style for the Quick Add
view, review actions and configuration controls. Cover the user-visible
distinction between saved and unsaved entries, mixed receipts, category
correction without reposting, manual follow-up completion, error messages and
editable learned rules. Prior art includes account-holdings component tests and
adapter command-parity tests. Keep assertions on rendered behavior and user
actions rather than component internals or CSS class snapshots. Verify shared
adapters where web/desktop behavior diverges.

### Real-provider evaluation and phone acceptance

Run a separate synthetic held-out corpus against the selected real Luna
integration to measure extraction correctness and usage. Provider calls are an
explicit evaluation activity, not a requirement for every unit or integration
test. Do not claim deterministic AI stubs prove accuracy. Phone acceptance stays
user-owned and covers copied/typed input, permissions, connectivity, receipts
and token revocation. Neither evaluation was performed during specification.

### Acceptance matrix

Use synthetic examples based on supported message structures, with fixture
labels for expected account, amount, direction, currency, date, event identity,
expected posting/review behavior and provenance. Real bank messages are not
committed as fixtures. Keep a held-out set distinct from prompt examples and
learning examples.

| Case family                                              | Required observation                                                                                   |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Clear bank/card alerts and typed notes                   | Correct fields, account and classification/default evidence; no invented account or transaction date   |
| Mixed blocks                                             | Independently valid events post; unresolved parts remain visible; no complete receipt for missing work |
| Balances/limits embedded in alerts                       | Paid amount is distinguished from balance and credit limit                                             |
| Unknown category                                         | One saved event plus linked category review; resolving it leaves activity count unchanged              |
| Missing/conflicting account/date/currency                | Review rather than incorrect automatic posting                                                         |
| Identical request retries, lost response and concurrency | Stable capture receipt and at most one financial write per event                                       |
| SMS then statement, and statement then SMS               | Reliable matches attach evidence; manual corrections survive; uncertain matches review                 |
| Two genuine same-day same-amount purchases               | No automatic merge based on similarity                                                                 |
| Transfers, card settlements and pending completion       | No fabricated counterpart, doubled spending or repeated completed event                                |
| Full/partial refunds and reversals                       | Original debit remains; confirmed credit is separate and linked appropriately                          |
| Manual loan/investment/EPF follow-up                     | Payment saved once; follow-up remains reviewable; no automatic related balance adjustment              |
| Correction learning                                      | Explicit rule wins; newer same-scope learning replaces older; history remains unchanged                |
| Malformed output, refusal, timeout, provider failure     | Accepted source survives and review/retry is actionable; no partial false success                      |
| Crash during posting/classification/review completion    | Restart resumes missing work without duplicate activities or lost review state                         |
| Revoked/wrong token and guessed capture ID               | No unauthorized submission/result access; no access to history/settings                                |
| Source cleanup                                           | Transactions, duplicate protection and unresolved evidence survive                                     |
| UI tokens/themes/keyboard/mobile                         | Existing design-system behavior, usable review controls and accurate receipts                          |

Measure exact-field accuracy among automatically posted events, false-post rate,
review rate, duplicate errors, per-capture usage/cost and p50/p95 processing
latency. Do not equate strict-schema output with financial accuracy. For
explicitly supported, unambiguous fixtures, require the expected automatic
posting, not merely an absence of wrong postings. Sending every message to
review fails the functional gate. For ambiguous fixtures, require the expected
review reason and no unjustified financial write. Report sample size and
bank/template coverage; zero errors on a finite set is not proof of universal
accuracy. Prompt/model changes rerun the regression corpus.

### Validation and completion evidence

The implementation handoff must state which code revision was tested, corpus
size/coverage, critical errors, review rate, measured costs/latencies,
idempotency/crash checks, migrated-schema checks, theme/accessibility checks and
phone-test results. Distinguish local tests, staging, deployment and unverified
checks. Do not mark the feature ready solely because a model returned plausible
JSON.

## Out of Scope

- Automatic loan principal calculations, inferred investment units/NAV changes
  and automatic EPF balance adjustments. Linked manual review remains required.
- New multi-user accounts or multitenancy; settings belong to the existing
  installation dataset.
- Money movement, bill payment, trading or retrieval of bank credentials.
- Replacing source statements with SMS as a complete financial record.
- Guaranteed unattended Messages automation, arbitrary SMS-history access or
  automatic offline delivery. Manual Shortcut entry is the initial flow.
- A replacement visual design system or use of the prototype’s standalone
  styling in production.
- Implementation, commits, deployment, paid model evaluation and production-data
  changes as part of this planning task.

## Further Notes

The Wayfinder decision map remains the record of the product discussions. This
spec replaces the earlier handoff as the canonical implementation input. The
earlier document will be retained only as a superseded planning archive.

| Contract                                                                             | Canonical decision                                                                           |
| ------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| Automatic posting, typed-note defaults, mixed input, category-only review            | [Capture lifecycle and automatic posting rules](issues/04-capture-lifecycle.md)              |
| Retries, repeated alerts, statement matches, transfers, refunds and pending messages | [Duplicate detection and statement reconciliation](issues/05-dedupe-reconcile.md)            |
| Manual related-balance review; no automatic principal, units or EPF calculation      | [Automatic loan and investment estimates, deferred](issues/06-related-balances.md)           |
| Settings, source retention, rule learning and Shortcut access                        | [Configuration, correction learning and capture credentials](issues/07-settings-learning.md) |
| Accepted flow and existing design tokens                                             | [Application Quick Add and Shortcut experience](issues/08-quick-add-experience.md)           |

Supporting evidence:
[integration research](research/01-existing-integration.md),
[iPhone research](research/02-iphone-entry.md),
[Luna research](research/03-model-extraction.md), and
[accepted walkthrough](prototype/quick-add-walkthrough.html).

The existing checkout has unrelated uncommitted Bedrock changes. Preserve them,
record the baseline used for implementation and isolate Quick Add changes before
any implementation commit. A later implementation task owns tests, code review
and the requested commit; deployment remains separately authorized.

Readiness: Yash confirmed the testing boundaries on 2026-09-12. All product
decisions and testing seams are settled, and this spec is published as
`ready-for-agent`. Implementation verification, model evaluation and user-owned
phone testing remain execution work, not completed tests.
