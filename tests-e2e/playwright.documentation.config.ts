import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for Hook0 documentation (Docusaurus) E2E tests.
 *
 * Serves the documentation `build/` output (produced by `npm run build` in
 * ../documentation locally, or the documentation.build CI artifact) via a
 * static server. The same webServer path is used locally and in CI so what is
 * validated locally is exactly what runs in CI.
 */
const BASE_URL = "http://localhost:3999";

export default defineConfig({
  testDir: "./tests/documentation",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ["html", { open: "never", outputFolder: "playwright-report-documentation" }],
    ["list"],
    ...(process.env.CI
      ? [["junit", { outputFile: "test-results/documentation-junit.xml" }] as const]
      : []),
  ],

  use: {
    baseURL: BASE_URL,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "on-first-retry",
  },

  snapshotPathTemplate:
    "{snapshotDir}/{testFileDir}/{testFileName}-snapshots/{arg}-{projectName}-{platform}{ext}",

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "Mobile Chrome",
      use: { ...devices["Pixel 5"] },
    },
  ],

  webServer: {
    command: "npx serve ../documentation/build -l 3999 --no-port-switching",
    url: BASE_URL,
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
