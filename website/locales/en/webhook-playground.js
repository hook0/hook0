// Per-page strings for webhook-playground (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
module.exports = {
  pageTitle: 'Free Webhook Tester Online — Test & Debug Webhooks Instantly | Hook0',
  pageDescription: 'Free webhook tester — no signup. Send test events, inspect payloads, verify HMAC signatures, debug delivery. Works with Stripe, GitHub, Shopify webhooks. Open-source.',
  hero: {
    badge: 'Free — No signup required',
    titleBefore: 'Test webhooks',
    titleAccent: 'in seconds',
    subtitle: 'Send events, inspect payloads, verify HMAC signatures, and debug delivery — all in your browser. The fastest way to test your webhook integration.',
    ctaPrimary: 'Open the Playground',
    ctaSecondary: 'See pricing — Free tier included',
  },
  features: {
    eyebrow: 'What you can do',
    h2: 'Everything you need to test webhooks',
    cards: [
      { title: 'Send test events', body: 'Fire webhook events with custom JSON payloads to any endpoint. See the response in real time.' },
      { title: 'Verify HMAC signatures', body: 'Check that your webhook receiver correctly validates HMAC-SHA256 signatures and rejects tampered payloads.' },
      { title: 'Inspect payloads', body: 'View HTTP headers, request body, status codes, and latency for every delivery attempt.' },
      { title: 'Test retry behavior', body: "Simulate endpoint failures and watch Hook0's configurable two-phase retry logic in action (fast retries, then slow retries)." },
      { title: 'Code examples', body: 'Copy-paste working code for Python, Node.js, Go, and Rust to integrate webhooks in minutes.' },
      { title: 'No signup, no install', body: 'Works instantly in your browser. No account creation, no CLI tool, no Docker setup needed to start testing.' },
    ],
  },
  toProduction: {
    h2: 'From testing to production in 5 minutes',
    subtitle: "When you're ready to send webhooks in production, Hook0 handles retries, HMAC signatures, delivery monitoring, and multi-tenant routing. Start with the free tier — no credit card required.",
    ctaPrimary: 'Try the Playground',
    ctaSecondary: 'View pricing',
  },
  faq: {
    items: [
      { q: 'Is Hook0 Playground free?', a: 'Yes. Hook0 Playground is completely free to use with no signup required. You can send webhook events, inspect payloads, verify HMAC signatures, and debug delivery issues — all in your browser.' },
      { q: 'What can I test with the webhook playground?', a: 'You can send test webhook events with custom payloads, inspect HTTP headers and response codes, verify HMAC-SHA256 signatures, test retry behavior, and debug endpoint connectivity. It supports all standard webhook patterns used by Stripe, GitHub, Shopify, and other platforms.' },
      { q: 'Do I need to create an account?', a: 'No. The playground works instantly without signup. If you want to save your configurations or send more than 100 events/day, you can create a free Hook0 account.' },
      { q: 'Is there a free webhook service I can use in production?', a: 'Yes. Hook0 Cloud scales from free to production — the Developer tier includes 100 events/day, HMAC signatures, delivery monitoring, and 7-day data retention. No credit card required. Self-hosting is also available for specific infrastructure requirements.' },
    ],
  },
};
