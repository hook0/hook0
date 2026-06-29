// Per-page strings for open-source-webhooks (EN base).
// VERBATIM extraction from the legacy inline template; do not humanize.
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  "pageTitle": "Best Open-Source Webhook Server (2026) | Hook0",
  "pageDescription": "Compare open-source webhook servers: Hook0 (SSPL, full-feature), Svix (open-core), Convoy (MIT). Cloud from €59/month, or self-host for compliance.",
  "track": "oss-webhooks",
  "hero": {
    "eyebrow": "Open-Source",
    "titleLine1": "Best Open-Source",
    "titleLine2": "Webhook Server",
    "subtitle": "Hook0 is fully open-source under SSPL; audit every line of code, self-host for compliance, or use Hook0 Cloud for managed infrastructure, automatic updates, and EU hosting. Bootstrapped, no open-core tricks.",
    "ctaPrimary": "Start Free on Cloud",
    "ctaPrimaryTrack": "oss-webhooks-hero-cloud-signup",
    "ctaSecondary": "Try the Playground",
    "ctaSecondaryHref": "https://play.hook0.com",
    "ctaSecondaryTrack": "oss-webhooks-hero-playground",
    "trustIndicators": [
      "100% open-source",
      "Self-host available (Docker / K8s)",
      "Bootstrapped, no VC"
    ]
  },
  "socialProof": true,
  "whyOss": {
    "eyebrow": "Why Open-Source",
    "h2": "Why Your Webhook Server Should Be Open-Source",
    "cards": [
      {
        "icon": "audit",
        "title": "Audit Every Line of Code",
        "body": "Webhooks carry sensitive payloads. With open-source, your security team can audit exactly how data is handled, signed, and delivered. No black boxes."
      },
      {
        "icon": "lock",
        "title": "No Vendor Lock-in",
        "body": "If the vendor disappears, raises prices, or pivots, you still have the code. Fork it, maintain it, or migrate at your own pace. Your webhook infrastructure is yours."
      },
      {
        "icon": "selfhost",
        "title": "Self-Host Anywhere",
        "body": "Deploy on your own servers, your own cloud, or air-gapped networks. Open-source means you choose where your data lives, not the vendor."
      },
      {
        "icon": "community",
        "title": "Community & Contributions",
        "body": "Report bugs, send PRs, request features. Open-source projects align incentives: the product gets better because users can shape it directly."
      }
    ]
  },
  "comparison": {
    "eyebrow": "Licensing",
    "h2": "Webhook Licensing Models Compared",
    "columns": {
      "criteria": "Criteria",
      "sspl": "SSPL (Hook0)",
      "openCore": "Open-Core (Svix)",
      "mit": "MIT (Convoy)",
      "proprietary": "Proprietary (Hookdeck)"
    },
    "rows": [
      {
        "criteria": "Source code available",
        "sspl": "Yes, 100% on GitHub & GitLab",
        "openCore": "Partial (core only)",
        "mit": "Yes, on GitHub",
        "proprietary": "No"
      },
      {
        "criteria": "Can audit code",
        "sspl": "Every line, including infra",
        "openCore": "Core only, enterprise is closed",
        "mit": "Yes",
        "proprietary": "No"
      },
      {
        "criteria": "Can self-host",
        "sspl": "Yes, free (Docker / K8s)",
        "openCore": "Enterprise plan only",
        "mit": "Yes, free",
        "proprietary": "No"
      },
      {
        "criteria": "Feature parity (cloud = self-host)",
        "sspl": "Same codebase, all features",
        "openCore": "Different editions, features gated",
        "mit": "Cloud is a separate product",
        "proprietary": "N/A (cloud-only)"
      },
      {
        "criteria": "Vendor lock-in risk",
        "sspl": "Low; fork anytime, standard PostgreSQL",
        "openCore": "Medium; enterprise features lost if you leave",
        "mit": "Low; MIT allows forking",
        "proprietary": "High; no source, no self-host"
      },
      {
        "criteria": "Data sovereignty",
        "sspl": "Full control (self-host or EU Cloud)",
        "openCore": "US cloud or enterprise self-host",
        "mit": "Self-host only",
        "proprietary": "US cloud, no self-host option"
      },
      {
        "criteria": "Community contributions",
        "sspl": "PRs welcome on full codebase",
        "openCore": "PRs on core only",
        "mit": "PRs welcome",
        "proprietary": "No community access"
      },
      {
        "criteria": "License restrictions",
        "sspl": "Cannot resell as managed service",
        "openCore": "Enterprise features require paid license",
        "mit": "None (permissive)",
        "proprietary": "All usage subject to vendor terms"
      }
    ]
  },
  "differentiators": {
    "eyebrow": "Hook0 Difference",
    "h2": "What Sets Hook0 Apart",
    "cards": [
      {
        "icon": "audit",
        "title": "Audit Every Line",
        "body": "Webhooks carry sensitive payloads. Your security and compliance teams can review the entire codebase; API, worker, database schema; before deploying to production. No closed-source black boxes."
      },
      {
        "icon": "lock",
        "title": "No Vendor Lock-in",
        "body": "Migrate anytime. No proprietary APIs, no proprietary data formats. Hook0 stores everything in standard PostgreSQL. If you leave, your data and infrastructure knowledge come with you."
      },
      {
        "icon": "cloud",
        "title": "Cloud When You Want It",
        "body": "Start with Hook0 Cloud for the fastest path to production. Switch to self-host later for compliance or data sovereignty; or the other way around. Same codebase, zero migration effort."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Common Questions",
    "items": [
      {
        "q": "Is Hook0 open-source?",
        "a": "Yes. Hook0 is fully open-source under the SSPL-1.0 license. Every line of code is on GitHub and GitLab. There is no proprietary enterprise edition."
      },
      {
        "q": "What license does Hook0 use?",
        "a": "SSPL-1.0 (Server Side Public License). You can freely self-host, modify, and audit the code. The only restriction is offering Hook0 as a managed service to third parties without open-sourcing your stack."
      },
      {
        "q": "What does self-hosting Hook0 require?",
        "a": "Self-hosting Hook0 requires Docker Compose or Kubernetes and a PostgreSQL database. You manage your own infrastructure, scaling, backups, updates, and monitoring. The self-hosted binary is built from the same codebase as Hook0 Cloud; no features are stripped out. Hook0 Cloud handles all of that for you if you prefer a managed path."
      },
      {
        "q": "What are the risks of open-core webhook tools?",
        "a": "Open-core webhook tools split their codebase into a free community edition and a paid enterprise edition. The risk: features you rely on today (SSO, advanced monitoring, self-hosting support) can be moved behind the paywall at any time. You cannot audit the closed-source parts for security. And if you self-host, you run a stripped-down version. Hook0 avoids this; the full codebase is available under SSPL, with no enterprise edition."
      },
      {
        "q": "Is Hook0 really free to self-host?",
        "a": "Yes. Hook0 is open-source and self-hostable at no license cost. Hook0 Cloud adds managed infrastructure, automatic updates, EU hosting, priority support, and SLA; so you can focus on your product instead of operating webhook infrastructure. Start with the free cloud tier (100 events/day, no credit card)."
      }
    ]
  },
  "related": {
    "h2": "Related",
    "links": [
      { "label": "Self-Hosted Webhooks", "href": "./self-hosted-webhooks" },
      { "label": "Hook0 vs Svix", "href": "./hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "./hook0-vs-hookdeck" },
      { "label": "Build vs Buy Webhooks", "href": "./build-vs-buy-webhooks" },
      { "label": "Hook0 Alternatives", "href": "./hook0-alternatives" }
    ]
  }
};
