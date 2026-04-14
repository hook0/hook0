import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for Hook0 Play E2E tests.
 *
 * Runs against the Play server (Rust binary) on port 3030.
 */
export default defineConfig({
  testDir: "./tests/play",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ["html", { open: "never", outputFolder: "playwright-report-play" }],
    ["list"],
    ...(process.env.CI
      ? [["junit", { outputFile: "test-results/play-junit.xml" }] as const]
      : []),
  ],

  use: {
    baseURL: process.env.PLAY_BASE_URL || "http://localhost:3030",
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

  ...(process.env.CI
    ? {}
    : {
        webServer: {
          command: "cargo run --manifest-path ../play/Cargo.toml",
          url: process.env.PLAY_BASE_URL || "http://localhost:3030",
          reuseExistingServer: true,
          timeout: 60000,
        },
      }),
});
