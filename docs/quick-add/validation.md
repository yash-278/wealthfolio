# Quick Add implementation validation

Date: 2026-09-12. Branch: `codex/quick-add-capture`. Implementation base:
`8f6f9898d30e84d7215e01d3d06cd65e02c9ab1b`.

## Local checks

- Frontend suite: 2,129 tests passed across 257 files. After the final settings
  correction, all nine Quick Add UI tests passed again.
- Workspace type checking passed. The final frontend-only check also passed.
- Web production build passed. It reported the existing large-chunk warning.
- Workspace lint completed with zero errors and existing warnings. The final
  Quick Add lint check reported no warnings.
- Rust workspace compiled both desktop and web targets. The full run completed
  with 3,348 passed, 15 failed and 17 ignored tests. The failures are in two AI
  snapshot targets: `tool_outputs_parity` and `tool_schemas`. Their producer
  code, fixtures and stored snapshots are unchanged from the implementation
  base. Differences concern existing tax, instrument-type and buy-amount output.
  These stale expectations were subsequently reviewed and refreshed during
  release validation, as recorded below.
- The core regression rerun passed 2,006 unit tests and 12 integration tests.
  The final reference-group expansion then passed all 21 API tests.
- Bedrock endpoint and client-routing tests passed, including rejection of
  non-AWS endpoints. Real-provider calls remain opt-in.
- Synthetic Bedrock evaluation: 12 regression cases and a broader 16-case run
  passed. See [evaluation details](bedrock-evaluation.md) for tuning history,
  token costs and limitations.

The API suite exercises temporary SQLite migration, durable capture acceptance,
versioned review, retries, budget limits, source cleanup, token permissions,
concurrent writes, crash recovery, statement reconciliation, category learning,
transfer linking, partial refunds and pending completion in both arrival orders.
All 21 API tests passed after the final same-batch reference correction; the
full-workspace counts above precede that correction.

## Review and release boundaries

Standards and specification reviews were performed against the implementation
base and updated after fixes. They identified credential routing, review flags,
qualification versioning, pagination, date provenance and pending completion;
those findings were addressed and checked at their relevant API/UI seams.

UI uses existing Wealthfolio components and semantic tokens. Automated UI tests
cover paste, receipts, review, settings, refund linking and token controls. No
interactive visual, keyboard, light/dark-theme or physical iPhone acceptance was
performed. The supplied Shortcut is a manual recipe and request template.

No production deployment or financial data changes were made. Automatic posting
still requires the user's supervised trial and qualification confirmation.

The original checkout's uncommitted Bedrock work was preserved. This branch was
integrated with fork main at `22d3d809`, retaining its explicit Bedrock region
selection and chat routing. After integration, 21 capture API tests, 97 AI unit
tests, 12 UI tests and the 16-case synthetic Luna evaluation passed. Workspace
formatting, TypeScript checks and the frontend production build also passed. The
evaluation configures its temporary provider with an explicit AWS region.

## Release snapshot correction

The server test dependency enables the AI `test-utils` feature across workspace
tests. This exposed 15 previously gated snapshot mismatches in code unchanged
from fork main. Expected outputs were checked against the current asset-taxonomy
and activity-draft implementations. Snapshots now include instrument types and
optional tax fields, and preserve an unstated BUY amount as null until commit.
No tool implementation changed. All 51 output snapshots and 20 schema snapshots
pass with `test-utils` explicitly enabled and snapshot updates disabled.
