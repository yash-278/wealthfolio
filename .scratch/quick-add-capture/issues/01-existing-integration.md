# Existing Wealthfolio integration and ownership boundaries

Type: research Label: wayfinder:research Status: resolved Assignee: Yash / Codex
research Blocked by: none Parent: [Quick Add transaction capture](../map.md)

## Question

Which existing activity, categorization, draft, token, settings, and
deduplication services can Quick Add reuse, and where are changes required?
Verify the single-instance versus per-user ownership model, REST versus MCP
token boundaries, transaction/category atomicity, concurrency guarantees, source
metadata preservation, and desktop/web compatibility. Read the actual working
tree, including custom AI-provider changes. Produce evidence and decision
options, not an implementation.

## Comments

2026-09-11 — Claimed by Yash / Codex research (Status: claimed); resolved after
cited source inspection.

## Answer

Existing core and settings services are reusable, but raw intake, capture state,
restricted HTTP authentication and posting/category atomicity need explicit
design. Same-key database uniqueness exists; cross-source deduplication and
response replay are not established. Current ownership is instance-wide. See
[cited findings](../research/01-existing-integration.md). Research branch:
`research/quick-add-integration`; worktree:
`/private/tmp/wealthfolio-quick-add-integration`. No implementation or external
changes.
