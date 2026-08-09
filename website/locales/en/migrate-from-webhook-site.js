// Per-page strings for migrate-from-webhook-site (EN base).
// VERBATIM extraction from the legacy inline template: do not humanize.
module.exports = {
  pageTitle: 'Webhook.site Alternative: Switch to Hook0 in 30 Min',
  pageDescription: 'Looking for a webhook.site alternative? Hook0 is the production-grade upgrade: HMAC signatures, configurable retries, subscriber portal, open-source. Free forever.',
  breadcrumb: 'Migrate from webhook.site',
  hero: {
    eyebrow: 'Webhook.site Alternative',
    titleBefore: 'Past webhook.site?',
    titleAccent: 'Switch to Hook0',
    subtitle: 'webhook.site catches inbound HTTP for debugging. Hook0 sends webhooks out to your customers with HMAC signing, retries, delivery logs and a subscriber portal. Different job, same domain. Open-source.',
    ctaPrimary: 'Switch to Hook0',
    ctaSecondary: 'Try the Playground',
    ctaNote: '100 events/day free. No credit card. Open-source.',
  },
  vsTable: {
    eyebrow: 'Two adjacent tools',
    h2: 'Inbound inspector vs outbound platform',
    sub: 'webhook.site receives. Hook0 sends. Picking the right one upfront saves you a refactor later.',
    headers: { need: 'Need', webhookSite: 'webhook.site', hook0: 'Hook0' },
    rows: [
      { need: 'Inspect incoming requests for debugging', webhookSite: 'Yes', webhookSitePositive: true, hook0: 'Yes (play.hook0.com)', hook0Positive: true },
      { need: 'Send webhooks to your customers in production', webhookSite: 'No', webhookSitePositive: false, hook0: 'Yes', hook0Positive: true },
      { need: 'HMAC-sign every payload', webhookSite: 'No', webhookSitePositive: false, hook0: 'Yes', hook0Positive: true },
      { need: 'Retries and dead letter queues', webhookSite: 'No', webhookSitePositive: false, hook0: 'Yes', hook0Positive: true },
      { need: 'Subscriber portal for your customers', webhookSite: 'No', webhookSitePositive: false, hook0: 'Yes', hook0Positive: true },
      { need: 'Self-host on your infra', webhookSite: 'No', webhookSitePositive: false, hook0: 'Free (SSPL-1.0)', hook0Positive: true },
      { need: 'Free tier', webhookSite: 'Yes', webhookSitePositive: true, hook0: 'Yes', hook0Positive: true },
    ],
  },
  migration: {
    eyebrow: 'Migration',
    h2: 'webhook.site to production in 30 minutes',
    steps: [
      { index: 'Step 1', title: 'Create the application', body: 'Sign up, create an application. You get an auth token and an application ID on the spot. No credit card.' },
      { index: 'Step 2', title: 'Swap the URL', body: 'Replace the webhook.site URL with a Hook0 API call. Python or Node.js SDK, or plain HTTP.' },
      { index: 'Step 3', title: 'Hand over the portal', body: 'Drop the subscriber portal in front of your customers. They register their own endpoints, rotate their own keys, read their own delivery logs.' },
    ],
    codeBlock: '// Before. webhook.site as a debug receiver:\nfetch("https://webhook.site/abcd-1234", {\n  method: "POST",\n  body: JSON.stringify(payload)\n});\n\n// After. Hook0 as a production webhook platform:\nawait hook0.message.create("&lt;application_id&gt;", {\n  event_type: "invoice.paid",\n  event_id:   "evt_Wqb1k73rXprtTm7Qdlr38G",\n  payload\n});\n\n// What changes for your subscribers: signed payloads,\n// automatic retries, delivery logs they can replay themselves.\n',
    docsLink: 'Read the getting-started guide →',
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Migration questions',
    items: [
      { q: 'Is Hook0 a webhook.site alternative?', a: 'Yes: Hook0 is the production-grade alternative when you outgrow webhook.site. Where webhook.site is a request inspector ("what payload did I receive?"), Hook0 is a webhook platform: it sends events to your subscribers, signs them with HMAC, retries on failure, and stores delivery logs. Use webhook.site to debug; use Hook0 in production.' },
      { q: 'How do I migrate from webhook.site to Hook0?', a: 'Sign up for Hook0 (free, no credit card), create an application, and replace the webhook.site URL in your code with one Hook0 REST API call. You get HMAC-signed delivery, retries, dead letter queues and a subscriber portal: no code change beyond the API endpoint.' },
      { q: 'Can I still inspect raw webhook payloads with Hook0?', a: 'Yes. Every event sent through Hook0 is logged with the full request, response, status code and latency. You can replay any event from the dashboard. For ad-hoc testing without an account, play.hook0.com lets you generate disposable webhook URLs the same way webhook.site does.' },
      { q: 'Is Hook0 open-source unlike webhook.site?', a: 'Yes. Hook0 is fully open-source under SSPL-1.0 and self-hostable on Docker Compose or Kubernetes. webhook.site is a closed-source SaaS. If you need to keep traffic on your own infrastructure, Hook0 is the answer.' },
    ],
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'webhook-platform', label: 'Webhook Platform' },
      { enSlug: 'webhook-api', label: 'Webhook API' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
    ],
  },
};
