import { expect, test } from "@playwright/test";

const receipt = {
  id: "synthetic-capture",
  version: 1,
  status: "needs_review",
  input: { text: "Synthetic bank alert " + "long-reference".repeat(35) },
  candidates: [
    {
      id: "synthetic-review",
      status: "needs_review",
      activityId: "",
      fields: {
        accountId: "demo",
        amount: "250",
        currency: "INR",
        date: "2026-09-13",
        direction: "debit",
        kind: "payment",
        merchant: "Demo cafe",
        reference: null,
      },
    },
  ],
  reviews: [
    { id: "synthetic-review", status: "open", reason: "account_required", activityId: null },
  ],
};

test.beforeEach(async ({ page }) => {
  await page.route("**/api/v1/**", async (route) => {
    const path = new URL(route.request().url()).pathname;
    let data: unknown = [];
    if (path.startsWith("/api/v1/taxonomies/")) data = { categories: [] };
    if (path === "/api/v1/captures") data = receipt;
    if (path === "/api/v1/capture-reviews") data = [receipt];
    if (path === "/api/v1/accounts")
      data = [
        {
          id: "demo",
          name: "Demo spending account",
          currency: "INR",
          isActive: true,
          isArchived: false,
        },
      ];
    if (path === "/api/v1/quick-add/settings")
      data = {
        provider: "bedrock",
        model: "openai.gpt-5.6-luna",
        monthlyBudgetMicros: 1000000,
        timezone: "Asia/Kolkata",
        mappings: [{ alias: "Spending", accountId: "demo" }],
        typedNoteAccountId: null,
        typedNoteToday: false,
        automaticPosting: false,
        evaluationPassed: false,
        supervisedTrialCompleted: false,
      };
    if (path === "/api/v1/quick-add/usage") data = { month: "2026-09", reservedOrUsedMicros: 0 };
    await route.fulfill({ json: data });
  });
});

for (const [width, height] of [
  [320, 568],
  [390, 664],
  [430, 932],
  [768, 1024],
  [1440, 900],
]) {
  test(`capture, review and settings fit ${width}x${height}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await page.goto("/e2e/mobile-ui/");
    await page.waitForLoadState("networkidle");
    const scroll = page.locator("[data-page-scroll-container]");
    await expect(page.getByRole("heading", { name: "Add a transaction" })).toBeVisible();
    if (width === 390 || width === 430) {
      const capture = await page
        .getByRole("button", { name: "Capture", exact: true })
        .boundingBox();
      const nav = await page.getByRole("navigation", { name: "Bottom navigation" }).boundingBox();
      expect(capture!.y + capture!.height).toBeLessThanOrEqual(nav!.y);
    }
    await page.screenshot({
      path: test.info().outputPath(`initial-${width}.png`),
    });
    await page.getByLabel("Transaction text").fill("Paid 250 INR for lunch today");
    await page.getByRole("button", { name: "Capture", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Capture received" })).toBeVisible();
    for (const route of ["capture", "review", "settings"]) {
      if (route === "review") await page.getByRole("link", { name: "Open review queue" }).click();
      if (route === "settings") {
        await page.getByRole("link", { name: "Add a transaction" }).click();
        await page.getByRole("link", { name: "Quick Add settings" }).click();
        await expect(page.getByRole("button", { name: "Save settings" })).toBeVisible();
      }
      await page.waitForLoadState("networkidle");
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(
        true,
      );
      // Native wheel/touchpad scrolling must reach the end of the single page scroller.
      await page.mouse.move(width / 2, height / 2);
      await page.mouse.wheel(0, 10000);
      await expect
        .poll(async () => {
          await page.mouse.wheel(0, 1000);
          return scroll.evaluate((el) =>
            Math.abs(el.scrollHeight - el.clientHeight - el.scrollTop),
          );
        })
        .toBeLessThan(2);
      const bounds = await scroll.boundingBox();
      expect(bounds!.height).toBeLessThanOrEqual(height);
      if (width < 768) {
        const last =
          route === "capture"
            ? page.getByText("1 items need review")
            : route === "review"
              ? page.getByRole("button", { name: "Dismiss review" })
              : page.getByRole("button", { name: "Create token" });
        if (await last.count()) {
          const lastBox = await last.boundingBox();
          const navBox = await page
            .getByRole("navigation", { name: "Bottom navigation" })
            .boundingBox();
          expect(lastBox!.y + lastBox!.height).toBeLessThanOrEqual(navBox!.y);
        }
      }
      await page.screenshot({
        path: test.info().outputPath(`${route}-${width}.png`),
      });
      await scroll.evaluate((el) => {
        el.scrollTop = 0;
      });
    }
  });
}

test("native touch scroll and a shortened viewport keep review controls reachable", async ({
  page,
  browserName,
}) => {
  test.skip(browserName !== "chromium", "CDP dispatches native touch input in Chromium");
  await page.setViewportSize({ width: 390, height: 664 });
  await page.goto("/e2e/mobile-ui/?route=/quick-add/review");
  await page.waitForLoadState("networkidle");
  const scroll = page.locator("[data-page-scroll-container]");
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Emulation.setTouchEmulationEnabled", { enabled: true });
  await cdp.send("Input.dispatchTouchEvent", {
    type: "touchStart",
    touchPoints: [{ x: 200, y: 520 }],
  });
  for (let y = 500; y >= 180; y -= 20) {
    await cdp.send("Input.dispatchTouchEvent", { type: "touchMove", touchPoints: [{ x: 200, y }] });
  }
  await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
  await expect.poll(() => scroll.evaluate((el) => el.scrollTop)).toBeGreaterThan(100);
  await page.setViewportSize({ width: 390, height: 380 });
  await scroll.evaluate((el) => {
    el.scrollTop = el.scrollHeight;
  });
  await expect
    .poll(async () => {
      const last = await page.getByRole("button", { name: "Dismiss review" }).boundingBox();
      return last!.y + last!.height;
    })
    .toBeLessThanOrEqual(304);
  await cdp.detach();
});
