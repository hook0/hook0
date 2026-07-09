// Per-page strings for eu-webhook-infrastructure (EN base).
// Legal constraints (non-negotiable, see website/CLAUDE.local.md):
//   - Data plane = Clever Cloud SAS (France, EEA). Cloudflare, Inc. (USA) CDN is
//     DISCLOSED, framed by SCC 2021 + TIA and, where applicable, the EU-US DPF.
//   - NEVER "100% sovereign", "no data leaves the EU", "CLOUD Act free".
//   - GDPR/NIS2/DORA = process/context claims ("designed for", "supports your
//     requirements"), never "certified".
//   - License = "open-source (SSPL-1.0)" whenever mentioned.
// Competitor facts sourced from the 2026-07-08 business.md snapshots
// (~/.claude/shared/orgs/hook0/competitors/*): Hookdeck Growth $499/mo,
// multi-region US/EU/Asia with exact regions unpublished; Svix Pro $490/mo, no
// EU-hosted managed cloud advertised; Convoy Premium $999/mo, no managed
// geographic residency. Managed on-prem pricing verified against
// src/includes/_pricing.ejs (Pro On-Premise: €1,000 setup + €500/mo excl. VAT,
// or €0 setup + €6,000/yr excl. VAT).
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  "pageTitle": "EU Webhook Infrastructure: Hosted in France by Default | Hook0",
  "pageDescription": "Hook0 runs its webhook data plane on Clever Cloud (France) from the free tier up. French-law company, public sub-processor list, self-host or on-prem anytime.",
  "pageModified": "2026-07-08",
  "track": "eu-webhook-infrastructure",
  "hero": {
    "eyebrow": "EU Webhook Infrastructure",
    "titleLine1": "EU by Default,",
    "titleLine2": "Not as a Paid Option",
    "subtitle": "Hook0's webhook data plane runs on Clever Cloud, in France, from the free tier up. The company behind it is incorporated under French law, with no US parent. And if you ever want out, the same codebase self-hosts — open-source (SSPL-1.0).",
    "ctaPrimary": "Start Free",
    "ctaSecondary": "See Pricing",
    "ctaSecondaryHref": "./pricing",
    "microcopy": "100 events/day free. No credit card. EU data plane on every plan."
  },
  "socialProof": true,
  "pillars": {
    "eyebrow": "Data Residency",
    "h2": "Where your webhook data actually lives",
    "cards": [
      {
        "title": "Data plane in France, from the free tier",
        "bodyHtml": "Webhook payloads, database and backups run on Clever Cloud SAS infrastructure in France, inside the European Economic Area. This is not an enterprise add-on: the free tier and every paid plan use the same EU data plane."
      },
      {
        "title": "A French-law company",
        "bodyHtml": "Hook0 is built and operated by a company incorporated under French law, with no US parent company. Your contract, your DPA and your data protection questions are handled under EU jurisdiction."
      },
      {
        "title": "A transparent edge: Cloudflare, disclosed",
        "bodyHtml": "Our CDN and DDoS protection layer is Cloudflare, Inc. (USA). We disclose it instead of burying it: those transfers are framed by the 2021 Standard Contractual Clauses, a documented Transfer Impact Assessment and, where applicable, the EU-US Data Privacy Framework. The full <a href=\"./gdpr-subprocessors\" class=\"text-green-400 hover:text-green-300 transition-colors\">sub-processor list</a> is public."
      },
      {
        "title": "A DPA you can actually read",
        "bodyHtml": "The Hook0 <a href=\"./data-processing-addendum\" class=\"text-green-400 hover:text-green-300 transition-colors\">Data Processing Addendum</a> lists every sub-processor and its transfer mechanism. No sales call required to find out where your data goes."
      }
    ]
  },
  "residency": {
    "eyebrow": "Comparison",
    "h2": "EU data residency, provider by provider",
    "subtitle": "What it takes, and what it costs, to keep webhook data in the EU with each managed provider. Public pricing, checked July 2026.",
    "headers": {
      "provider": "Provider",
      "residency": "EU residency on the managed cloud",
      "price": "Where the pricing starts"
    },
    "rows": [
      {
        "highlight": true,
        "provider": "Hook0",
        "residencyHtml": "Default. Data plane on Clever Cloud (France) on every plan, including the free tier.",
        "priceHtml": "€0 (free tier); paid plans from €59/month"
      },
      {
        "highlight": false,
        "provider": "Hookdeck",
        "residencyHtml": "US, EU and Asia regions on the managed platform; exact EU regions are not published.",
        "priceHtml": "Free tier available; the SLA-backed Growth plan is $499/month"
      },
      {
        "highlight": false,
        "provider": "Svix",
        "residencyHtml": "No EU-hosted managed cloud advertised; data residency is mentioned without an explicit EU region.",
        "priceHtml": "Managed Pro starts at $490/month; self-hosting (MIT) to control residency yourself"
      },
      {
        "highlight": false,
        "provider": "Convoy",
        "residencyHtml": "No managed geographic residency; choosing your region requires self-hosting.",
        "priceHtml": "Self-hosted Community is free (Elastic License v2); the managed Premium plan is $999/month"
      }
    ],
    "footnote": "Sources: each provider's public pricing and documentation pages, last checked on 2026-07-08. Spotted something out of date? Tell us and we will fix it."
  },
  "reversibility": {
    "eyebrow": "Reversibility",
    "h2": "Reversibility is the other half of sovereignty",
    "intro": "EU hosting matters less if leaving is impossible. Hook0's cloud, self-hosted and on-premise deployments share a single codebase.",
    "cards": [
      {
        "title": "Self-host it yourself",
        "bodyHtml": "The full Hook0 codebase is open-source (SSPL-1.0). Docker Compose or Kubernetes, PostgreSQL underneath. Your webhook payloads stay inside your own network. See <a href=\"./self-hosted-webhooks\" class=\"text-green-400 hover:text-green-300 transition-colors\">self-hosted webhooks</a>."
      },
      {
        "title": "Managed on-premise",
        "bodyHtml": "We deploy a dedicated Hook0 instance in your environment and keep it maintained and updated: €1,000 setup + €500/month (excl. VAT), or €0 setup + €6,000/year (excl. VAT). Your infrastructure, our maintenance."
      }
    ]
  },
  "compliance": {
    "eyebrow": "Compliance",
    "h2": "Built for teams with GDPR, NIS2 or DORA requirements",
    "intro": "Hook0 does not sell you a compliance stamp. It gives you the concrete properties your auditors ask about.",
    "cards": [
      {
        "title": "GDPR",
        "bodyHtml": "The data plane stays in France (EEA), sub-processors are documented with their transfer mechanisms, and a DPA is available before you sign anything. Hook0 is designed for GDPR compliance — evidence you can point to, not a badge."
      },
      {
        "title": "NIS2",
        "bodyHtml": "NIS2 is a directive that applies to your organisation, not a product certification. Hook0 supports your requirements with EU data residency by default, a public sub-processor list, per-attempt delivery logs and a documented <a href=\"./security\" class=\"text-green-400 hover:text-green-300 transition-colors\">security page</a>."
      },
      {
        "title": "DORA",
        "bodyHtml": "Financial entities under DORA scrutinise ICT third-party risk and exit strategies. Hook0 supports that analysis: EU data plane, documented sub-processors, and a real exit path — self-hosting or on-premise on the same codebase."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "EU webhook infrastructure questions",
    "items": [
      {
        "q": "Where is Hook0's webhook data hosted?",
        "a": "Hook0's webhook data plane — payloads, database and backups — runs on Clever Cloud SAS infrastructure in France, inside the European Economic Area. The CDN and DDoS protection layer in front of the website and API is Cloudflare, Inc. (USA), disclosed in our public sub-processor list and framed by the 2021 Standard Contractual Clauses, a documented Transfer Impact Assessment and, where applicable, the EU-US Data Privacy Framework."
      },
      {
        "q": "Is Hook0 GDPR-compliant?",
        "a": "Hook0 is designed for GDPR compliance: an EU data plane, a public sub-processor list with transfer mechanisms, and a Data Processing Addendum you can review before signing anything. There is no official GDPR badge for webhook providers, so whoever you evaluate, ask for the DPA and the sub-processor list. Ours are public."
      },
      {
        "q": "Does Hook0 support NIS2 or DORA requirements?",
        "a": "NIS2 and DORA apply to your organisation; no webhook vendor can be \"certified\" against them. What Hook0 provides is the material your compliance team needs: EU data residency by default, documented sub-processors, delivery logs for every attempt, and an exit strategy (self-hosting or managed on-premise, both on the same codebase as the cloud)."
      },
      {
        "q": "Can I leave the Hook0 cloud later?",
        "a": "Yes. The cloud, self-hosted and on-premise versions share one codebase, open-source under SSPL-1.0. You can self-host with Docker Compose or Kubernetes, or ask us to run a dedicated instance in your environment for €1,000 setup + €500/month (excl. VAT). Either way you keep the same API, so your integration code does not change."
      },
      {
        "q": "Which webhook providers offer EU data residency on their managed cloud?",
        "a": "As of July 2026: Hook0 hosts its data plane in France on every plan. Hookdeck advertises US, EU and Asia regions on its managed platform, without publishing exact EU regions. Svix does not advertise an EU-hosted managed cloud. Convoy offers no managed geographic residency — you pick your region by self-hosting. Providers change their offerings, so check their current documentation too."
      }
    ]
  },
  "related": {
    "h2": "Related",
    "links": [
      { "label": "Self-Hosted Webhooks", "href": "./self-hosted-webhooks" },
      { "label": "Pricing", "href": "./pricing" },
      { "label": "Webhook Cost Comparison", "href": "./webhook-cost-comparison" },
      { "label": "GDPR Sub-processors", "href": "./gdpr-subprocessors" },
      { "label": "Security", "href": "./security" },
      { "label": "Data Processing Addendum", "href": "./data-processing-addendum" }
    ]
  }
};
