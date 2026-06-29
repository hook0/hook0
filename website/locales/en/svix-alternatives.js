// Per-page strings for svix-alternatives (EN base).
// VERBATIM extraction from the legacy inline template: do not humanize.
module.exports = {
  pageTitle: 'Best Svix Alternatives (2026): Open-Source Webhook Platforms Compared',
  pageDescription: 'Evaluating Svix? Compare Hook0, Hookdeck, Convoy & more. Side-by-side on pricing, self-hosting, licensing, and what "open-source" means in practice.',
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Looking for a Svix Alternative?',
    titleAccent: 'Open-Source Webhook Platforms Compared',
    subtitle: "Svix is a good webhook platform. It is not the only one. If you care about full open-source licensing, free self-hosting, EU data residency, or a vendor that won't jack up prices after a Series B, this page breaks down your options.",
    ctaPrimary: 'Start Free with Hook0',
    ctaSecondary: 'Try the Playground',
  },
  whyLookBeyond: {
    eyebrow: 'Why Look Beyond Svix',
    h2: 'Why Teams Look Elsewhere',
    cards: [
      { title: 'Open-Core Limitations', body: "Svix's MIT base is real open-source. The catch: enterprise features like SSO, advanced analytics, and dedicated support are proprietary. When you scale, you hit a paywall. If your team needs full source access, that's a problem." },
      { title: '$17M in VC Means Pressure', body: "Venture capital expects a return. Svix raised $17M -- that money has to come back somehow, usually through price hikes, feature gating, or an acquisition. A bootstrapped vendor doesn't have that pressure." },
      { title: 'No European Hosting', body: "Svix is US-based and has no EU cloud option. If you're subject to GDPR or data sovereignty rules, that's a blocker. You could self-host, but that requires their enterprise plan." },
    ],
  },
  comparison: {
    eyebrow: 'Feature Comparison',
    h2: 'Svix vs the Alternatives',
    sub: 'Five webhook platforms side by side. Data speaks louder than marketing pages.',
    headers: { criteria: 'Criteria', svix: 'Svix', hook0: 'Hook0', hookdeck: 'Hookdeck', convoy: 'Convoy', hostedhooks: 'HostedHooks' },
    rows: [
      { criteria: 'License', svixHtml: 'MIT (open-core, enterprise closed)', hook0Html: 'SSPL-1.0 (full source available)', hookdeckHtml: 'Closed-source', convoyHtml: 'MPL-2.0', hostedhooksHtml: 'Closed-source' },
      { criteria: 'Funding', svixHtml: '$17M VC-funded', hook0Html: '100% Bootstrapped', hookdeckHtml: '$3.5M VC-funded', convoyHtml: 'VC-funded', hostedhooksHtml: 'Bootstrapped' },
      { criteria: 'Self-Hosting', svixHtml: 'Enterprise plan only (full features)', hook0Html: 'Free (Docker / K8s)', hookdeckHtml: 'No', convoyHtml: 'Yes (self-managed)', hostedhooksHtml: 'No' },
      { criteria: 'Free Tier', svixHtml: 'Yes', hook0Html: 'Yes, no credit card', hookdeckHtml: 'Yes (100k events/mo)', convoyHtml: 'Community edition only', hostedhooksHtml: 'Yes (limited)' },
      { criteria: 'HMAC Signatures', svixHtml: 'Included', hook0Html: 'Included (all plans)', hookdeckHtml: 'Verification only', convoyHtml: 'Included', hostedhooksHtml: 'Included' },
      { criteria: 'Retry Logic', svixHtml: 'Automatic retries', hook0Html: 'Configurable per subscription (fast + slow phases)', hookdeckHtml: 'Automatic retries', convoyHtml: 'Automatic retries', hostedhooksHtml: 'Automatic retries' },
      { criteria: 'Data Hosting', svixHtml: 'US-based', hook0Html: 'Europe (GDPR) or self-host', hookdeckHtml: 'US-based', convoyHtml: 'Self-host only', hostedhooksHtml: 'US-based' },
      { criteria: 'Open-Source Level', svixHtml: 'Partial (open-core)', hook0Html: 'Full (SSPL, no closed add-ons)', hookdeckHtml: 'None', convoyHtml: 'Full (MPL-2.0)', hostedhooksHtml: 'None' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Svix truly open-source?', a: "Partially. Svix's base is MIT-licensed, but enterprise features (SSO, advanced analytics, priority support) are proprietary and closed-source. This is called open-core. You can run the community edition, but key production features require a paid plan. Hook0, by contrast, ships everything under SSPL-1.0 with no closed-source add-ons." },
      { q: 'Can I self-host Svix for free?', a: "You can self-host the MIT-licensed community edition, but enterprise features are not included. Full self-hosting with all features requires Svix's enterprise plan. Hook0 and Convoy both offer free self-hosting with full feature parity." },
      { q: 'What is the best Svix alternative for startups?', a: "Hook0 works well for startups. Free tier, no credit card, per-event pricing starting at €59/month, and free self-hosting via Docker or Kubernetes. The company is 100% bootstrapped, so there's no VC pushing to raise prices next quarter. Convoy is worth a look too if MPL-2.0 licensing matters to you." },
      { q: 'How does Svix pricing compare to alternatives?', a: 'Svix offers a free tier and per-event paid plans, but self-hosting and enterprise features require enterprise pricing (contact sales). Hook0 Cloud starts at €59/month with transparent pricing and includes self-hosting for free on any plan. Hookdeck is cloud-only with per-event pricing. Convoy is self-hosted only with enterprise pricing for support. HostedHooks offers cloud-only paid plans.' },
    ],
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Hook0 Alternatives' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
    ],
  },
};
