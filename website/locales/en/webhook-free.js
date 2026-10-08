// Per-page strings for webhook-free (EN base).
// Search intent: "free webhook" covers two jobs, receiving/inspecting a webhook
// someone else sends (play.hook0.com) and sending your own (Cloud free tier or
// self-hosted server). The page routes the reader to the right free path before
// anything else, then states exactly where each free option stops.
// Facts to keep true (re-check before editing):
//   - Cloud free Developer tier = src/includes/_pricing.ejs: 1 developer,
//     1 application, 10 event types, 10 subscriptions, 100 events/day, extra
//     events blocked, 7 days retention.
//   - License = SSPL-1.0 (LICENSE.txt at the repo root).
//   - play.hook0.com works without an account.
//   - Competitor cells mirror the wording already published on
//     svix-alternatives / webhook-service (Svix: MIT server, US and EU regions;
//     Hookdeck: Outpost self-hostable under Apache-2.0, Event Gateway cloud-only).
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
// SVG diagram labels live here too so the FR/DE diagrams read in their language.
module.exports = {
  "pageTitle": "Free Webhooks: Cloud, Self-Hosted or Playground | Hook0",
  "pageDescription": "Three genuinely free ways to use Hook0: test webhooks in your browser with no account, self-host the open-source server (SSPL-1.0) unlimited, or a free cloud tier with no credit card.",
  "pageModified": "2026-10-08",
  "track": "webhook-free",
  "selfHostHref": "https://documentation.hook0.com/self-hosting/docker-compose",
  "hero": {
    "eyebrow": "Free Webhooks",
    "titleLine1": "Free webhooks, three honest ways",
    "titleLine2": "Test one now, or run the server for free",
    "subtitleHtml": "Three things at Hook0 are genuinely free, with no hidden paywall. <strong class=\"text-gray-200\">play.hook0.com</strong> inspects and tests any webhook, no account. The <strong class=\"text-gray-200\">open-source server</strong> (SSPL-1.0) self-hosts unlimited on your own infra. And <strong class=\"text-gray-200\">Hook0 Cloud</strong> has a free tier: 100 events/day, no credit card, no feature gating. Pick what you actually need.",
    "ctaPlay": "Test a webhook now",
    "ctaSelfHost": "Self-host the server free",
    "microcopy": "No account to test. Unlimited self-host. Free cloud tier, no credit card."
  },
  "flow": {
    "title": "How Hook0 delivers a webhook",
    "desc": "An event is signed with HMAC, delivered, automatically retried after a 503 error, then accepted with a 200 response.",
    "event": "event",
    "sign": "sign",
    "deliver": "deliver",
    "retry": "retry",
    "caption": "Every event is signed (HMAC), delivered, retried automatically on failure, then logged, on the cloud and self-hosted alike."
  },
  "router": {
    "eyebrow": "Start here",
    "h2": "Receive or send? Pick your free path",
    "subtitle": "\"Free webhook\" covers two different jobs. The diagram shows which one you need. Both are free.",
    "diagram": {
      "title": "Receiving versus sending webhooks with Hook0",
      "desc": "On the left, you receive and inspect a webhook a service sends you, using the free play.hook0.com tool. On the right, you send your own HMAC-signed webhooks to your users, with the free Cloud tier or the self-hosted open-source server.",
      "receiveEyebrow": "RECEIVE & DEBUG",
      "receiveSub": "play.hook0.com, no account",
      "service": "a service",
      "serviceSub": "Stripe, GitHub…",
      "playSub": "inspect, replay",
      "sendEyebrow": "SEND SIGNED",
      "sendSub": "Cloud or self-host, production",
      "hook0Sub": "sign, retry",
      "signed": "HMAC signed",
      "autoRetry": "auto-retry on 5xx",
      "yourUser": "your user"
    },
    "captionHtml": "<strong class=\"text-gray-300\">Left</strong>, you receive &amp; inspect a webhook with the free <a href=\"https://play.hook0.com\" target=\"_blank\" rel=\"noopener\" class=\"text-indigo-400 hover:text-indigo-300 underline\" onclick=\"trackEvent('CTA', 'Click', 'webhook-free-roles-play')\">play.hook0.com</a> tool. <strong class=\"text-gray-300\">Right</strong>, you send your own signed webhooks to your users, free on the <a href=\"./self-hosted-webhooks\" class=\"text-indigo-400 hover:text-indigo-300 underline\" onclick=\"trackEvent('CTA', 'Click', 'webhook-free-roles-selfhost')\">self-hosted server</a> or the Cloud free tier.",
    "test": {
      "title": "I just need to test a webhook",
      "body": "Generate a public URL, inspect the payloads a service sends you, replay them. In your browser, nothing to install, no signup.",
      "cta": "Open play.hook0.com →",
      "note": "Free forever. Public tool, open-source."
    },
    "infra": {
      "title": "I need webhook infra for my product",
      "body": "Sign, retry, fan-out and monitor the webhooks you send to your own users. Self-host the open-source server, or start on the cloud free tier.",
      "ctaSelfHost": "Self-host free (SSPL)",
      "ctaCloud": "Start on Cloud free",
      "note": "Open-source unlimited, or 100 events/day on Cloud."
    }
  },
  "ways": {
    "eyebrow": "Three free ways",
    "h2": "Three ways to run Hook0 free, and where each stops",
    "subtitle": "No surprise paywall. Here is exactly what is free and where each option stops.",
    "cards": [
      {
        "id": "play",
        "title": "Playground",
        "badge": "Free forever, no account",
        "body": "Receive, inspect, debug and replay webhook payloads in the browser. The fastest way to see what a service is actually sending.",
        "bestFor": "Best for: debugging an integration right now.",
        "cta": "Open the playground →",
        "href": "https://play.hook0.com",
        "external": true
      },
      {
        "id": "selfhost",
        "title": "Self-hosted (open-source)",
        "badge": "Free forever, unlimited volume",
        "body": "Run the full Hook0 server on your own infra under SSPL-1.0. Same codebase as the cloud, all delivery features, your data stays with you. No license cost, no volume cap from Hook0.",
        "bestFor": "Best for: production with full control and no metering.",
        "cta": "Self-hosting guide →",
        "href": "https://documentation.hook0.com/self-hosting/docker-compose",
        "external": false
      },
      {
        "id": "cloud",
        "title": "Cloud free tier",
        "badge": "Free forever, no credit card",
        "body": "Managed by us, nothing to run. 1 application, 10 event types, 10 subscriptions, up to 100 events/day, 7 days retention. All features included. Extra events above 100/day are blocked.",
        "bestFor": "Best for: side-projects and a serious trial, no infra to own.",
        "cta": "Start free on Cloud →",
        "href": "https://app.hook0.com/register",
        "external": false
      }
    ]
  },
  "compare": {
    "eyebrow": "Honest comparison",
    "h2": "How \"free\" compares across webhook tools",
    "subtitle": "Most \"free\" webhook offers stop at a metered cloud tier. Hook0 self-hosts for free with the same feature set as its cloud.",
    "headers": {
      "criteria": "Criteria",
      "hook0": "Hook0",
      "svix": "Svix",
      "hookdeck": "Hookdeck",
      "webhooksite": "webhook.site"
    },
    "rows": [
      { "criteria": "Free cloud tier", "hook0": "Yes, no credit card", "svix": "Yes", "hookdeck": "Yes", "webhooksite": "Yes (tester only)" },
      { "criteria": "Self-host for free", "hook0": "Yes, full feature set (SSPL)", "svix": "MIT server, some hosted features missing", "hookdeck": "Outpost only (Apache-2.0)", "webhooksite": "No" },
      { "criteria": "Send webhooks to your users", "hook0": "Yes", "svix": "Yes", "hookdeck": "Yes", "webhooksite": "No (receive only)" },
      { "criteria": "Free tool to test webhooks", "hook0": "Yes (play.hook0.com)", "svix": "Yes", "hookdeck": "Console", "webhooksite": "Yes" },
      { "criteria": "Full source available", "hook0": "Yes, 100% (GitHub & GitLab)", "svix": "Core server only", "hookdeck": "Outpost only", "webhooksite": "No" },
      { "criteria": "Where the data runs", "hook0": "EU data plane on every plan, or self-host", "svix": "US and EU regions, or self-host", "hookdeck": "US, EU and Asia regions", "webhooksite": "Varies" }
    ],
    "footnote": "Comparison of publicly documented offers, last checked October 2026. Check each vendor's current plans before deciding."
  },
  "limits": {
    "eyebrow": "No surprises",
    "h2": "Where \"free\" ends",
    "intro": "Being upfront beats a hidden paywall. Here is exactly when you pay:",
    "itemsHtml": [
      "<strong class=\"text-gray-200\">Self-hosted stays free forever</strong>: no license cost, no volume cap. You only pay for your own infrastructure.",
      "<strong class=\"text-gray-200\">Cloud paid plans</strong> (Startup at €59/mo excl. VAT, then Pro) unlock higher volume, more applications and dedicated support. No feature is locked behind them; the free tier already has HMAC signing, retries, logs and the portal.",
      "<strong class=\"text-gray-200\">Scale and support are what you pay for.</strong> If you need more than 100 events/day on the cloud, or a managed SLA, that is the paid line."
    ],
    "cta": "See full pricing →",
    "ctaHref": "./pricing"
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Free webhook questions",
    "items": [
      {
        "q": "Is Hook0 really free?",
        "a": "Yes, in three honest ways. play.hook0.com is a free tool to test and inspect webhooks in your browser, no account needed. The open-source server is free forever to self-host with no volume cap imposed by Hook0 (SSPL-1.0). And Hook0 Cloud has a free Developer tier: 100 events/day, no credit card, no feature gating. No bait-and-switch."
      },
      {
        "q": "Is there a free open-source webhook server?",
        "a": "Yes. Hook0 is a free, open-source webhook server under SSPL-1.0, published on GitHub and GitLab. It ships with Docker Compose and Kubernetes manifests and runs on PostgreSQL. Self-hosting is free with no license cost and no volume cap from Hook0. It is the same codebase as the cloud, with the same features."
      },
      {
        "q": "What are the free tier limits on Hook0 Cloud?",
        "a": "The free Developer tier includes 1 developer, 1 application, 10 event types, 10 subscriptions, up to 100 events/day, and 7 days of data retention. All delivery features are included: HMAC signing, retries, delivery logs and the subscriber portal. No credit card is required and the tier is free forever for side-projects."
      },
      {
        "q": "Can I test a webhook for free without an account?",
        "a": "Yes. play.hook0.com gives you a public URL to receive, inspect and replay webhook payloads directly in your browser. There is nothing to install and no signup. It is free and itself open-source."
      },
      {
        "q": "Is self-hosting Hook0 free?",
        "a": "Yes. There is no license cost and no volume cap imposed by Hook0 when you self-host. You provide the infrastructure (Docker or Kubernetes plus PostgreSQL) and get community support on Discord. Managed scaling, an uptime SLA and managed updates are the things you take on yourself when self-hosting."
      },
      {
        "q": "What is the difference between play.hook0.com and Hook0 Cloud?",
        "a": "play.hook0.com is a free tool to receive, inspect and debug webhooks that other services send you. Hook0 Cloud is the infrastructure to send signed webhooks to your own users in production, with retries, a dead letter queue and a subscriber portal. Use Play to debug, use Cloud or self-hosted to ship."
      },
      {
        "q": "Does the free cloud tier expire or require a credit card?",
        "a": "No. The free Developer tier has no time limit and requires no credit card. It is free forever for side-projects."
      },
      {
        "q": "What happens when I exceed the free cloud tier?",
        "a": "Events beyond 100/day on the free tier are blocked until the next day. To send more, you upgrade to a paid plan (Startup at €59/mo excl. VAT, or Pro). Paid plans only unlock higher volume and support; no feature is locked behind them. If you want no volume cap at all, self-host the open-source server for free."
      }
    ]
  },
  "related": {
    "h2": "Related",
    "links": [
      { "label": "play.hook0.com", "href": "https://play.hook0.com", "external": true },
      { "label": "Webhook Playground", "href": "./webhook-playground" },
      { "label": "Stripe Webhook Tester", "href": "./stripe-webhook-tester" },
      { "label": "GitHub Webhook Tester", "href": "./github-webhook-tester" },
      { "label": "Shopify Webhook Tester", "href": "./shopify-webhook-tester" },
      { "label": "Open-Source Webhooks", "href": "./open-source-webhooks" },
      { "label": "Self-Hosted Webhooks", "href": "./self-hosted-webhooks" },
      { "label": "Webhook Platform", "href": "./webhook-platform" },
      { "label": "Pricing", "href": "./pricing" },
      { "label": "Hook0 vs Svix", "href": "./hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "./hook0-vs-hookdeck" }
    ]
  }
};
