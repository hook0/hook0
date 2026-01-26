import { test, expect } from "@playwright/test";

/**
 * Application management E2E tests.
 *
 * Tests for creating, viewing, and managing Hook0 applications.
 */
test.describe("Applications", () => {
  let testUserEmail: string;
  let testUserPassword: string;

  test.beforeAll(async ({ request }) => {
    const timestamp = Date.now();
    testUserEmail = `test-apps-${timestamp}@hook0.local`;
    testUserPassword = `TestPass123!${timestamp}`;

    const response = await request.post("/api/v1/register", {
      data: {
        email: testUserEmail,
        password: testUserPassword,
        password_confirmation: testUserPassword,
      },
    });
    expect(response.status()).toBeLessThan(400);
  });

  test.beforeEach(async ({ page }) => {
    await page.goto("/login");
    await page.locator('[data-test="email-input"]').fill(testUserEmail);
    await page.locator('[data-test="password-input"]').fill(testUserPassword);
    await page.locator('[data-test="login-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations/, { timeout: 15000 });
  });

  test("should create new application with required fields only and verify API response", async ({
    page,
  }) => {
    const timestamp = Date.now();
    const appName = `Test App ${timestamp}`;

    await page.locator('[data-test="create-application-button"]').click();
    await expect(page.locator('[data-test="create-application-dialog"]')).toBeVisible();

    await page.locator('[data-test="application-name-input"]').fill(appName);

    const responsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST",
      { timeout: 15000 }
    );

    await page.locator('[data-test="create-application-submit"]').click();

    const response = await responsePromise;
    expect(response.status()).toBeLessThan(400);

    const responseBody = await response.json();
    expect(responseBody).toHaveProperty("application_id");
    expect(responseBody.name).toBe(appName);

    await expect(page.locator('[data-test="create-application-dialog"]')).not.toBeVisible({
      timeout: 10000,
    });

    await expect(page.locator('[data-test="application-card"]').filter({ hasText: appName })).toBeVisible({
      timeout: 10000,
    });
  });

  test("should display application details after creation", async ({ page, request }) => {
    const timestamp = Date.now();
    const appName = `Details App ${timestamp}`;

    await page.goto("/login");
    await page.locator('[data-test="email-input"]').fill(testUserEmail);
    await page.locator('[data-test="password-input"]').fill(testUserPassword);
    await page.locator('[data-test="login-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations/, { timeout: 15000 });

    await page.locator('[data-test="create-application-button"]').click();
    await page.locator('[data-test="application-name-input"]').fill(appName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST"
    );
    await page.locator('[data-test="create-application-submit"]').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    await page.locator('[data-test="application-card"]').filter({ hasText: appName }).click();

    await expect(page).toHaveURL(new RegExp(`/applications/${app.application_id}`), {
      timeout: 10000,
    });
    await expect(page.locator('[data-test="application-name"]')).toContainText(appName);
  });

  test("should delete application and verify removal from list", async ({ page }) => {
    const timestamp = Date.now();
    const appName = `Delete App ${timestamp}`;

    await page.locator('[data-test="create-application-button"]').click();
    await page.locator('[data-test="application-name-input"]').fill(appName);

    const createResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes("/api/v1/applications") && response.request().method() === "POST"
    );
    await page.locator('[data-test="create-application-submit"]').click();

    const createResponse = await createResponsePromise;
    expect(createResponse.status()).toBeLessThan(400);
    const app = await createResponse.json();

    await page.locator('[data-test="application-card"]').filter({ hasText: appName }).click();

    await page.locator('[data-test="application-settings-button"]').click();
    await page.locator('[data-test="delete-application-button"]').click();

    await expect(page.locator('[data-test="confirm-delete-dialog"]')).toBeVisible();
    await page.locator('[data-test="confirm-delete-input"]').fill(appName);

    const deleteResponsePromise = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/v1/applications/${app.application_id}`) &&
        response.request().method() === "DELETE"
    );

    await page.locator('[data-test="confirm-delete-button"]').click();

    const deleteResponse = await deleteResponsePromise;
    expect(deleteResponse.status()).toBeLessThan(400);

    await expect(page).toHaveURL(/\/dashboard|\/applications/, { timeout: 10000 });
    await expect(page.locator('[data-test="application-card"]').filter({ hasText: appName })).not.toBeVisible({
      timeout: 10000,
    });
  });
});
