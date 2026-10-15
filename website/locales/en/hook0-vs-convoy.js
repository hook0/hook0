// Per-page strings for hook0-vs-convoy (EN base).
// Facts refreshed 2026-09-29 against the Convoy competitor snapshot:
// Convoy is ACTIVE (v26.8.0 on 2026-09-28, daily commits, ~2,875 stars), licensed
// Elastic License v2.0 (not MPL-2.0). Pricing rechecked 2026-10-15 on getconvoy.io/pricing:
// Cloud Pro $99/month flat (25 events/s, 7-day retention), Cloud Premium $499/month,
// US or EU region picked at signup, 14-day trial; self-hosted Premium $999/month.
module.exports = {
  pageTitle: 'Hook0 vs Convoy: Webhook Platforms Compared (2026)',
  pageDescription: 'Compare Hook0 (Rust, SSPL-1.0, EU-hosted cloud from €59/month) and Convoy (Go, Elastic License v2.0, cloud from $99/month). Features, licensing and pricing side by side.',
  pageModified: '2026-10-15',
  breadcrumb: 'Hook0 vs Convoy',
  tldr: {
    h2: 'The Short Answer',
    body: "Hook0 and Convoy both publish their full source and run on PostgreSQL, so the real trade-offs sit elsewhere. Hook0 is written in Rust under SSPL-1.0, with a managed EU cloud from €59/month and a Pro tier at €190. Convoy is written in Go under the Elastic License v2.0. Its cloud has a 14-day trial, then Pro at $99/month flat for 25 events per second and Premium at $499/month, in a US or EU region; its self-hosted Premium licence is $999/month. Choose Convoy for inbound plus outbound in one product and flat throughput pricing; choose Hook0 for a Rust data plane, a permanent free tier, a French operator, and a self-hosted server with every feature included.",
  },
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Convoy',
    titleAccent: 'Same Problem, Different Trade-offs',
    subtitle: "Both publish their full source code. Both are built on PostgreSQL. The real differences sit elsewhere: Rust vs Go, SSPL-1.0 vs Elastic License v2.0, and daily event quotas vs flat throughput tiers. This page breaks down what actually matters when you're picking one for production.",
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  differentiators: {
    eyebrow: 'Why Hook0',
    h2: 'Key Differences',
    cards: [
      { title: 'Event Quotas vs Throughput Tiers', body: "Convoy prices its cloud by throughput: Pro is $99/month flat for 25 events per second, Premium is $499/month with custom rate limits, and nothing stays free after the 14-day trial. Self-hosted, its Premium features (JS transformations, RBAC, white-labelling) cost $999/month. Hook0 Cloud prices by daily volume: free up to 100 events a day, Startup at €59/month, Pro at €190/month for 100,000 events a day, and overage is billed per event instead of blocking delivery. The self-hosted Hook0 server includes every feature at no licence cost." },
      { title: 'An EU Region vs a French Operator', body: 'Hook0\'s managed cloud runs its application data plane in France at Clever Cloud (the US Cloudflare CDN is disclosed in our <a href="/data-processing-addendum" class="underline">DPA</a>), designed for GDPR compliance, and Hook0 is operated by a French company. Convoy Cloud lets you pick a US or an EU region at signup and lists data residency options on its Enterprise plan; Convoy\'s company address is in San Francisco. Where your data sits and which law binds the vendor are two separate questions.' },
      { title: 'Rust vs Go', body: 'Hook0 is written in Rust. No garbage collector means no GC pauses, lower memory usage, and more predictable latency under load. Convoy is written in Go, which has good throughput but does use garbage collection. At high volume, the difference shows up in tail latencies.' },
      { title: 'SSPL-1.0 vs Elastic License v2.0', body: 'Convoy uses the Elastic License v2.0: full source available, but offering Convoy as a managed service requires a commercial agreement. Hook0 uses SSPL-1.0: full source available, but cloud providers cannot resell it as a competing service. Both are source-available licenses and neither is OSI-approved. The practical difference is which activity is restricted, not how much code you can read.' },
    ],
  },
  comparison: {
    eyebrow: 'Feature Comparison',
    h2: 'Side by Side',
    headers: { feature: 'Feature', hook0: 'Hook0', convoy: 'Convoy' },
    rows: [
      { feature: 'License', hook0Html: 'SSPL-1.0 (full source available, not OSI-approved)', convoyHtml: 'Elastic License v2.0 (full source available, not OSI-approved)' },
      { feature: 'Language', hook0Html: 'Rust', convoyHtml: 'Go' },
      { feature: 'Database', hook0Html: 'PostgreSQL only', convoyHtml: 'PostgreSQL + Redis' },
      { feature: 'Webhook Direction', hook0Html: 'Outbound (sending)', convoyHtml: 'Outbound + inbound' },
      { feature: 'Managed Cloud', hook0Html: 'Yes (Clever Cloud FR, US Cloudflare CDN disclosed)', convoyHtml: 'Yes (US or EU region, picked at signup)' },
      { feature: 'Self-Hosting', hook0Html: 'Free (Docker / K8s)', convoyHtml: 'Free (Community tier)' },
      { feature: 'Paid Plans', hook0Html: 'Startup €59/month, Pro €190/month', convoyHtml: 'Cloud Pro $99/month, Cloud Premium $499/month, self-hosted Premium $999/month, Enterprise on quote' },
      { feature: 'SOC 2', hook0Html: 'Planned', convoyHtml: 'SOC 2 Type 1' },
      { feature: 'HMAC Signatures', hook0Html: 'Yes', convoyHtml: 'Yes' },
      { feature: 'Retry Logic', hook0Html: 'Configurable 2-phase (fast + slow, smart defaults)', convoyHtml: 'Configurable' },
      { feature: 'Primary Repo', hook0Html: '<a href="https://github.com/hook0/hook0" class="underline">GitHub</a> + <a href="https://gitlab.com/hook0/hook0" class="underline">GitLab</a>', convoyHtml: '<a href="https://github.com/frain-dev/convoy" class="underline">GitHub</a> (~2.9k stars)' },
      { feature: 'Funding', hook0Html: '100% Bootstrapped', convoyHtml: 'VC-backed (YC W22, Frain Technologies)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Convoy open-source?', a: 'Convoy publishes its full source code under the Elastic License v2.0, which is not an OSI-approved open-source license: it forbids offering Convoy as a managed service without a commercial agreement. Hook0 is in the same family: full source code under SSPL-1.0, also not OSI-approved, with a restriction aimed at cloud providers reselling it. If your procurement policy strictly requires an OSI license, neither qualifies.' },
      { q: 'Does Convoy have a managed cloud?', a: "Yes. Convoy Cloud has a 14-day trial (1 project, 100 events per day), then Pro at $99/month flat for 25 events per second and Premium at $499/month with custom rate limits. You pick a US or an EU region at signup. Hook0's managed cloud is EU-hosted from the free tier onward, with paid plans at €59 and €190 per month." },
      { q: 'How do Hook0 and Convoy compare on pricing?', a: "Self-hosted, both servers are free, but Convoy's Premium features cost $999/month while the Hook0 server ships every feature. In the cloud, Convoy charges a flat price per throughput tier: $99/month for 25 events per second, $499/month for custom limits. Hook0 Cloud is free up to 100 events a day, then €59/month for 30,000 events a day and €190/month for 100,000, with overage billed per event. At a steady high volume Convoy Pro can cost less; for a small or growing volume, and for a free tier that never expires, Hook0 costs less." },
      { q: 'How do Hook0 and Convoy compare on performance?', a: 'Hook0 is written in Rust, so there are no garbage collection pauses. That means more predictable latency and lower memory usage under load. Convoy is written in Go, which performs well but does have GC overhead. Infrastructure-wise, both need PostgreSQL, but Convoy also requires Redis.' },
      { q: 'What does Convoy do better than Hook0?', a: 'Convoy handles both inbound and outbound webhooks in one product, while Hook0 focuses on outbound delivery. Convoy also has a SOC 2 Type 1 attestation, more GitHub stars (~2,875), publicly referenced fintech customers, and flat throughput-based cloud plans from $99/month that some teams prefer for billing predictability.' },
      { q: 'Is Convoy still maintained?', a: "Yes. Convoy ships 2 to 3 releases per month (v26.8.0 landed in September 2026), its blog is active and the GitHub repository has multiple regular contributors. Treat any 'Convoy is dead' claim you find online as outdated." },
    ],
  },
  comparisonSources: {
    competitors: ['Convoy'],
    scope: 'This page compares the public plans, license and maintenance status of Hook0 and Convoy. Prices are each vendor\'s published figures; maintenance is taken from Convoy\'s public GitHub activity.',
    sources: [
      { label: 'Convoy releases on GitHub', url: 'https://github.com/frain-dev/convoy/releases', consulted: '2026-09-29' },
      { label: 'Convoy pricing', url: 'https://www.getconvoy.io/pricing', consulted: '2026-10-15', archive: 'https://web.archive.org/web/20260604143101/https://www.getconvoy.io/pricing' },
      { label: 'Convoy license on GitHub', url: 'https://github.com/frain-dev/convoy/blob/main/LICENSE', consulted: '2026-10-04' },
    ],
    updated: '2026-10-15',
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Hook0 Alternatives' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
      { enSlug: 'webhook-cost-comparison', label: 'Webhook Cost Comparison' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU Webhook Infrastructure' },
      { enSlug: 'sovereign-webhooks', label: 'Sovereign Webhooks' },
      { enSlug: 'webhooks-for-ai-agents', label: 'Webhook MCP Server' },
    ],
  },
};
