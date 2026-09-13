# Mobile layout changes

Implemented locally on 2026-09-13 in branch `codex/mobile-ui`, based on
`c672ab498`. The checkout is `/Users/yash/Personal/wealthfolio-mobile-ui`. The
original Bedrock checkout and the Quick Add implementation checkout were left
unchanged.

## Changes

- Removed mobile rules that applied horizontal overflow to every flex and grid
  row. CSS computes the other axis as `auto`, creating unintended nested scroll
  containers. Wide tables retain their own explicitly declared scroll behavior.
- The app shell uses dynamic viewport height. Narrow iPad windows select mobile
  navigation. Desktop drag interception is only mounted in Tauri.
- Quick Add and review use shared page containers, sticky headers and bottom
  navigation clearance. Review pagination and actions wrap on narrow screens.
- Quick Add has compact header links, a larger capture button on phones,
  readable text fields and a bounded desktop form. Review and settings selects
  have 44-pixel minimum touch targets.
- Cancelled refresh gestures reset without refreshing. Text-field gestures,
  multitouch starts and gestures beginning on a scrolled page do not activate
  pull-to-refresh. The hook no longer disables native touch handling
  mid-gesture.

## Verification

- Full frontend suite passed, 2,228 tests. The two new refresh regressions
  passed separately. After the final form changes, all 11 Quick Add and refresh
  tests passed again.
- Workspace type checks passed. Web and Tauri frontend builds passed.
- Repository lint completed with no errors; existing warnings remain. The new
  browser fixture has the same Fast Refresh warning as the existing fixture.
- Chromium and WebKit passed all 10 responsive capture/review/settings cases.
  Chromium native touch and shortened-viewport check passed. The equivalent
  CDP-only test is explicitly skipped under WebKit.
- All 48 existing net-worth browser layout regressions passed.
- Phone and desktop screenshots were inspected; header actions were compacted
  after the first visual pass to keep Capture nearer the first phone viewport.

These are local component/browser checks using synthetic HTTP responses. The
browser fixture reserves bottom-navigation space but does not exercise the full
navigation menu or every authenticated application page. Physical iPhone
acceptance and production deployment have not been performed. No production
financial records were changed. Release was authorized on 2026-09-13. The merge
and deployment outcome is recorded in Atlas.
