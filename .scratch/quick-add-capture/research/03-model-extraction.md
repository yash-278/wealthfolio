# Quick Add model extraction research

As of 2026-09-11. Documentation and code inspection only; no inference calls or
account-access checks.

## Findings

`gpt-5.6-luna` supports structured outputs, text input/output and reasoning
effort `none`. Published token rates are $0.20 input, $0.02 cached input and
$1.20 output per million tokens. Access and throughput depend on the API account
tier; no measured latency or extraction accuracy is established here. Start
evaluation with reasoning disabled.
[Official model documentation](https://developers.openai.com/api/docs/models/gpt-5.6-luna).

Illustrative costs, assuming uncached input and billed output totals, without
retries or infrastructure: 1,000 input + 300 output tokens = $0.00056/capture,
$0.56/1,000 captures. 3,000 input + 600 output = $0.00132/capture, $1.32/1,000.
These token counts are scenarios, not measurements; account mappings, schema and
category lists contribute to input. Record actual usage before making budget or
latency promises.

Structured Outputs supports strict JSON Schema; use a root object, required
properties, nullable unknown values and `additionalProperties:false`. Chat
Completions uses `response_format`; Responses uses `text.format`. Refusals may
not match the schema. Check refusal and completion status before parsing;
truncated, filtered or otherwise incomplete output cannot be posted. Schema
compliance does not establish financial correctness.
[Official structured output guide](https://developers.openai.com/api/docs/guides/structured-outputs).

## Current code evidence

Evidence below is the original working tree at
`/Users/yash/Personal/wealthfolio`, including uncommitted Bedrock changes, not
the clean research branch.

- [Provider clients](/Users/yash/Personal/wealthfolio/crates/ai/src/chat/provider_clients.rs:65)
  already constructs an OpenAI `CompletionsClient`, explicitly avoiding
  Responses for existing multi-turn chat. Bedrock delegates to this client after
  endpoint validation. A single-turn extraction request can use Chat Completions
  without migrating chat.
- [Provider configuration](/Users/yash/Personal/wealthfolio/crates/ai/src/provider_service.rs:472)
  resolves provider configuration; model listing begins at line 542. Reuse
  configuration and secret resolution.
- [Provider model](/Users/yash/Personal/wealthfolio/crates/ai/src/provider_model.rs:33)
  capabilities cover tools/thinking/vision/streaming, with no
  structured-extraction capability. Existing model metadata is insufficient to
  approve arbitrary configured providers for automatic posting.
- [OpenAI catalog](/Users/yash/Personal/wealthfolio/crates/ai/src/ai_providers.json:202)
  currently lists GPT-5.4 models, not Luna.
  [Bedrock catalog](/Users/yash/Personal/wealthfolio/crates/ai/src/ai_providers.json:248)
  lists GPT-OSS models. Do not infer Luna availability or OpenAI prices for
  Bedrock.
- [Chat streaming](/Users/yash/Personal/wealthfolio/crates/ai/src/chat/streaming.rs:307)
  builds tool-enabled agents and merges additional parameters.
  [AI dependencies](/Users/yash/Personal/wealthfolio/crates/ai/Cargo.toml:34)
  pin rig-core 0.30. These establish reusable plumbing, not a ready structured
  extraction service.

## Proposed contract

Add a dedicated, tool-free Quick Add extraction operation in `crates/ai`, with a
configured provider/model independent from chat selection. Initially qualify
direct OpenAI/Luna. Other providers require adapter-specific schema support and
synthetic evaluation before automatic posting; an incompatible provider produces
a review item with a configuration reason. Reuse the existing provider settings
and secret store rather than introducing another API-key mechanism.

Return a versioned list of candidate events, source spans, decimal amount
strings, currency, direction, date/time evidence, account hints, merchant,
category candidate and ambiguity reasons. Unknown facts remain null. The
application resolves authorized account IDs and categories, checks duplicates
and validates posting rules. Model confidence alone never authorizes writes,
loan calculations or investment quantities.

Treat submitted text as untrusted data in a user content block. Fixed developer
instructions define extraction; no tools, dynamic instructions from messages,
arbitrary URLs, or settings changes. Supply only bounded account aliases and
category context needed for extraction. Retained originals belong in the capture
record under retention settings, never application logs. Every accepted capture
with failed extraction or unresolved validation reaches review with a reason and
retry/manual correction path.

Before implementation completion, verify the pinned rig client preserves
refusal, finish reason and usage metadata. If it does not, implement the narrow
typed HTTP extraction request using existing `reqwest`, rather than accepting
bare text from an agent prompt. Do not silently fall back to free-form JSON or
another provider.

Evaluation must cover synthetic bank/card alerts, typed notes, multiple events,
ambiguous dates, missing currency, duplicate forwards, transfers, card payments,
misleading balance amounts, malformed output, refusal, truncation and prompt
injection. Measure field correctness, false automatic-post rate, review rate,
actual token usage and p50/p95 latency. No paid evaluation was run in this
planning task.

## Research isolation

Created branch `research/quick-add-luna` and worktree
`/private/tmp/wealthfolio-quick-add-luna` from HEAD `8f6f989`. This report is
the only research-worktree addition. No commits, pushes, deployment or
original-code edits. A copy is retained in the main tracker so the temporary
worktree is not the sole record.
