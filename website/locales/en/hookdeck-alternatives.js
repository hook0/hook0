// Per-page strings for hookdeck-alternatives (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
module.exports = {
  pageTitle: 'Best Hookdeck Alternatives (2026): Webhook Platforms That Do More',
  pageDescription: 'Hookdeck is a webhook proxy, not a webhook platform. Compare real alternatives: Hook0, Svix, Convoy for sending, managing, and monitoring webhooks.',
  breadcrumb: 'Hookdeck alternatives',
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hookdeck Alternatives',
    titleAccent: 'Hookdeck Is a Proxy. You Might Need a Platform',
    subtitleHtml: 'Hookdeck is a webhook gateway: it receives and routes incoming webhooks. If you need to <strong class="text-white">send</strong> webhooks to your users -- with retries, signatures, and subscriber management -- Hookdeck doesn\'t do that. These alternatives do.',
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  gatewayVsPlatform: {
    eyebrow: 'Key Distinction',
    h2: "Gateway vs Platform: What's the Difference?",
    sub: "Pick the wrong category and you'll end up building the missing half yourself.",
    cards: [
      { title: 'Webhook Gateway (Hookdeck)', bodyHtml: "A gateway sits between a third-party webhook sender and your application. It receives incoming webhooks, buffers them, retries failed deliveries, and routes events to the right endpoint. It's basically a reverse proxy for webhooks. <strong class=\"text-white\">You are the consumer.</strong>", color: 'indigo' },
      { title: 'Webhook Platform (Hook0, Svix, Convoy)', bodyHtml: 'A platform lets you send webhooks to your users. You publish events, the platform delivers them with retries, HMAC signatures, and a subscriber management portal. <strong class="text-white">You are the producer.</strong> This is what you need to add webhooks to your product.', color: 'green' },
    ],
  },
  comparison: {
    eyebrow: 'Feature Comparison',
    h2: 'Hookdeck vs the Alternatives',
    sub: 'Five options, one table. What matters most is whether you need to send webhooks, receive them, or both.',
    headers: { criteria: 'Criteria', hookdeck: 'Hookdeck', hook0: 'Hook0', svix: 'Svix', convoy: 'Convoy', awsEventbridge: 'AWS EventBridge' },
    rows: [
      { criteria: 'Type', hookdeckHtml: 'Webhook gateway / proxy', hook0Html: 'Full webhook platform', svixHtml: 'Webhook platform (open-core)', convoyHtml: 'Webhook platform', awsEventbridgeHtml: 'Event bus (AWS ecosystem)' },
      { criteria: 'Sending Webhooks', hookdeckHtml: 'No', hook0Html: 'Yes (core feature)', svixHtml: 'Yes', convoyHtml: 'Yes', awsEventbridgeHtml: 'Yes (via API Destinations)' },
      { criteria: 'Receiving Webhooks', hookdeckHtml: 'Yes (core feature)', hook0Html: 'No (by design)', svixHtml: 'No', convoyHtml: 'Yes (incoming + outgoing)', awsEventbridgeHtml: 'Yes (event ingestion)' },
      { criteria: 'Self-Hosting', hookdeckHtml: 'No', hook0Html: 'Free (Docker / K8s)', svixHtml: 'Enterprise plan only', convoyHtml: 'Yes (self-managed)', awsEventbridgeHtml: 'No (AWS only)' },
      { criteria: 'Open Source', hookdeckHtml: 'No (closed-source)', hook0Html: 'Yes (SSPL-1.0, full source)', svixHtml: 'Partial (open-core, enterprise closed)', convoyHtml: 'Yes (MPL-2.0)', awsEventbridgeHtml: 'No (AWS proprietary)' },
      { criteria: 'Free Tier', hookdeckHtml: 'Yes (100k events/mo)', hook0Html: 'Yes, no credit card', svixHtml: 'Yes', convoyHtml: 'Community edition only', awsEventbridgeHtml: 'Pay-per-use (AWS billing)' },
      { criteria: 'Data Hosting', hookdeckHtml: 'US-based', hook0Html: 'Europe (GDPR) or self-host', svixHtml: 'US-based', convoyHtml: 'Self-host only', awsEventbridgeHtml: 'Multi-region (AWS)' },
      { criteria: 'Funding', hookdeckHtml: '$3.5M VC-funded', hook0Html: '100% Bootstrapped', svixHtml: '$17M VC-funded', convoyHtml: 'VC-funded', awsEventbridgeHtml: 'Amazon (public company)' },
    ],
  },
  whyLookBeyond: {
    eyebrow: 'Why Look Beyond Hookdeck',
    h2: "When Hookdeck Isn't Enough",
    sub: 'Hookdeck does one thing well: receiving and routing webhooks. But there are clear cases where it falls short.',
    cards: [
      { title: 'You Need to Send Webhooks', body: "Hookdeck doesn't send webhooks. Period. If your product needs to notify customers via webhooks with retries, HMAC signatures, and delivery logs, you need a webhook platform: Hook0, Svix, or Convoy.", color: 'green' },
      { title: 'You Want to Self-Host', body: 'Hookdeck is cloud-only. There is no self-hosting option. If compliance or data sovereignty rules require you to run on your own infrastructure, Hook0 and Convoy are both self-hostable at no cost.', color: 'indigo' },
      { title: 'You Need European Data Hosting', body: "Hookdeck is US-based. Hook0 Cloud is hosted in Europe with GDPR compliance built in. If you're an EU company handling sensitive data, the choice is straightforward.", color: 'green' },
      { title: 'You Want to Audit the Source Code', body: "Hookdeck is closed-source. You can't see how your webhook data is processed. Hook0's entire codebase is open under SSPL-1.0, so you can read and audit every line.", color: 'indigo' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Hookdeck open-source?', a: 'No. Hookdeck is closed-source and cloud-only. You cannot inspect the code, audit it, or self-host it. If open-source matters to you, alternatives like Hook0 (SSPL-1.0) or Convoy (MPL-2.0) are fully open-source.' },
      { q: 'Can I self-host Hookdeck?', a: 'No. Hookdeck does not offer a self-hosted option. It is cloud-only. If you need to run your webhook infrastructure on your own servers for compliance, data sovereignty, or cost reasons, Hook0 and Convoy both support self-hosting.' },
      { q: "What's the difference between a webhook proxy and a webhook platform?", a: 'A webhook proxy (like Hookdeck) sits between a webhook sender and your application. It receives, routes, and retries incoming webhooks. A webhook platform (like Hook0 or Svix) lets you send webhooks to your users. It handles delivery, retries, signatures, and subscriber management for you. If you want to add webhooks to your product, you need a platform, not a proxy.' },
      { q: "What's the best Hookdeck alternative for sending webhooks?", a: 'Hook0, if you need to send webhooks. You publish events, Hook0 delivers them to your subscribers with retries, HMAC signatures, and a management dashboard. The code is open-source (SSPL-1.0), you can self-host it, the company is bootstrapped, and the cloud runs in Europe.' },
    ],
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Hook0 Alternatives' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
    ],
  },
};
