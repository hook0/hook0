// Per-page strings for webhook-api (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  "pageTitle": "Webhook API: One REST Call to Deliver Events | Hook0",
  "pageDescription": "Hook0 exposes a clean REST webhook API: one call to send an event, automatic HMAC signing, configurable retries and SDKs for Python, Node.js and more. Free forever.",
  "track": "webhook-api",
  "hero": {
    "eyebrow": "Webhook API",
    "titleLine1": "The Webhook API That",
    "titleLine2": "Just Works in 30 Minutes",
    "subtitle": "One POST from your backend. Hook0 does the HMAC, the retries, the DLQ and the delivery logs. SDKs in Python and Node.js. Open-source, free tier, no credit card.",
    "ctaPrimary": "Start Free",
    "ctaSecondary": "Read the API Reference",
    "microcopy": "100 events/day free. No credit card. Open-source."
  },
  "socialProof": true,
  "codeExample": {
    "eyebrow": "Code",
    "h2": "Send your first event in 30 seconds",
    "subtitle": "One endpoint, one payload. No SDK required, no webhook concepts to learn first.",
    "restCode": "POST https://api.hook0.com/api/v1/event\nAuthorization: Bearer &lt;APPLICATION_AUTH_TOKEN&gt;\nContent-Type: application/json\n\n{\n  \"application_id\": \"c0ea6ffa-1972-4435-b434-ec9e93d38f42\",\n  \"event_type\":     \"invoice.paid\",\n  \"event_id\":       \"evt_Wqb1k73rXprtTm7Qdlr38G\",\n  \"payload\": {\n    \"invoice_id\": \"in_8X9aBcDeFgHiJk\",\n    \"status\":     \"paid\",\n    \"amount_eur\": 4990\n  },\n  \"labels\": { \"tenant\": \"acme\", \"env\": \"prod\" }\n}\n",
    "pythonCode": "hook0 = Hook0(\"AUTH_TOKEN\")\nhook0.message.create(\n  \"app_id\",\n  MessageIn(\n    event_type=\"invoice.paid\",\n    event_id=\"evt_123\",\n    payload={\"status\": \"paid\"}\n  )\n)",
    "nodeCode": "const hook0 = Hook0(\"AUTH_TOKEN\");\nawait hook0.message.create(\"app_id\", {\n  event_type: \"invoice.paid\",\n  event_id:   \"evt_123\",\n  payload:    { status: \"paid\" }\n});",
    "docsLabel": "Browse the full API reference →",
    "docsHref": "https://documentation.hook0.com/reference/",
    "docsTrack": "webhook-api-docs"
  },
  "capabilities": {
    "eyebrow": "Inside the API",
    "h2": "What the webhook API does for you",
    "cards": [
      {
        "title": "HMAC-SHA256 signing",
        "body": "Payloads carry a signature and a timestamp. Subscribers verify both. Replay attacks fail the timestamp check."
      },
      {
        "title": "Two-phase retries",
        "body": "Fast retries within the first few minutes for flaky endpoints. Slow retries over hours and days for real outages. DLQ when the budget runs out."
      },
      {
        "title": "Idempotent event IDs",
        "body": "Pass your own <code class=\"text-green-400\">event_id</code>. Hook0 deduplicates on it, so the API call is safe to retry without firing twice downstream."
      },
      {
        "title": "Delivery logs and replay",
        "body": "Headers, body, response code, latency. Stored per attempt. Replay any event by ID, from the dashboard or the API."
      },
      {
        "title": "Open-source SDKs",
        "body": "Python and Node.js. Generated from the OpenAPI spec, so the client and the API stay in sync."
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
        "q": "What is the Hook0 webhook API?",
        "a": "The Hook0 webhook API is a REST interface that lets your backend trigger an event with a single HTTP call. Hook0 then signs the payload with HMAC, delivers it to every matching subscriber, retries on failure with a configurable two-phase backoff, and logs every attempt. SDKs are available for Python, Node.js and other languages."
      },
      {
        "q": "How do I authenticate with the webhook API?",
        "a": "Authentication uses a Bearer token (application authentication token) passed in the <code class=\"text-green-400\">Authorization</code> header. Tokens are scoped to an application and can be rotated from the dashboard at any time."
      },
      {
        "q": "Does the webhook API include retries and HMAC signatures?",
        "a": "Yes. Every event triggered via the webhook API is automatically signed with HMAC (so subscribers can verify it) and retried using a two-phase backoff strategy on delivery failure. Dead letter queues capture events that exhaust their retry budget."
      },
      {
        "q": "What SDKs are available for the Hook0 webhook API?",
        "a": "Official SDKs include Python and Node.js, with community libraries available for more languages. The REST API is fully documented in the API reference, so any HTTP client works."
      },
      {
        "q": "Is the webhook API rate-limited?",
        "a": "Yes. Rate limits scale with the plan: the free tier allows 100 events per day, paid tiers raise both the daily volume and the burst rate. Self-hosted deployments are not rate-limited by Hook0."
      }
    ]
  },
  "related": {
    "h2": "Related",
    "links": [
      { "label": "Webhook Platform", "href": "./webhook-platform" },
      { "label": "Hook0 vs Svix", "href": "./hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "./hook0-vs-hookdeck" },
      { "label": "Build vs Buy Webhooks", "href": "./build-vs-buy-webhooks" },
      { "label": "Self-Hosted Webhooks", "href": "./self-hosted-webhooks" },
      { "label": "Open-Source Webhooks", "href": "./open-source-webhooks" }
    ]
  }
};
