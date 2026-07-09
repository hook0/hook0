// Per-page strings for hook0-vs-convoy (EN base).
// Facts refreshed 2026-07-08 against the Convoy competitor snapshot:
// Convoy is ACTIVE (v26.6.2 on 2026-07-08), licensed Elastic License v2.0 (not MPL-2.0),
// has a cloud offering (no public pricing, no managed EU residency), pricing $0 -> $999/month flat.
module.exports = {
  pageTitle: 'Hook0 vs Convoy: Webhook Platforms Compared (2026)',
  pageDescription: 'Compare Hook0 (Rust, SSPL-1.0, EU-hosted cloud from €59/month) and Convoy (Go, Elastic License v2.0, $0 to $999/month). Features, licensing and pricing side by side.',
  pageModified: '2026-07-08',
  breadcrumb: 'Hook0 vs Convoy',
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Convoy',
    titleAccent: 'Same Problem, Different Trade-offs',
    subtitle: "Both publish their full source code. Both are built on PostgreSQL. The real differences sit elsewhere: Rust vs Go, SSPL-1.0 vs Elastic License v2.0, and a pricing ladder vs a flat $999/month. This page breaks down what actually matters when you're picking one for production.",
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  differentiators: {
    eyebrow: 'Why Hook0',
    h2: 'Key Differences',
    cards: [
      { title: 'A Middle Tier vs a $0 to $999 Jump', body: "Convoy's paid ladder has one step: the Community tier is free, then Premium lands at $999/month flat (JS transformations, RBAC and white-labeling included). There is nothing in between. Hook0 Cloud starts free, then moves to Startup at €59/month and Pro at €190/month, so a growing team never faces an abrupt pricing cliff with nothing in between." },
      { title: 'Managed EU Cloud vs Self-Host for Data Residency', body: 'Hook0\'s managed cloud runs its application data plane in France at Clever Cloud (the US Cloudflare CDN is disclosed in our <a href="/data-processing-addendum" class="underline">DPA</a>), designed for GDPR compliance. Convoy also has a cloud offering, but no managed EU data-residency option: picking the region your webhook data lives in requires self-hosting, which means you also own monitoring, backups, scaling and uptime.' },
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
      { feature: 'Managed Cloud', hook0Html: 'Yes (Clever Cloud FR, US Cloudflare CDN disclosed)', convoyHtml: 'Yes (no public pricing, no managed EU residency)' },
      { feature: 'Self-Hosting', hook0Html: 'Free (Docker / K8s)', convoyHtml: 'Free (Community tier)' },
      { feature: 'Paid Plans', hook0Html: 'Startup €59/month, Pro €190/month', convoyHtml: 'Premium $999/month (flat), Enterprise on quote' },
      { feature: 'SOC 2', hook0Html: 'Planned', convoyHtml: 'SOC 2 Type 1' },
      { feature: 'HMAC Signatures', hook0Html: 'Yes', convoyHtml: 'Yes' },
      { feature: 'Retry Logic', hook0Html: 'Configurable 2-phase (fast + slow, smart defaults)', convoyHtml: 'Configurable' },
      { feature: 'Primary Repo', hook0Html: '<a href="https://github.com/hook0/hook0" class="underline">GitHub</a> + <a href="https://gitlab.com/hook0/hook0" class="underline">GitLab</a>', convoyHtml: '<a href="https://github.com/frain-dev/convoy" class="underline">GitHub</a> (~2.8k stars)' },
      { feature: 'Funding', hook0Html: '100% Bootstrapped', convoyHtml: 'VC-backed (YC W22, Frain Technologies)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Convoy open-source?', a: 'Convoy publishes its full source code under the Elastic License v2.0, which is not an OSI-approved open-source license: it forbids offering Convoy as a managed service without a commercial agreement. Hook0 is in the same family: full source code under SSPL-1.0, also not OSI-approved, with a restriction aimed at cloud providers reselling it. If your procurement policy strictly requires an OSI license, neither qualifies.' },
      { q: 'Does Convoy have a managed cloud?', a: "Yes. Convoy offers a cloud version (the trial gives 1 project and 100 events per day) but does not publish cloud pricing, and there is no managed EU data-residency option: choosing where your webhook data lives requires self-hosting. Hook0's managed cloud is EU-hosted from the free tier onward, with paid plans at €59 and €190 per month." },
      { q: 'How do Hook0 and Convoy compare on pricing?', a: "For self-hosting, both are free. For paid features, Convoy goes straight from the free Community tier to Premium at $999/month flat, with nothing in between. Hook0 Cloud has a free tier, then Startup at €59/month and Pro at €190/month. If a flat all-inclusive bill fits your team, Convoy's Premium is predictable; if you want to start small and scale, Hook0 covers the middle of the market Convoy skips." },
      { q: 'How do Hook0 and Convoy compare on performance?', a: 'Hook0 is written in Rust, so there are no garbage collection pauses. That means more predictable latency and lower memory usage under load. Convoy is written in Go, which performs well but does have GC overhead. Infrastructure-wise, both need PostgreSQL, but Convoy also requires Redis.' },
      { q: 'What does Convoy do better than Hook0?', a: 'Convoy handles both inbound and outbound webhooks in one product, while Hook0 focuses on outbound delivery. Convoy also has a SOC 2 Type 1 attestation, more GitHub stars (~2,800), reference fintech customers such as Xendit and PiggyVest, and a flat $999/month Premium tier that some teams prefer for billing predictability.' },
      { q: 'Is Convoy still maintained?', a: "Yes. Convoy ships 2 to 3 releases per month (v26.6.2 landed in July 2026), its blog is active and the GitHub repository has multiple regular contributors. Treat any 'Convoy is dead' claim you find online as outdated." },
    ],
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
    ],
  },
};
