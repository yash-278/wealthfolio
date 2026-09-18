# Native client parity

The target is full reference-client parity. An implemented screen is not a claim
that every workflow in that area is finished or verified on a physical device.

| Area | Current implementation | Remaining parity |
| --- | --- | --- |
| Rendering | Native SwiftUI TabView, NavigationStack, List, Form, Chart, sheets and system toolbar controls | Physical layout, accessibility and interaction acceptance |
| Engine | Existing Rust services/SQLite/migrations through an in-process ABI | Full error/cancellation and binary/stream transport coverage |
| Offline | Local reads and writes; durable existing sync queue | Airplane-mode device acceptance and restart testing |
| Own server | Pair/login, snapshot, incremental upload/download, pause, conflict acceptance | Device end-to-end test and expired-session UX acceptance |
| Setup | Native currency/timezone setup; separate bundle/data | Localization and complete existing onboarding options |
| Dashboard | Core valuation, history chart, account list, privacy | All chart periods, performance metrics, portfolio scope selection |
| Accounts | Create and edit basic account fields; detail links | Archive/delete, groups, holdings-mode setup, provider-linked restrictions |
| Activities | Paging, asset search, create/edit/delete | Full filters, transfer linking, bulk editing, reviewed asset resolution, import workflows |
| Holdings | List and detail financial values | Lots, allocation drill-down, manual snapshots and asset editing |
| Net worth | Current totals | Alternative assets, liabilities, valuation editing and history |
| Insights | Performance chart/returns/attribution/risk and quality warnings; taxonomy allocation, currency breakdown | Benchmarks, allocation drill-down/targets, drift/rebalancing |
| Spending | Month totals, daily chart, opted-in accounts | Categories, splits, budgets, events, rules and refunds |
| Quick Add | Text intake, receipt, review correction, retry/dismiss and basic settings | Categorization/refund linking, account aliases, usage controls, automatic-posting qualification and shortcut tokens |
| Goals | List/progress and basic save-up creation | Editing/deleting, funding rules, detailed projections and retirement planning |
| Assistant | Threads, streamed text conversation and provider credentials/settings | Tools/approval cards, attachments, edit/retry, tags/pinning and richer results |
| Preferences | Base currency/timezone, Keychain provider credentials, persistent privacy | Appearance, locale/formatting, health checks, market data/provider configuration |
| Files | Native CSV export/share using existing engine formatting | CSV/PDF import, backup/restore |
| Widgets/Shortcuts | Not implemented | Native extensions/App Intents with scoped shared storage |
| Addons | No web addon UI is loaded | Decide native equivalents per installed addon without embedding web UI |

## Validation recorded

2026-09-13: the signed iPhone build and signature verification passed. Both native Rust integration tests
passed with synthetic data. The sync test covers a web Quick Add activity arriving
on the native client. No production data was copied to this new app during tests.
The native build installed on the paired iPhone and the launch command succeeded.
Visual and feature acceptance remain pending. Swift XCTest cases were added for
financial precision, missing values and dates; they have not yet run in Xcode.
