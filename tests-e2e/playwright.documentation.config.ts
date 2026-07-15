import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright configuration for Hook0 documentation (Docusaurus) E2E tests.
 *
 * Runs against a static server serving the documentation `build/` output.
 * Locally, the webServer below serves ../documentation/build (run
 * `npm run build` in ../documentation first). In CI the server is started by
 * the pipeline script and DOCUMENTATION_BASE_URL is provided.
 */
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
    baseURL: process.env.DOCUMENTATION_BASE_URL || "http://localhost:3999",
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
          command: "npx serve ../documentation/build -l 3999 --no-port-switching",
          url: process.env.DOCUMENTATION_BASE_URL || "http://localhost:3999",
          reuseExistingServer: true,
          timeout: 60000,
        },
      }),
});
