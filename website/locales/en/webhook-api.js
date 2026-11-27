// Per-page strings for webhook-api (EN base).
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
// Every technical fact on this page is checked against the code it describes:
// the ingest contract (api/src/handlers/events.rs, EventPost), the duplicate
// event_id answer (problems.rs, EventAlreadyIngested = 409), the retry
// schedule (documentation/explanation/webhook-retry-logic.md) and the SDK
// quick starts (clients/python, clients/typescript). If one of those changes,
// change the matching sentence here and in the fr/de locales.
module.exports = {
  tldr: {
    h2: 'What is a webhook API?',
    body: 'A webhook API pushes an HTTP request to a URL you registered as soon as an event happens, instead of waiting for you to ask. A REST API answers when the client calls it; a webhook API calls the client. Hook0 runs the sending side for your SaaS. Your backend POSTs each event once to /api/v1/event, and Hook0 signs it with HMAC-SHA256, delivers it to every matching subscription, retries failed deliveries (first retry after about 3 seconds, later ones up to 10 hours apart) and keeps every attempt with its response so you can replay it. If you only need to receive webhooks from Stripe or GitHub, an inbound webhook gateway is the better fit.',
  },
  pageTitle: 'Webhook API: Send Signed Webhooks With One REST Call | Hook0',
  pageDescription: 'What a webhook API is, how it differs from a REST API, and how Hook0 sends signed webhooks with retries, delivery logs and replay. Free tier, no card.',
  pageModified: '2026-11-26',
  "track": "webhook-api",
  "hero": {
    "eyebrow": "Webhook API",
    "titleLine1": "The Webhook API",
    "titleLine2": "for Sending Events to Your Users",
    "subtitle": "A webhook API lets your backend notify your customers' systems the moment something happens. With Hook0 that is one POST: Hook0 signs the event, delivers it, retries failures and logs every attempt. Official SDKs in 11 languages, open-source server, free tier.",
    "ctaPrimary": "Start Free",
    "ctaSecondary": "Read the API Reference",
    "microcopy": "100 events/day free. No credit card. Open-source.",
    "updated": "Updated November 26, 2026"
  },
  "socialProof": true,
  "vsApi": {
    "h2": "Webhook vs REST API: who makes the call",
    "intro": "Both run over HTTP and usually carry JSON. What changes is who starts the request, and who has to handle a receiver that is down.",
    "headers": { "aspect": "Aspect", "rest": "REST API (pull)", "webhook": "Webhook (push)" },
    "rows": [
      { "aspect": "Who starts the request", "rest": "The client calls the server", "webhook": "The sender calls a URL the receiver registered" },
      { "aspect": "When data arrives", "rest": "Whenever the client polls", "webhook": "Seconds after the event" },
      { "aspect": "Wasted requests", "rest": "Most polls return nothing new", "webhook": "One request per event and subscription" },
      { "aspect": "Receiver is down", "rest": "The client retries its own call", "webhook": "The sender has to queue and retry" },
      { "aspect": "Authentication", "rest": "The client sends a token", "webhook": "The receiver verifies an HMAC signature" },
      { "aspect": "Typical use", "rest": "Read or change data on demand", "webhook": "Tell another system that something happened" }
    ],
    "outro": "Most SaaS products ship a REST API to read and write data and webhooks to announce changes. Hook0 handles the webhook half and exposes it as a REST API of its own. The <a href=\"https://documentation.hook0.com/explanation/webhook-vs-polling\" class=\"text-indigo-400 hover:text-indigo-300 underline\">webhook vs polling guide</a> covers the trade-offs in depth."
  },
  "codeExample": {
    "eyebrow": "Code",
    "h2": "Send your first event in 30 seconds",
    "subtitle": "One endpoint. The payload travels as a string with its content type, so subscribers receive the exact bytes you sent.",
    "restCode": "POST https://app.hook0.com/api/v1/event\nAuthorization: Bearer &lt;APPLICATION_TOKEN&gt;\nContent-Type: application/json\n\n{\n  \"application_id\":       \"c0ea6ffa-1972-4435-b434-ec9e93d38f42\",\n  \"event_id\":             \"0192f5b4-7c1e-7d3a-9f2b-4e8a1c6d2b90\",\n  \"event_type\":           \"billing.invoice.paid\",\n  \"payload\":              \"{\\\"invoice_id\\\":\\\"in_8X9aBcDeFgHiJk\\\",\\\"amount_eur\\\":4990}\",\n  \"payload_content_type\": \"application/json\",\n  \"occurred_at\":          \"2026-11-26T09:30:00Z\",\n  \"labels\":               { \"tenant\": \"acme\", \"env\": \"prod\" }\n}\n",
    "pythonCode": "from hook0 import Event, Hook0Client\n\nclient = Hook0Client(\"https://app.hook0.com/api/v1\", APP_ID, TOKEN)\nclient.send_event(Event(\n    event_type=\"billing.invoice.paid\",\n    payload='{\"invoice_id\": \"in_123\"}',\n    payload_content_type=\"application/json\",\n    labels={\"env\": \"prod\"},\n))",
    "nodeCode": "import { Event, Hook0Client } from 'hook0-client';\n\nconst hook0 = new Hook0Client('https://app.hook0.com/api/v1', APP_ID, TOKEN);\nawait hook0.sendEvent(new Event(\n  'billing.invoice.paid',\n  JSON.stringify({ invoice_id: 'in_123' }),\n  'application/json',\n  { env: 'prod' }\n));",
    "docsLabel": "Follow the 5-minute quickstart →",
    "docsHref": "https://documentation.hook0.com/tutorials/getting-started",
    "docsTrack": "webhook-api-docs"
  },
  "capabilities": {
    "eyebrow": "Inside the API",
    "h2": "What the webhook API does for you",
    "cards": [
      {
        "title": "HMAC-SHA256 signing",
        "body": "Each delivery carries an <code class=\"text-green-400\">X-Hook0-Signature</code> header with a timestamp and an HMAC-SHA256 of the payload. Subscribers check both, so a replayed old request fails the timestamp check."
      },
      {
        "title": "Retries on a fixed schedule",
        "body": "A failed delivery is retried after about 3 seconds, then 10 seconds, 3 minutes, 30 minutes, and on up to 10 hours between attempts, 24 retries by default. When they run out, the last attempt stays failed with its response, ready to replay."
      },
      {
        "title": "Idempotent event IDs",
        "body": "Pass your own <code class=\"text-green-400\">event_id</code> (a UUID). Sending the same ID twice returns 409 Conflict instead of creating a second event, so a retried API call never fires twice downstream."
      },
      {
        "title": "Delivery logs and replay",
        "body": "Headers, body, response code, latency. Stored per attempt. Replay any event by ID, from the dashboard or the API."
      },
      {
        "title": "Open-source SDKs",
        "body": "JavaScript/TypeScript, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, Lua and Zig. Generated from the OpenAPI spec, so the client and the API stay in sync."
      },
      {
        "title": "Free tier, no gates",
        "body": "100 events per day, no credit card. Paid plans scale volume. Every feature on this page is in the free tier."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Webhook API questions",
    "items": [
      {
        "q": "What is a webhook API?",
        "a": "A webhook API sends an HTTP POST to a URL you registered when an event happens, so the receiver learns about it without polling. The Hook0 webhook API is the sending side of that exchange. Your backend triggers an event with one HTTP call, and Hook0 signs the payload with HMAC, delivers it to every matching subscriber, retries failures and logs every attempt. Official SDKs cover 11 languages."
      },
      {
        "q": "What is the difference between a webhook and an API?",
        "a": "With a REST API, the client asks and the server answers. With a webhook, the server calls the client when something changes. Webhooks remove polling and arrive seconds after the event, but the sender then has to sign requests, queue them and retry when the receiver is down. Hook0 takes that sender-side work off your hands."
      },
      {
        "q": "Which event platform has both an API and webhooks?",
        "a": "Hook0 is an event platform with a REST API in front and webhooks out the back. You send events through the API (or one of the 11 SDKs), declare event types and subscriptions through the same API, and Hook0 delivers each event as a signed webhook to every subscribed endpoint, with retries, delivery logs and replay."
      },
      {
        "q": "How do I authenticate with the webhook API?",
        "a": "Authentication uses a Bearer token (application authentication token) passed in the <code class=\"text-green-400\">Authorization</code> header. Tokens are scoped to an application and can be rotated from the dashboard at any time."
      },
      {
        "q": "Does the webhook API include retries and HMAC signatures?",
        "a": "Yes. Every event sent through the webhook API is signed with HMAC-SHA256 so subscribers can verify it. A failed delivery is retried on an increasing schedule, from about 3 seconds to 10 hours between attempts. When an event exhausts its retries, the last attempt stays failed with the endpoint's response, and one API call replays it."
      },
      {
        "q": "What SDKs are available for the Hook0 webhook API?",
        "a": "Hook0 maintains 11 official SDKs: JavaScript/TypeScript, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, Lua and Zig. Each installs from its language's package registry (Zig from a tagged archive), sends events and verifies webhook signatures. The REST API is fully documented in the API reference, so any HTTP client works too."
      },
      {
        "q": "Is the webhook API rate-limited?",
        "a": "Yes. Rate limits scale with the plan. The free tier allows 100 events per day, paid tiers raise both the daily volume and the burst rate. Self-hosted deployments are not rate-limited by Hook0."
      }
    ]
  },
  "related": {
    "h2": "Related",
    "links": [
      { "label": "API Reference", "href": "https://documentation.hook0.com/openapi/intro" },
      { "label": "Retry Schedule", "href": "https://documentation.hook0.com/explanation/webhook-retry-logic" },
      { "label": "Webhook vs Polling", "href": "https://documentation.hook0.com/explanation/webhook-vs-polling" },
      { "label": "How Webhooks Work", "href": "./how-webhooks-work" },
      { "label": "Webhook Platform", "href": "./webhook-platform" },
      { "label": "Webhook MCP Server", "href": "./webhooks-for-ai-agents" },
      { "label": "Hook0 vs Svix", "href": "./hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "./hook0-vs-hookdeck" },
      { "label": "Build vs Buy Webhooks", "href": "./build-vs-buy-webhooks" },
      { "label": "Self-Hosted Webhooks", "href": "./self-hosted-webhooks" },
      { "label": "Open-Source Webhooks", "href": "./open-source-webhooks" }
    ]
  }
};
