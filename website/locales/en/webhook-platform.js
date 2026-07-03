// Per-page strings for webhook-platform (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
module.exports = {
  "pageTitle": "Webhook Platform: Send, Sign, Retry, Monitor | Hook0",
  "pageDescription": "Open-source webhook platform delivering events with HMAC signatures, retries, dead letter queues and a subscriber portal. Free forever, no credit card.",
  "track": "webhook-platform",
  "hero": {
    "eyebrow": "Webhook Platform",
    "titleLine1": "The Open-Source Webhook Platform",
    "titleLine2": "Built for Production",
    "subtitle": "Hook0 handles the parts you keep putting off. HMAC-SHA256 signing on every payload. Retries that actually back off properly. A DLQ when a customer's endpoint has been 502'ing for six hours. And a subscriber portal so your users stop opening tickets to rotate a secret. Self-host it or use the cloud. No vendor lock-in either way.",
    "ctaPrimary": "Start Free",
    "ctaSecondary": "Try the Playground",
    "microcopy": "100 events/day free. No credit card. Open-source."
  },
  "socialProof": true,
  "capabilities": {
    "eyebrow": "Capabilities",
    "h2": "A webhook platform you don't have to write yourself",
    "subtitle": "A production webhook system is roughly 3 sprints of plumbing. Skip that.",
    "cards": [
      {
        "title": "HMAC signing",
        "body": "Payloads are signed so subscribers can verify the request came from you. Timestamps cover replay attacks."
      },
      {
        "title": "Two-phase retries",
        "body": "Fast retries for flaky endpoints, slow retries over days for real outages. Tunable per subscription, with a DLQ at the end."
      },
      {
        "title": "Delivery logs",
        "body": "Request, response, status code, latency. Replay any event from the dashboard. Support stops guessing."
      },
      {
        "title": "Subscriber portal",
        "body": "A drop-in UI where your users manage their endpoints, secrets and event filters. No more support tickets to rotate a key."
      },
      {
        "title": "SSPL, no open-core",
        "body": "Same code in cloud and self-hosted. Docker Compose or Kubernetes. No enterprise tier hiding the useful features."
      },
      {
        "title": "Hosted in Europe",
        "body": "EU data residency, GDPR by default. Matters when your customers care where their events land."
      }
    ]
  },
  "howItWorks": {
    "eyebrow": "Code",
    "h2": "One API call to deliver an event",
    "code": "// Trigger an event from anywhere in your backend.\nawait hook0.message.create(\"&lt;application_id&gt;\", {\n  event_type: \"invoice.paid\",\n  event_id:   \"evt_Wqb1k73rXprtTm7Qdlr38G\",\n  payload: {\n    invoice_id: \"in_8X9aBcDeFgHiJk\",\n    status:     \"paid\",\n    amount_eur: 4990\n  }\n});\n\n// What Hook0 does after this call:\n// signs the payload with HMAC, fans out to every matching\n// subscriber, retries failed deliveries on a two-phase schedule,\n// and stores the request/response for replay.\n",
    "docsLabel": "Read the getting-started guide →",
    "docsHref": "https://documentation.hook0.com/docs/getting-started",
    "docsTrack": "webhook-platform-docs"
  },
  "buildVsBuy": {
    "eyebrow": "Build vs Buy",
    "h2": "What rolling your own actually costs",
    "head": [
      "Concern",
      "Build it yourself",
      "Hook0 webhook platform"
    ],
    "rows": [
      [
        "Time to first webhook",
        "3+ sprints",
        "30 minutes"
      ],
      [
        "Retry logic",
        "You design backoff and DLQ",
        "Two-phase, configurable"
      ],
      [
        "HMAC signing and verification",
        "You implement and document",
        "Included, documented"
      ],
      [
        "Subscriber UI",
        "Another sprint",
        "Drop-in portal"
      ],
      [
        "Replay and debugging",
        "Custom dashboards",
        "Built-in logs and replay"
      ],
      [
        "Ongoing maintenance",
        "Forever, your team",
        "Ours"
      ],
      [
        "Lock-in risk",
        "N/A (your code)",
        "None. Full source, self-hostable."
      ]
    ],
    "footLabel": "Read the full build vs buy breakdown →",
    "footHref": "./build-vs-buy-webhooks"
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Webhook platform questions",
    "items": [
      {
        "q": "What is a webhook platform?",
        "a": "A webhook platform is the production infrastructure that sits between your application and your customers' endpoints. It signs payloads with HMAC, retries on failures with configurable backoff, stores delivery logs, and exposes a subscriber portal so your users can manage their own endpoints. Hook0 ships all of this as a managed service or a self-hosted Docker stack."
      },
      {
        "q": "Why use a webhook platform instead of building your own?",
        "a": "Building a production-grade webhook system means 3+ engineering sprints for retries, dead letter queues, HMAC signatures, monitoring, and a subscriber UI, plus ongoing maintenance forever. A webhook platform like Hook0 lets you ship in 30 minutes and reclaim that engineering time for product work."
      },
      {
        "q": "Is Hook0's webhook platform free?",
        "a": "Yes. Hook0 has a free forever tier with no credit card required. You also get the option to self-host the same codebase on your own infrastructure for free under SSPL-1.0. Paid plans only unlock higher volume and dedicated support; no feature gating."
      },
      {
        "q": "Can I self-host the Hook0 webhook platform?",
        "a": "Yes. The entire Hook0 webhook platform is open-source under SSPL-1.0 and ships with Docker Compose and Kubernetes manifests. Self-hosting is free, with no enterprise tier required, and the self-hosted edition has the same features as the cloud."
      },
      {
        "q": "How does Hook0 compare to Svix, Hookdeck or Convoy?",
        "a": "Hook0 is fully open-source (not open-core), 100% bootstrapped (no VC pressure on pricing), and self-hostable on any plan. Svix is open-core with an enterprise tier; Hookdeck is closed-source and US-only; Convoy is open-source but community-maintained. See the side-by-side comparisons for the details."
      }
    ]
  },
  "related": {
    "h2": "Related",
    "links": [
      {
        "label": "Webhook API",
        "href": "./webhook-api"
      },
      {
        "label": "Hook0 vs Svix",
        "href": "./hook0-vs-svix"
      },
      {
        "label": "Hook0 vs Hookdeck",
        "href": "./hook0-vs-hookdeck"
      },
      {
        "label": "Hook0 vs Convoy",
        "href": "./hook0-vs-convoy"
      },
      {
        "label": "Build vs Buy Webhooks",
        "href": "./build-vs-buy-webhooks"
      },
      {
        "label": "Self-Hosted Webhooks",
        "href": "./self-hosted-webhooks"
      },
      {
        "label": "Open-Source Webhooks",
        "href": "./open-source-webhooks"
      }
    ]
  }
};
