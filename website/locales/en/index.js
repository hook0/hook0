// Per-page strings for the homepage (EN base). VERBATIM extraction from the
// legacy src/index.ejs (pre-i18n conversion). Section copy that the homepage
// reuses across includes (pricing, FAQ, features, etc.) lives in
// `locales/en/_chrome.js` under `includes.*` and is exposed via
// `locals.t.includes` by `getPageLocals`. EN identity gate enforces byte-for-
// byte match with the live homepage.
module.exports = {
  pageTitle: 'Hook0: Free Webhook Platform, HMAC Signatures, EU Hosting',
  pageDescription: 'Free webhook platform with HMAC signatures, configurable retries, and delivery monitoring. Self-host the SSPL code or use our EU-hosted cloud. No credit card.',
  pageModified: '2026-06-28',
  hero: {
    badgePillBootstrapped: '100% Bootstrapped',
    badgeOpenSource: 'Hook0 is Open-Source',
    headlineLine1: 'Webhooks',
    headlineLine2: 'As A Service',
    description: 'Hook0 is an Open-Source Webhooks-as-a-service (WaaS) that makes it easy for developers to send webhooks. Developers make one API call, and Hook0 takes care of deliverability, retries, security, and more.',
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Quick Start Guide',
    trustFreePlan: 'Free Plan',
    trustNoCard: 'No Credit Card',
    trustHostedEurope: 'Hosted in Europe',
    trustGdpr: 'GDPR Compliant',
    scrollLabel: 'Scroll',
    socialProductHuntAlt: 'Hook0 on Product Hunt',
    socialGithubStars: 'GitHub Stars',
  },
  goDeeper: {
    h2: 'Go deeper',
    cards: [
      { label: 'Built to Last', href: './built-to-last', color: 'green' },
      { label: 'Security', href: './security', color: 'indigo' },
      { label: 'Quick Start', href: 'https://documentation.hook0.com/tutorials/getting-started', color: 'yellow' },
      { label: 'Compare Alternatives', href: 'https://documentation.hook0.com/comparisons', color: 'purple' },
      { label: 'OSS Friends', href: './oss-friends', color: 'pink' },
      { label: 'Webhook Tester', href: 'https://play.hook0.com', color: 'emerald', external: true },
    ],
  },
  // Schema.org graph for the homepage. siteUrl is interpolated from the
  // data.js locals at render time so the file stays JSON-pure.
  pageSchema: {
    '@context': 'https://schema.org',
    '@graph': [
      {
        '@type': 'Organization',
        inLanguage: 'en',
        name: 'Hook0',
        alternateName: ['hook0', 'Hook Zero'],
        url: 'https://www.hook0.com',
        logo: 'https://www.hook0.com/mediakit/logo/110x110-white.png',
        description: 'Free webhook platform, open-source. Send webhooks with one API call; Hook0 handles retries, HMAC signatures, and delivery monitoring. Free forever, no credit card. Self-host or use our cloud.',
        foundingDate: '2021',
        founders: [
          { '@type': 'Person', name: 'Francois-Guillaume Ribreau' },
          { '@type': 'Person', name: 'David Sferruzza' },
        ],
        sameAs: [
          'https://twitter.com/hook0_',
          'https://github.com/hook0',
          'https://www.linkedin.com/company/hook0',
          'https://www.youtube.com/channel/UCFGvNaoV6Ycdb6uh1rIvMcg',
          'https://www.hook0.com/community',
        ],
        contactPoint: {
          '@type': 'ContactPoint',
          email: 'support@hook0.com',
          contactType: 'customer service',
        },
      },
      {
        '@type': 'SoftwareApplication',
        inLanguage: 'en',
        name: 'Hook0',
        alternateName: ['hook0', 'Hook Zero'],
        applicationCategory: 'DeveloperApplication',
        operatingSystem: 'Any',
        url: 'https://www.hook0.com',
        description: 'Open-source webhook infrastructure. Send webhooks with one API call; retries, HMAC signatures, and delivery monitoring included.',
        offers: {
          '@type': 'Offer',
          price: '0',
          priceCurrency: 'USD',
          description: 'Free tier: 100 webhook events/day, HMAC signatures, delivery monitoring. No credit card required.',
        },
        author: { '@type': 'Organization', name: 'Hook0' },
        license: 'https://spdx.org/licenses/SSPL-1.0.html',
        downloadUrl: 'https://github.com/hook0/hook0',
        softwareVersion: '2026',
        featureList: 'Webhook delivery, HMAC signatures, Configurable retries, Delivery monitoring, Subscriber portal, Self-hosting, REST API, SDKs',
      },
    ],
  },
  // Homepage FAQ used both for visible <details> (currently rendered by
  // _faq.ejs from chrome.includes.faq) AND for the FAQPage JSON-LD schema.
  // Kept here separately because the homepage ships a richer schema than the
  // visible 4-question include; these are the SEO-targeted answers.
  faq: {
    items: [
      { q: 'Does Hook0 have a free tier?', a: 'Yes. Hook0 has a free tier that includes 100 webhook events per day, HMAC signatures, and delivery monitoring. No credit card required. Hook0 is also open-source: you can self-host it at no cost.' },
      { q: 'What is webhooks as a service?', a: "Webhooks as a service means you send events to Hook0 via a REST API, and Hook0 delivers them to your users' endpoints. It handles retries, cryptographic signatures, delivery logging, and subscriber management so you don't have to build that infrastructure yourself." },
      { q: 'Is Hook0 open-source?', a: 'Yes. The source code is available on GitHub and GitLab under the SSPL-1.0 license (client SDKs are MIT). You can self-host it with Docker Compose or Kubernetes, or use the managed cloud service hosted in Europe.' },
      { q: 'Is SSPL really open source?', a: "Hook0's full server source is published under SSPL-1.0, and the client SDKs under MIT. SSPL is a source-available copyleft license: you can read, modify, self-host, and run Hook0 freely. It's stricter than MIT in one case. If you resell Hook0 as a managed service, you have to open-source your own infrastructure stack. If you're building on Hook0 or self-hosting it, SSPL adds no obligation over MIT." },
      { q: 'Can I self-host Hook0?', a: 'Yes. Hook0 supports self-hosting via Docker Compose or Kubernetes. The self-hosted version has the same features as the cloud version. Documentation is at documentation.hook0.com/self-hosting.' },
      { q: 'How does Hook0 compare to Svix, Hookdeck, or HostedHooks?', a: "Hook0 ships its full server source under SSPL-1.0 with no proprietary enterprise tier, and self-hosting is free on every plan. Svix's core is MIT but reserves several features for paid plans. Hookdeck is a webhook gateway and HostedHooks is closed-source. Hook0 is bootstrapped with no VC funding and runs both in the cloud (hosted in the EU) and on-premise." },
    ],
  },
};
