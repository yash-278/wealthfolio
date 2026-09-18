import { defineConfig, devices } from "@playwright/test";
export default defineConfig({
  testDir: "./e2e/mobile-ui",
  outputDir: "./test-results/mobile-ui",
  workers: 2,
  reporter: "line",
  use: {
    baseURL: "http://127.0.0.1:4176",
    screenshot: "only-on-failure",
    trace: "retain-on-failure",
  },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "webkit", use: { ...devices["Desktop Safari"] } },
  ],
  webServer: {
    command: "BUILD_TARGET=web pnpm --filter frontend exec vite --host 127.0.0.1 --port 4176",
    url: "http://127.0.0.1:4176/e2e/mobile-ui/",
    reuseExistingServer: !process.env.CI,
  },
});
