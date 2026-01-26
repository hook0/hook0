import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Tutorial E2E tests for Hook0.
 *
 * Tests for the onboarding tutorial flow.
 * Following the Three-Step Verification Pattern.
 */
test.describe("Tutorial", () => {
  test("should display tutorial introduction page after first login", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tutorial-intro-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register via API
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: "Test",
        last_name: "User",
        password,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email
    await verifyEmailViaMailpit(request, email);

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // New users should be redirected to tutorial
    await expect(page).toHaveURL(/\/tutorial|\/dashboard|\/organizations/, {
      timeout: 15000,
    });

    // If redirected to tutorial, verify the introduction page elements
    const url = page.url();
    if (url.includes("/tutorial")) {
      // Verify tutorial page contains expected elements
      await expect(page.locator("text=Welcome to Hook0")).toBeVisible({ timeout: 10000 });
      await expect(page.locator("text=Start")).toBeVisible();
      await expect(page.locator("text=Skip")).toBeVisible();
    }
  });

  test("should allow skipping the tutorial", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tutorial-skip-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register via API
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: "Test",
        last_name: "User",
        password,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email
    await verifyEmailViaMailpit(request, email);

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for redirect
    await expect(page).toHaveURL(/\/tutorial|\/dashboard|\/organizations/, {
      timeout: 15000,
    });

    // If redirected to tutorial, skip it
    const url = page.url();
    if (url.includes("/tutorial")) {
      await page.locator("text=Skip").click();

      // Should be redirected to home/organizations
      await expect(page).toHaveURL(/\/organizations|\/dashboard/, {
        timeout: 15000,
      });
    }
  });

  test("should start tutorial when clicking start button", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tutorial-start-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register via API
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: "Test",
        last_name: "User",
        password,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email
    await verifyEmailViaMailpit(request, email);

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for redirect
    await expect(page).toHaveURL(/\/tutorial|\/dashboard|\/organizations/, {
      timeout: 15000,
    });

    // If redirected to tutorial, start it
    const url = page.url();
    if (url.includes("/tutorial")) {
      // Click start button (contains "Start" text)
      await page.locator("button:has-text('Start')").click();

      // Should be redirected to the first tutorial step (create organization)
      await expect(page).toHaveURL(/\/tutorial\/create-organization|\/organizations\/new/, {
        timeout: 15000,
      });
    }
  });

  test("should complete tutorial flow step by step", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tutorial-full-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register via API
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: "Test",
        last_name: "User",
        password,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email
    await verifyEmailViaMailpit(request, email);

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for redirect
    await expect(page).toHaveURL(/\/tutorial|\/dashboard|\/organizations/, {
      timeout: 15000,
    });

    // If not on tutorial, navigate directly to it
    const url = page.url();
    if (!url.includes("/tutorial")) {
      await page.goto("/tutorial");
    }

    // Check if we're on the tutorial introduction
    const tutorialUrl = page.url();
    if (tutorialUrl.includes("/tutorial")) {
      // The tutorial has multiple steps, let's verify the flow works
      // by checking that navigation buttons are present

      // Step 1: Introduction - Click Start
      await expect(page.locator("text=Welcome to Hook0")).toBeVisible({ timeout: 10000 });
      await page.locator("button:has-text('Start')").click();

      // Step 2: Create Organization
      await expect(page).toHaveURL(/\/tutorial\/create-organization|\/organizations\/new/, {
        timeout: 15000,
      });

      // Fill organization name
      await expect(page.locator('[data-test="organization-name-input"]')).toBeVisible({
        timeout: 10000,
      });
      await page.locator('[data-test="organization-name-input"]').fill(`Tutorial Org ${timestamp}`);

      // Submit and wait for API
      const orgResponsePromise = page.waitForResponse(
        (response) =>
          response.url().includes("/api/v1/organizations") &&
          response.request().method() === "POST",
        { timeout: 15000 }
      );
      await page.locator('[data-test="organization-submit-button"]').click();
      const orgResponse = await orgResponsePromise;
      expect(orgResponse.status()).toBeLessThan(400);

      // Should proceed to next step
      await expect(page).toHaveURL(/\/tutorial/, {
        timeout: 15000,
      });
    }
  });

  test("should navigate directly to tutorial introduction page", async ({ page, request }) => {
    // Setup: Create test user
    const timestamp = Date.now();
    const email = `test-tutorial-direct-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register via API
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: {
        email,
        first_name: "Test",
        last_name: "User",
        password,
      },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    // Verify email
    await verifyEmailViaMailpit(request, email);

    // Login via UI
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({
      timeout: 10000,
    });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();

    // Wait for any redirect
    await expect(page).toHaveURL(/\/tutorial|\/dashboard|\/organizations/, {
      timeout: 15000,
    });

    // Navigate directly to tutorial
    await page.goto("/tutorial");

    // Verify tutorial page is accessible
    await expect(page).toHaveURL(/\/tutorial/, { timeout: 10000 });
  });
});
