import { test, expect } from "@playwright/test";

/**
 * Documentation E2E — API Reference page (/api), rendered client-side by Scalar.
 *
 * Regression guard: "the API documentation no longer displays".
 *
 * The Docusaurus Footer imports `website/data.js` into the browser bundle via the
 * `@shared/website-data` alias. That module evaluates `process.env.LOCAL_PREVIEW_URL`
 * at load time; since the browser has no `process`, the bare reference threw
 * "ReferenceError: process is not defined" during hydration and blanked the fully
 * client-rendered /api page. Fixed by the `define-shared-env` DefinePlugin in
 * documentation/docusaurus.config.js, which inlines the value at build time.
 *
 * Without the fix this test fails twice over: the "Hook0 API" heading never renders
 * (blank page) and a `process is not defined` page error is captured.
 */
test.describe("Documentation /api reference", () => {
  test("displays the API reference without leaking a bare process reference", async ({ page }) => {
    const pageErrors: string[] = [];
    page.on("pageerror", (err) => pageErrors.push(String(err)));

    await page.goto("/api", { waitUntil: "domcontentloaded" });

    // The reference must actually display (this is what regressed to a blank page).
    // Scalar renders the OpenAPI `info.title` ("Hook0 API") as the top-level heading.
    await expect(page.getByRole("heading", { level: 1, name: /hook0 api/i })).toBeVisible({
      timeout: 20000,
    });

    // Root cause: no unresolved `process.env.*` from the shared website data may reach
    // the browser bundle. Assert the exact reported crash is absent.
    const processErrors = pageErrors.filter((m) => /process is not defined/i.test(m));
    expect(
      processErrors,
      `Client bundle leaked a bare process reference:\n${processErrors.join("\n")}`
    ).toEqual([]);

    // No uncaught runtime error at all on the API reference page.
    expect(pageErrors, `Unexpected page errors:\n${pageErrors.join("\n")}`).toEqual([]);
  });

  // The Scalar viewer bundle is loaded by /api itself (documentation/src/scalar/lazy-plugin.js).
  // Before, @scalar/docusaurus put it as a synchronous <script> on every page, which held
  // back the first paint of every guide. These two tests pin both halves of the change:
  // guides no longer fetch it, and /api still renders when reached through client-side
  // navigation (the hard-load case is covered by the test above).
  test("guide pages do not download the Scalar API reference bundle", async ({ page }) => {
    const scalarRequests: string[] = [];
    page.on("request", (req) => {
      if (req.url().includes("@scalar/api-reference")) scalarRequests.push(req.url());
    });

    await page.goto("/how-to-guides/monitor-webhook-performance", { waitUntil: "load" });
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();

    expect(scalarRequests, "a guide page requested the Scalar bundle").toEqual([]);
  });

  test("renders the API reference after client-side navigation from a guide", async ({ page }) => {
    const pageErrors: string[] = [];
    page.on("pageerror", (err) => pageErrors.push(String(err)));

    await page.goto("/how-to-guides/monitor-webhook-performance", { waitUntil: "load" });
    await page
      .getByRole("navigation")
      .getByRole("link", { name: "Reference", exact: true })
      .first()
      .click();

    await expect(page).toHaveURL(/\/api$/);
    await expect(page.getByRole("heading", { level: 1, name: /hook0 api/i })).toBeVisible({
      timeout: 20000,
    });
    expect(pageErrors, `Unexpected page errors:\n${pageErrors.join("\n")}`).toEqual([]);
  });
});
