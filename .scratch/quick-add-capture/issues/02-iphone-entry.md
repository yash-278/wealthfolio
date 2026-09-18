# iPhone Shortcut entry and delivery constraints

Type: research Label: wayfinder:research Status: resolved Assignee:
iphone_research Blocked by: none Parent:
[Quick Add transaction capture](../map.md)

## Question

What supported iPhone Shortcut flow can submit selected, copied, or entered
bank-message text to a self-hosted HTTPS API and present created, duplicate,
review, and failure results? Verify Messages sharing constraints, optional
message automation, metadata availability, token provisioning/revocation,
background limits, and practical retry behavior using Apple primary sources.
Distinguish manual forwarding from unattended SMS access. Propose a minimum
user-tested setup without claiming it has been tested on Yash's phone.

## Comments

## Answer

Use a manual text Shortcut with clipboard or typed input and HTTPS JSON POST to
the shared capture API. Direct Messages share-sheet support is not established
by Apple documentation. Message automations may run without confirmation, but
metadata and delivery guarantees require device validation. Use revocable
capture tokens, persisted server intake, idempotency and distinct
created/duplicate/review/processing/delivery-error responses. Any accepted
capture that cannot post stays visible for review; a request never received by
the server cannot enter review.

[Apple-source findings and proposed acceptance run](../research/02-iphone-entry.md).
Research branch `research/quick-add-iphone`; worktree
`/Users/yash/Personal/wealthfolio-quick-add-iphone`. No device tests or commits.
