import { test, expect } from "@playwright/test";

/**
 * Homepage tests for Hook0.
 * Note: The homepage (/) requires authentication and redirects to login if not logged in.
 */
test.describe("Homepage", () => {
  test("should redirect to login when not authenticated", async ({ page }) => {
    await page.goto("/");

    // Should redirect to login page when not authenticated
    await expect(page).toHaveURL(/\/login/);
  });

  test("should display login page title", async ({ page }) => {
    await page.goto("/login");

    // Check the page loads with Hook0 branding
    await expect(page.locator("img[alt='Hook0']")).toBeVisible({
      timeout: 10000,
    });
  });

  test("should have working navigation to register from login", async ({
    page,
  }) => {
    await page.goto("/login");

    // Click "Create an account" link
    await page.getByRole("link", { name: /create an account/i }).click();

    // Should navigate to register page
    await expect(page).toHaveURL(/\/register/);
  });
});
