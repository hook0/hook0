// EN chrome strings (header, footer, CTA, social-proof). VERBATIM extraction
// from src/includes/_header.ejs, _footer.ejs, _cta-inline.ejs,
// _social-proof-bar.ejs — must stay byte-identical with the legacy includes
// (EN identity gate).
//
// SCOPE (MVP): translate only chrome strings that are language-independent
// of page routing — column titles, tagline, copyright, cookie settings,
// CTA copy, nav labels. Footer link LABELS (e.g. "Pricing", "Security")
// stay EN for now; they will be re-translated in a second pass once more
// target pages are localized (the link href routing depends on slugs.js).
module.exports = {
  header: {
    nav: {
      howItWorks: 'How it works',
      useCases: 'Use cases',
      pricing: 'Pricing',
      documentation: 'Documentation',
      play: 'Play',
      contact: 'Contact',
      login: 'Login',
      startFree: 'Start Free',
    },
    a11y: {
      openMenu: 'Open menu',
    },
  },
  footer: {
    srTitle: 'Footer',
    tagline: 'Open-Source Webhooks-as-a-Service. Built by developers for developers.',
    madeInEurope: 'Made in Europe',
    copyright: 'All rights reserved.',
    bootstrapped: '100% bootstrapped, no VCs. We are here to stay.',
    cookieSettings: 'Cookie Settings',
    titles: {
      about: 'About',
      compare: 'Compare',
      guides: 'Guides',
      developers: 'Developers',
      community: 'Community',
    },
  },
  cta: {
    h2: 'You have better things to build',
    subtitle: 'Stop building webhook infrastructure. Start shipping features. Get started in minutes.',
    pillAuth: 'Authentication',
    pillMonitoring: 'Monitoring',
    btnStartFree: 'Start Free',
    btnQuickStart: 'Quick Start Guide',
    trustNoCard: 'No credit card required',
    trustSetup: 'Setup in 5 minutes',
    trustCancel: 'Cancel anytime',
  },
  socialProof: {
    label: 'Trusted by teams at',
  },
};
