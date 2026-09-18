# Dashboard feature map

Source audit: 2026-09-13, local web app source in this checkout. This does not assert that every local change is deployed. Companion mockup: `dashboard-complete-prototype.html`.

The question: how should the native dashboard retain the web dashboard's feature set while following the accepted native design system?

Keep Dashboard as one bottom tab with Investments, Net Worth and optional Spending sections. Keep a left-aligned page title, privacy and contextual actions. Use native glass for navigation and controls, readable surfaces for data. No introductory marketing copy or card subtitles.

## Source coverage and proposed native placement

| Web source | Feature | Native dashboard destination | Current native gap / mockup scope |
| --- | --- | --- | --- |
| `pages/dashboard/portfolio-page.tsx` | Remembered Investments / Net Worth tabs; Spending appears when enabled; swipe navigation | Full-width section switcher below the header | Native dashboard currently only shows investments. Mockup switches all three; optional visibility, persistence and swiping need implementation. |
| `pages/dashboard/dashboard-content.tsx` | Current value, true period P&L and return, unavailable states | Value and return above chart | Native currently displays balance change. Must reuse the web dashboard performance profile and helpers' semantics, not subtract two balances. Mockup returns are illustrative. |
| Same; `components/history-chart` | Valuation and net-contribution history, direct inspection, responsive domain | Chart with contribution overlay toggle | Native lacks contribution overlay. Mockup toggles it and supports pointer/keyboard inspection. |
| Same; interval selector | 1D / 1W / 1M / 3M / 6M / YTD / 1Y / 5Y / All, remembered interval | Full-width period control with overflow on small devices | Native period set is incomplete. Mockup includes the full set; synthetic series changes per selection. |
| `pages/dashboard/portfolio-update-trigger.tsx` | Source timestamp, warnings, update/recalculation entry | Compact data-health control below content / actions | Mockup health sheet represents these states. Actual warning handling needs parity. |
| `pages/dashboard/accounts-summary.tsx` | Account values and performance, grouped or flat layout, expandable groups, account drill-down | Accounts section | Mockup toggles sample groups; native currently basic list without equivalent grouped performance. |
| `pages/dashboard/top-holdings.tsx` | Exclude cash/alternative assets; daily / unrealized / total P&L / return; sort by value or gain; symbol/name; detail and view all | Holdings section with filter sheet | Mockup changes sort, display name and metric labels; full native calculation/selection behavior missing. |
| `pages/dashboard/goals.tsx` | Progress and prioritized goals, view all, empty state | Goals below holdings | Native goal screen exists but dashboard widget missing; mockup shows a sample goal. |
| `pages/dashboard/dashboard-actions.tsx` | Record transaction, conditional broker/device sync, update prices, rebuild history, verify data | Contextual action sheet | Keep our optional own-server sync. Broker/provider and upstream device-sync actions are conditional, not a replacement for our server. Mockup actions are stubs. |
| `pages/net-worth/net-worth-content.tsx`, `net-worth-chart.tsx` | Net worth and period change, assets/liabilities history and period selector; missing/stale valuation warnings | Net Worth hero and history | Native currently summary only. Mockup shows history, period selection and warning entry; no real calculations. |
| `pages/net-worth/components/breakdown-table.tsx`, detail sheets | Category composition, amounts, category trends; investment allocation detail; category asset detail | Breakdown rows opening native sheets | Mockup representative category sheets; actual charts and record drill-down require implementation. |
| `pages/net-worth/components/velocity-card.tsx` | Monthly pace; market gains, contributions and equity built; trailing-year comparison | Monthly pace card | Mockup component breakdown. Must port existing calculation logic without changing meaning. |
| `pages/net-worth/components/momentum-card.tsx` | Current vs prior period, monthly movement bars | Momentum card | Mockup period comparison; full monthly bars remain to be designed in native. |
| `pages/dashboard/portfolio-page.tsx` | Add asset / liability, linked mortgage flow | Net Worth add menu | Mockup action entry only. Native alternative-asset forms not yet present. |
| `features/spending/components/spending-tab-content.tsx` | Optional setup, settings/rules/assistant paths; timezone-aware periods, specific month and custom range | Spending switcher, period/calendar control, settings | Mockup main periods and settings entry; month/custom range/setup are documented requirements, not implemented flows. |
| Same; `cash-flow-strip.tsx` | Spending total, income/savings/net flow; daily or period bars and activity drill-down | Spending hero and cash-flow summary | Native only basic monthly report. Mockup daily chart and selected sample value; exact period aggregation not implemented. |
| Same; category rollup | Where it went treemap / category rows, savings handling, uncategorized review and transactions | Selectable category map and review control | Mockup sample categories and review sheet; category hierarchy and categorization remain native gaps. |
| `budget-line-chart-card.tsx` | Budget month navigation, actual vs target / pace, category allocations, editing | Budget card and sheet | Mockup totals and entry points only; actual-vs-planned interactive curve, historical pace and editor remain implementation work. |
| `recent-activity-card.tsx`, `events-card.tsx` | Recent cash activity and event-linked spending | Recent activity and Events | Mockup sample entries; native event flow missing. |
| Spending dashboard signals and reports links | Notable changes, Where / What changed / When reports with period carried forward | Signals and Reports | Mockup sheets explain destinations; advanced native reports missing. |

## Layout study

- A, Chart first: three section tabs, value/history first, then detailed sections. Recommended continuation of the accepted dashboard.
- B, Overview first: cross-section value strip plus contained chart and account/category block. Useful if switching between the three totals is frequent.
- C, Accounts first: smaller hero/chart with a denser account/category list. Keeps the same controls and detail destinations.

All layouts support light/dark appearance. Data is synthetic, state stays in memory, and action sheets never mutate real data. The mockup is not native Liquid Glass and does not claim feature completion.

## Implementation order after layout selection

1. Native three-section dashboard shell and shared period/privacy state, retaining offline engine and optional own-server sync.
2. Investments parity: proper performance profile, contribution history, grouped accounts, holding controls and goals.
3. Net Worth history, category sheets, pace/momentum and stale-valuation handling; asset/liability forms.
4. Spending parity: optional setup, full period model, categories, budgets, activity/event drill-down and reports.
5. Device review in light/dark, compact width and accessibility text sizes; verify chart selection and period semantics against synthetic fixtures and the existing Rust contracts.
