// Per-page strings for webhook-cost-comparison (EN base).
// Hook0 figures verified against locales/en/pricing.js + src/includes/_pricing.ejs.
// Competitor figures sourced from their public pricing pages, checked 2026-07-08.
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  pageTitle: 'Webhook Cost Comparison: Hook0 vs Svix vs Hookdeck vs Convoy',
  pageDescription: 'What a webhook service costs at 100k to 10M events/month: Hook0 from €59, Svix from $490, Hookdeck Outpost $10/M, Convoy $0 or $999. Public pricing, July 2026.',
  pageModified: '2026-07-08',
  breadcrumb: 'Webhook Cost Comparison',
  track: 'webhook-cost-comparison',
  hero: {
    eyebrow: 'Cost Comparison',
    titleBefore: 'Webhook Cost Comparison:',
    titleAccent: 'What You Actually Pay in 2026',
    subtitle: 'Four webhook services, four billing models. We put the public prices of Hook0, Svix, Hookdeck Outpost and Convoy on the same volumes, from 100,000 to 10 million events per month, with the asterisks pricing pages tend to leave out.',
    ctaPrimary: 'Start Free',
    ctaSecondary: 'See Hook0 Pricing',
    microcopy: '100 events/day free. No credit card. Open-source (SSPL-1.0).',
  },
  socialProof: true,
  costTable: {
    eyebrow: 'Cloud vs Cloud',
    h2: 'Monthly price by event volume',
    subtitle: 'Managed cloud plans only, public prices checked on 8 July 2026. Self-hosting is covered further down.',
    headers: {
      provider: 'Service',
      volumes: ['100k events/mo', '1M events/mo', '3M events/mo', '10M events/mo'],
      eu: 'EU data residency (entry price)',
    },
    rows: [
      {
        provider: 'Hook0 Cloud¹',
        highlight: true,
        cells: ['€59 (Startup)', '€190 (Pro)', '€190 (Pro)', '≈ €890 (Pro + overage)'],
        eu: 'Included on every plan, free tier included. Data plane hosted in France (Clever Cloud).⁶',
      },
      {
        provider: 'Svix Free²',
        highlight: false,
        cells: ['≈ $5', '≈ $95', '≈ $295', '≈ $995'],
        eu: 'No EU cloud region documented on the public pricing page.',
      },
      {
        provider: 'Svix Professional³',
        highlight: false,
        cells: ['≈ $495', '≈ $585', '≈ $785', '≈ $1,485'],
        eu: 'From $490/mo: EEA DPA included, no documented EU-hosted region.',
      },
      {
        provider: 'Hookdeck Outpost (managed)⁴',
        highlight: false,
        cells: ['$1', '$10', '$30', '$100'],
        eu: 'EU region available at the same $10/M (exact regions not published).',
      },
      {
        provider: 'Convoy⁵',
        highlight: false,
        cells: ['n/a', 'n/a', 'n/a', 'n/a'],
        eu: 'No managed EU option, you pick your region by self-hosting.',
      },
    ],
    footnotes: [
      '¹ Hook0 quotas are daily; figures assume events spread evenly over a 30-day month. Startup (€59/mo) covers up to 30,000 events/day, then €0.003/event. Pro (€190/mo, or €1,824/yr with the 20% annual discount) covers up to 100,000 events/day, then €0.0001/event. Subscriptions and retries are free, and overage never blocks delivery.',
      '² Svix Free includes 50,000 messages/month, then $0.0001/message, capped at 200 msg/s with one connector and Svix branding. SOC 2 Type II, no-branding, static IPs and 90-day retention start on Professional.',
      '³ Svix Professional starts at $490/month with 50,000 messages included, then $0.0001/message. Volume discounts apply at higher volumes, so the 10M figure is an upper bound.',
      '⁴ Hookdeck Outpost managed is pay-as-you-go outbound delivery infrastructure at $10 per million events, no monthly minimum. Hookdeck also sells an inbound Event Gateway as a separate product with its own plans ($0 to $499/month).',
      '⁵ Convoy publishes no managed-cloud pricing (its cloud trial is limited to 1 project and 100 events/day). Self-hosted: Community is free, the Premium feature set is licensed at $999/month flat, infrastructure not included.',
      '⁶ The Hook0 data plane (API and database) runs on Clever Cloud in France, operated by a French company. Cloudflare (US) serves as CDN and is listed in our DPA.',
    ],
    pricesChecked: 'All prices were read from public pricing pages on 8 July 2026. Spotted an outdated figure? Tell us and we will fix it.',
  },
  methodology: {
    eyebrow: 'Methodology',
    h2: 'How to read this table',
    items: [
      'Managed cloud plans only. Self-hosting (licences plus infrastructure) is compared in the next section.',
      'Currencies as published: euros for Hook0, dollars for the rest. No conversion applied.',
      'Hook0 bills per event, not per delivery: one event fanning out to several subscriptions, retries included, counts once. Svix bills per message, Hookdeck Outpost per event.',
      'The sticker price is not the TCO: retention, throughput, EU data residency, support and compliance differ widely between plans. The footnotes carry the caveats.',
    ],
  },
  selfHost: {
    eyebrow: 'Self-Hosting',
    h2: 'Self-hosted webhook TCO: the licence is free, the ops are not',
    intro: 'All four products can run on your own infrastructure. The licence line is where they differ; everything else costs the same.',
    cards: [
      {
        title: 'Hook0 (SSPL-1.0)',
        body: 'Open-source (SSPL-1.0): the full server is published, nothing held back for an enterprise tier. Licence cost €0. Runs on Docker Compose or Kubernetes with PostgreSQL.',
      },
      {
        title: 'Svix (MIT, open-core)',
        body: 'The core server is MIT, but several enterprise features stay closed-source and Svix positions self-hosting for its enterprise customers. Licence cost $0 for the core.',
      },
      {
        title: 'Hookdeck Outpost (Apache 2.0)',
        body: 'Outpost is Apache 2.0 with no private fork: the self-hosted codebase matches the managed one. Licence cost $0. The managed service adds serverless scaling, SOC 2, SSO, RBAC and support.',
      },
      {
        title: 'Convoy (Elastic License v2)',
        body: 'Source-available but not OSI-approved: the licence forbids offering Convoy as a managed service. Community is free; the Premium feature set (JS transformations, RBAC, white-label portal) is licensed at $999/month.',
      },
    ],
    opsCard: {
      title: 'The bill nobody puts on their pricing page',
      body: 'Whichever licence you pick, self-hosting costs roughly the same: compute for the API and workers, a production PostgreSQL (plus queues or Redis depending on the product), monitoring, backups, security patching, version upgrades and an on-call rotation. Those line items do not care whose logo is on the repository. Compare licences and feature sets, then price the ops honestly: they quickly outgrow a €59 or $490 subscription.',
      close: 'That is also why we publish both: Hook0 Cloud when you want the ops included, self-hosting when you have the platform team for it.',
    },
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Webhook cost questions',
    items: [
      {
        q: 'How much does Hook0 cost?',
        a: 'Hook0 Cloud has a free Developer tier: 100 events/day, no credit card. Startup is €59/month for up to 30,000 events/day, then €0.003/event. Pro is €190/month for up to 100,000 events/day, then €0.0001/event, or €1,824/year with the 20% annual discount. Enterprise is custom. Self-hosting the open-source (SSPL-1.0) code is free.',
      },
      {
        q: 'What does a webhook service cost at 1 million events per month?',
        a: 'At 1M events/month on public July 2026 prices: Hook0 Pro is €190, Svix Professional is about $585, Hookdeck Outpost managed is about $10 and Convoy publishes no cloud price (self-hosting is free, its Premium licence is $999/month). The per-event sticker is only part of the bill: retention, throughput, EU data residency, support and compliance differ widely between those plans.',
      },
      {
        q: 'Why is Hookdeck Outpost so much cheaper per event?',
        a: '$10 per million events is a genuinely low delivery price and we will not pretend otherwise. Outpost managed is metered outbound delivery infrastructure. Hook0 sells flat plans that include the dashboard, per-subscription filtering on business attributes, 7 to 30 days of retention depending on the plan, an EU data plane and an on-premise option. Depending on which of those you need, either can be the cheaper total.',
      },
      {
        q: 'Is self-hosting a webhook service really free?',
        a: 'The licence usually is: Hook0 (SSPL-1.0), the Svix core (MIT) and Hookdeck Outpost (Apache 2.0) cost nothing to run yourself, while Convoy is free in Community and $999/month for Premium features. The infrastructure and operations are never free: compute, PostgreSQL, monitoring, upgrades and on-call time are the real self-hosted TCO, and they are the same whichever product you deploy.',
      },
      {
        q: 'Which webhook service includes EU data residency?',
        a: 'Hook0 includes it on every plan, free tier included: the data plane runs on Clever Cloud in France, and Cloudflare (a US company) serves as CDN, as disclosed in our DPA. Hookdeck Outpost managed offers an EU region at the same $10/M. Svix documents an EEA DPA from Professional ($490/month) but no EU-hosted region on its public pricing page. Convoy has no managed EU option; you choose your region by self-hosting.',
      },
    ],
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'pricing', label: 'Hook0 Pricing' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
    ],
  },
};
