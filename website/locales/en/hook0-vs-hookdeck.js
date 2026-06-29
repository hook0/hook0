// Per-page strings for hook0-vs-hookdeck (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
module.exports = {
  pageTitle: 'Hook0 vs Hookdeck: Webhook Platform vs Gateway',
  pageDescription: 'Compare Hook0 and Hookdeck: webhook platform vs gateway, open-source vs closed-source, self-hostable vs cloud-only. See the key differences.',
  pageModified: '2026-06-22',
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Hookdeck',
    titleAccent: 'Open-Source Webhook Platform Alternative',
    subtitle: 'Looking for a Hookdeck alternative? Hook0 is a 100% open-source, EU-hosted webhook platform with no vendor lock-in. Hookdeck is a webhook gateway. They solve different problems. Here is what each one actually covers.',
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  platformVsGateway: {
    eyebrow: 'Core Difference',
    h2: 'Platform vs Gateway',
    intro: 'Hook0 and Hookdeck solve different problems. One sends webhooks, the other proxies them.',
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
      title: 'Hookdeck: Webhook Gateway',
      bullets: [
        'Proxy layer between senders and receivers',
        'Adds retries and queuing to existing webhooks',
        'Does not generate or send webhooks',
        'Closed-source, cloud-only',
        'No self-hosting option',
      ],
    },
  },
  comparison: {
    eyebrow: 'Feature Comparison',
    h2: 'Side by Side',
    headers: { feature: 'Feature', hook0: 'Hook0', hookdeck: 'Hookdeck' },
    rows: [
      { feature: 'Type', hook0Html: 'Full webhook platform', hookdeckHtml: 'Webhook gateway / proxy' },
      { feature: 'Open-Source', hook0Html: 'Yes (SSPL-1.0)', hookdeckHtml: 'No (closed-source)' },
      { feature: 'Self-Hosting', hook0Html: 'Yes (Docker / K8s)', hookdeckHtml: 'No' },
      { feature: 'Send Webhooks', hook0Html: 'Yes (core feature)', hookdeckHtml: 'No (proxy only)' },
      { feature: 'Subscriber Management', hook0Html: 'Built-in portal', hookdeckHtml: 'Not applicable' },
      { feature: 'HMAC Signatures', hook0Html: 'Generated automatically', hookdeckHtml: 'Verification only' },
      { feature: 'Event Type Management', hook0Html: 'Full event type registry', hookdeckHtml: 'No' },
      { feature: 'Free Tier', hook0Html: '100/day free, EU-hosted', hookdeckHtml: '100,000 events/month' },
      { feature: 'Data Hosting', hook0Html: 'Europe (GDPR) or self-host', hookdeckHtml: 'US-based' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    lastReviewed: 'Last reviewed June 2026.',
    items: [
      { q: 'What is the difference between Hook0 and Hookdeck?', a: "Hook0 is a webhook platform: you send events via API, Hook0 delivers them to subscribers with retries, signatures, and monitoring. Hookdeck is a gateway that sits between existing webhook senders and receivers to add reliability. It doesn't send webhooks itself." },
      { q: 'Is Hook0 open-source?', a: "Hook0's server is published under SSPL-1.0 and the SDKs under MIT. SSPL is a source-available copyleft license: you can inspect, modify, and self-host the entire platform freely. Hookdeck is closed-source and only available as a managed SaaS." },
      { q: 'Can I self-host Hook0?', a: 'Yes. Hook0 supports self-hosting via Docker Compose or Kubernetes at no cost. Hookdeck does not offer self-hosting: it is a cloud-only service.' },
      { q: 'Which should I choose?', a: "If you need to add webhooks to your product (send events to your users' endpoints), use Hook0. If you already receive webhooks from third parties and just need a reliability proxy, Hookdeck may fit. They're different tools for different problems." },
      { q: 'Is Hook0 hosted in the EU, unlike Hookdeck?', a: 'Hook0 Cloud is operated by a French company (FGRibreau SARL) on French infrastructure (Clever Cloud), so it falls outside US CLOUD Act jurisdiction, and your webhook payloads stay in the EU. Hookdeck is a US company. You can also self-host Hook0 so no webhook data leaves your network.' },
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
    ],
  },
};
