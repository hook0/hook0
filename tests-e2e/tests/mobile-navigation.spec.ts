import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Mobile Navigation E2E tests for Hook0.
 *
 * Tests mobile-specific behavior: compact context bar (names hidden, icons visible),
 * tab bar with icons-only navigation, and org/app switching on mobile viewport.
 */
test.describe("Mobile Navigation", () => {
  test.use({ viewport: { width: 375, height: 812 } });

  test("should display tab bar with icons on mobile viewport", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-mobile-tabs-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    // Navigate to org dashboard — tab bar should appear
    await page.goto(`/organizations/${organizationId}/dashboard`);

    // Tab bar nav should be visible (Row 2 of the top nav)
    const tabBar = page.locator('[data-test="tab-bar"]');
    await expect(tabBar).toBeVisible({ timeout: 10000 });

    // Tab icons should be visible (labels hidden on mobile via CSS)
    const tabIcons = tabBar.locator(".hook0-topnav__tab-icon");
    const iconCount = await tabIcons.count();
    expect(iconCount).toBeGreaterThan(0);
  });

  test("should navigate via tab bar icons", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-mobile-tabnav-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    // Navigate to org-level applications page
    await page.goto(`/organizations/${organizationId}/applications`);

    // Tab bar should be visible
    const tabBar = page.locator('[data-test="tab-bar"]');
    await expect(tabBar).toBeVisible({ timeout: 10000 });

    // Click a different tab (e.g., the second tab which should be "Applications")
    const tabs = tabBar.locator("a");
    const tabCount = await tabs.count();
    expect(tabCount).toBeGreaterThan(1);

    // Click the last tab to navigate somewhere different
    const initialUrl = page.url();
    await tabs.last().click();

    // URL should change (navigated to a different page)
    await expect(page).not.toHaveURL(initialUrl, { timeout: 10000 });
  });

  test("should open org switcher dropdown on mobile", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-mobile-orgswitch-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    // Navigate to org dashboard
    await page.goto(`/organizations/${organizationId}/dashboard`);

    // Org switcher button should be visible on mobile
    const orgSwitcher = page.locator('[data-test="context-bar-org-switcher"]');
    await expect(orgSwitcher).toBeVisible({ timeout: 10000 });

    // Click to open dropdown
    await orgSwitcher.click();

    // Dropdown should appear with role="menu"
    const dropdown = page.locator('[role="menu"]');
    await expect(dropdown).toBeVisible({ timeout: 5000 });

    // Dropdown should contain menu items
    const menuItems = dropdown.locator('[role="menuitem"]');
    const itemCount = await menuItems.count();
    expect(itemCount).toBeGreaterThanOrEqual(1);
  });

  test("should close dropdown on Escape", async ({ page, request }) => {
    const timestamp = Date.now();
    const email = `test-mobile-escape-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    // Register and verify
    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);

    const verificationResult = await verifyEmailViaMailpit(request, email);
    const organizationId = verificationResult.organizationId;
    expect(organizationId).toBeTruthy();

    // Login
    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    // Navigate to org dashboard
    await page.goto(`/organizations/${organizationId}/dashboard`);

    // Open org switcher dropdown
    const orgSwitcher = page.locator('[data-test="context-bar-org-switcher"]');
    await expect(orgSwitcher).toBeVisible({ timeout: 10000 });
    await orgSwitcher.click();

    // Dropdown should be visible
    const dropdown = page.locator('[role="menu"]');
    await expect(dropdown).toBeVisible({ timeout: 5000 });

    // Press Escape to close
    await page.keyboard.press("Escape");

    // Dropdown should be hidden
    await expect(dropdown).toBeHidden({ timeout: 5000 });
  });
});
