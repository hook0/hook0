import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for Hook0 E2E tests.
 *
 * Local: Uses docker-compose.yaml to start the full stack
 * CI: Services are started by the CI script (API + serve for frontend)
 */
export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [["html", { open: "never" }], ["list"]],

  use: {
    baseURL: process.env.BASE_URL || "http://localhost:8001",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "on-first-retry",
  },

  expect: {
    toHaveScreenshot: {
      maxDiffPixels: 100,
      threshold: 0.2,
    },
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
    {
      name: "firefox",
      use: { ...devices["Desktop Firefox"] },
    },
    {
      name: "webkit",
      use: { ...devices["Desktop Safari"] },
    },
    {
      name: "Mobile Chrome",
      use: { ...devices["Pixel 5"] },
    },
    {
      name: "Mobile Safari",
      use: { ...devices["iPhone 13"] },
    },
  ],

  // Only use webServer locally - in CI, services are started by the CI script
  ...(process.env.CI
    ? {}
    : {
        webServer: {
          command:
            "docker compose -f ../docker-compose.yaml up -d && docker compose -f ../docker-compose.yaml logs -f frontend",
          url: "http://localhost:8001",
          reuseExistingServer: true,
          timeout: 180000,
        },
      }),
});
