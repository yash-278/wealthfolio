# Capture from an iPhone

Quick Add accepts copied bank/card alerts and typed notes through the same API
as the application. A reachable Wealthfolio web server with HTTPS is required.
Desktop-only installations do not provide a remote Shortcut endpoint.

## Set up

1. In Wealthfolio, open Settings → Quick Add. Configure Bedrock Luna, account
   mappings, timezone and an AI budget. Start with automatic posting disabled.
2. Create a named Shortcut token. Copy it when shown. This token only submits
   captures and reads receipts submitted with that token.
3. In Apple Shortcuts, create a shortcut named “Wealthfolio capture”. Add “Ask
   for Input”, with text input. You can paste a copied alert or type an expense
   note. Receiving text from a compatible share sheet is optional.
4. Add “Generate UUID”. Store it as `Request ID`. Generate it once for a new
   capture, then reuse it with the unchanged text if the response is lost.
5. Add a Dictionary with `clientRequestId` set to Request ID, `text` set to the
   input and `inputKind` set to `bank_alert`, `card_alert`, `typed_note` or
   `unknown`.
6. Add “Get Contents of URL” for `https://YOUR-WEALTHFOLIO/api/v1/captures`. Set
   method POST and request body JSON to the Dictionary. Add headers
   `Authorization: Bearer YOUR_SHORTCUT_TOKEN` and
   `Content-Type: application/json`.
7. Read `id`, `status`, `candidates` and `reviews` from the response. Save the
   capture ID. A successful HTTP response means the text was accepted, not that
   every transaction was posted.
8. If status is `processing`, wait two seconds and GET
   `/api/v1/captures/CAPTURE_ID` with the same Authorization header. Repeat at
   most three times. The server continues processing after the Shortcut closes.
9. Show saved and already-recorded candidates separately from open reviews.
   Offer “Open URLs” to `https://YOUR-WEALTHFOLIO/quick-add/review` for
   unresolved work.

The
[request template](../../apps/frontend/public/quick-add/request-template.json)
contains placeholders only. Enter your own address and token in the Shortcut. Do
not put a model-provider key in the Shortcut.

## Recover an uncertain submission

A network failure means the receipt is unconfirmed. Preserve the exact text and
Request ID, then retry that same request. If you know the capture ID, fetch its
receipt first. Changing the text requires a new Request ID. Reusing an ID with
different text returns HTTP 409.

An expired or revoked token returns HTTP 401. Create a replacement in the
application. A replacement token cannot read receipts belonging to an old token;
use the application's review queue for those captures.

## Phone acceptance

Complete these checks on your phone before enabling automatic posting after the
supervised trial:

- Pasted and typed input reaches the expected review item.
- The Shortcut displays processing, saved, already-recorded and needs-review
  results accurately.
- Retrying the same Request ID creates one capture and one financial event.
- A connection failure is displayed as unconfirmed, without claiming the
  transaction was saved.
- Revoking the token stops new capture requests.
- The review link opens the correct Wealthfolio installation.

This is a manual Shortcut recipe. It has not been signed, installed or tested on
an iPhone. Direct forwarding from Messages and message automations are not
guaranteed.
