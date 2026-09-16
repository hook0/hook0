// Per-page strings for hookdeck-alternatives (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
module.exports = {
  pageTitle: 'Hookdeck Alternatives 2026: Open-Source, EU-Hosted Webhook Platforms',
  pageDescription: 'Hookdeck splits inbound and outbound into two products. See how Hook0, Svix, and Convoy compare on open-source licensing, free self-hosting, and EU data residency.',
  pageModified: '2026-09-16',
  breadcrumb: 'Hookdeck alternatives',
  tldr: {
    h2: 'The Short Answer',
    body: "Hookdeck's Event Gateway receives incoming webhooks, but sending them to your users runs through a separate product, Outpost. For a single open-source platform with EU data residency and free self-hosting, Hook0 fits. It runs under SSPL-1.0 with the full source open, keeps its data plane on Clever Cloud in France, bills per event from €59/month, and runs the same code in its cloud and on your own servers. Convoy and AWS EventBridge each cover part of that, and the table below shows where every option lands.",
  },
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hookdeck Alternatives',
    titleAccent: 'Hookdeck Is Two Products. You Might Want One Platform',
    subtitleHtml: 'Hookdeck\'s Event Gateway receives and routes incoming webhooks, and sending runs through a second product, Outpost. If you want a single platform to <strong class="text-white">send</strong> webhooks to your users, with retries, signatures, and subscriber management, running the same code in the cloud and self-hosted, these alternatives do that.',
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  gatewayVsPlatform: {
    eyebrow: 'Key Distinction',
    h2: "Gateway vs Platform: What's the Difference?",
    sub: "Pick the wrong category and you'll end up building the missing half yourself.",
    cards: [
      { title: 'Webhook Gateway (Hookdeck Event Gateway)', bodyHtml: "A gateway sits between a third-party webhook sender and your application. It receives incoming webhooks, buffers them, retries failed deliveries, and routes events to the right endpoint. It's basically a reverse proxy for webhooks. <strong class=\"text-white\">You are the consumer.</strong>", color: 'indigo' },
      { title: 'Webhook Platform (Hook0, Svix, Convoy)', bodyHtml: 'A platform lets you send webhooks to your users. You publish events, the platform delivers them with retries, HMAC signatures, and a subscriber management portal. <strong class="text-white">You are the producer.</strong> This is what you need to add webhooks to your product.', color: 'green' },
    ],
  },
  comparison: {
    eyebrow: 'Feature Comparison',
    h2: 'Hookdeck vs the Alternatives',
    sub: 'Five options, one table. What matters most is whether you need to send webhooks, receive them, or both.',
    headers: { criteria: 'Criteria', hookdeck: 'Hookdeck', hook0: 'Hook0', svix: 'Svix', convoy: 'Convoy', awsEventbridge: 'AWS EventBridge' },
    rows: [
      { criteria: 'Type', hookdeckHtml: 'Event Gateway (inbound) + Outpost (outbound)', hook0Html: 'Full webhook platform', svixHtml: 'Webhook platform (open-core)', convoyHtml: 'Webhook platform', awsEventbridgeHtml: 'Event bus (AWS ecosystem)' },
      { criteria: 'Sending Webhooks', hookdeckHtml: 'Yes (Outpost, separate product)', hook0Html: 'Yes (core feature)', svixHtml: 'Yes', convoyHtml: 'Yes', awsEventbridgeHtml: 'Yes (via API Destinations)' },
      { criteria: 'Receiving Webhooks', hookdeckHtml: 'Yes (core feature)', hook0Html: 'No (by design)', svixHtml: 'No', convoyHtml: 'Yes (incoming + outgoing)', awsEventbridgeHtml: 'Yes (event ingestion)' },
      { criteria: 'Self-Hosting', hookdeckHtml: 'Outpost only (Apache-2.0)', hook0Html: 'Free (Docker / K8s)', svixHtml: 'Enterprise plan only', convoyHtml: 'Yes (self-managed)', awsEventbridgeHtml: 'No (AWS only)' },
      { criteria: 'Open Source', hookdeckHtml: 'Partial (Outpost Apache-2.0, Gateway closed)', hook0Html: 'Yes (SSPL-1.0, full source)', svixHtml: 'Partial (open-core, enterprise closed)', convoyHtml: 'Source-available (Elastic License 2.0)', awsEventbridgeHtml: 'No (AWS proprietary)' },
      { criteria: 'Free Tier', hookdeckHtml: 'Yes (100k events/mo)', hook0Html: 'Yes, no credit card', svixHtml: 'Yes', convoyHtml: 'Community edition only', awsEventbridgeHtml: 'Pay-per-use (AWS billing)' },
      { criteria: 'Data Hosting', hookdeckHtml: 'Canada-based, EU region available', hook0Html: 'Europe (GDPR) or self-host', svixHtml: 'US-based', convoyHtml: 'Self-host only', awsEventbridgeHtml: 'Multi-region (AWS)' },
      { criteria: 'Funding', hookdeckHtml: '$3.5M VC-funded', hook0Html: '100% Bootstrapped', svixHtml: '$17M VC-funded', convoyHtml: 'VC-funded', awsEventbridgeHtml: 'Amazon (public company)' },
    ],
  },
  whyLookBeyond: {
    eyebrow: 'Why Look Beyond Hookdeck',
    h2: "When Hookdeck Isn't Enough",
    sub: 'Hookdeck does one thing well: receiving and routing webhooks. But there are clear cases where it falls short.',
    cards: [
      { title: 'You Need to Send Webhooks', body: "Hookdeck's Event Gateway doesn't send webhooks. That job belongs to Outpost, a separate product with its own plans. If you want one platform that publishes events and delivers them with retries, HMAC signatures, and delivery logs, look at Hook0, Svix, or Convoy.", color: 'green' },
      { title: 'You Want to Self-Host', body: 'Hookdeck opens only its Outpost component, published under Apache-2.0 and self-hostable. The Event Gateway stays closed-source and cloud-only. Hook0 and Convoy both self-host the whole product at no licence cost, and with Hook0 the self-hosted code is the code that runs the managed cloud.', color: 'indigo' },
      { title: 'You Need European Data Hosting', body: "Hookdeck is Canada-based, with an EU region available on managed Outpost. Hook0 Cloud is hosted in Europe with GDPR compliance built in. If you're an EU company handling sensitive data, the choice is straightforward.", color: 'green' },
      { title: 'You Want to Audit the Source Code', body: "Outpost is Apache-2.0, so Hookdeck's delivery code is readable, but the Event Gateway that ingests your webhooks stays closed. Hook0's entire codebase is open-source under SSPL-1.0, and the managed cloud runs that same code, so you can read and audit every line.", color: 'indigo' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Hookdeck open-source?', a: 'Partly. Outpost, Hookdeck\'s delivery component, is Apache-2.0 and self-hostable, and the repository is active. The Event Gateway that ingests webhooks stays closed-source and cloud-only. Hook0 is fully open-source under SSPL-1.0, with the whole platform self-hostable. Convoy runs on the Elastic License 2.0, which the OSI does not approve and which forbids offering Convoy as a managed service.' },
      { q: 'Can I self-host Hookdeck?', a: 'Partly. Outpost is Apache-2.0 and runs on your own infrastructure; the Event Gateway has no self-hosted option. Note that on Hookdeck Outpost managed, SSO, RBAC and SCIM are not in the $10/M Starter tier: they start on the Growth tier, at $499/month minimum on top of the per-event cost. If you need your whole webhook infrastructure on your own servers, Hook0 and Convoy both self-host end to end, and Hook0 runs the same code in its managed cloud with no feature gated behind an enterprise tier.' },
      { q: "What's the difference between a webhook proxy and a webhook platform?", a: 'A webhook proxy (like Hookdeck\'s Event Gateway) sits between a webhook sender and your application. It receives, routes, and retries incoming webhooks. A webhook platform (like Hook0 or Svix) lets you send webhooks to your users. It handles delivery, retries, signatures, and subscriber management for you. If you want to add webhooks to your product, you need a platform, not a proxy.' },
      { q: "What's the best Hookdeck alternative for sending webhooks?", a: 'Hook0, if you need to send webhooks. You publish events, Hook0 delivers them to your subscribers with retries, HMAC signatures, and a management dashboard. The code is open-source (SSPL-1.0), you can self-host it, the company is bootstrapped, and the cloud runs in Europe.' },
      { q: 'Which Hookdeck alternative is EU-hosted and open-source?', a: 'Hook0. Hookdeck opens only Outpost, its delivery component, under Apache-2.0; the Event Gateway stays closed-source and cloud-only. Hook0 runs its data plane on Clever Cloud in France (inside the EU), is open-source under SSPL-1.0, and self-hosts on Docker or Kubernetes, so you can read the code or keep webhook data inside your own network. The CDN in front of Hook0 Cloud is Cloudflare (US), disclosed in the public sub-processor list.' },
      { q: 'How do I migrate from Hookdeck to Hook0?', a: 'The two tools overlap on outbound delivery, so a migration mostly moves your sending path. Point your event publishing at Hook0\'s API, recreate your event types and subscriptions, and keep your subscribers\' endpoint URLs as they are. Both sign payloads with HMAC, so consumers verify the signature the same way once you share the new secret. Hook0 is open-source under SSPL-1.0, so you can run it in Hook0 Cloud or self-host the same code while you cut over.' },
      { q: 'Is there a free, open-source Hookdeck alternative?', a: 'Two options. Hook0 is fully open-source under SSPL-1.0 and self-hosts for free on Docker or Kubernetes, with the managed cloud running the same code. Convoy is source-available under the Elastic License 2.0, which the OSI does not approve and which forbids offering Convoy as a managed service. Hookdeck itself opens only Outpost, its delivery component, under Apache-2.0; the Event Gateway that ingests webhooks stays closed.' },
      { q: 'How does Hookdeck pricing compare to Hook0?', a: 'Hookdeck\'s Event Gateway has a free tier around 100k events per month. Outpost managed starts at $10 per million events on the Starter tier, but SSO, RBAC, and SCIM only appear on the Growth tier, which starts at $499 per month on top of usage. Hook0 bills per event from €59/month, publishes its full pricing, and self-hosts for free with no feature gated behind an enterprise plan.' },
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
