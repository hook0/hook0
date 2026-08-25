// Per-page strings for webhooks-for-ai-agents (EN base).
// Search intent. The demand on this territory arrives in two shapes: short MCP
// vocabulary ("mcp webhook", "webhook mcp", "how to use webhooks with mcp
// tools"), and whole questions of the kind answer engines surface ("what is
// the best way to deliver webhooks reliably to llm agents when downstream
// endpoints are slow"). Hence MCP in the title, an extractable answer block
// near the top, and an FAQ phrased the way those questions arrive rather than
// the way a product page would phrase them.
// The page covers an agent operating the webhooks a product SENDS. It stays
// distinct from ./webhook-api (endpoint/SDK reference framing) and from
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
// the FAQPage JSON-LD is auto-generated from this same array. Keep answers
// plain text — the card renders them unescaped.
module.exports = {
  "pageTitle": "Webhook MCP Server for AI Agents | Hook0",
  "pageDescription": "Hook0's MCP server gives Claude, Cursor and Windsurf 17 tools to create subscriptions, emit events and replay failed deliveries on the webhooks you send.",
  "pageModified": "2026-08-25",
  "track": "webhooks-for-ai-agents",
  "hero": {
    "eyebrow": "Webhook MCP Server",
    "titleLine1": "Your Agent Can Read Webhooks.",
    "titleLine2": "It Can Also Send Them.",
    "subtitle": "Wiring an assistant to webhooks usually means the events someone else emits. Hook0 works the other way round: an MCP server that lets Claude, Cursor or Windsurf operate the webhooks your own product sends. Register event types, create subscriptions, ingest events, replay what failed.",
    "ctaPrimary": "Start Free",
    "ctaSecondary": "Read the MCP Docs",
    "ctaSecondaryHref": "https://documentation.hook0.com/reference/mcp",
    "microcopy": "100 events/day free. No credit card. The MCP server runs on your machine."
  },
  "answer": {
    "h2": "In short",
    "lead": "Hook0's MCP server is a local process that exposes your Hook0 account to an MCP-compatible assistant as 17 tools. Nine read your applications, event types, subscriptions, events and delivery attempts. Eight write: create an event type, wire a subscription, emit an event, replay a failed delivery.",
    "factsLabel": "The short version",
    "facts": [
      { "k": "Install", "v": "<code class=\"text-green-400\">cargo install hook0-mcp</code>" },
      { "k": "Tools", "v": "17 (9 read, 8 write)" },
      { "k": "Clients", "v": "Claude Desktop, Cursor, Windsurf, Cline, and any MCP client" },
      { "k": "Transport", "v": "stdio by default, SSE if you run it as a service" },
      { "k": "Runs against", "v": "Hook0 Cloud, or your own instance via <code class=\"text-green-400\">HOOK0_API_URL</code>" },
      { "k": "Restricting it", "v": "<code class=\"text-green-400\">HOOK0_READ_ONLY=true</code> plus an attenuated service token" },
      { "k": "License", "v": "Open-source (SSPL-1.0), no open-core holdback" }
    ]
  },
  "socialProof": true,
  "sides": {
    "eyebrow": "Two Directions",
    "h2": "Webhooks and agents point two different ways",
    "subtitle": "The two are often discussed as one subject. They are not the same problem and they are not solved by the same tool.",
    "cards": [
      {
        "title": "An agent that receives webhooks",
        "bodyHtml": "Something happens in Stripe, GitHub or a CRM, and an agent needs to react to it. The hard parts are inbound: a public endpoint or a tunnel, signature verification, deduplication, and not firing the same agent run twice. Tooling for this direction is well covered."
      },
      {
        "title": "An agent that operates what you send",
        "bodyHtml": "You are the one emitting events. Your customers subscribe to <code class=\"text-green-400\">order.completed</code> and expect it to arrive. The work is outbound: registering event types, wiring subscriptions, reading why an attempt failed, replaying it once the receiver is fixed. That is the side Hook0's MCP server exposes to your assistant."
      }
    ],
    "note": "If you ship a product other systems subscribe to, the second direction is the one that costs you support tickets."
  },
  "versus": {
    "eyebrow": "MCP vs Webhooks",
    "h2": "Two protocols, two layers",
    "intro": "They get compared because both carry events around. They sit at different layers, and a setup that works usually has both.",
    "rows": [
      {
        "label": "A webhook",
        "bodyHtml": "An HTTP request your product sends to a URL a customer registered, when something happens on your side. One-way, machine-to-machine, signed so the receiver can trust it, retried when the receiver is down. It is the data plane: it moves the event."
      },
      {
        "label": "MCP",
        "bodyHtml": "The Model Context Protocol, how an assistant discovers and calls tools. Request/response, driven by a person in a conversation, running locally. It is a control plane: it operates the thing that moves the event."
      },
      {
        "label": "Together",
        "bodyHtml": "Your product keeps emitting events through the REST API or an SDK, unchanged. MCP is what you reach for when you need to register a new event type, point a subscription somewhere else, or work out why yesterday's delivery failed, without opening the dashboard."
      }
    ]
  },
  "tools": {
    "eyebrow": "MCP Server",
    "h2": "What the assistant can actually do",
    "subtitle": "Seventeen tools, split between reading and writing. This is the shipped list, not a roadmap. Each one is documented, with its example prompt, in the MCP reference.",
    "headers": {
      "tool": "Tool",
      "does": "What it does",
      "say": "What you type"
    },
    "groups": [
      {
        "label": "Read (9 tools)",
        "rows": [
          { "tool": "list_organizations", "does": "List the organizations you can reach", "say": "&ldquo;Show my organizations&rdquo;" },
          { "tool": "list_applications", "does": "List the applications in an organization", "say": "&ldquo;What apps do I have?&rdquo;" },
          { "tool": "get_application", "does": "Read one application's details", "say": "&ldquo;Show details for app X&rdquo;" },
          { "tool": "list_event_types", "does": "List the event types registered on an application", "say": "&ldquo;What event types are registered?&rdquo;" },
          { "tool": "list_subscriptions", "does": "List webhook subscriptions and their configuration", "say": "&ldquo;Show all my webhooks&rdquo;" },
          { "tool": "get_subscription", "does": "Read one subscription's configuration", "say": "&ldquo;Show webhook configuration for&hellip;&rdquo;" },
          { "tool": "list_events", "does": "List events emitted by an application", "say": "&ldquo;Show recent events&rdquo;" },
          { "tool": "get_event", "does": "Read one event, payload included", "say": "&ldquo;Show event abc123&rdquo;" },
          { "tool": "list_request_attempts", "does": "List delivery attempts for an event", "say": "&ldquo;Show delivery history for event X&rdquo;" }
        ]
      },
      {
        "label": "Write (8 tools)",
        "highlight": true,
        "rows": [
          { "tool": "create_application", "does": "Create an application", "say": "&ldquo;Create an app called Order Service&rdquo;" },
          { "tool": "delete_application", "does": "Delete an application", "say": "&ldquo;Delete the test application&rdquo;" },
          { "tool": "create_event_type", "does": "Register a new event type", "say": "&ldquo;Add event type order.completed&rdquo;" },
          { "tool": "create_subscription", "does": "Create a webhook subscription against a URL", "say": "&ldquo;Create a webhook to https://&hellip;&rdquo;" },
          { "tool": "update_subscription", "does": "Change or disable an existing subscription", "say": "&ldquo;Disable the webhook for&hellip;&rdquo;" },
          { "tool": "delete_subscription", "does": "Delete a subscription", "say": "&ldquo;Remove the staging webhook&rdquo;" },
          { "tool": "ingest_event", "does": "Emit an event, the sending side, from the assistant", "say": "&ldquo;Send a test user.created event&rdquo;" },
          { "tool": "retry_delivery", "does": "Replay a delivery that failed", "say": "&ldquo;Retry the failed delivery for event X&rdquo;" }
        ]
      }
    ],
    "footnote": "Plus eight resource URIs under hook0:// for direct lookups, and three guided prompts for the flows people repeat: create a subscription, debug a delivery, set up an application.",
    "footHref": "https://documentation.hook0.com/reference/mcp",
    "footLabel": "Full tool reference, with the setup for Claude Desktop, Cursor, Windsurf and Cline"
  },
  "setup": {
    "eyebrow": "Setup",
    "h2": "Three steps to your first prompt",
    "intro": "The server is a Rust binary you install once. Nothing runs on Hook0's side that was not already running.",
    "steps": [
      {
        "n": "1",
        "title": "Install the server",
        "bodyHtml": "It ships on crates.io and builds to a single binary.",
        "code": "cargo install hook0-mcp"
      },
      {
        "n": "2",
        "title": "Create a service token",
        "bodyHtml": "In the Hook0 dashboard, under your organization's service tokens. Attenuate it to the applications the assistant should reach before you paste it anywhere.",
        "code": ""
      },
      {
        "n": "3",
        "title": "Declare it in your assistant",
        "bodyHtml": "Claude Desktop reads <code class=\"text-green-400\">claude_desktop_config.json</code>. Cursor, Windsurf and Cline take the same block in their own config file.",
        "code": "{\n  \"mcpServers\": {\n    \"hook0\": {\n      \"command\": \"hook0-mcp\",\n      \"env\": {\n        \"HOOK0_API_TOKEN\": \"your-service-token-here\"\n      }\n    }\n  }\n}"
      }
    ],
    "outro": "Restart the assistant and ask it something you would otherwise click through: &ldquo;why did my last webhook delivery fail?&rdquo;",
    "docsHref": "https://documentation.hook0.com/reference/mcp",
    "docsLabel": "Config file paths per assistant, environment variables and SSE mode"
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
        "bodyHtml": "Read-only mode narrows the tool list; the token itself still carries whatever it was granted. Attenuation restricts a token to specific applications and can carry an expiry, enforced at the API rather than in the client. Use both. They cover different failure modes."
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
    "intro": "Natural language changes how you operate webhook delivery. It does not deliver anything by itself. This is what does.",
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
        "q": "How do I use webhooks with MCP tools?",
        "a": "Install hook0-mcp, create a Hook0 service token, and declare the server in your assistant's configuration file. From then on the assistant has seventeen tools against your account: it can list what exists, register an event type, create a subscription against a URL, emit a test event, and replay a delivery that failed. Your application keeps emitting its real events through the REST API or an SDK, and that path is unchanged."
      },
      {
        "q": "What is the difference between MCP and a webhook?",
        "a": "A webhook is an HTTP request your product sends to a URL a customer registered, one-way and machine-to-machine, signed so the receiver can trust it. MCP is how an assistant discovers and calls tools, request/response, driven by a person in a conversation. The webhook moves the event; MCP operates the system that moves it. Most setups that use both keep the webhook path untouched and use MCP for the operating work."
      },
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
        "q": "What happens to events when a subscriber endpoint is down for a while?",
        "a": "Hook0 retries the delivery on a two-phase, configurable schedule rather than dropping it on the first failure, and it records every attempt with its response. Once the receiver is fixed you replay what failed, from the dashboard, from the API, or by asking the assistant to retry that delivery. The event itself is not lost while the endpoint is unreachable."
      },
      {
        "q": "How does the receiving side verify a payload Hook0 sent?",
        "a": "Every attempt carries an HMAC-SHA256 signature computed from the payload and the subscription's secret. The receiver recomputes it and compares before acting on the event, which is what stops a forged request from triggering a workflow. The signature scheme and a verification snippet are in the Hook0 documentation."
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
