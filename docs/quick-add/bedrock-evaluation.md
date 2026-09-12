# Bedrock Luna evaluation

Date: 2026-09-12 Provider: AWS Bedrock Mantle, us-east-1 Model:
`openai.gpt-5.6-luna` Prompt: transaction-kind definitions revised after the
initial diagnostic run Implementation base:
`8f6f9898d30e84d7215e01d3d06cd65e02c9ab1b`, uncommitted Quick Add worktree

## Observed result

The revised prompt passed 12 synthetic API cases against a temporary SQLite
ledger. Three new transactions were saved with the expected account, amount,
currency, direction and date. A repeated reference matched the existing
transaction. Eight uncertain or unsupported cases remained in review. No
incorrect or duplicate transaction was observed in this run.

| Measurement                       | Result        |
| --------------------------------- | ------------- |
| Input tokens                      | 5044          |
| Output tokens                     | 979           |
| Estimated token cost              | $0.002402     |
| Median capture latency            | 1.447 seconds |
| p95 capture latency, nearest rank | 2.884 seconds |
| Review-only captures              | 8 of 12       |

The cases cover a debit, credit, balance embedded in an alert, unknown account,
declined payment, pending payment, OTP, missing date, ambiguous dates, owned
transfer, an uncertain note and a duplicate reference. New payments also
retained category review where no category rule existed.

The initial prompt failed to post clear transactions because it used ambiguous
transaction-kind definitions. It confused card purchases with credit-card bill
repayments. Those diagnostic cases informed the prompt revision, so this set is
now a regression set, not an independent held-out accuracy estimate.

## Broader v3 evaluation

The 16-case set initially failed: masked hints such as `XX1234` did not match
configured suffix `1234`, and one provider request timed out. These captures
stayed in review; no wrong writes were observed. The resolver now strips only
conventional masking from numeric suffixes and continues rejecting ambiguous
account mappings.

After that fix, all 16 cases passed with six expected ledger transactions and no
duplicate postings. The run reported 8,082 input tokens, 1,857 output tokens,
and an estimated $0.004230 token cost. Cases include card purchases, typed
notes, EMI payments, mixed blocks, instruction text, conflicting references,
currency mismatch and partial refunds. Because its failures informed the
resolver fix, this set is now also regression evidence rather than an untouched
accuracy sample. A transient provider failure remains reviewable and can be
retried manually.

## Limits

This small synthetic run does not qualify every bank template, typed-note
default, multi-event message or locale. The supervised trial and iPhone
acceptance remain pending. No production financial data was submitted or
changed. Cost uses reported tokens at $0.22 per million input and $1.32 per
million output tokens; provider billing and failed-call charges can differ. The
table covers the final successful run only.

## Reproduce explicitly

Run the ignored `bedrock_luna_qualification` integration test with
`QUICK_ADD_BEDROCK_KEY_FILE` pointing to a private local file and
`QUICK_ADD_QUALIFICATION_REPORT` pointing to a local output file. The test uses
a temporary database and a $0.25 application budget. Ordinary test runs skip
real-provider calls. Never commit the key file.
