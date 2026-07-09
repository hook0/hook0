// Per-page strings for hook0-vs-svix (EN base).
// VERBATIM extraction from the legacy inline template: do not humanize.
module.exports = {
  pageTitle: 'Hook0 vs Svix: Open-Source Webhook Platform Comparison',
  pageDescription: 'Compare Hook0 and Svix: open-source SSPL vs open-core, bootstrapped vs VC-funded, EU-hosted vs US, self-hostable on every plan. An honest side-by-side.',
  pageModified: '2026-06-22',
  breadcrumb: 'Hook0 vs Svix',
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Svix',
    titleAccent: 'Open-Source Webhook Platform Comparison',
    subtitle: 'Looking for a Svix alternative? Both are webhook platforms, but they differ on licensing, funding model, hosting, and what "open-source" actually means in practice. Hook0 is 100% open-source, bootstrapped, and EU-hosted, with no vendor lock-in.',
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  differentiators: {
    eyebrow: 'Why Hook0',
    h2: 'Key Differences',
    cards: [
      { title: 'Source-Available, No Closed Add-Ons', body: "Hook0's server ships under SSPL-1.0 and the SDKs under MIT. You get the whole platform: read it, modify it, self-host it. Svix's core is MIT, but enterprise features (SSO, advanced analytics, dedicated support) stay closed-source on paid plans." },
      { title: 'Bootstrapped From Day One', body: 'Svix is venture-funded. Investors expect a return, which means pressure to raise prices or get acquired. Hook0 is 100% bootstrapped. No board to please, no growth-at-all-costs mandate.' },
      { title: 'No Vendor Lock-In', body: 'Hook0 Cloud runs the same open-source code you can read and audit. If you ever need to, you can export and run it yourself (free, Docker or Kubernetes), so you are never trapped in a proprietary platform. Svix restricts self-hosting to enterprise customers.' },
      { title: 'EU Hosting, Outside the CLOUD Act', body: 'Hook0 Cloud runs on French infrastructure (Clever Cloud) and is operated by a French company, so it falls outside US CLOUD Act jurisdiction. Your webhook payloads stay in the EU. Svix is US-based. You can also self-host so no webhook data leaves your network.' },
    ],
  },
  comparison: {
    eyebrow: 'Feature Comparison',
    h2: 'Side by Side',
    headers: { feature: 'Feature', hook0: 'Hook0', svix: 'Svix' },
    rows: [
      { feature: 'License', hook0Html: 'SSPL-1.0 (full source available)', svixHtml: 'MIT (open-core, enterprise closed)' },
      { feature: 'Funding', hook0Html: '100% Bootstrapped', svixHtml: 'VC-funded' },
      { feature: 'Self-Hosting', hook0Html: 'Free (Docker / K8s)', svixHtml: 'Enterprise plan only' },
      { feature: 'Free Tier', hook0Html: 'Yes, no credit card', svixHtml: 'Yes' },
      { feature: 'HMAC Signatures', hook0Html: 'Included (all plans)', svixHtml: 'Included' },
      { feature: 'Retry Logic', hook0Html: 'Configurable per subscription (fast + slow phases, smart defaults)', svixHtml: 'Automatic retries' },
      { feature: 'Data Hosting', hook0Html: 'Europe (GDPR)', svixHtml: 'US-based' },
      { feature: 'Subscription Management', hook0Html: 'Included', svixHtml: 'App Portal (paid plans)' },
      { feature: 'Vendor Lock-in Risk', hook0Html: 'None (full source, self-hostable)', svixHtml: 'Moderate (enterprise features closed)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    lastReviewed: 'Last reviewed June 2026.',
    items: [
      { q: 'Is Hook0 open-source like Svix?', a: "Hook0's server is published under SSPL-1.0 and the client SDKs under MIT, with no proprietary enterprise tier. SSPL is a source-available copyleft license: you can read, modify, and self-host the whole platform freely. Svix's core is MIT, but several enterprise features are closed-source and only available on paid plans." },
      { q: "How does Hook0's free tier compare to Svix's?", a: "Hook0's free tier is free forever with no credit card: 100 events per day, HMAC signatures, and delivery monitoring, hosted in the EU. Paid plans scale with your volume on the same managed infrastructure, with every feature included and no enterprise paywall. Svix reserves several features for paid plans." },
      { q: 'Does Hook0 support Standard Webhooks?', a: 'Standard Webhooks is a specification authored by Svix. Hook0 signs every payload with HMAC-SHA256 and documents the scheme. Standard Webhooks support is planned.' },
      { q: 'Can I use Hook0 for regulated or compliance-sensitive workloads?', a: 'Yes. Hook0 Cloud keeps your webhook data in the EU, on French infrastructure operated by a French company, outside US CLOUD Act jurisdiction, which is what most compliance-sensitive teams need first. Because the full server source is open (SSPL-1.0), you can audit exactly how data is handled and you are never locked in. Formal third-party attestations such as SOC 2, HIPAA and PCI-DSS are planned.' },
      { q: 'Is Hook0 hosted in the EU and outside the US CLOUD Act?', a: 'Hook0 Cloud is operated by a French company (FGRibreau SARL) on French infrastructure (Clever Cloud), so it falls outside US CLOUD Act jurisdiction. Your webhook payloads, which often carry customer data, stay in the EU. Svix and Hookdeck are US companies. You can also self-host Hook0 so no webhook data leaves your network.' },
      { q: 'Can I self-host Hook0 for free?', a: 'Yes. The same open-source code runs free on Docker Compose or Kubernetes, which is what keeps you from ever being locked in. Most teams start on Hook0 Cloud (managed, EU-hosted, free tier) and keep self-hosting as their exit option. Svix offers self-hosting only on its enterprise plan.' },
      { q: 'Is Hook0 bootstrapped?', a: 'Yes. Hook0 is 100% bootstrapped with zero VC funding. Svix is venture-funded. Bootstrapped means Hook0 answers to its users, not to investors looking for an exit.' },
      { q: 'Do Svix and Hookdeck consider Hook0 a competitor?', a: 'Svix and Hookdeck each publish comparison pages that include Hook0. You can read their own assessments alongside ours.' },
    ],
  },
  deepDive: {
    prefix: 'Want more detail?',
    linkText: 'Read the feature-by-feature comparison in our docs',
    linkHref: 'https://documentation.hook0.com/comparisons/svix-vs-hook0',
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'svix-alternatives', label: 'Svix Alternatives' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
      { enSlug: 'webhook-cost-comparison', label: 'Webhook Cost Comparison' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU Webhook Infrastructure' },
    ],
  },
};
