// Per-page strings for hook0-vs-hookbase (EN base).
//
// Facts captured 2026-11-24 from hookbase.app (home, pricing, retries, customer
// portal, payload transforms, signature verification, MCP, the /compare/svix and
// /compare/hookdeck pages, privacy and terms), listed in comparisonSources.
// The page's structuring nuance: Hookbase covers BOTH directions (an inbound
// gateway with sources, routes, transforms and tunnels, plus outbound apps with a
// React portal), managed-only on Cloudflare. Hook0 covers the OUTBOUND side only,
// with the same code in its EU cloud and on your own servers.
// Honesty gate (content-seo doctrine #4):
//   - Hookbase's entry price is lower than Hook0's: say so, do not bury it.
//   - Hookbase states it is "managed-only": quote that, never "closed" or worse.
//   - No Hookbase page states a hosting region or an EU data-residency option on
//     the date above, so the page writes "Not stated", never "US-only".
//   - Hook0 does not transform payloads and has no production inbound gateway.
//   - No certification claim either way (product-marketing interdicts).
// SSPL-1.0 is written "open source (not OSI-approved)", never "fully open source".
module.exports = {
  pageTitle: 'Hook0 vs Hookbase: Open Source, EU-Hosted Webhooks',
  pageDescription: 'Hookbase is a managed webhook gateway on Cloudflare for inbound and outbound traffic. Hook0 sends webhooks to your users, open source and EU-hosted.',
  pageModified: '2026-11-24',
  breadcrumb: 'Hook0 vs Hookbase',
  tldr: {
    h2: 'Hook0 or Hookbase: which one fits your team?',
    body: "Pick Hookbase if you mostly receive webhooks from Stripe, GitHub or Shopify and want one managed gateway that verifies, transforms and routes them, with a free plan and paid plans from $7 a month. Pick Hook0 if you send webhooks to your own customers and need to run the same code in an EU cloud or on your own servers. Hook0 retries a failed delivery up to 24 times over 8 days on every plan, signs every request with HMAC-SHA256 and is published by a French company. Hookbase states it is managed-only.",
  },
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Hookbase',
    titleAccent: 'Edge Gateway or EU Sending Platform',
    subtitle: "Hookbase runs a webhook gateway on Cloudflare's network that receives, transforms and routes webhooks, and can also send them. Hook0 focuses on sending webhooks to your users, with free retries, HMAC signatures, per-attempt logs and a subscriber portal, in an EU cloud or on your own servers. Here is where each one is the better choice.",
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  differentiators: {
    eyebrow: 'What Each One Does',
    h2: 'Key Differences',
    cards: [
      { title: 'Self-Host the Same Code, or Stay Managed', body: "Hook0's server is open source under SSPL-1.0 (not OSI-approved), so the code behind Hook0 Cloud also runs on your Docker or Kubernetes cluster, and its 11 client SDKs are MIT. Hookbase's own comparison page describes Hookbase as <em>managed-only</em>, and no Hookbase page offers a licence or a self-hosted edition." },
      { title: 'Retries That Do Not Depend on the Plan', body: 'Hook0 retries a failed delivery up to 24 times within 8 days, from 3 seconds to 10 hours apart, and the schedule is the same on the free plan and on paid ones. Hookbase sets the attempt count by plan: 5 on Free (about 13 minutes), 11 on Starter, 13 on Pro and 15 on Business (about 5 days). See <a href="https://documentation.hook0.com/explanation/webhook-retry-logic" class="text-indigo-400 hover:text-indigo-300 underline">Hook0\'s exact retry schedule</a>.' },
      { title: 'EU Publisher, EU Cloud by Default', body: "Hook0 is published by a French company with no US parent, and Hook0 Cloud runs in the EU, hosted in France by default. Hookbase processes events on Cloudflare Workers across 300+ locations and stores payloads in Cloudflare R2. Its public pages name no legal entity and no hosting region, and its terms are governed by the laws of Delaware." },
      { title: 'One Job Done Deeply, or a Wider Gateway', body: 'Hookbase covers more ground: an inbound gateway with signature checks for 70+ providers, payload transforms in 8 languages, queue and storage destinations, local tunnels and an MCP server. Hook0 stays on outbound delivery. It sends the payload you publish unchanged, filters by event type and label, logs every attempt with its status code and response, and lets you replay any event from the dashboard or the API.' },
    ],
  },
  comparison: {
    eyebrow: 'Side by Side',
    h2: 'Feature Comparison',
    headers: { feature: 'Feature', hook0: 'Hook0', hookbase: 'Hookbase' },
    rows: [
      { feature: 'Webhook direction', hook0Html: 'Outbound (you send to your users)', hookbaseHtml: 'Inbound and outbound' },
      { feature: 'Free plan', hook0Html: '100 events a day, no credit card', hookbaseHtml: '1,000 events a month, hard cap' },
      { feature: 'Paid plans', hook0Html: 'From €59/month, overage billed per event', hookbaseHtml: '$7, $29 and $79/month, or $1 per 1,000 events' },
      { feature: 'Automatic retries', hook0Html: 'Up to 24 within 8 days, on every plan', hookbaseHtml: '5 (Free) to 15 (Business) attempts' },
      { feature: 'Signatures', hook0Html: 'HMAC-SHA256 on every request + TLS', hookbaseHtml: 'Standard Webhooks spec for outbound' },
      { feature: 'Subscriber portal', hook0Html: 'Embeddable', hookbaseHtml: 'React package (@hookbase/portal)' },
      { feature: 'Filtering', hook0Html: 'Event types and labels', hookbaseHtml: 'Event types, labels, route filters' },
      { feature: 'Payload transforms', hook0Html: 'No, payload sent as published', hookbaseHtml: 'Yes, 8 languages (from Starter)' },
      { feature: 'Inbound gateway', hook0Html: 'No (the CLI tunnel is for local development)', hookbaseHtml: 'Yes, 70+ provider signature checks' },
      { feature: 'Replay', hook0Html: 'Per event, from the dashboard or API', hookbaseHtml: 'Yes, with a dead letter queue' },
      { feature: 'Hosting', hook0Html: 'Managed EU cloud (Clever Cloud, France) + self-host', hookbaseHtml: 'Cloudflare Workers and R2, region not stated' },
      { feature: 'Source code', hook0Html: 'Server SSPL-1.0, 11 SDKs MIT', hookbaseHtml: 'Managed-only, no licence published' },
      { feature: 'Publisher', hook0Html: 'French company, no US parent', hookbaseHtml: 'Not stated (terms under Delaware law)' },
    ],
  },
  fit: {
    h2: 'Where Hookbase Fits Better',
    bullets: [
      'Most of your traffic is webhooks you receive from Stripe, GitHub, Shopify or Slack, and you want one place that checks their signatures, reshapes the payloads and routes them to your services, or to S3, SQS or Pub/Sub on Pro and Business. Hook0 does not do inbound.',
      'You need payload transforms written in JSONata, jq, JavaScript or Liquid between the provider and your app.',
      'You want a paid plan below €59 a month. Hookbase Starter is $7 a month for 25,000 inbound events.',
      'You want to manage webhooks from an AI assistant today, and Hookbase ships an MCP server, hosted or local.',
    ],
    linkPrefix: 'Comparing other inbound gateways as well? See',
    link: { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Hook0 an alternative to Hookbase?', a: 'For sending webhooks to your customers, yes. Both deliver events to subscriber endpoints with signatures, retries, logs and a portal. Hook0 adds an open-source server you can self-host, an EU publisher and EU hosting, and the same 24-retry schedule on every plan. For receiving and transforming webhooks from third parties, Hookbase covers ground Hook0 does not.' },
      { q: 'Can I self-host Hookbase?', a: "Not according to its public pages. Hookbase's comparison with Svix describes Hookbase as managed-only, and no page lists a licence or a self-hosted edition. Hook0's server is open source under SSPL-1.0 and runs on Docker or Kubernetes, the same code as Hook0 Cloud." },
      { q: 'Where is Hookbase hosted?', a: 'Hookbase says it processes events on Cloudflare Workers across 300+ locations and stores payloads encrypted at rest in Cloudflare R2. Its public pages do not state a hosting region or an EU data-residency option. Hook0 Cloud runs in the EU, hosted in France by default.' },
      { q: 'How many times does Hookbase retry a failed webhook?', a: 'It depends on the plan: 5 attempts on Free (about 13 minutes), 11 on Starter (about 28 hours), 13 on Pro (about 3 days) and 15 on Business (about 5 days), and the count is not configurable. Hook0 retries up to 24 times within 8 days on every plan, from 3 seconds to 10 hours apart.' },
      { q: 'Is Hookbase cheaper than Hook0?', a: 'At the entry level, yes. Hookbase has a free plan capped at 1,000 events a month and paid plans at $7, $29 and $79 a month. Hook0 has a free plan with 100 events a day and paid plans from €59 a month, with overage billed per event so traffic above the quota is still delivered. Compare at your own volume on the pricing pages.' },
      { q: 'Does Hook0 transform payloads like Hookbase?', a: 'No. Hook0 delivers the payload you publish exactly as you sent it, and chooses which subscriptions receive it by event type and labels. If you need to reshape third-party webhooks before they reach your app, that is a Hookbase feature.' },
      { q: 'Which should I choose, Hook0 or Hookbase?', a: 'Choose by direction and by where the data must live. Receiving and transforming webhooks from many providers on a managed edge gateway: Hookbase. Sending webhooks to your own customers, with an open-source server, EU hosting and a French publisher: Hook0.' },
      { q: 'Is Hook0 affiliated with Hookbase?', a: 'No. Hookbase is a trademark of its respective owner, and Hook0 is independent and not affiliated with or endorsed by Hookbase. This page is a factual comparison for teams evaluating both tools.' },
    ],
  },
  comparisonSources: {
    competitors: ['Hookbase'],
    scope: "This page compares the direction, pricing, retries, signatures, portal, transforms, hosting, source code and publisher of Hook0 and Hookbase. Hookbase facts come from its public website, pricing page, product pages, comparison pages, privacy policy and terms, all read on the date shown. Hook0 facts come from the Hook0 documentation and pricing page.",
    sources: [
      { label: 'Hookbase home page', url: 'https://www.hookbase.app/', consulted: '2026-11-24' },
      { label: 'Hookbase pricing', url: 'https://www.hookbase.app/pricing', consulted: '2026-11-24' },
      { label: 'Hookbase retries', url: 'https://www.hookbase.app/webhook-retries', consulted: '2026-11-24' },
      { label: 'Hookbase customer portal', url: 'https://www.hookbase.app/customer-portal', consulted: '2026-11-24' },
      { label: 'Hookbase payload transforms', url: 'https://www.hookbase.app/payload-transforms', consulted: '2026-11-24' },
      { label: 'Hookbase vs Svix (managed-only)', url: 'https://www.hookbase.app/compare/svix', consulted: '2026-11-24' },
      { label: 'Hookbase vs Hookdeck (Cloudflare Workers)', url: 'https://www.hookbase.app/compare/hookdeck', consulted: '2026-11-24' },
      { label: 'Hookbase terms (governing law)', url: 'https://www.hookbase.app/terms', consulted: '2026-11-24' },
      { label: 'Hook0 retry schedule', url: 'https://documentation.hook0.com/explanation/webhook-retry-logic', consulted: '2026-11-24' },
    ],
    updated: '2026-11-24',
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'hook0-vs-webhook-relay', label: 'Hook0 vs Webhook Relay' },
      { enSlug: 'hook0-vs-requeue', label: 'Hook0 vs Requeue' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Open-Source Webhooks' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU Webhook Infrastructure' },
      { enSlug: 'hook0-alternatives', label: 'Hook0 Alternatives' },
    ],
  },
};
