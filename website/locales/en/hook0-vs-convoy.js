// Per-page strings for hook0-vs-convoy (EN base).
// VERBATIM extraction from the legacy inline template: do not humanize.
module.exports = {
  pageTitle: 'Hook0 vs Convoy: Open-Source Webhook Platforms Compared (2026)',
  pageDescription: 'Both open-source, both on PostgreSQL. Compare Hook0 (Rust, SSPL, managed cloud) and Convoy (Go, MPL-2.0, self-host only). Side-by-side features and trade-offs.',
  breadcrumb: 'Hook0 vs Convoy',
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Convoy',
    titleAccent: 'Same Problem, Different Trade-offs',
    subtitle: "Both open-source. Both built on PostgreSQL. But the similarities stop there: Rust vs Go, managed cloud vs self-host only, SSPL vs MPL-2.0. This page breaks down what actually matters when you're picking one for production.",
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  differentiators: {
    eyebrow: 'Why Hook0',
    h2: 'Key Differences',
    cards: [
      { title: 'Managed Cloud vs Self-Host Only', body: 'Convoy is self-host only. No managed cloud, period. You run it, you maintain it. Hook0 lets you pick: use the managed cloud (hosted in Europe) or self-host for free with Docker or Kubernetes.' },
      { title: 'Rust vs Go', body: 'Hook0 is written in Rust. No garbage collector means no GC pauses, lower memory usage, and more predictable latency under load. Convoy is written in Go, which has good throughput but does use garbage collection. At high volume, the difference shows up in tail latencies.' },
      { title: 'SSPL vs MPL-2.0', body: "Convoy uses MPL-2.0. Very permissive, no restrictions on redistribution. Hook0 uses SSPL-1.0: full source is available, but cloud providers can't resell it as a competing service. Both are open-source. The difference is about what third parties can do with the code." },
      { title: 'European Hosting vs DIY Infrastructure', body: "Hook0's cloud runs in Europe, GDPR-compliant out of the box. With Convoy, you pick your own hosting location, but you also own the entire ops stack: monitoring, backups, scaling, uptime. There is no managed option, so it's all on you." },
    ],
  },
  comparison: {
    eyebrow: 'Feature Comparison',
    h2: 'Side by Side',
    headers: { feature: 'Feature', hook0: 'Hook0', convoy: 'Convoy' },
    rows: [
      { feature: 'License', hook0Html: 'SSPL-1.0 (full source available)', convoyHtml: 'MPL-2.0' },
      { feature: 'Language', hook0Html: 'Rust', convoyHtml: 'Go' },
      { feature: 'Database', hook0Html: 'PostgreSQL only', convoyHtml: 'PostgreSQL + Redis' },
      { feature: 'Managed Cloud', hook0Html: 'Yes (EU-hosted)', convoyHtml: 'No' },
      { feature: 'Self-Hosting', hook0Html: 'Free (Docker / K8s)', convoyHtml: 'Yes (only option)' },
      { feature: 'Free Tier', hook0Html: 'Yes (cloud)', convoyHtml: 'N/A (self-host only)' },
      { feature: 'HMAC Signatures', hook0Html: 'Yes', convoyHtml: 'Yes' },
      { feature: 'Retry Logic', hook0Html: 'Configurable 2-phase (fast + slow, smart defaults)', convoyHtml: 'Configurable' },
      { feature: 'Primary Repo', hook0Html: '<a href="https://github.com/hook0/hook0" class="underline">GitHub</a> + <a href="https://gitlab.com/hook0/hook0" class="underline">GitLab</a>', convoyHtml: '<a href="https://github.com/frain-dev/convoy" class="underline">GitHub</a> (~2.8k stars)' },
      { feature: 'Funding', hook0Html: '100% Bootstrapped', convoyHtml: 'VC-backed (Frain Technologies)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Convoy fully open-source?', a: 'Yes. Convoy uses the MPL-2.0 license, Hook0 uses SSPL-1.0. Both publish their full source code. The practical difference is in redistribution: MPL-2.0 has fewer restrictions, while SSPL-1.0 prevents cloud providers from offering the software as a competing managed service.' },
      { q: 'Does Convoy have a managed cloud?', a: 'No. Convoy is self-host only, so you run and maintain everything yourself. Hook0 has a managed cloud (hosted in Europe) and also supports free self-hosting with Docker or Kubernetes.' },
      { q: 'How do Hook0 and Convoy compare on performance?', a: 'Hook0 is written in Rust, so there are no garbage collection pauses. That means more predictable latency and lower memory usage under load. Convoy is written in Go, which performs well but does have GC overhead. Infrastructure-wise, both need PostgreSQL, but Convoy also requires Redis.' },
      { q: 'Which is better for self-hosting?', a: "Both can be self-hosted, but with Convoy that's your only option. Hook0 supports Docker Compose and Kubernetes for free self-hosting, and also has a managed cloud if you'd rather skip the ops work. One practical difference: Hook0 only needs PostgreSQL. Convoy needs PostgreSQL and Redis." },
    ],
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Hook0 Alternatives' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
    ],
  },
};
