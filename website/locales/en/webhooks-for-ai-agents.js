// Per-page strings for webhooks-for-ai-agents (EN base).
// Search intent: someone wiring an AI assistant to webhook infrastructure, who
// finds only one half of the subject written down. Everything published on this
// territory so far covers an agent CONSUMING webhooks; this page covers the
// other direction — an agent operating the webhooks a product SENDS. The page
// stays distinct from ./webhook-api (endpoint/SDK reference framing) and from
// ./webhook-platform (product-and-capabilities framing).
// Every capability claimed below maps to a tool documented in
// documentation/reference/mcp.md (9 read tools, 8 write tools, 17 total),
// or to a platform capability already claimed on locales/en/webhook-platform.js
// (HMAC signing, two-phase retries, per-attempt delivery logs, replay,
// subscriber portal, EU data plane, SSPL no open-core). Nothing beyond that.
// Deliberately NOT claimed, and not to be added later without proof:
//   - that no competitor does this (the competitor watch records what rivals
//     have PUBLISHED, never what they lack) — describe Hook0, not their gaps;
//   - any Agent Skill, plugin or assistant-directory listing: none is shipped.
// Legal constraints (non-negotiable, same set as ./webhook-service):
//   - Data plane = Clever Cloud SAS (France, EEA). Cloudflare, Inc. (USA) CDN is
//     DISCLOSED, framed by SCC 2021 + TIA and, where applicable, the EU-US DPF.
//   - NEVER "100% sovereign", "no data leaves the EU", "CLOUD Act free".
//   - GDPR = process/context claim ("designed for"), never "certified".
//   - License = "open-source (SSPL-1.0)" whenever mentioned.
//   - No SOC 2, no ISO claim: Hook0 holds neither.
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  "pageTitle": "Webhooks for AI Agents: The Sending Side | Hook0",
  "pageDescription": "Most agent tooling covers webhooks an agent receives. Hook0's MCP server covers the other direction: your assistant creates subscriptions, sends events and retries failed deliveries against the webhooks your product emits.",
  "pageModified": "2026-08-17",
  "track": "webhooks-for-ai-agents",
  "hero": {
    "eyebrow": "Webhooks for AI Agents",
    "titleLine1": "Your Agent Can Read Webhooks.",
    "titleLine2": "It Can Also Send Them.",
    "subtitle": "Almost everything written about agents and webhooks describes an agent consuming events someone else emits. Hook0 works the other way round: an MCP server that lets Claude, Cursor or Windsurf operate the webhooks your own product sends — register event types, create subscriptions, ingest events, replay what failed.",
    "ctaPrimary": "Start Free",
    "ctaSecondary": "Read the MCP Docs",
    "ctaSecondaryHref": "https://documentation.hook0.com/reference/mcp",
    "microcopy": "100 events/day free. No credit card. The MCP server runs on your machine."
  },
  "socialProof": true,
  "sides": {
    "eyebrow": "Two Directions",
    "h2": "Webhooks and agents point two different ways",
    "subtitle": "The two are often discussed as one subject. They are not the same problem, they are not solved by the same tool, and only one of them is usually written down.",
    "cards": [
      {
        "title": "An agent that receives webhooks",
        "bodyHtml": "Something happens in Stripe, GitHub or a CRM, and an agent needs to react to it. The hard parts are inbound: a public endpoint or a tunnel, signature verification, deduplication, and not firing the same agent run twice. This is the direction most agent tooling addresses, and it is well covered."
      },
      {
        "title": "An agent that operates what you send",
        "bodyHtml": "You are the one emitting events. Your customers subscribe to <code class=\"text-green-400\">order.completed</code> and expect it to arrive. The work is outbound: registering event types, wiring subscriptions, reading why an attempt failed, replaying it once the receiver is fixed. That is the side Hook0's MCP server exposes to your assistant."
      }
    ],
    "note": "If you ship a product other systems subscribe to, the second direction is the one that costs you support tickets."
  },
  "tools": {
    "eyebrow": "MCP Server",
    "h2": "What the assistant can actually do",
    "subtitle": "Seventeen tools, split between reading and writing. This is the shipped list, not a roadmap — each one is documented, with its example prompt, in the MCP reference.",
    "headers": {
      "tool": "Tool",
      "does": "What it does",
      "say": "What you type"
    },
    "groups": [
      {
        "label": "Read",
        "rows": [
          { "tool": "list_applications", "does": "List the applications in an organization", "say": "&ldquo;What apps do I have?&rdquo;" },
          { "tool": "list_event_types", "does": "List the event types registered on an application", "say": "&ldquo;What event types are registered?&rdquo;" },
          { "tool": "list_subscriptions", "does": "List webhook subscriptions and their configuration", "say": "&ldquo;Show all my webhooks&rdquo;" },
          { "tool": "list_events", "does": "List events emitted by an application", "say": "&ldquo;Show recent events&rdquo;" },
          { "tool": "list_request_attempts", "does": "List delivery attempts for an event", "say": "&ldquo;Show delivery history for event X&rdquo;" }
        ]
      },
      {
        "label": "Write",
        "highlight": true,
        "rows": [
          { "tool": "create_event_type", "does": "Register a new event type", "say": "&ldquo;Add event type order.completed&rdquo;" },
          { "tool": "create_subscription", "does": "Create a webhook subscription against a URL", "say": "&ldquo;Create a webhook to https://&hellip;&rdquo;" },
          { "tool": "update_subscription", "does": "Change or disable an existing subscription", "say": "&ldquo;Disable the webhook for&hellip;&rdquo;" },
          { "tool": "ingest_event", "does": "Emit an event — the sending side, from the assistant", "say": "&ldquo;Send a test user.created event&rdquo;" },
          { "tool": "retry_delivery", "does": "Replay a delivery that failed", "say": "&ldquo;Retry the failed delivery for event X&rdquo;" }
        ]
      }
    ],
    "footnote": "Plus application and organization management, resource URIs for direct lookups, and three guided prompts for the flows people repeat: create a subscription, debug a delivery, set up an application.",
    "footHref": "https://documentation.hook0.com/reference/mcp",
    "footLabel": "Full tool reference, with the setup for Claude Desktop, Cursor, Windsurf and Cline"
  },
  "guardrails": {
    "eyebrow": "Guardrails",
    "h2": "Handing an agent your delivery infrastructure",
    "intro": "Write access to production webhooks is not something to grant casually. Three controls ship with the server, and they compose.",
    "cards": [
      {
        "title": "Read-only mode",
        "bodyHtml": "Set <code class=\"text-green-400\">HOOK0_READ_ONLY=true</code> and the server advertises only the nine read tools. The assistant can investigate a failed delivery and cannot change anything while doing it."
      },
      {
        "title": "Attenuated tokens",
        "bodyHtml": "A service token carries full organization access by default, so narrow it. Token attenuation restricts a token to specific applications and can carry an expiry, enforced at the API rather than in the client. Read-only mode and an attenuated token are complementary, not redundant."
      },
      {
        "title": "It runs where you run it",
        "bodyHtml": "The MCP server is a local process talking to the Hook0 API. The assistant sees what the tools it calls return, and nothing else is forwarded anywhere. Point <code class=\"text-green-400\">HOOK0_API_URL</code> at your own instance and the same tools drive a self-hosted deployment."
      }
    ]
  },
  "platform": {
    "eyebrow": "Underneath",
    "h2": "The agent is an interface, not the guarantee",
    "intro": "Natural language changes how you operate webhook delivery. It does not deliver anything by itself — this is what does.",
    "cards": [
      {
        "title": "Signed, retried, logged",
        "bodyHtml": "Every attempt carries an HMAC-SHA256 signature. Retries are two-phase and configurable, so a subscriber that is down for an hour does not cost you that hour of events. Every attempt is logged, which is what makes &ldquo;why did this fail&rdquo; answerable by a lookup instead of a guess."
      },
      {
        "title": "An EU data plane, on every plan",
        "bodyHtml": "Payloads, database and backups run on Clever Cloud SAS infrastructure in France, inside the EEA, including on the free tier. The CDN in front is Cloudflare, Inc. (USA), disclosed in the public <a href=\"./gdpr-subprocessors\" class=\"text-green-400 hover:text-green-300 transition-colors\">sub-processor list</a> with its transfer mechanism. Details on <a href=\"./eu-webhook-infrastructure\" class=\"text-green-400 hover:text-green-300 transition-colors\">EU webhook infrastructure</a>."
      },
      {
        "title": "A codebase you can take",
        "bodyHtml": "Hook0 is open-source (SSPL-1.0), with no open-core holdback: the hosted service runs the code you can run. The MCP server, the API and the subscriber portal behave the same way against a self-hosted instance."
      }
    ]
  },
  "faq": {
    "eyebrow": "Questions",
    "h2": "Before you point an assistant at production",
    "items": [
      {
        "q": "Does the assistant see my event payloads?",
        "a": "It sees what the tools it calls return, and reading an event returns that event's payload. Nothing is forwarded to a third party by Hook0: the MCP server is a local process that talks to the Hook0 API directly. If payloads should stay out of the conversation entirely, attenuate the token to the applications that carry no sensitive data."
      },
      {
        "q": "Which assistants work with it?",
        "a": "Claude Desktop, Cursor, Windsurf and Cline are documented with their configuration file, and any MCP-compatible client works the same way. ChatGPT does not support MCP natively today."
      },
      {
        "q": "What stops an agent from deleting a production subscription?",
        "a": "Two things, and they are worth combining. Read-only mode removes the write tools from the list the assistant can see. Token attenuation restricts what the token itself may touch, enforced at the API, so a mistake in the client cannot exceed it. Deleted resources are not automatically restorable, which is the reason to set both before pointing an assistant at production."
      },
      {
        "q": "Does this work with self-hosted Hook0?",
        "a": "Yes. Set HOOK0_API_URL to your instance and the seventeen tools behave identically. The whole product is open-source (SSPL-1.0) with no open-core holdback, so the self-hosted deployment is the same software as the cloud one."
      },
      {
        "q": "Is this an Agent Skill or a plugin?",
        "a": "No. It is an MCP server, installed with cargo install hook0-mcp and declared in your assistant's configuration. It runs over stdio by default, or over SSE if you would rather run it as a service."
      },
      {
        "q": "Do I need the MCP server to send webhooks?",
        "a": "No. The REST API and the SDKs are the normal path for your application to emit events, and they are unaffected by any of this. The MCP server is for the human operating the setup: registering an event type, wiring a subscription, working out why one delivery failed at four in the afternoon."
      }
    ]
  },
  "related": {
    "h2": "Related",
    "links": [
      { "href": "./webhook-api", "label": "Webhook API" },
      { "href": "./webhook-platform", "label": "Webhook platform" },
      { "href": "./webhook-service", "label": "Managed webhook service" },
      { "href": "./self-hosted-webhooks", "label": "Self-hosted webhooks" },
      { "href": "https://documentation.hook0.com/reference/mcp", "label": "MCP server documentation" }
    ]
  }
};
