// Per-page strings for sovereign-webhooks (EN base).
// Angle: digital sovereignty as a positioning lens, NOT a compliance claim.
// Non-negotiable guardrails (see website/CLAUDE.local.md + the sovereignty
// reference sheet):
//   - NEVER claim a certification Hook0 does not hold: no SecNumCloud, HDS,
//     SEAL level, ISO 27001 or SOC 2 of its own. No badges.
//   - NEVER a negative certification mention ("not SOC 2 / ISO certified").
//     The page states positive, verifiable facts and stays silent on certs
//     Hook0 does not hold.
//   - NEVER "100% open source" / "only true open source" / "source-available"
//     as a slur. License is "open-source (SSPL-1.0)".
//   - NEVER claim at-rest encryption is on by default (it is not).
//   - The CLOUD Act section is EDUCATIONAL + a generic vendor checklist. It does
//     NOT assert that Hook0 is outside CLOUD Act scope (that legal conclusion is
//     /legal-gated). It only restates facts already published on-site: the
//     publisher is a French-law company with no US parent, the data plane runs
//     in France, and the Cloudflare (USA) edge is disclosed under 2021 SCC + TIA
//     + EU-US DPF. No "no byte leaves the EU".
//   - No competitor is named on this page; related links point to the existing
//     comparison pages (which carry their own non-affiliation + legal review).
// The faq.items[].a text MUST match the visible card body byte-for-byte; the
// FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  "pageTitle": "Sovereign Webhooks: Digital Sovereignty by Design | Hook0",
  "pageDescription": "Digital sovereignty is about who can be legally compelled to produce your data, not just where it sits. Hook0: French-law editor, EU data plane, open-source, reversible.",
  "pageModified": "2026-10-06",
  "track": "sovereign-webhooks",
  "tldr": {
    "h2": "The Short Answer",
    "body": "Digital sovereignty is not about where a server sits — it is about who can be legally compelled to produce your data. Hook0 answers that with concrete, verifiable properties rather than a badge. The company behind it is incorporated under French law, with no US parent. Its webhook data plane runs in France (Clever Cloud) on every plan, including the free tier. The whole platform is open-source — server under SSPL-1.0, SDKs under MIT — and the self-hosted code is the same code that runs the managed cloud, so you can move your webhook infrastructure in-house with no vendor left to depend on. The CDN and DDoS edge is Cloudflare, Inc. (USA), disclosed in a public sub-processor list. Hook0 is designed for GDPR, NIS2 and DORA requirements rather than certified against them. Free for 100 events a day, no credit card."
  },
  "hero": {
    "eyebrow": "Digital Sovereignty",
    "titleLine1": "Sovereign Webhooks,",
    "titleLine2": "Not Just EU-Hosted",
    "subtitle": "Localising data is the easy part. Sovereignty is about who can be legally compelled to hand it over. Hook0 is run by a French-law company with no US parent, hosts its data plane in France, and is open-source end to end — so leaving means running the same code yourself.",
    "ctaPrimary": "Start Free",
    "ctaSecondary": "Where your data lives",
    "ctaSecondaryHref": "./eu-webhook-infrastructure",
    "microcopy": "100 events/day free. No credit card. EU data plane on every plan."
  },
  "socialProof": true,
  "dimensions": {
    "eyebrow": "Localise ≠ Sovereign",
    "h2": "Sovereignty is three questions, not one",
    "intro": "A datacenter in Paris is necessary, not sufficient. The EU's own sovereignty frameworks break the word into three distinct questions — and data location only answers the first.",
    "cards": [
      {
        "title": "Legal",
        "bodyHtml": "Which law applies to the provider, and which law can compel it? A datacenter inside the EU, operated by a company subject to non-EU law, is still exposed to extraterritorial access requests. The jurisdiction of the company matters as much as the location of the disk."
      },
      {
        "title": "Operational",
        "bodyHtml": "Who runs, supports, updates and can switch off the service? Where are the on-call teams, and who holds the encryption keys? Sovereignty erodes when day-to-day control sits outside the jurisdiction you chose."
      },
      {
        "title": "Technological",
        "bodyHtml": "What is the stack made of — software, database, identity, DNS — and is it open, auditable and replaceable? A closed, cloud-only service leaves you with an export and a rebuild as your only exit. Open source keeps the stack substitutable."
      }
    ]
  },
  "compulsion": {
    "eyebrow": "The Real Question",
    "h2": "Who can be legally compelled to produce your data?",
    "intro": "The US CLOUD Act (2018) lets US authorities require a provider subject to US law to produce data it controls, wherever that data is stored — including in an EU region. So the question to ask any webhook vendor is not only where the servers are, but who could be ordered to hand the data over.",
    "checklistTitle": "Seven questions to ask any webhook vendor",
    "checklist": [
      "Who owns and controls the company, and under which law?",
      "Where do the data <strong>and</strong> the operations actually run?",
      "Who holds the encryption keys?",
      "Which hidden dependencies ride along — identity, DNS, CDN, email, container registries, AI?",
      "How reversible is it: can you export and run it elsewhere, and has that been tested?",
      "What level is claimed, and with which audit evidence you can actually read?",
      "What is the plan if the provider disappears or is cut off?"
    ],
    "hook0Title": "How Hook0 answers, in verifiable facts",
    "hook0Html": "The company behind Hook0 is incorporated under French law, with no US parent. Its webhook data plane — payloads, database and backups — runs in France (Clever Cloud), on every plan. The platform is open-source (SSPL-1.0), so the keys, the stack and the exit path are yours to inspect and to take. The one disclosed US dependency is the Cloudflare CDN and DDoS edge, listed in a public <a href=\"./gdpr-subprocessors\" class=\"text-green-400 hover:text-green-300 transition-colors\">sub-processor list</a> and framed by the 2021 Standard Contractual Clauses, a documented Transfer Impact Assessment and, where applicable, the EU-US Data Privacy Framework. Read the <a href=\"./data-processing-addendum\" class=\"text-green-400 hover:text-green-300 transition-colors\">DPA</a> and decide for yourself — no sales call required.",
    // legalTitle/legalHtml: /legal-validated bounded statement (verbatim, do not
    // rephrase). Ties the CLOUD Act analysis to the publishing entity and the data
    // it controls only; names Cloudflare (US) as staying in scope; keeps the "no
    // absolute guarantee" posture. Changing this wording requires a new /legal pass.
    "legalTitle": "Hook0 and the US CLOUD Act",
    "legalHtml": "Hook0's publisher, FGRibreau SARL, is a French company with no US subsidiary, parent, or establishment. The CLOUD Act (18 U.S.C. § 2713, amending the Stored Communications Act) requires a provider subject to US jurisdiction to produce data within its \"possession, custody, or control.\" Absent any US nexus, the publisher is not, to our knowledge and under current law, a provider that US authorities could compel, under the CLOUD Act, to produce the data it controls. This analysis concerns the publishing entity and the data it controls; it does not extend to third parties subject to US law — in particular Cloudflare (US), used at the edge, which remains within the CLOUD Act's scope for the data it processes. The data plane is hosted in France by default (Clever Cloud). As Hook0 is open source and self-hostable, a self-hosted deployment involves no third party that could be compelled."
  },
  "levers": {
    "eyebrow": "The Levers",
    "h2": "What Hook0 actually gives you",
    "intro": "These are the levers recognised for digital resilience and reversibility. Hook0 hands you the properties, not a label.",
    "cards": [
      {
        "title": "EU jurisdiction and hosting",
        "bodyHtml": "A French-law company, no US parent, with the webhook data plane hosted in France (Clever Cloud) on every plan including the free tier — not an enterprise add-on. See <a href=\"./eu-webhook-infrastructure\" class=\"text-green-400 hover:text-green-300 transition-colors\">EU webhook infrastructure</a> for the full residency picture."
      },
      {
        "title": "Open source, end to end",
        "bodyHtml": "The server is open-source under SSPL-1.0, the SDKs under MIT. There is no closed enterprise edition: the code you self-host is the code that runs the managed cloud. Nothing is hidden behind a paywall you cannot audit. See <a href=\"./open-source-webhooks\" class=\"text-green-400 hover:text-green-300 transition-colors\">open-source webhooks</a>."
      },
      {
        "title": "Reversibility, tested",
        "bodyHtml": "Run the same platform on your own infrastructure with Docker Compose or Kubernetes, or have us operate a dedicated instance on-premise. Self-hosted, there is no third-party provider left that can be compelled — you own the code and the servers. See <a href=\"./self-hosted-webhooks\" class=\"text-green-400 hover:text-green-300 transition-colors\">self-hosted webhooks</a>."
      },
      {
        "title": "Disclosed dependencies",
        "bodyHtml": "Every sub-processor and its transfer mechanism is published, not buried. The Cloudflare (USA) edge is named, and the <a href=\"./data-processing-addendum\" class=\"text-green-400 hover:text-green-300 transition-colors\">Data Processing Addendum</a> is available before you sign anything. Transparency is the part of sovereignty you can check today."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Sovereign webhook questions",
    "items": [
      {
        "q": "What makes a webhook platform sovereign?",
        "a": "Sovereignty is more than EU hosting. It comes down to who can be legally compelled to produce your data, who operates the service and holds the keys, and whether the stack is open and reversible. The verifiable levers are the jurisdiction of the company, the location of both data and operations, open-source code, disclosed dependencies and a tested exit path. Hook0 provides those levers rather than a certification badge."
      },
      {
        "q": "What is the US CLOUD Act and how does it relate to webhook data?",
        "a": "The CLOUD Act (2018) lets US authorities require a provider subject to US law to produce data it controls, wherever that data is stored, including in an EU region. That is why data location alone does not settle the sovereignty question. The company behind Hook0 is incorporated under French law with no US parent, its webhook data plane runs in France, and its one disclosed US dependency is the Cloudflare CDN and DDoS edge, published in the sub-processor list with its transfer safeguards. Review the DPA and sub-processor list to assess your own requirements."
      },
      {
        "q": "Is Hook0 open source?",
        "a": "Yes. The server is open-source under SSPL-1.0 and the SDKs under MIT. There is no closed enterprise edition: the code you self-host is the same code that runs the managed cloud, so the whole stack stays auditable and reversible."
      },
      {
        "q": "Where does Hook0 host webhook data?",
        "a": "Hook0's webhook data plane — payloads, database and backups — runs on Clever Cloud infrastructure in France, inside the European Economic Area, on every plan including the free tier. The CDN and DDoS edge in front of the site and API is Cloudflare, Inc. (USA), disclosed in a public sub-processor list with its transfer mechanisms."
      },
      {
        "q": "Can I run Hook0 with no third-party provider at all?",
        "a": "Yes. Self-host the open-source (SSPL-1.0) server with Docker Compose or Kubernetes and your webhook payloads stay inside your own network, with no third-party provider that can be compelled. You keep the same API, so your integration code does not change if you later move between self-hosted, on-premise and cloud."
      },
      {
        "q": "Is Hook0 subject to the CLOUD Act?",
        "a": "The publisher (a French company with no US nexus) is not, to our knowledge and under current law, a provider reachable by a CLOUD Act order for the data it controls. Our edge provider Cloudflare, a US company, does fall under the CLOUD Act for the data it processes."
      },
      {
        "q": "Can a US authority obtain my data?",
        "a": "Not by compelling the publisher under the CLOUD Act, as long as it has no US nexus. But no absolute guarantee is possible: data transiting our Cloudflare (US) edge is subject to US law, and other avenues (international judicial cooperation) exist independently of the CLOUD Act. Self-hosting removes any third party that could be compelled."
      },
      {
        "q": "Does the upcoming Luxembourg entity change this?",
        "a": "No, on one condition: the Luxembourg entity remains an EU entity. The reasoning holds as long as it has no US establishment, subsidiary, parent, or other US nexus."
      }
    ]
  },
  "related": {
    "h2": "Related",
    "links": [
      { "enSlug": "eu-webhook-infrastructure", "label": "EU Webhook Infrastructure" },
      { "enSlug": "open-source-webhooks", "label": "Open-Source Webhooks" },
      { "enSlug": "self-hosted-webhooks", "label": "Self-Hosted Webhooks" },
      { "enSlug": "gdpr-subprocessors", "label": "GDPR Sub-processors" },
      { "enSlug": "security", "label": "Security" },
      { "enSlug": "pricing", "label": "Pricing" },
      { "enSlug": "webhook-cost-comparison", "label": "Webhook Cost Comparison" }
    ]
  }
};
