import { test, expect } from "@playwright/test";
import { loginAsNewUser } from "../fixtures/test-setup";

/**
 * Error Handling E2E tests for Hook0.
 *
 * Tests error states when navigating to non-existent resources
 * (valid routes but invalid IDs that trigger API errors).
 * Distinct from error-404.spec.ts which tests unknown routes.
 */
test.describe("Error Handling", () => {
  test("should show error for non-existent organization", async ({ page, request }) => {
    await loginAsNewUser(page, request, "bad-org");

    // Navigate to a non-existent organization UUID
    const fakeOrgId = "00000000-0000-0000-0000-000000000000";
    await page.goto(`/organizations/${fakeOrgId}/dashboard`);

    // The page should show an error card (Hook0ErrorCard renders .hook0-error-card)
    // OrganizationsDashboard.vue shows Hook0ErrorCard on orgError or when !organization
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: 15000 });

    // The error card should contain error information
    await expect(page.locator('[data-test="error-card-title"]')).toBeVisible();
    await expect(page.locator('[data-test="error-card-detail"]')).toBeVisible();
  });

  test("should show error for non-existent application in valid organization", async ({
    page,
    request,
  }) => {
    const env = await loginAsNewUser(page, request, "bad-app");

    // Navigate to a valid org but fake application UUID
    const fakeAppId = "00000000-0000-0000-0000-000000000000";
    await page.goto(
      `/organizations/${env.organizationId}/applications/${fakeAppId}/dashboard`
    );

    // The page should show an error card
    // ApplicationsDashboard.vue shows Hook0ErrorCard on appError
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: 15000 });

    // The error card should contain error information
    await expect(page.locator('[data-test="error-card-title"]')).toBeVisible();
    await expect(page.locator('[data-test="error-card-detail"]')).toBeVisible();
  });

  test("should show error for non-existent application sub-page", async ({ page, request }) => {
    const env = await loginAsNewUser(page, request, "bad-app-sub");

    // Navigate to a valid org but fake app UUID on a sub-page (event_types)
    const fakeAppId = "00000000-0000-0000-0000-000000000000";
    await page.goto(
      `/organizations/${env.organizationId}/applications/${fakeAppId}/event_types`
    );

    // Should show an error card or error state
    await expect(page.locator('[data-test="error-card"]')).toBeVisible({ timeout: 15000 });
  });

  /**
   * The two settings screens, which never showed this state at all.
   *
   * Their skeleton was guarded on the data being absent, and a failed fetch leaves it absent for
   * good, so the skeleton kept the state and the error card below it was unreachable. Measured
   * before the fix: two skeletons still on screen twelve seconds after the request was refused,
   * no card, no retry. That reads as slow rather than as broken, which is why nobody reported it.
   *
   * Each test carries its own control. Two of them assert a card is visible, and an assertion
   * like that passes for free the day the selector stops matching anything — so each one first
   * asserts the same selector on a screen that has always guarded this correctly.
   */
  /**
   * The symptom, read where it showed up, rather than the guard that now prevents it.
   *
   * What this defect looked like was not a missing card, it was a skeleton that never went away:
   * the screen read as slow rather than as broken, which is why it went unreported. So the card
   * being there is half the answer and the skeleton being gone is the other half, and a screen
   * that somehow showed both would be a regression this would catch.
   *
   * One read of the DOM returns the pair, and polling it is what waits for the screen to settle,
   * so a failure prints the state that was actually on screen rather than "element not found".
   * The card count in the expectation doubles as the check that the read happened at all, which
   * is what a bare "no skeletons" assertion would lack.
   */
  function expectTheCardReplacedTheSkeleton(page: import("@playwright/test").Page, screen: string) {
    return expect
      .poll(
        () =>
          page.evaluate(() => ({
            errorCards: document.querySelectorAll('[data-test="error-card"]').length,
            skeletons: document.querySelectorAll(".hook0-skeleton").length,
          })),
        {
          timeout: 15000,
          message: `${screen} must say the load failed rather than show a skeleton for good`,
        }
      )
      .toEqual({ errorCards: 1, skeletons: 0 });
  }

  const MISSING = "00000000-0000-0000-0000-000000000000";

  test("should show error when an application's settings cannot be loaded", async ({
    page,
    request,
  }) => {
    const env = await loginAsNewUser(page, request, "bad-app-settings");

    // Control: the dashboard guards this correctly and has done all along.
    await page.goto(`/organizations/${env.organizationId}/applications/${MISSING}/dashboard`);
    await expect(
      page.locator('[data-test="error-card"]'),
      "the selector must find a card where one is known to render"
    ).toBeVisible({ timeout: 15000 });

    await page.goto(`/organizations/${env.organizationId}/applications/${MISSING}/settings`);
    await expectTheCardReplacedTheSkeleton(page, "application settings");
    await expect(
      page.locator('[data-test="error-card-retry"]'),
      "and offer the retry, which is the only way back that is not a page reload"
    ).toBeVisible();
  });

  test("should show error when an organization's settings cannot be loaded", async ({
    page,
    request,
  }) => {
    await loginAsNewUser(page, request, "bad-org-settings");

    // Control, as above.
    await page.goto(`/organizations/${MISSING}/dashboard`);
    await expect(
      page.locator('[data-test="error-card"]'),
      "the selector must find a card where one is known to render"
    ).toBeVisible({ timeout: 15000 });

    await page.goto(`/organizations/${MISSING}/settings`);
    await expectTheCardReplacedTheSkeleton(page, "organization settings");
    await expect(
      page.locator('[data-test="error-card-retry"]'),
      "and offer the retry, which is the only way back that is not a page reload"
    ).toBeVisible();
  });
});
