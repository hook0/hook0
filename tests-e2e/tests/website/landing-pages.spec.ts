import { test, expect } from "@playwright/test";

const LANDING_PAGES = [
  {
    path: "/hook0-vs-svix",
    h1Contains: "Hook0 vs Svix",
    title: "Hook0 vs Svix",
    faqCount: 4,
  },
  {
    path: "/hook0-vs-hookdeck",
    h1Contains: "Hook0 vs Hookdeck",
    title: "Hook0 vs Hookdeck",
    faqCount: 4,
  },
  {
    path: "/build-vs-buy-webhooks",
    h1Contains: "Stop Building Webhooks",
    title: "Build vs Buy",
    faqCount: 4,
  },
  {
    path: "/self-hosted-webhooks",
    h1Contains: "Self-Hosted",
    title: "Self-Hosted",
    faqCount: 5,
  },
];

for (const page of LANDING_PAGES) {
  test.describe(`Landing page: ${page.path}`, () => {
    test("returns HTTP 200 and correct H1", async ({ request }) => {
      const response = await request.get(page.path);
      expect(response.status()).toBe(200);

      const html = await response.text();
      expect(html).toContain(page.h1Contains);
    });

    test("has title tag", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      expect(html).toMatch(/<title>[^<]*<\/title>/);
      expect(html).toContain(page.title);
    });

    test("has meta description", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      expect(html).toMatch(/<meta\s+name="description"\s+content="[^"]+"/);
    });

    test("has canonical URL", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      expect(html).toContain(`<link rel="canonical"`);
      expect(html).toContain(page.path);
    });

    test("has FAQ schema JSON-LD", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      expect(html).toContain('"@type":"FAQPage"');

      // Count Question entries
      const questionCount = (html.match(/"@type":"Question"/g) || []).length;
      expect(questionCount).toBe(page.faqCount);
    });

    test("has Open Graph tags", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      expect(html).toMatch(/property="og:title"/);
      expect(html).toMatch(/property="og:description"/);
      expect(html).toMatch(/property="og:url"/);
    });

    test("has social proof bar (customer logos)", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      expect(html).toContain('alt="Coinbase"');
      expect(html).toContain('alt="WoodWing"');
      expect(html).toContain('alt="Optery"');
    });

    test("has GitHub stars widget", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      expect(html).toContain("ghbtns.com/github-btn.html");
    });

    test("has Product Hunt badge", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      expect(html).toContain("producthunt.com/widgets/embed-image");
    });

    test("has CTA linking to app.hook0.com/register", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      expect(html).toContain("https://app.hook0.com/register");
    });

    test("has Related pages links", async ({ request }) => {
      const response = await request.get(page.path);
      const html = await response.text();
      // Each page should link to at least 2 other landing pages
      const otherPages = LANDING_PAGES.filter((p) => p.path !== page.path);
      let linkCount = 0;
      for (const other of otherPages) {
        if (html.includes(other.path)) linkCount++;
      }
      expect(linkCount).toBeGreaterThanOrEqual(2);
    });
  });
}
