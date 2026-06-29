// Per-page strings for hook0-alternatives (EN base).
// VERBATIM extraction from the legacy inline template; do not humanize.
module.exports = {
  pageTitle: 'Hook0 Alternatives; Honest Comparison (2026)',
  pageDescription: 'Looking for Hook0 alternatives? Compare Hook0, Svix, Hookdeck, and Convoy side by side on licensing, self-hosting, pricing, and features.',
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 Alternatives',
    titleAccent: 'An Honest Comparison',
    subtitle: 'Looking for a webhook platform? Someone published a "Hook0 Alternatives" page, so here is our side of the story. No spin; just facts, side by side.',
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  comparison: {
    eyebrow: 'Feature Comparison',
    h2: 'Hook0 vs the Alternatives',
    sub: 'Four webhook platforms, one table. Judge for yourself.',
    headers: { criteria: 'Criteria', hook0: 'Hook0', svix: 'Svix', hookdeck: 'Hookdeck', convoy: 'Convoy' },
    rows: [
      { criteria: 'Open-Source', hook0Html: 'Yes (SSPL-1.0, full source)', svixHtml: 'Partial (open-core, enterprise closed)', hookdeckHtml: 'No (closed-source)', convoyHtml: 'Yes (MPL-2.0)' },
      { criteria: 'Self-Hosting', hook0Html: 'Free (Docker / K8s)', svixHtml: 'Enterprise plan only', hookdeckHtml: 'No', convoyHtml: 'Yes (self-managed)' },
      { criteria: 'Free Tier', hook0Html: 'Yes, no credit card', svixHtml: 'Yes', hookdeckHtml: 'Yes (100k events/mo)', convoyHtml: 'Community edition only' },
      { criteria: 'Pricing Model', hook0Html: 'Per-event, transparent', svixHtml: 'Per-event + enterprise tiers', hookdeckHtml: 'Per-event, cloud-only', convoyHtml: 'Enterprise pricing' },
      { criteria: 'HMAC Signatures', hook0Html: 'Included (all plans)', svixHtml: 'Included', hookdeckHtml: 'Verification only', convoyHtml: 'Included' },
      { criteria: 'Retry Logic', hook0Html: 'Configurable per subscription (fast + slow phases)', svixHtml: 'Automatic retries', hookdeckHtml: 'Automatic retries', convoyHtml: 'Automatic retries' },
      { criteria: 'Funding', hook0Html: '100% Bootstrapped', svixHtml: '$17M VC-funded', hookdeckHtml: '$3.5M VC-funded', convoyHtml: 'VC-funded' },
      { criteria: 'Data Hosting', hook0Html: 'Europe (GDPR) or self-host', svixHtml: 'US-based', hookdeckHtml: 'US-based', convoyHtml: 'Self-host only' },
      { criteria: 'Type', hook0Html: 'Full webhook platform', svixHtml: 'Webhook platform (open-core)', hookdeckHtml: 'Webhook gateway / proxy', convoyHtml: 'Webhook platform' },
    ],
  },
  whatTheyLeftOut: {
    eyebrow: 'The Full Picture',
    h2: "What Their Comparison Page Doesn't Tell You",
    sub: 'Hookdeck published a "Hook0 Alternatives" page. We appreciate the attention. Here is what they left out.',
    cards: [
      { title: '"Hook0 is HTTPS-only"', body: 'Yes; and that is a feature, not a limitation. Sending webhook payloads over plain HTTP means your customers\' data transits in clear text. Every serious production system uses HTTPS. We enforce it because security is not optional.', color: 'green' },
      { title: '"No published SLA"', body: 'Hook0 Cloud Enterprise includes a custom SLA with dedicated support. If uptime guarantees matter, that is the fastest path; no infrastructure to manage, no ops team needed. Hook0 is also open-source, so you always have the option to self-host if your compliance requirements demand it.', color: 'indigo' },
      { title: '"Pricing is unclear"', body: 'Our pricing is public and per-event. No sales call required. No "contact us" wall. Cloud starts at €59/month; 8x cheaper than Svix for comparable features. Try getting that transparency from a VC-funded, closed-source competitor.', color: 'green' },
      { title: "What they won't mention: funding", body: 'Hookdeck raised $3.5M in VC. Svix raised $17M. Convoy is VC-funded too. Hook0 is 100% bootstrapped. When your webhook provider needs to 10x revenue to satisfy investors, guess whose prices go up? Not ours.', color: 'indigo' },
    ],
  },
  difference: {
    eyebrow: 'Why Hook0',
    h2: 'The Hook0 Difference',
    cards: [
      { title: 'Not Just a Proxy', body: 'Unlike Hookdeck, Hook0 sends webhooks on your behalf; retries, signatures, subscriber management. Not a middleware layer.' },
      { title: 'No Enterprise Paywall', body: "Unlike Svix, every feature ships in every plan. Self-hosting isn't locked behind a sales call." },
      { title: 'European & GDPR', body: 'Cloud hosted in EU. Data sovereignty built in. Bootstrapped; no US VC board deciding your data policy.' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'What are the best Hook0 alternatives?', a: 'The main alternatives to Hook0 are Svix (open-core, VC-funded), Hookdeck (closed-source webhook gateway), and Convoy (open-source, VC-funded). Each solves a different slice of the webhook problem. Hook0 is the only one that is fully open-source, bootstrapped, and self-hostable for free.' },
      { q: 'Is Hookdeck better than Hook0?', a: 'Hookdeck is a webhook gateway; it proxies existing webhooks for reliability. Hook0 is a webhook platform; it sends webhooks on your behalf with retries, signatures, and subscriber management. They solve different problems. If you need to add webhooks to your product, Hook0 is the right tool.' },
      { q: 'Should I use Svix or Hook0?', a: 'Both are webhook platforms, but they differ on licensing and funding. Svix is open-core (enterprise features are closed-source) and raised $17M in VC. Hook0 is fully open-source under SSPL, bootstrapped, and offers free self-hosting. If vendor independence and long-term pricing stability matter, Hook0 is the safer bet.' },
      { q: 'What does Hook0 cost?', a: 'Hook0 has a free tier with no credit card required. Hook0 is also open-source and self-hostable for compliance requirements. Hook0 Cloud adds managed infrastructure, EU hosting, automatic updates, and priority support. Paid plans start at €59/month with per-event pricing.' },
      { q: 'Does Hook0 work at scale?', a: "Yes. Hook0's architecture supports PostgreSQL-only for simplicity or Pulsar + S3 for high throughput. Cloud customers process millions of events per day. The same architecture runs identically when self-hosted." },
    ],
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'svix-alternatives', label: 'Svix Alternatives' },
      { enSlug: 'hookdeck-alternatives', label: 'Hookdeck Alternatives' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Best Open-Source Webhook Server' },
    ],
  },
};
