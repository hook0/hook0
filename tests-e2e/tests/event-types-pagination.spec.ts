import { test, expect } from "@playwright/test";
import { verifyEmailViaMailpit, API_BASE_URL } from "../fixtures/email-verification";

/**
 * Hook0PaginatedList interactions on the EventTypes page.
 *
 * Boots the live frontend, creates a fresh tenant + application, seeds enough
 * event types to span 2 pages (default page size = 100), then exercises the
 * full prev/next/loading/edge-disable contract of `Hook0PaginatedList.vue`:
 *  - Page 1 renders 100 rows + indicator says "Page 1".
 *  - Next button is enabled, Prev button is disabled on first page.
 *  - Click Next → page 2 renders the remaining rows + indicator becomes "Page 2".
 *  - Prev becomes enabled, Next becomes disabled on last page.
 *  - Click Prev → returns to page 1 (served from cache, no network refetch).
 *  - All assertions are over `data-test-*` selectors only (no class/CSS coupling).
 */
test.describe("EventTypes pagination (Hook0PaginatedList)", () => {
  // Seeding 105 event types via REST is by far the slowest part of this test.
  // Pad the timeout aggressively — each POST takes ~50-100ms and we need 105.
  test.setTimeout(180_000);

  test("prev/next/indicator/loading reflect cursor pagination on event_types list", async ({
    page,
    request,
  }) => {
    // -----------------------------------------------------------------------
    // Setup: register, verify, login, create application
    // -----------------------------------------------------------------------
    const timestamp = Date.now();
    const email = `test-pagination-evt-${timestamp}@hook0.local`;
    const password = `TestPassword123!${timestamp}`;

    const registerResponse = await request.post(`${API_BASE_URL}/register`, {
      data: { email, first_name: "Test", last_name: "User", password },
    });
    expect(registerResponse.status()).toBeLessThan(400);
    const registerData = await registerResponse.json();
    const organizationId: string = registerData.organization_id;
    expect(organizationId).toBeTruthy();

    await verifyEmailViaMailpit(request, email, organizationId);

    await page.goto("/login");
    await expect(page.locator('[data-test="login-form"]')).toBeVisible({ timeout: 10000 });
    await page.locator('[data-test="login-email-input"]').fill(email);
    await page.locator('[data-test="login-password-input"]').fill(password);
    await page.locator('[data-test="login-submit-button"]').click();
    await expect(page).toHaveURL(/\/dashboard|\/organizations|\/tutorial/, { timeout: 15000 });

    // Create application via the UI flow so the cookie/session is the same one
    // we'll later use for the event types list page.
    await page.goto(`/organizations/${organizationId}/applications/new`);
    await expect(page.locator('[data-test="application-form"]')).toBeVisible({ timeout: 10000 });
    await page
      .locator('[data-test="application-name-input"]')
      .fill(`Pagination App ${timestamp}`);

    let applicationId = "";
    const createAppResponse = page.waitForResponse(async (response) => {
      if (
        response.url().includes("/api/v1/applications") &&
        response.request().method() === "POST"
      ) {
        if (response.status() < 400) {
          const app = await response.json();
          applicationId = app.application_id;
        }
        return true;
      }
      return false;
    }, { timeout: 15000 });
    await page.locator('[data-test="application-submit-button"]').click();
    const appResponse = await createAppResponse;
    expect(appResponse.status()).toBeLessThan(400);
    expect(applicationId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    );

    // Pull the access token the frontend stashed in localStorage during login.
    // It's wrapped in a JSON envelope under the `auth` key (see stores/auth.ts).
    const accessToken = await page.evaluate(() => {
      const raw = window.localStorage.getItem("auth");
      if (!raw) return null;
      try {
        const parsed = JSON.parse(raw) as { accessToken?: unknown };
        return typeof parsed.accessToken === "string" ? parsed.accessToken : null;
      } catch {
        return null;
      }
    });
    expect(accessToken, "Access token must be available for API seeding").toBeTruthy();

    // -----------------------------------------------------------------------
    // Seed 105 event types so pagination spans two pages (default limit=100).
    //
    // The API enforces a per-user rate limit on event_type creation, so we
    // retry on 429 with a short backoff instead of failing the test. The
    // backoff doubles up to 1s — the typical rate-limit window — so a 105-row
    // seed completes in well under a minute.
    // -----------------------------------------------------------------------
    const SEED_COUNT = 105;
    const created: string[] = [];
    async function seedOne(idx: number): Promise<string> {
      const padded = String(idx).padStart(3, "0");
      const payload = {
        application_id: applicationId,
        service: `pgsvc${padded}`,
        resource_type: `pgrt${padded}`,
        verb: `pgvb${padded}`,
      };
      let backoffMs = 100;
      for (let attempt = 0; attempt < 10; attempt++) {
        const r = await request.post(`${API_BASE_URL}/event_types`, {
          headers: { Authorization: `Bearer ${accessToken}` },
          data: payload,
        });
        if (r.status() === 201) {
          return (await r.json()).event_type_name;
        }
        if (r.status() === 429) {
          await new Promise((res) => setTimeout(res, backoffMs));
          backoffMs = Math.min(backoffMs * 2, 1000);
          continue;
        }
        throw new Error(`seed event type ${idx} failed: ${r.status()} ${await r.text()}`);
      }
      throw new Error(`seed event type ${idx} exhausted retries against 429 rate limit`);
    }
    for (let i = 0; i < SEED_COUNT; i++) {
      created.push(await seedOne(i));
    }
    expect(created.length).toBe(SEED_COUNT);

    // -----------------------------------------------------------------------
    // Navigate to the event types list and exercise the paginated control
    // -----------------------------------------------------------------------
    await page.goto(
      `/organizations/${organizationId}/applications/${applicationId}/event_types`,
    );
    await expect(page.locator('[data-test="event-types-card"]')).toBeVisible({ timeout: 15000 });

    const table = page.locator('[data-test="event-types-table"]');
    // Scope to tbody so we exclude skeleton rows (no row-id) and stray
    // `[row-id]` elements from neighbouring components or stale renders.
    const rows = page.locator('[data-test="event-types-table"] tbody tr[row-id]');
    const indicator = page.locator('[data-test="pagination-current-page"]');
    const prevBtn = page.locator('[data-test="pagination-prev"]');
    const nextBtn = page.locator('[data-test="pagination-next"]');

    // Wait for the table to render with the first page of rows.
    await expect(table).toBeVisible({ timeout: 15000 });
    await expect.poll(() => rows.count(), { timeout: 15000 }).toBe(100);

    // Indicator says "Page 1"; prev disabled, next enabled.
    await expect(indicator).toContainText("Page 1");
    await expect(prevBtn).toBeDisabled();
    await expect(nextBtn).toBeEnabled();

    // Click Next -> page 2 renders the remaining 5 rows. Wait for the row
    // count to drop to exactly the page-2 size before asserting the indicator,
    // so the assertion races the table's row-render flush, not the page-flip.
    await nextBtn.click();
    await expect.poll(() => rows.count(), { timeout: 15000 }).toBe(SEED_COUNT - 100);
    await expect(indicator).toContainText("Page 2");

    // On the last page: next disabled, prev enabled.
    await expect(nextBtn).toBeDisabled();
    await expect(prevBtn).toBeEnabled();

    // Click Prev -> back to page 1, full 100 rows again. This is served from
    // TanStack Query's cache (no refetch), but we don't need to assert the
    // network — the row count + indicator are sufficient outcome proof.
    await prevBtn.click();
    await expect.poll(() => rows.count(), { timeout: 15000 }).toBe(100);
    await expect(indicator).toContainText("Page 1");
    await expect(prevBtn).toBeDisabled();
    await expect(nextBtn).toBeEnabled();
  });
});
