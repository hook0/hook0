// Per-page strings for webhook-service (EN base).
// Search intent: someone shopping for a managed webhook service, not reading a
// feature list. The page is an evaluation page (what the service owes you, what
// it costs, how you leave) so it stays distinct from ./webhook-platform, which
// is product-and-capabilities framed.
// Legal constraints (non-negotiable, see website/CLAUDE.local.md):
//   - Data plane = Clever Cloud SAS (France, EEA). Cloudflare, Inc. (USA) CDN is
//     DISCLOSED, framed by SCC 2021 + TIA and, where applicable, the EU-US DPF.
//   - NEVER "100% sovereign", "no data leaves the EU", "CLOUD Act free".
//   - GDPR = process/context claim ("designed for"), never "certified".
//   - License = "open-source (SSPL-1.0)" whenever mentioned.
// Every capability claimed below maps to one shipped on locales/en/webhook-platform.js
// (HMAC signing, two-phase retries, delivery logs, subscriber portal, SSPL no
// open-core, hosted in Europe). Nothing beyond that list is claimed.
// Competitor pricing from the competitor snapshots last checked 2026-07-16:
// Hookdeck Growth $499/mo, Svix Pro $490/mo, Convoy Premium $999/mo. Hook0 prices
// verified against src/includes/_pricing.ejs.
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  "pageTitle": "Webhook Service: Managed Delivery, With a Way Out | Hook0",
  "pageDescription": "A managed webhook service with HMAC signing, two-phase retries and per-attempt logs. EU data plane on every plan, and the same open-source (SSPL-1.0) codebase self-hosts when you want out.",
  "pageModified": "2026-08-13",
  "track": "webhook-service",
  "hero": {
    "eyebrow": "Webhook Service",
    "titleLine1": "A Managed Webhook Service",
    "titleLine2": "You Can Take With You",
    "subtitle": "Hook0 delivers your events, signs them, retries them when the receiver is down, and logs every attempt. It runs on an EU data plane from the free tier up. And the day a managed webhook service stops being the right answer, the same codebase self-hosts, open-source under SSPL-1.0.",
    "ctaPrimary": "Start Free",
    "ctaSecondary": "See Pricing",
    "ctaSecondaryHref": "./pricing",
    "microcopy": "100 events/day free. No credit card. EU data plane on every plan."
  },
  "socialProof": true,
  "checklist": {
    "eyebrow": "Evaluation",
    "h2": "What a webhook service owes you",
    "subtitle": "The parts teams discover after they have already shipped their own delivery loop. Here is where Hook0 stands on each one.",
    "cards": [
      {
        "title": "Signed payloads, every attempt",
        "bodyHtml": "Every delivery carries an HMAC-SHA256 signature, so the receiver can tell your events from anything else that found the endpoint. The verification steps are documented rather than left as an exercise."
      },
      {
        "title": "Retries that survive a bad afternoon",
        "bodyHtml": "Two-phase, configurable retries. A subscriber that is down for an hour does not cost you the events that happened during that hour, and you do not write the backoff schedule yourself."
      },
      {
        "title": "Proof of what actually happened",
        "bodyHtml": "Per-attempt delivery logs. When a customer says they never received the event, the answer is a lookup rather than an argument."
      },
      {
        "title": "Replay, once the bug is fixed",
        "bodyHtml": "Deliveries can be replayed from the logs. The receiver's outage or its parsing bug stops being a data-loss incident and becomes a replay."
      },
      {
        "title": "Something your customers can use alone",
        "bodyHtml": "A drop-in subscriber portal where your users register their own endpoints and read their own delivery history, instead of opening a support ticket for every 500."
      },
      {
        "title": "A data plane you can point to",
        "bodyHtml": "Payloads, database and backups run on Clever Cloud SAS infrastructure in France, inside the EEA, on every plan including the free tier. The CDN in front is Cloudflare, Inc. (USA), disclosed in the public <a href=\"./gdpr-subprocessors\" class=\"text-green-400 hover:text-green-300 transition-colors\">sub-processor list</a> with its transfer mechanism. See <a href=\"./eu-webhook-infrastructure\" class=\"text-green-400 hover:text-green-300 transition-colors\">EU webhook infrastructure</a>."
      },
      {
        "title": "An exit that is not a rewrite",
        "bodyHtml": "The whole product is open-source under SSPL-1.0, with no open-core holdback, so the cloud runs the code you can run. Leaving means hosting the same software rather than rebuilding against a new API."
      }
    ]
  },
  "cost": {
    "eyebrow": "Cost",
    "h2": "What a managed webhook service costs",
    "subtitle": "Where each provider's public pricing starts once you want a supported, managed service rather than a trial.",
    "headers": {
      "provider": "Provider",
      "offer": "How you run it",
      "price": "Where the pricing starts"
    },
    "rows": [
      {
        "highlight": true,
        "provider": "Hook0",
        "offerHtml": "Managed cloud, self-hosted or managed on-premise, all on one codebase.",
        "priceHtml": "€0 (free tier, 100 events/day); paid plans from €59/month"
      },
      {
        "highlight": false,
        "provider": "Hookdeck",
        "offerHtml": "Managed platform, US, EU and Asia regions.",
        "priceHtml": "Free tier available; the SLA-backed Growth plan is $499/month"
      },
      {
        "highlight": false,
        "provider": "Svix",
        "offerHtml": "Managed cloud, or self-hosting under MIT.",
        "priceHtml": "Managed Pro starts at $490/month"
      },
      {
        "highlight": false,
        "provider": "Convoy",
        "offerHtml": "Self-hosted Community, or the managed Premium plan.",
        "priceHtml": "Community is free (Elastic License v2); managed Premium is $999/month"
      }
    ],
    "footnote": "Sources: each provider's public pricing pages, last checked on 2026-07-16. Providers move their prices, so check theirs too. Spotted something out of date? Tell us and we will fix it.",
    "footLabel": "Read the full webhook cost comparison →",
    "footHref": "./webhook-cost-comparison"
  },
  "ownership": {
    "eyebrow": "Ownership",
    "h2": "Managed today, yours tomorrow",
    "intro": "A webhook service is a dependency in your delivery path. It is worth knowing, before you sign, what happens the day you want it out.",
    "cards": [
      {
        "title": "Self-host the same thing",
        "bodyHtml": "Docker Compose or Kubernetes, PostgreSQL underneath, open-source under SSPL-1.0. Your payloads stay inside your own network. See <a href=\"./self-hosted-webhooks\" class=\"text-green-400 hover:text-green-300 transition-colors\">self-hosted webhooks</a>."
      },
      {
        "title": "Or keep it managed, in your environment",
        "bodyHtml": "We deploy a dedicated instance in your infrastructure and keep it maintained: €1,000 setup + €500/month (excl. VAT), or €0 setup + €6,000/year (excl. VAT)."
      },
      {
        "title": "The API does not change",
        "bodyHtml": "Cloud, self-hosted and on-premise run the same <a href=\"./webhook-api\" class=\"text-green-400 hover:text-green-300 transition-colors\">webhook API</a>. Moving between them changes your deployment while your integration code stays as it is."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Webhook service questions",
    "items": [
      {
        "q": "What is a webhook service?",
        "a": "A webhook service takes an event from your backend and delivers it to the HTTP endpoints your customers registered. It owns the four parts that are tedious to own yourself, signing each payload so the receiver can trust it, retrying when the receiver is down, keeping a log of every attempt, and giving your customers somewhere to manage their own endpoints. You make one API call; the service handles the rest of the delivery."
      },
      {
        "q": "What is the difference between a webhook service and a webhook platform?",
        "a": "In practice the two words describe the same product, and vendors use them interchangeably. \"Service\" tends to describe the managed side (someone else runs the delivery for you), while \"platform\" tends to describe the surface you build on: API, portal, event types, logs. Hook0 is both, which is why the same features show up on this page and on the webhook platform page."
      },
      {
        "q": "How much does a webhook service cost?",
        "a": "Hook0 is free up to 100 events/day, with paid plans from €59/month. Among the other managed providers, as last checked in July 2026, Hookdeck's SLA-backed Growth plan is $499/month, Svix's managed Pro starts at $490/month, and Convoy's managed Premium is $999/month. Self-hosting shifts the cost from a subscription to your own infrastructure and maintenance time."
      },
      {
        "q": "Where does Hook0 host webhook data?",
        "a": "The webhook data plane, meaning payloads, database and backups, runs on Clever Cloud SAS infrastructure in France, inside the European Economic Area, on every plan including the free tier. The CDN and DDoS layer in front of the website and API is Cloudflare, Inc. (USA), disclosed in our public sub-processor list and framed by the 2021 Standard Contractual Clauses, a documented Transfer Impact Assessment and, where applicable, the EU-US Data Privacy Framework."
      },
      {
        "q": "Can I move off the managed service later?",
        "a": "Yes, and it is the reason the codebase is open-source under SSPL-1.0 with no open-core holdback. You can self-host the same software with Docker Compose or Kubernetes, or have us run a dedicated instance inside your own environment. The API stays the same in all three cases, so your integration code does not change."
      },
      {
        "q": "Do I still need a webhook service if I already send HTTP requests?",
        "a": "Sending the first request is the easy part. The sprints go into everything after it, from a retry schedule that does not hammer a struggling receiver to signatures your customers can verify, per-attempt logs for the support conversation, replay after an outage, and a portal so subscribers manage their own endpoints. If you have already built all of that and enjoy maintaining it, you do not need a webhook service."
      }
    ]
  },
  "related": {
    "h2": "Related",
    "links": [
      { "label": "Webhook Platform", "href": "./webhook-platform" },
      { "label": "Webhook API", "href": "./webhook-api" },
      { "label": "Build vs Buy Webhooks", "href": "./build-vs-buy-webhooks" },
      { "label": "Webhook Cost Comparison", "href": "./webhook-cost-comparison" },
      { "label": "EU Webhook Infrastructure", "href": "./eu-webhook-infrastructure" },
      { "label": "Self-Hosted Webhooks", "href": "./self-hosted-webhooks" },
      { "label": "Pricing", "href": "./pricing" }
    ]
  }
};
