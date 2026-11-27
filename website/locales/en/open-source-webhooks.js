// Per-page strings for open-source-webhooks (EN base).
// VERBATIM extraction from the legacy inline template; do not humanize.
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  "tldr": {
    "h2": "Which open-source webhook server should you pick?",
    "body": "Pick Hook0 if you want the code you self-host to be the code that runs in the managed cloud. It is open-source under SSPL-1.0, runs on Docker or Kubernetes with every feature included, and Hook0 Cloud is EU-hosted from €59/month. The other projects differ mostly on license and on where the customer portal lives. Svix has an MIT core, and its App Portal is documented for the hosted service. Convoy uses the Elastic License 2.0 and puts Portal Links in a paid self-hosted plan. Hookdeck Outpost (Apache-2.0) includes a per-tenant portal but needs PostgreSQL, Redis and a message queue. WebhookX (Apache-2.0) keeps multi-tenancy behind a commercial license and documents no managed cloud."
  },
  "pageTitle": "Best Open-Source Webhook Server (2026) | Hook0",
  "pageDescription": "Open-source webhook servers compared: Hook0 (SSPL-1.0), Svix (MIT core), Convoy (ELv2), Hookdeck Outpost and WebhookX (Apache-2.0). License, stack, portal.",
  "pageModified": "2026-11-25",
  "track": "oss-webhooks",
  "hero": {
    "eyebrow": "Open-Source",
    "titleLine1": "Best Open-Source",
    "titleLine2": "Webhook Server",
    "subtitle": "Hook0 is open-source under SSPL-1.0; audit every line of code, self-host for compliance, or use Hook0 Cloud for managed infrastructure, automatic updates, and EU hosting. Bootstrapped, no open-core tricks.",
    "ctaPrimary": "Start Free on Cloud",
    "ctaPrimaryTrack": "oss-webhooks-hero-cloud-signup",
    "ctaSecondary": "Try the Playground",
    "ctaSecondaryHref": "https://play.hook0.com",
    "ctaSecondaryTrack": "oss-webhooks-hero-playground",
    "trustIndicators": [
      "Open-source under SSPL-1.0",
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
  "projects": {
    "eyebrow": "Projects",
    "h2": "Five open-source webhook servers side by side",
    "intro": "Each line comes from the project's public repository, documentation or pricing page, checked on 24 and 25 November 2026. The sources are listed at the bottom of this page.",
    "columns": {
      "project": "Project",
      "license": "License",
      "stack": "Stack",
      "selfHost": "Self-host",
      "portal": "Customer portal",
      "managed": "Managed cloud"
    },
    "rows": [
      {
        "project": "Hook0",
        "license": "SSPL-1.0",
        "stack": "Rust, PostgreSQL (Pulsar + S3 optional)",
        "selfHost": "Docker Compose, Kubernetes",
        "portal": "Subscriptions API and dashboard",
        "managed": "Hook0 Cloud, EU-hosted, free tier then from €59/month"
      },
      {
        "project": "Svix",
        "license": "MIT core, closed enterprise features",
        "stack": "Rust, PostgreSQL, Redis optional",
        "selfHost": "Docker Compose or a single container",
        "portal": "App Portal, documented for the hosted service",
        "managed": "Svix Cloud, free up to 50,000 messages/month"
      },
      {
        "project": "Convoy",
        "license": "Elastic License 2.0",
        "stack": "Go, PostgreSQL, Redis",
        "selfHost": "Docker Compose, Helm",
        "portal": "Portal Links, in the paid Premium self-hosted plan",
        "managed": "Convoy Cloud"
      },
      {
        "project": "Hookdeck Outpost",
        "license": "Apache-2.0",
        "stack": "Go, PostgreSQL, Redis, RabbitMQ or SQS",
        "selfHost": "Docker Compose, Helm, Railway",
        "portal": "Per-tenant portal included",
        "managed": "Hookdeck, $10 per million events, first 50,000 free"
      },
      {
        "project": "WebhookX",
        "license": "Apache-2.0; multi-tenancy, secret management and connect-auth need a commercial license",
        "stack": "Go, PostgreSQL 13+, Redis 6.2+",
        "selfHost": "Docker Compose or binary",
        "portal": "None documented",
        "managed": "None documented"
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
      "mit": "MIT (permissive)",
      "proprietary": "Proprietary (closed SaaS)"
    },
    "rows": [
      {
        "criteria": "Source code available",
        "sspl": "Yes, the whole codebase on GitHub & GitLab",
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
        "openCore": "Yes, MIT core server; supported on-prem on Enterprise",
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
        "openCore": "US or EU cloud, or self-host the core",
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
        "a": "Yes. Hook0 is open-source under the SSPL-1.0 license. Every line of code is on GitHub and GitLab. There is no proprietary enterprise edition."
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
      },
      {
        "q": "Is there an open-source webhook platform that is also EU-hosted?",
        "a": "Yes. Hook0 is open-source under SSPL-1.0, and Hook0 Cloud runs its data plane on Clever Cloud in France (inside the EU) on every plan. Many EU-hosted webhook services are closed-source and cloud-only, so you cannot read the code or run it yourself. With Hook0 you can audit the code, self-host it, or use the EU cloud; the CDN in front of the cloud is Cloudflare (US), disclosed in our public sub-processor list."
      },
      {
        "q": "Is WebhookX a good open-source alternative for sending webhooks?",
        "a": "WebhookX is an Apache-2.0 webhook gateway written in Go that receives, validates, transforms and delivers events. It needs PostgreSQL 13+ and Redis 6.2+, and its latest release on 25 November 2026 was v1.2.0. Multi-tenancy, secret management and the connect-auth plugin require a commercial license, and its README documents no managed cloud or customer portal. If you send webhooks to many customers and want tenant isolation without a separate license, Hook0 includes it in the SSPL-1.0 code."
      },
      {
        "q": "What is Hookdeck Outpost, and how does it compare with Hook0?",
        "a": "Outpost is the outbound delivery engine from Hookdeck: Go, Apache-2.0, multi-tenant, with a portal per tenant. Self-hosted, it runs on PostgreSQL, Redis and a RabbitMQ or SQS queue; managed, Hookdeck charges $10 per million events after the first 50,000. Hook0 runs on PostgreSQL alone for most setups, adding Pulsar and S3 only for high throughput, and its managed cloud is hosted in the EU."
      }
    ]
  },
  "comparisonSources": {
    "competitors": [
      "Svix",
      "Convoy",
      "Hookdeck Outpost",
      "WebhookX"
    ],
    "scope": "This page compares open-source and source-available webhook servers on license, stack, self-hosting, customer portal and managed offer. Facts come from each project's public repository, documentation and pricing page.",
    "sources": [
      {
        "label": "Svix source, license and runtime dependencies on GitHub",
        "url": "https://github.com/svix/svix-webhooks",
        "consulted": "2026-11-25"
      },
      {
        "label": "Svix App Portal documentation",
        "url": "https://docs.svix.com/app-portal",
        "consulted": "2026-11-25"
      },
      {
        "label": "Svix pricing",
        "url": "https://www.svix.com/pricing/",
        "consulted": "2026-11-24"
      },
      {
        "label": "Convoy source and license on GitHub",
        "url": "https://github.com/frain-dev/convoy",
        "consulted": "2026-11-25"
      },
      {
        "label": "Convoy Kubernetes installation guide",
        "url": "https://getconvoy.io/docs/deployment/install-convoy/kubernetes",
        "consulted": "2026-11-25"
      },
      {
        "label": "Convoy pricing",
        "url": "https://www.getconvoy.io/pricing",
        "consulted": "2026-11-25"
      },
      {
        "label": "Hookdeck Outpost source and license on GitHub",
        "url": "https://github.com/hookdeck/outpost",
        "consulted": "2026-11-25"
      },
      {
        "label": "Hookdeck Outpost pricing",
        "url": "https://hookdeck.com/outpost/pricing",
        "consulted": "2026-11-25"
      },
      {
        "label": "WebhookX source and license on GitHub",
        "url": "https://github.com/webhookx-io/webhookx",
        "consulted": "2026-11-25"
      },
      {
        "label": "WebhookX v1.2.0 release notes",
        "url": "https://github.com/webhookx-io/webhookx/releases/tag/v1.2.0",
        "consulted": "2026-11-25"
      }
    ],
    "updated": "2026-11-25"
  },
  "related": {
    "h2": "Related",
    "links": [
      { "label": "Self-Hosted Webhooks", "href": "./self-hosted-webhooks" },
      { "label": "Hook0 vs Svix", "href": "./hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "./hook0-vs-hookdeck" },
      { "label": "Hook0 vs Convoy", "href": "./hook0-vs-convoy" },
      { "label": "Build vs Buy Webhooks", "href": "./build-vs-buy-webhooks" },
      { "label": "Hook0 Alternatives", "href": "./hook0-alternatives" }
    ]
  }
};
