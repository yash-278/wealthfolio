# Mobile UI regression checks

Run with the repository's pinned pnpm version:

```sh
pnpm exec playwright install chromium webkit
pnpm exec playwright test --config playwright.mobile-ui.config.ts
```

The fixture renders the production Quick Add, review and settings components,
shared application shell and mobile navigation scroll container. HTTP responses
use synthetic data. The floating navigation placeholder reserves the same space
as the app navigation; this suite does not test its menu or the full
authenticated application startup.

Checks cover capture submission, route navigation, horizontal containment,
scrolling to the last action above navigation at 320, 390, 430, 768 and 1440
pixels, and long unbroken review text. Both Chromium and WebKit run the layout
checks. Chromium also receives native touch input and a shortened viewport. The
duplicate WebKit touch test is skipped because CDP input is Chromium-specific.

Screenshots and failure traces are saved under `test-results/mobile-ui`.
Physical iPhone keyboard, browser toolbar and standalone PWA behavior still need
device acceptance. Resizing a desktop browser is not physical-device validation.
