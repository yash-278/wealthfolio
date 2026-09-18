# Amazon Bedrock provider

This build adds Amazon Bedrock to Settings → AI Providers. It uses AWS's
OpenAI-compatible Chat Completions API, including streaming and Wealthfolio's
existing financial tools. No gateway or extra Railway service is required.

## Configure

1. Obtain an Amazon Bedrock API key in your own AWS account. An AWS access key
   ID and secret access key are not interchangeable with a Bedrock API key.
2. Open Amazon Bedrock in Settings → AI Providers and save your AWS region, such
   as `us-east-1`. There is no default region.
3. Save the Bedrock API key using the provider's API-key field.
4. Refresh the model list. This checks connectivity and authentication against
   that region. It does not prove that every listed model is enabled for your
   account or supports every assistant feature.
5. Select the model you want and enable the provider. For models outside the
   built-in catalog, set tool, image and reasoning capabilities to match the
   model's documented support. Models without tool support cannot access your
   accounts through the assistant. The GPT OSS catalog entries are text-only.
6. Start a new chat and select the Bedrock model. Test with a synthetic prompt
   before asking a question about your accounts.

The region creates an endpoint in this form:
`https://bedrock-mantle.REGION.api.aws/v1`. Only HTTPS regional
AWS Mantle endpoints are used. Saved Runtime URLs are normalized to Mantle in
the same region. Custom proxies, China endpoints,
SigV4 and AWS profiles are outside this implementation.

## Credentials and usage

For Railway, set these service variables and redeploy:

```text
AWS_REGION=us-east-1
AWS_BEARER_TOKEN_BEDROCK=<your Bedrock API key>
```

The backend reads these variables; they are not baked into the frontend.
Alternatively, configure the region and key through Settings. Values saved in
Settings take precedence over environment variables. The saved key uses
Wealthfolio's existing encrypted secret store under `ai_bedrock`, separately
from `ai_openai`, and is never stored in provider settings.

AWS bills model usage separately from a ChatGPT subscription and Railway
hosting. The provider starts disabled. Requests do not switch to OpenAI or a
different provider after failure. Chat titles use the selected Bedrock chat
model, so title generation can also incur AWS usage.

Replace an expired key in Settings or Railway, depending on where you configured it.
If both are configured, removing the saved key exposes the environment fallback.
Remove both keys or disable the provider to stop using Bedrock.
Changing the region or key clears cached model discovery. HTTP 401/403 errors
from discovery point to key expiry or access; 429 means AWS throttling.
Discovery has a 30-second timeout. A successful model-list request alone does
not verify inference permission.

## Validation

Local tests cover region validation, credential isolation and removal, missing
configuration, and OpenAI-compatible streaming text and tool-call assembly. They
use synthetic credentials and a local HTTP fixture, not AWS.

Before enabling the provider, verify a model-list request and a synthetic streamed
chat with your AWS account, then an account-read tool call. Confirm denied model
access and expired credentials produce errors. Back up Wealthfolio's persistent
volume before replacing its application image. No database migration is required
by this provider change.

## Sources

- [AWS Chat Completions API](https://docs.aws.amazon.com/bedrock/latest/userguide/inference-chat-completions-mantle.html)
- [Amazon Bedrock API keys](https://docs.aws.amazon.com/bedrock/latest/userguide/api-keys.html)

Implementation baseline: Wealthfolio v3.8.0, commit
`8f6f9898d30e84d7215e01d3d06cd65e02c9ab1b`.

## Railway build

The Docker build uses four compiler jobs, release mode with workspace optimization
level 0, and dependency optimization level 1. Two higher-optimization builds ended
after approximately 20 minutes without a compiler diagnosis; this configuration
completed in 11m 05s. Runtime performance under heavier workloads is unmeasured.
Railway supplies the existing persistent volume at `/data`; the Dockerfile does
not declare a Docker-managed volume.
