import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for Hook0 website (marketing) E2E tests.
 *
 * Runs against the live website or a local Parcel dev server.
 * Tests cover: landing pages, tracking (gtag.js), RGPD/consent, social proof, SEO meta.
 */
export default defineConfig({
  testDir: "./tests/website",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : undefined,
  reporter: [
    ["html", { open: "never", outputFolder: "playwright-report-website" }],
    ["list"],
    ...(process.env.CI
      ? [["junit", { outputFile: "test-results/website-junit.xml" }] as const]
      : []),
  ],

  use: {
    baseURL: process.env.WEBSITE_BASE_URL || "https://www.hook0.com",
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
});
