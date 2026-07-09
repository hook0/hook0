// Per-page strings for self-hosted-webhooks (EN base).
// VERBATIM extraction from the legacy inline template: do not humanize.
// The faq.items[].a text MUST match the visible card body byte-for-byte;
// the FAQPage JSON-LD is auto-generated from this same array.
module.exports = {
  "pageTitle": "Self-Hosted Webhooks: Docker & Kubernetes | Hook0",
  "pageDescription": "Deploy Hook0 on your infrastructure with Docker or Kubernetes. Fully open-source, same code as cloud. No data leaves your network.",
  "pageModified": "2026-06-22",
  "track": "self-hosted",
  "hero": {
    "eyebrow": "Self-Hosted",
    "titleLine1": "Self-Hosted",
    "titleLine2": "Webhook Platform",
    "subtitle": "Deploy webhooks on-premise with the same codebase as our cloud version. Your webhook payloads never leave your network. Docker Compose or Kubernetes. Open-source under SSPL v1, no vendor lock-in.",
    "ctaPrimary": "Start Free",
    "ctaPrimaryTrack": "self-hosted-hero-register",
    "ctaSecondary": "Installation Guide",
    "ctaSecondaryHref": "https://documentation.hook0.com/self-hosting/docker-compose",
    "ctaSecondaryTrack": "self-hosted-hero-docs",
    "trustIndicators": [
      "Same code as cloud",
      "SSPL-1.0 License",
      "No telemetry"
    ]
  },
  "socialProof": true,
  "whySelfHost": {
    "eyebrow": "Why Self-Host",
    "h2": "Your Data, Your Infrastructure",
    "cards": [
      {
        "icon": "shield",
        "title": "Data Sovereignty",
        "body": "On-premise webhooks where payloads stay inside your perimeter. Period. No third-party ever sees your data. CISO-friendly for healthcare, finance, government, and GDPR compliance."
      },
      {
        "icon": "code",
        "title": "Fully Open-Source",
        "body": "SSPL-1.0 licensed. No open-core tricks and no feature gates. Every line of code is on GitHub and GitLab. You can audit it, fork it, or send a PR."
      },
      {
        "icon": "server",
        "title": "Docker & Kubernetes",
        "body": "Docker Compose for dev and small deployments. Helm chart for production Kubernetes clusters. Both work out of the box."
      },
      {
        "icon": "sync",
        "title": "Same Code, Same Features",
        "body": "One codebase. The binary you deploy is built from the same repo as our cloud. Retries, signatures, monitoring, subscription management: nothing is stripped out."
      }
    ]
  },
  "deployment": {
    "eyebrow": "Deployment",
    "h2": "Two Ways to Deploy",
    "options": [
      {
        "kind": "docker",
        "title": "Docker Compose",
        "body": "Good for dev, testing, and small-scale production. Three commands, everything starts.",
        "code": "git clone https://github.com/hook0/hook0.git<br>cd hook0<br>docker compose up -d",
        "docsHref": "https://documentation.hook0.com/self-hosting/docker-compose",
        "docsLabel": "Docker Compose guide",
        "docsTrack": "self-hosted-docker-docs"
      },
      {
        "kind": "kubernetes",
        "title": "Kubernetes",
        "body": "For production. Horizontal scaling, health checks, rolling updates via Helm.",
        "code": "helm repo add hook0 https://charts.hook0.com<br>helm install hook0 hook0/hook0",
        "docsHref": "https://documentation.hook0.com/self-hosting/kubernetes",
        "docsLabel": "Kubernetes guide",
        "docsTrack": "self-hosted-k8s-docs"
      }
    ]
  },
  "whoSelfHosts": {
    "eyebrow": "Use Cases",
    "h2": "Who Self-Hosts Hook0?",
    "cards": [
      {
        "icon": "industry",
        "title": "Regulated Industries",
        "body": "Healthcare, finance, government. When your compliance team says \"no external SaaS for this data,\" you still need webhooks."
      },
      {
        "icon": "globe",
        "title": "Data Sovereignty",
        "body": "European companies under GDPR, or anyone who needs to prove exactly where data is processed and stored."
      },
      {
        "icon": "lock",
        "title": "Air-Gapped Networks",
        "body": "No internet? No problem. Hook0 has zero phone-home, zero telemetry, zero external dependencies."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Common Questions",
    "items": [
      {
        "q": "Is the self-hosted version the same as the cloud version?",
        "a": "Yes. One codebase, no \"community edition.\" What runs on our cloud is what you deploy on yours."
      },
      {
        "q": "What infrastructure do I need?",
        "a": "Docker Compose for simple setups, Kubernetes (Helm) for production. PostgreSQL for storage. A single node handles thousands of events per minute."
      },
      {
        "q": "Does my data leave my network?",
        "a": "No. Everything stays on your infrastructure. No telemetry, no phone-home, no external calls. If you prefer a managed service, Hook0 Cloud runs on French infrastructure (Clever Cloud), so your data stays in the EU and outside the US CLOUD Act."
      },
      {
        "q": "Can I get support for self-hosted deployments?",
        "a": "Yes. Commercial support covers installation help, configuration review, and priority bug fixes."
      },
      {
        "q": "Can I try Hook0 before self-hosting?",
        "a": "Yes. Our cloud version offers a free tier with 100 events per day, no credit card required. Try it out, then deploy on-premise when you are ready."
      }
    ]
  },
  "deepDive": {
    "prefix": "Want more detail?",
    "linkLabel": "Read the full self-hosting guide in our docs",
    "linkHref": "https://documentation.hook0.com/self-hosting/docker-compose",
    "suffix": "."
  },
  "related": {
    "h2": "Related",
    "links": [
      { "enSlug": "hook0-vs-svix", "label": "Hook0 vs Svix" },
      { "enSlug": "hook0-vs-hookdeck", "label": "Hook0 vs Hookdeck" },
      { "enSlug": "build-vs-buy-webhooks", "label": "Build vs Buy Webhooks" },
      { "enSlug": "webhook-cost-comparison", "label": "Webhook Cost Comparison" },
      { "enSlug": "eu-webhook-infrastructure", "label": "EU Webhook Infrastructure" }
    ]
  }
};
