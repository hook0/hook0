import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for Hook0 E2E tests.
 *
 * Local: Uses docker-compose.yaml to start the full stack
 * CI: Services are started by the CI script (API + serve for frontend)
 */
/**
 * Marks the tests that deliberately spend this machine's allowance on the
 * endpoints that send mail, until the API refuses the next call.
 *
 * That allowance is held per source address, and every test in a run shares one
 * — a browser cannot pretend to come from somewhere else, because the API only
 * accepts a forwarded-for address from a peer it trusts and never lets a page
 * set that header past the CORS allow-list. So a test that empties the bucket
 * takes it away from every test that runs after it, for as long as the refill
 * takes. Hence a project of their own, ordered after the rest.
 */
const RATE_LIMIT_TAG = /@rate-limit/;

/** Keeps those tests out of every project that is not built to absorb them. */
const NEVER_THE_RATE_LIMIT_TESTS = { grepInvert: RATE_LIMIT_TAG } as const;

export default defineConfig({
  testDir: "./tests",
  testIgnore: ["**/play/**", "**/website/**", "**/documentation/**"],
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ["html", { open: "never" }],
    ["list"],
    ...(process.env.CI
      ? [["junit", { outputFile: "test-results/junit.xml" }] as const]
      : []),
  ],

  use: {
    baseURL: process.env.BASE_URL || "http://localhost:8001",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "on-first-retry",
  },

  snapshotPathTemplate: "{snapshotDir}/{testFileDir}/{testFileName}-snapshots/{arg}-{projectName}-{platform}{ext}",

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
      ...NEVER_THE_RATE_LIMIT_TESTS,
    },
    {
      name: "firefox",
      use: { ...devices["Desktop Firefox"] },
      ...NEVER_THE_RATE_LIMIT_TESTS,
    },
    {
      name: "webkit",
      use: { ...devices["Desktop Safari"] },
      ...NEVER_THE_RATE_LIMIT_TESTS,
    },
    {
      name: "Mobile Chrome",
      use: { ...devices["Pixel 5"] },
      ...NEVER_THE_RATE_LIMIT_TESTS,
    },
    {
      name: "Mobile Safari",
      use: { ...devices["iPhone 13"] },
      ...NEVER_THE_RATE_LIMIT_TESTS,
    },
    {
      // Runs strictly after the whole chromium suite, and that ordering is the
      // point rather than a preference: see RATE_LIMIT_TAG. Declaring it as a
      // dependency is what makes it a guarantee instead of a filename that
      // happens to sort last. The cost is that Playwright skips a project whose
      // dependency failed, so these tests do not run on an already-red suite —
      // acceptable, since a red suite is not a silent state. To run them alone
      // while working on them: --project=chromium-rate-limit --no-deps.
      name: "chromium-rate-limit",
      use: { ...devices["Desktop Chrome"] },
      grep: RATE_LIMIT_TAG,
      dependencies: ["chromium"],
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
