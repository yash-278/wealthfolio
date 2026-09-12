# Application Quick Add and Shortcut experience

Type: prototype
Label: wayfinder:prototype
Status: resolved
Assignee: Yash / Codex
Blocked by: 02, 04, 05, 07
Parent: [Quick Add transaction capture](../map.md)

## Question

What should users see when pasting one transaction or a block of text into Quick Add, reviewing a mixed-result submission, and configuring the Shortcut? Produce a disposable interaction outline or mockup for live feedback. Include linked manual loan/investment/EPF balance-review items without duplicating the captured payment, save receipts, retry visibility, review navigation and actionable errors, with equivalent semantics across application and Shortcut. No production implementation.

## Comments

### Prototype approach, 2026-09-12

Use the logic-prototype branch of the prototype skill: a self-contained HTML walkthrough of capture results, review resolution and Shortcut receipts. This tests behavior and information hierarchy, not final visual design. Synthetic examples only, in-memory state, no production API calls. The ticket remains claimed until Yash provides feedback.

### Walkthrough ready for feedback, 2026-09-12

[Interactive Quick Add walkthrough](../prototype/quick-add-walkthrough.html) is a self-contained synthetic demo. It covers a mixed block, category-only review of a saved payment, linked manual balance follow-up, same-request retry and delivery failure. It includes a static settings preview and simulated Shortcut receipt. State is in memory; no real parsing, credentials, persistence or financial API calls occur.

Prototype proposals requiring feedback: side-by-side saved transactions and review items, clear "Payment already saved" versus "Not yet saved" labels, per-item review actions, marking a manually completed balance update done, a shared capture receipt in the Shortcut and application, and the settings grouping. Retention-period choices in the demo are illustrative. A manual balance follow-up is not an automatic balance edit.

JavaScript syntax checked. No automated UI interaction or iPhone testing performed. Awaiting Yash's reaction; do not resolve this HITL ticket or treat the proposed layout as accepted yet. No production implementation, commit or deployment occurred.

## Answer

Resolved with Yash on 2026-09-12: "Works but we make sure anything on website follows its design tokens".

The walkthrough's interaction flow is accepted: distinguish saved payments from unposted candidates, show outstanding review work and per-item actions, and expose clear outcomes in both Quick Add and Shortcut receipts. Related-balance follow-ups remain manual and must not duplicate payments.

All production web UI must use Wealthfolio's existing design tokens, shared components and theme behavior. Reuse `apps/frontend/src/globals.css`, `@wealthfolio/ui` and established application patterns for typography, spacing, colors, borders, controls and feedback states. The prototype's hardcoded visual styles are not the production design specification. Confirm light/dark theme behavior, responsive layout and accessibility during implementation verification.

The prototype remains a local, throwaway reference linked above. No production implementation or commit is authorized by this acceptance; preserve the artifact in the planning directory. Illustrative retention intervals are not newly confirmed product requirements.
