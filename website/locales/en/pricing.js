// Per-page strings for pricing (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
// The faq.items[].a text MUST match the visible <details> body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  "pageTitle": "Hook0 Pricing: Free Tier, No Credit Card | Webhook Plans",
  "pageDescription": "Free forever tier. Cloud from €59/month, self-hostable SSPL code. Compare plans: events/day, subscriptions, retention, support. No hidden fees.",
  "track": "pricing",
  "hero": {
    "h1": "Hook0 Pricing",
    "tagline": "Choose the plan that fits your team. Start free, scale to production."
  },
  "differentiators": {
    "h2": "Why Hook0 pricing is different",
    "cards": [
      {
        "title": "Bootstrapped, no VC",
        "body": "No pressure to raise prices. We grow with you, not against you."
      },
      {
        "title": "Open-source, no lock-in",
        "body": "Audit every line of code. Self-host for compliance. Start with Cloud for fastest path to production."
      },
      {
        "title": "No hidden fees",
        "body": "Retries are free. HMAC signatures included. No per-endpoint charges. Overage rates are shown on each plan."
      }
    ]
  },
  "faq": {
    "h2": "Pricing FAQ",
    "items": [
      {
        "q": "What happens if I exceed my daily event limit?",
        "a": "On the free Developer plan, extra events are blocked (HTTP 429). On paid plans (Startup and Pro), extra events are <strong>never blocked</strong>; they are billed at a per-event rate (€0.003/event on Startup, €0.0001/event on Pro). We chose not to interrupt delivery to avoid causing issues to customers building products on top of Hook0."
      },
      {
        "q": "How can I monitor my usage?",
        "a": "The Organization Dashboard in the Hook0 app shows your event consumption for the current day and past days. For billing details and invoice history, check your Stripe billing portal."
      },
      {
        "q": "Is Hook0 free?",
        "a": "Yes. Hook0 has a free Developer tier that includes 100 webhook events per day, HMAC signatures, and delivery monitoring. No credit card required. Hook0 is also open-source and self-hostable if you need data sovereignty or specific infrastructure requirements."
      },
      {
        "q": "Can I self-host Hook0 for free?",
        "a": "Yes. Hook0 is fully open-source under the SSPL-1.0 license. You can self-host it with Docker Compose or Kubernetes. When self-hosting, you manage your own infrastructure, scaling, updates, and monitoring. Most teams start with Hook0 Cloud for the fastest path to production."
      },
      {
        "q": "How does Hook0 pricing compare to Svix and Hookdeck?",
        "a": "Hook0 Cloud starts at €59/month vs Svix at $490/month for comparable features. Svix locks self-hosting behind enterprise pricing. Hookdeck has no self-hosted option. Hook0 is also fully open-source under SSPL, so you can self-host if you need data sovereignty."
      }
    ]
  }
};
