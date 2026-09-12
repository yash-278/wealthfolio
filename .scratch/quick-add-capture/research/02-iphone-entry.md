# iPhone Quick Add entry research

As of 2026-09-11. Apple primary documentation only. No device testing, private messages, application changes, commits or pushes.

Research branch: `research/quick-add-iphone`, based on `8f6f989`.
Worktree: `/Users/yash/Personal/wealthfolio-quick-add-iphone`.
Tracker copy: `/Users/yash/Personal/wealthfolio/.scratch/quick-add-capture/research/02-iphone-entry.md`.

## Answer

Use one Quick Add capture API for the application and iPhone Shortcut. Ship a manual text client first. The user can paste or type text, or pass text from an app that exposes it to the share sheet. Treat automatic Messages triggers as an optional, separately validated setup. They are supported, but are not a reliable-delivery contract.

## Verified platform facts

- Shortcuts accepts text from compatible sharing apps. When no input arrives, it can ask for input or use the clipboard. This supports a manual paste/type flow even without a source app's share sheet. [Apple input handling](https://support.apple.com/guide/shortcuts/limit-the-input-for-a-shortcut-apd8195f96d6/9.0/ios/26), [input types](https://support.apple.com/en-ie/guide/shortcuts/apd7644168e1/ios).
- Apple's Messages forwarding instructions select bubbles, then address a new message to recipients. They do not document forwarding an ordinary SMS bubble to a Shortcut. The onscreen-content guide promises access only from supported apps, with examples such as Safari, Maps and Photos. Do not advertise direct Messages share-sheet capture or arbitrary SMS-history access. Copy/paste is the conservative onboarding path, with the exact Messages copy gesture checked on the target phone. [Messages forwarding](https://support.apple.com/en-gb/guide/iphone/iph125628311/ios), [onscreen items](https://support.apple.com/en-au/guide/shortcuts/apd350ce757a/ios).
- Get Contents of URL makes API requests and supports POST with a JSON request body populated from variables. Parse its JSON response to present the result. [API requests](https://support.apple.com/guide/shortcuts/request-your-first-api-apd58d46713f/ios), [JSON](https://support.apple.com/en-gu/guide/shortcuts/apd0f2e057df/ios).
- Message automation supports sender and message-contains criteria. Multiple criteria must all match. Message is listed among automations that can run without confirmation, although individual actions can still require configuration. This is permission for a configured event flow, not evidence of unrestricted inbox access. [Communication triggers](https://support.apple.com/en-mt/guide/shortcuts/apdd711f9dff/ios), [automatic execution](https://support.apple.com/en-euro/guide/shortcuts/apd602971e63/ios).
- Personal automation belongs to a device and does not sync to other devices, although iCloud backs it up. Setup therefore includes instructions on each phone. [Personal automation](https://support.apple.com/en-gb/guide/shortcuts/apd690170742/9.0/ios/26).
- Shortcuts may request permission to access needed data, with Allow Once, Always Allow and Don't Allow choices. Import questions can clear personal configuration when sharing and collect the recipient's own values. [Privacy](https://support.apple.com/en-euro/guide/shortcuts/apd961a4fc65/9.0/ios/26), [import questions](https://support.apple.com/en-ca/guide/shortcuts/apdf330fd3a0/ios).
- Apple's web API guide lists OAuth 2 as unsupported and calls out rate limits. Use a dedicated capture credential instead of relying on the website's interactive login session. [API limitations](https://support.apple.com/en-ie/guide/shortcuts/apd891a6c84e/ios).

## Proposed product contract

These are design recommendations, not claims that Apple provides these guarantees.

1. Settings creates a named, revocable Shortcut token restricted to capture submission and its result lookup. Provision the HTTPS base URL and token through blank import questions. Keep model credentials on the Wealthfolio server. Display token once, hash it at rest, show last use, and offer revoke/replace. Never include a credential in a shared Shortcut template or URL query. A token embedded in a user's Shortcut is accessible to that user; do not describe it as a Keychain-backed secret. Header configuration and permission behavior need device acceptance testing.
2. Shortcut receives text or asks for pasted/typed text, creates a request ID, and POSTs `{text, source: "iphone_shortcut", client_request_id, submitted_at, timezone}`. Sender and original message timestamp are optional only when actually supplied by a tested source. Never equate submission time with bank transaction time. A missing transaction date in an old pasted alert goes to review unless an explicit date policy resolves it.
3. API persists an authenticated, valid capture before invoking the model. Respond promptly with its ID and state. Process server-side so closing the Shortcut does not abandon an accepted capture. Do not make the Shortcut wait through an unbounded model run.
4. Return distinct `created`, `duplicate`, `review`, and `processing` results with a short summary and an application link. A duplicate result points to the existing capture or transaction. Review and processing are accepted captures. Review explains what is missing. For a block of text, display counts and the capture link rather than a misleading single success label.
5. Use an idempotency key for retries of the same submission, plus server transaction-level duplicate checks across newly submitted copies and imports. Retrying a timeout must never blindly create another transaction. Preserve the same request ID if a retry button or saved retry record is implemented.
6. Authentication failure, oversized input, and failure to reach or persist on the server are delivery errors, not review items. Say "Not confirmed received" after an ambiguous network failure. The user can reopen Quick Add and resubmit; server duplicate detection must cover that case. Do not claim every failed request entered review. Once accepted, model errors and ambiguous or unsupported financial events remain visible for review/retry rather than disappearing.
7. First release can use a short bounded result check, then show "Received; processing" with an app link. No background polling loop or promised automatic offline retry. Add a durable on-device outbox only if later requirements and device tests justify it.

## What Apple documentation does not establish

The reviewed pages do not specify a guaranteed HTTP timeout, background execution duration, delivery while locked, offline replay, stable SMS identifier, or full Message-trigger metadata schema. They do not prove that copied message text preserves sender or timestamp. No such guarantees should enter the specification. Device testing must establish any stronger claims, including alphanumeric bank senders and handling of denied access.

## Minimum acceptance run before publishing setup

Use synthetic alerts against a non-production capture fixture. Record iOS version and Shortcut version. Test typed input, copied message text, compatible share-sheet text, missing input, first-run permission, denied permission, cellular connectivity, and revoked token. Verify created, duplicate, review, processing and unavailable-server outputs. Simulate server acceptance followed by a lost response, and confirm retry cannot double-post. For optional Message automation, separately verify matching bank-style senders, available input fields, locked phone behavior and offline behavior on the target device. Do not mark this tested until a person performs the phone steps.
