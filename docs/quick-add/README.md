# Quick Add

Paste a bank/card alert or type an expense note in Quick Add. Each accepted
submission is saved before model processing. Clear entries can be posted
automatically after qualification and a supervised trial. Uncertain entries
remain in Review.

## Configure

1. Open AI Providers and configure AWS Bedrock with a Bedrock API key and a
   supported Mantle endpoint. The default is
   `https://bedrock-mantle.us-east-1.api.aws/openai/v1`. Credentials use the
   existing application secret store.
2. Open Quick Add settings. Choose Bedrock Luna, set a monthly USD budget and
   timezone, and map account suffixes or aliases to existing accounts.
3. Optionally choose an account and local-date default for typed notes. Bank and
   card alerts do not use these defaults.
4. Leave automatic posting off while qualifying the extractor and reviewing a
   small supervised trial. Confirm both gates before enabling posting. A changed
   extraction version requires renewed qualification; a stale settings tab
   cannot supply it.
5. Set optional source retention. Blank means keep text until deletion. Cleanup
   preserves financial transactions, duplicate-prevention identifiers and
   unresolved review evidence.

Estimated monthly usage includes conservative allowances for pending calls and
calls without usage information. At the configured limit, new text is accepted
into review while AI processing pauses. Budget-blocked captures can resume when
the month changes or the budget increases.

## Resolve review

- Correct unposted account, amount, date, direction or transaction kind, then
  save the payment.
- Categorize a saved payment without creating another transaction. Merchant
  corrections become editable rules for future captures. Explicit rules take
  precedence. Resetting learned rules leaves history intact.
- Record a confirmed transfer side without inventing the other side. A uniquely
  matching confirmed counterpart can be linked. Otherwise, use the existing
  transaction link flow and check the link from Review.
- A refund remains a separate credit. Select its original debit in refund
  review. Linking does not change either amount.
- Update loan, investment and EPF balances manually, then complete their linked
  follow-up. Quick Add does not estimate principal, units or returns.
- Retry failed extraction, or explicitly dismiss irrelevant input. Successful
  review actions can be retried without another payment.

Transaction, booking and value dates remain available separately when extracted.
Conflicting evidence stays in review. Reference matching checks account,
direction, amount, currency and date; similarity alone does not establish a
duplicate. Contradictory or multiply matched references require review,
including references imported before Quick Add existed. A pending alert with
source-backed fields and a unique compatible reference can link to a completed
payment in either arrival order. This resolves the pending review without
creating another payment. Unsupported or ambiguous pending alerts stay in
review.

## iPhone

Follow the [manual Shortcut recipe](iphone-shortcut.md). A reachable HTTPS web
installation is required. Shortcut tokens can submit text and read only their
own receipts. They cannot inspect account history or change settings. An
accepted capture continues after the Shortcut closes.

## API

Application authentication protects settings, review and related actions.
Shortcut tokens use `Authorization: Bearer …` only for capture submission and
their own receipt retrieval.

| Method and path                                                           | Purpose                                                                   |
| ------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| POST `/api/v1/captures`                                                   | Persist text using an immutable `clientRequestId`                         |
| GET `/api/v1/captures/{id}`                                               | Read processing and per-candidate outcomes                                |
| GET `/api/v1/capture-reviews?page=0&pageSize=25&reason=category_required` | Filtered, paginated unresolved captures; page size 1–100                  |
| POST `/api/v1/capture-reviews/{id}/resolve`                               | Versioned correction, categorization, refund link or follow-up completion |
| POST `/api/v1/capture-reviews/{id}/dismiss`                               | Versioned dismissal                                                       |
| POST `/api/v1/captures/{id}/retry`                                        | Retry eligible unfinished extraction                                      |
| DELETE `/api/v1/captures/{id}/source`                                     | Clear source text after review is complete                                |
| GET/PUT `/api/v1/quick-add/settings`                                      | Read or configure Quick Add                                               |
| GET `/api/v1/quick-add/usage`                                             | Current monthly usage and allowances                                      |

A capture response is not an all-or-nothing transaction success. Inspect
`candidates[].status` and open `reviews`. `posted` and `already_recorded`
identify saved activities; a saved activity may still have category or balance
review. `processing` continues in the background. Reusing a request ID with
changed text returns a conflict.

See [Bedrock evaluation evidence](bedrock-evaluation.md) for the measured
development run and its limits. Production deployment, a supervised trial and
phone acceptance are separate from local automated checks.

See [implementation validation](validation.md) for completed checks and
remaining user-owned release acceptance.
