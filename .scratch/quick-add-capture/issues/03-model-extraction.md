# GPT-5.6 Luna extraction and cost contract

Type: research Label: wayfinder:research Status: resolved Assignee:
luna_research Blocked by: none Parent:
[Quick Add transaction capture](../map.md)

## Question

Can GPT-5.6 Luna provide the structured extraction required for bank alerts,
card alerts and typed notes, through the current provider abstraction? Establish
exact model identifier, schema support, latency/cost assumptions,
refusal/incomplete response handling, prompt input boundaries, and configurable
provider support from primary sources and code. Quantify example per-capture
costs without inventing token usage or account availability. Treat submitted
text as data, never authority to invoke tools or change settings.

## Comments

## Answer

Use a dedicated tool-free structured extractor, initially direct OpenAI
`gpt-5.6-luna` with reasoning disabled. Reuse provider settings and credentials;
current chat plumbing is reusable but does not yet provide a qualified
structured extraction contract. Refusal, incomplete output, unsupported
providers and unresolved validation go to review. Published example: 1,000
input + 300 output tokens costs $0.00056; actual accuracy, usage, latency and
account access remain unmeasured.

Evidence and integration contract:
[Research findings](../research/03-model-extraction.md). Research branch
`research/quick-add-luna`, worktree `/private/tmp/wealthfolio-quick-add-luna`;
no commits or runtime changes.
