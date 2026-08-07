// Per-page strings for hook0-vs-hookdeck (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
module.exports = {
  pageTitle: 'Hook0 vs Hookdeck: Webhook Platform vs Gateway',
  pageDescription: 'Compare Hook0 and Hookdeck: one webhook platform vs a gateway and a separate delivery product, licences, self-hosting scope and enterprise pricing tiers.',
  pageModified: '2026-07-16',
  breadcrumb: 'Hook0 vs Hookdeck',
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Hookdeck',
    titleAccent: 'Open-Source Webhook Platform Alternative',
    subtitle: 'Looking for a Hookdeck alternative? Hook0 is a source-available (SSPL-1.0), EU-hosted webhook platform with no vendor lock-in. Hookdeck splits the job between an Event Gateway and Outpost, its delivery product. Here is what each one actually covers.',
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  platformVsGateway: {
    eyebrow: 'Core Difference',
    h2: 'Platform vs Gateway',
    intro: 'Hook0 sends webhooks from a single platform. Hookdeck proxies inbound traffic with its Event Gateway and sells outbound delivery separately as Outpost.',
    hook0: {
      title: 'Hook0: Webhook Platform',
      bullets: [
        "Send webhooks to your users' endpoints",
        'Manage subscriptions, event types, retries',
        'HMAC signatures, delivery logs, subscription management',
        'One API call to trigger an event',
        'Open-source, self-hostable',
      ],
    },
    hookdeck: {
      title: 'Hookdeck: Webhook Gateway + Outpost',
      bullets: [
        'Proxy layer between senders and receivers',
        'Adds retries and queuing to existing webhooks',
        'Sending lives in a separate product, Outpost',
        'Outpost is Apache-2.0, the Event Gateway is closed',
        'Self-hosting covers Outpost only',
      ],
    },
  },
  comparison: {
    eyebrow: 'Feature Comparison',
    h2: 'Side by Side',
    headers: { feature: 'Feature', hook0: 'Hook0', hookdeck: 'Hookdeck' },
    rows: [
      { feature: 'Type', hook0Html: 'Full webhook platform', hookdeckHtml: 'Event Gateway (inbound) + Outpost (outbound)' },
      { feature: 'Open-Source', hook0Html: 'Yes (SSPL-1.0)', hookdeckHtml: 'Partial (Outpost Apache-2.0, Gateway closed)' },
      { feature: 'Self-Hosting', hook0Html: 'Yes (Docker / K8s)', hookdeckHtml: 'Outpost only' },
      { feature: 'Send Webhooks', hook0Html: 'Yes (core feature)', hookdeckHtml: 'Yes (via Outpost)' },
      { feature: 'Subscriber Management', hook0Html: 'Built-in portal', hookdeckHtml: 'Not applicable' },
      { feature: 'HMAC Signatures', hook0Html: 'Generated automatically', hookdeckHtml: 'Verification only' },
      { feature: 'Event Type Management', hook0Html: 'Full event type registry', hookdeckHtml: 'No' },
      { feature: 'Free Tier', hook0Html: '100/day free, EU-hosted', hookdeckHtml: '100,000 events/month' },
      { feature: 'Data Hosting', hook0Html: 'Europe (GDPR) or self-host', hookdeckHtml: 'Canada-based, EU region available' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    lastReviewed: 'Last reviewed July 2026.',
    items: [
      { q: 'What is the difference between Hook0 and Hookdeck?', a: "Hook0 is a webhook platform: you send events via API, Hook0 delivers them to subscribers with retries, signatures, and monitoring. Hookdeck's Event Gateway sits between existing webhook senders and receivers to add reliability. It doesn't send webhooks itself: that is Outpost, Hookdeck's second product." },
      { q: 'Is Hook0 open-source?', a: "Hook0's server is published under SSPL-1.0 and the SDKs under MIT. SSPL is a source-available copyleft license: you can inspect, modify, and self-host the entire platform freely. Hookdeck publishes Outpost, its delivery component, under Apache-2.0, and keeps its Event Gateway closed-source and managed-only." },
      { q: 'Can I self-host Hook0?', a: 'Yes. Hook0 supports self-hosting via Docker Compose or Kubernetes at no cost, and the managed cloud runs that same code with no feature reserved for an enterprise tier. Hookdeck self-hosts Outpost under Apache-2.0, its Event Gateway is cloud-only, and on Outpost managed, SSO, RBAC and SCIM start on the Growth tier at $499/month minimum on top of the per-event cost.' },
      { q: 'Which should I choose?', a: "If you need to add webhooks to your product (send events to your users' endpoints), use Hook0. If you already receive webhooks from third parties and just need a reliability proxy, Hookdeck may fit. They're different tools for different problems." },
      { q: 'Is Hook0 hosted in the EU, unlike Hookdeck?', a: 'Hook0 Cloud is operated by a French company (FGRibreau SARL), with its data plane on Clever Cloud in France. The CDN and DDoS layer in front is Cloudflare (US), disclosed in a public sub-processor list with its transfer mechanism. Hookdeck is a Canadian company. And because Hook0 self-hosts on the same code, you can keep webhook data entirely inside your own network.' },
      { q: 'Does Hookdeck consider Hook0 an alternative?', a: 'Hookdeck publishes comparison pages that include Hook0, and so does Svix. You can read their own assessments alongside ours.' },
    ],
  },
  deepDive: {
    prefix: 'Want more detail?',
    linkText: 'Read the full comparison with architecture diagrams in our docs',
    linkHref: 'https://documentation.hook0.com/comparisons/hookdeck-vs-hook0',
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'hookdeck-alternatives', label: 'Hookdeck Alternatives' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
      { enSlug: 'webhook-cost-comparison', label: 'Webhook Cost Comparison' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU Webhook Infrastructure' },
    ],
  },
};
