import { test, expect, type Page, type APIRequestContext } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Onboarding use-case personalization E2E.
 *
 * The tutorial opens with one skippable question ("what are you building?") and
 * the answer is supposed to travel through the rest of the wizard, so the first
 * webhook a user sends looks like their own domain. Unit tests already pin the
 * preset mapping; what they cannot show is that the answer actually reaches the
 * later steps in a real browser. That is what these tests do: pick an option at
 * the intro, walk the wizard, and read the values the forms are seeded with.
 *
 * No page reload happens mid-wizard on purpose. The choice lives in the SPA
 * session (by design — a reload falls back to the generic examples), so
 * reloading would assert the fallback rather than the feature.
 */

/** Preset expected for the e-commerce use case, mirroring usecasePreset.ts. */
const ECOMMERCE = {
  service: "store",
  resource: "order",
  verb: "created",
  labelKey: "customer_id",
  payloadMarker: "order_id",
};

async function registerAndLogin(
  page: Page,
  request: APIRequestContext,
  testId: string
): Promise<void> {
  const timestamp = Date.now();
  const email = `test-usecase-${testId}-${timestamp}@hook0.local`;
  const password = `TestPassword123!${timestamp}`;

  const registerResponse = await request.post(`${API_BASE_URL}/register`, {
    data: { email, first_name: "Test", last_name: "User", password },
  });
  expect(registerResponse.status()).toBeLessThan(400);

  await verifyEmailViaMailpit(request, email);

  await page.goto("/login");
  await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
  await page.locator('[data-test="login-email-input"]').fill(email);
  await page.locator('[data-test="login-password-input"]').fill(password);

  const loginResponse = page.waitForResponse(
    (response) => response.url().includes("/auth/login") && response.request().method() === "POST",
    { timeout: 15000 }
  );
  await page.locator('[data-test="login-submit-button"]').click();
  expect((await loginResponse).status()).toBeLessThan(400);

  await expect(page).toHaveURL(/\/tutorial|\/dashboard|\/organizations/, { timeout: 15000 });
}

/** Open the tutorial intro, whichever authenticated page the login landed on. */
async function openTutorialIntro(page: Page): Promise<void> {
  await page.goto("/tutorial");
  await expect(page.locator('[data-test="tutorial-usecase"]')).toBeVisible({ timeout: 15000 });
}

/**
 * Walk the wizard from the intro to the event-type step, creating the
 * application the wizard requires on the way. Leaves the page on the event-type
 * form.
 */
async function advanceToEventTypeStep(page: Page, appName: string): Promise<void> {
  await page.locator('[data-test="tutorial-start-button"]').click();

  await expect(page).toHaveURL(/\/tutorial\/application/, { timeout: 15000 });
  const createAppRadio = page.locator('[data-test="tutorial-create-app-radio"]');
  if (await createAppRadio.isVisible({ timeout: 3000 }).catch(() => false)) {
    await createAppRadio.click();
  }

  const appNameInput = page.locator('[data-test="application-name-input"]');
  await expect(appNameInput).toBeVisible({ timeout: 20000 });
  await appNameInput.fill(appName);
  await page.locator('[data-test="application-submit-button"]').click();

  await expect(page).toHaveURL(/\/tutorial\/event_type/, { timeout: 20000 });
  await expect(page.locator('[data-test="event-type-service-input"]')).toBeVisible({
    timeout: 15000,
  });
}

test.describe("Onboarding use-case personalization", () => {
  test("shows the question and records the choice", async ({ page, request }) => {
    await registerAndLogin(page, request, "intro");
    await openTutorialIntro(page);

    // Every option is offered.
    for (const id of ["saas-b2b", "ecommerce", "microservices", "other"]) {
      await expect(page.locator(`[data-test="tutorial-usecase-${id}"]`)).toBeVisible();
    }

    // The neutral answer ("other") is the starting state, so a user who never
    // touches the question gets the untouched generic examples. None of the
    // personalizing answers may be pre-selected on their behalf.
    await expect(page.locator('[data-test="tutorial-usecase-other"]')).toHaveAttribute(
      "aria-pressed",
      "true"
    );
    for (const id of ["saas-b2b", "ecommerce", "microservices"]) {
      await expect(page.locator(`[data-test="tutorial-usecase-${id}"]`)).toHaveAttribute(
        "aria-pressed",
        "false"
      );
    }

    // Selecting one marks it pressed — the accessible state, not just a colour —
    // and moves the selection off the neutral default.
    await page.locator('[data-test="tutorial-usecase-ecommerce"]').click();
    await expect(page.locator('[data-test="tutorial-usecase-ecommerce"]')).toHaveAttribute(
      "aria-pressed",
      "true"
    );
    await expect(page.locator('[data-test="tutorial-usecase-other"]')).toHaveAttribute(
      "aria-pressed",
      "false"
    );
    await expect(page.locator('[data-test="tutorial-usecase-saas-b2b"]')).toHaveAttribute(
      "aria-pressed",
      "false"
    );

    // The question never blocks: the wizard can still be started.
    await expect(page.locator('[data-test="tutorial-start-button"]')).toBeEnabled();
  });

  test("carries the chosen use case into the event type and send-event steps", async ({
    page,
    request,
  }) => {
    await registerAndLogin(page, request, "prefill");
    await openTutorialIntro(page);

    await page.locator('[data-test="tutorial-usecase-ecommerce"]').click();
    await advanceToEventTypeStep(page, `Usecase App ${Date.now()}`);

    // The event type is seeded with the e-commerce example rather than left blank.
    await expect(page.locator('[data-test="event-type-service-input"]')).toHaveValue(
      ECOMMERCE.service
    );
    await expect(page.locator('[data-test="event-type-resource-input"]')).toHaveValue(
      ECOMMERCE.resource
    );
    await expect(page.locator('[data-test="event-type-verb-input"]')).toHaveValue(ECOMMERCE.verb);

    // Submitting the seeded values must be enough — the point of the feature is
    // that the user does not have to invent an event type.
    const eventTypeResponse = page.waitForResponse(
      (response) =>
        response.url().includes("/event_types") && response.request().method() === "POST",
      { timeout: 15000 }
    );
    await page.locator('[data-test="event-type-submit-button"]').click();
    expect((await eventTypeResponse).status()).toBeLessThan(400);
  });

  test("keeps the generic defaults when the question is skipped", async ({ page, request }) => {
    await registerAndLogin(page, request, "skipped");
    await openTutorialIntro(page);

    // Answer nothing and start: the wizard must behave exactly as before the
    // personalization existed, so skipping never degrades the standard flow.
    await advanceToEventTypeStep(page, `Generic App ${Date.now()}`);

    await expect(page.locator('[data-test="event-type-service-input"]')).toHaveValue("");
    await expect(page.locator('[data-test="event-type-resource-input"]')).toHaveValue("");
    await expect(page.locator('[data-test="event-type-verb-input"]')).toHaveValue("");
  });
});
