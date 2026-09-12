# Configuration, correction learning and capture credentials

Type: grilling Label: wayfinder:grilling Status: resolved Assignee: Yash / Codex
Blocked by: 01, 03, 04 Parent: [Quick Add transaction capture](../map.md)

## Question

What settings should a reusable Quick Add flow expose for model/provider,
account aliases and suffixes, source mappings, default account/date policies,
auto-post rules, original-text retention, and Shortcut credentials? Yash accepts
retaining raw submitted text and wants automatic learning from corrections.
Decide learning scope, precedence, conflict/undo/reset behavior, ambiguous
merchant identifiers, revocation and capture-only permissions. Reuse current
ownership boundaries; evaluate any need for user isolation explicitly.

## Comments

### Confirmed first round, 2026-09-12

Yash answered "yes" to the four proposed defaults and installation-wide
ownership clarification.

- Quick Add has its own model selector, initially GPT-5.6 Luna, and reuses AI
  Providers credentials. Changing the chat model does not change Quick Add.
- Account matching uses configured bank/card suffixes and aliases. Ambiguous
  matches go to review. Default accounts apply only to typed notes.
- Corrections automatically inform future captures for the same identifiable
  merchant. Learned rules can be inspected, edited or reset. Historical
  transactions are not silently recategorized.
- Each Shortcut has a named, revocable token for submitting captures and
  retrieving its own capture results, without access to the full financial
  history or application settings.
- Configuration belongs to the installation's existing shared dataset. Separate
  user accounts and multitenancy are deferred.

These remaining choices were settled in the second round below. Existing
requirements still apply: keep original text for review, and do not silently
drop accepted captures on provider failures.

### Confirmed second round, 2026-09-12

Yash confirmed both remaining defaults with "yes".

- Retain original submitted text until the user deletes it, with optional
  automatic cleanup. Deleting source text preserves saved transactions.
  Unresolved review items retain their supporting text.
- Explicit user-authored rules take precedence over learned rules. A newer
  correction replaces an older learned rule for the same identifiable merchant
  and scope. Ambiguous conflicts go to review.

## Answer

Resolved with Yash on 2026-09-12. The two confirmed rounds above define the
configuration contract. Quick Add configuration belongs to the existing
installation-wide dataset, not a new multi-user tenancy system. It has
independent model selection using existing provider credentials, configurable
account matching, automatic but inspectable correction learning, and named
revocable capture-only Shortcut tokens.

Rule changes affect future captures; historical transactions are not silently
rewritten. Resetting a learned rule does not undo past transaction corrections.
Source-text cleanup does not delete financial records or remove evidence still
needed for unresolved review. Token result access is limited to captures
associated with that token, not arbitrary capture identifiers or
financial-history access. Model credentials stay on the server.

Detailed controls, retention scheduling and credential setup interactions belong
to Application Quick Add and Shortcut experience. Permission tests, safe
cleanup, default validation and rule-conflict acceptance cases belong to
Evaluation and implementation handoff. No configuration or financial data was
changed in the live application by this planning decision.
