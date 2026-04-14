import { test, expect } from "@playwright/test";

test.describe("Play Theme", () => {
  test("theme toggle switches between dark and light mode", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    // Default is dark (no data-theme attribute means dark, or data-theme="dark")
    const initialTheme = await page.evaluate(() =>
      document.documentElement.getAttribute("data-theme"),
    );

    // Click theme toggle to switch to the other theme
    await page.locator("#btnTheme").click();

    const afterFirstClick = await page.evaluate(() =>
      document.documentElement.getAttribute("data-theme"),
    );

    // Should have changed
    expect(afterFirstClick).not.toBe(initialTheme);

    // Click again to toggle back
    await page.locator("#btnTheme").click();

    const afterSecondClick = await page.evaluate(() =>
      document.documentElement.getAttribute("data-theme"),
    );

    // Should match the value after first click was toggled away from
    // (light->dark->light or dark->light->dark)
    expect(afterSecondClick).not.toBe(afterFirstClick);
  });

  test("theme preference persists in localStorage across page reload", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    // Toggle theme to light
    // First, set to a known state: keep clicking until we get "light"
    await page.locator("#btnTheme").click();
    const themeAfterClick = await page.evaluate(() =>
      document.documentElement.getAttribute("data-theme"),
    );

    // Verify localStorage was updated
    const savedTheme = await page.evaluate(() =>
      localStorage.getItem("hook0play_theme"),
    );
    expect(savedTheme).toBe(themeAfterClick);

    // Reload the page
    await page.reload();
    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    // Theme should be restored from localStorage
    const themeAfterReload = await page.evaluate(() =>
      document.documentElement.getAttribute("data-theme"),
    );
    expect(themeAfterReload).toBe(themeAfterClick);
  });

  test("default theme is dark when no preference is saved", async ({
    browser,
    baseURL,
  }) => {
    // Use a fresh context with no localStorage
    const context = await browser.newContext({
      baseURL: baseURL!,
      colorScheme: "dark",
    });
    const page = await context.newPage();

    // Clear any stored theme preference
    await page.goto("/");
    await page.evaluate(() => localStorage.removeItem("hook0play_theme"));
    await page.reload();
    await expect(page.locator("#webhookUrl")).toBeVisible({ timeout: 10000 });

    // With prefers-color-scheme: dark and no saved preference, theme should be dark
    // The CSS default is dark (:root variables are dark), and initTheme() does nothing without saved pref
    const theme = await page.evaluate(() =>
      document.documentElement.getAttribute("data-theme"),
    );
    // No explicit data-theme attribute or "dark" means dark mode is active
    expect(theme === null || theme === "dark").toBe(true);

    await context.close();
  });
});
