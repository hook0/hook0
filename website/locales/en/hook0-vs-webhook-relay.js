// Per-page strings for hook0-vs-webhook-relay (EN base).
//
// Facts captured 2026-10-06 from webhookrelay.com (home page and pricing) and
// the UK Companies House register; Hook0 from its home page, the CLI source and
// the relay server (play/). The page's structuring nuance: the two products sit
// on OPPOSITE sides of the webhook pipe.
//   - Webhook Relay = INBOUND gateway: it receives webhooks from third parties,
//     filters/transforms them and forwards them to localhost, private servers,
//     Kubernetes or cloud queues.
//   - Hook0 = OUTBOUND: you send webhooks to your own users; Hook0 owns
//     delivery, retries, HMAC signing, logs, replay, subscriber portal.
// They overlap only on local development (`hook0 listen` vs Webhook Relay
// tunnels). Honesty gate (content-seo doctrine #4): Webhook Relay DOES list a
// self-hosted option (sales-led) on its home page; never write that it is
// hosted-only. Any attribute we cannot source is "Not stated".
module.exports = {
  pageTitle: 'Hook0 vs Webhook Relay: Sending vs Receiving Webhooks',
  pageDescription: "Webhook Relay receives third-party webhooks and routes them to localhost or private servers. Hook0 sends webhooks to your users, EU-hosted.",
  pageModified: '2026-10-06',
  breadcrumb: 'Hook0 vs Webhook Relay',
  tldr: {
    h2: 'The Short Answer',
    body: "Webhook Relay and Hook0 handle opposite ends of the webhook pipe. Webhook Relay is an inbound gateway that receives webhooks from services like GitHub or Stripe, filters and transforms them, and forwards them to localhost, servers behind a firewall, Kubernetes or cloud queues. Hook0 is outbound webhooks-as-a-service. It sends your product's events to your users' endpoints, with per-attempt logs, free retries, HMAC signatures and replay. They meet on local development, where the Hook0 CLI also forwards webhooks to localhost. If you receive webhooks, pick Webhook Relay. If you send them, pick Hook0.",
  },
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Webhook Relay',
    titleAccent: 'Sending vs Receiving',
    subtitle: "Webhook Relay brings third-party webhooks into networks that have no public IP. Hook0 delivers your own events to your customers' endpoints. This page shows which direction each tool covers, where they overlap, and when Webhook Relay is the right call.",
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  differentiators: {
    eyebrow: 'What Each One Does',
    h2: 'Key Differences',
    cards: [
      { title: 'Inbound Gateway or Outbound Delivery', body: "Webhook Relay sits in front of your systems and takes in webhooks that GitHub, Stripe or Shopify send you, then routes them where you need them. Hook0 sits behind your product and publishes its events to every subscriber, so the delivery it manages goes <em>out</em> to your users." },
      { title: 'Both Can Reach Localhost', body: "Webhook Relay forwards to localhost and to servers with no public IP through its agent and tunnels. The Hook0 CLI covers the development case with <code>hook0 listen 3000</code>, a WebSocket tunnel to your local port that needs no account and runs through a relay you can host yourself." },
      { title: 'EU Publisher, EU Cloud by Default', body: "Hook0 is published by a French company with no US parent, subsidiary or establishment, and its managed cloud runs in the EU by default. Webhook Relay is published by AppScension Ltd, a UK company, and says it runs on Google Cloud; its public pages do not state a data region." },
      { title: 'Open Source You Can Run Today', body: "Hook0's server and relay are open source under SSPL-1.0 (source-available, not OSI-approved), and its client SDKs are MIT. You can self-host the whole stack without talking to anyone. Webhook Relay lists a self-hosted option through its sales team." },
    ],
  },
  comparison: {
    eyebrow: 'Side by Side',
    h2: 'What Each One Handles',
    headers: { feature: 'Feature', hook0: 'Hook0', relay: 'Webhook Relay' },
    rows: [
      { feature: 'Primary role', hook0Html: 'Outbound webhooks-as-a-service', relayHtml: 'Inbound webhook gateway and tunnels' },
      { feature: 'Webhook direction', hook0Html: 'Outbound (you send to your users)', relayHtml: 'Inbound (you receive from third parties)' },
      { feature: 'Destinations', hook0Html: "Your subscribers' HTTP endpoints", relayHtml: 'Public URLs, private servers, localhost, Kubernetes, AWS, GCP and Azure services' },
      { feature: 'Local development', hook0Html: '<code>hook0 listen</code> tunnel, no account', relayHtml: 'Tunnels from the Basic plan' },
      { feature: 'Transformations', hook0Html: 'Not offered (payloads are sent as published)', relayHtml: 'JavaScript and Lua functions, filters' },
      { feature: 'Signatures', hook0Html: 'HMAC-SHA256 on every delivery', relayHtml: 'Not stated' },
      { feature: 'Subscriber portal', hook0Html: 'Embeddable', relayHtml: 'Not applicable (not an outbound sender)' },
      { feature: 'Free plan', hook0Html: '100 events per day', relayHtml: '150 webhooks per month, no tunnels' },
      { feature: 'Publisher', hook0Html: 'French company, no US entity', relayHtml: 'AppScension Ltd (UK)' },
      { feature: 'Hosting', hook0Html: 'Managed EU cloud (Clever Cloud FR) + self-host', relayHtml: 'Google Cloud (region not stated) + self-hosted option via sales' },
      { feature: 'Source', hook0Html: 'Server and relay SSPL-1.0, SDKs MIT', relayHtml: 'Not stated' },
    ],
  },
  fit: {
    h2: 'Where Webhook Relay Fits Better',
    body: "Pick Webhook Relay when your problem is on the receiving side. Typical cases are getting GitHub, Stripe or Shopify webhooks into a CI server or a Kubernetes service with no public IP, reshaping payloads with a function before they land, or fanning one inbound webhook out to several internal targets and cloud queues. Hook0 does not transform inbound traffic, and its tunnel is a development tool.",
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Hook0 an alternative to Webhook Relay?', a: 'Only where they overlap. Both can forward webhooks to localhost during development. Beyond that they face opposite directions: Webhook Relay receives and routes the webhooks you get from third parties, and Hook0 delivers the webhooks your product sends to its users.' },
      { q: 'Can Hook0 forward webhooks to localhost?', a: 'Yes, for development. Run hook0 listen 3000 and the CLI prints a public URL on play.hook0.com. Every request sent to it is forwarded to localhost:3000 through a WebSocket tunnel. No account is needed, and you can run your own relay with --relay-url.' },
      { q: 'Does Hook0 transform or route inbound webhooks?', a: 'No. Hook0 is outbound webhooks-as-a-service. It signs, delivers, retries and logs the events you publish. Filtering, transforming and routing webhooks you receive from third parties is what Webhook Relay is built for.' },
      { q: 'Where is my data hosted?', a: 'Hook0 Cloud runs in the EU by default, and Hook0 is published by a French company with no US entity. Webhook Relay states that it runs on Google Cloud, and its public pages do not name a region. Both can be self-hosted, Hook0 from its open-source code and Webhook Relay through its sales team.' },
      { q: 'Which should I choose, Hook0 or Webhook Relay?', a: 'Pick by direction. Sending webhooks to your own customers with signatures, per-attempt logs, free retries and a subscriber portal: Hook0. Receiving webhooks from third-party services and routing them into private networks, CI or cloud queues: Webhook Relay.' },
      { q: 'Is Hook0 affiliated with Webhook Relay?', a: 'No. Webhook Relay is a trademark of its owner. Hook0 is independent and is not affiliated with or endorsed by Webhook Relay. This page is a factual comparison for teams evaluating both.' },
    ],
  },
  comparisonSources: {
    competitors: ['Webhook Relay'],
    scope: 'This page compares the role, destinations, hosting and public plans of Hook0 and Webhook Relay. Prices and quotas are each vendor\'s published figures. Company details come from the UK Companies House register.',
    sources: [
      { label: 'Webhook Relay home page', url: 'https://webhookrelay.com/', consulted: '2026-10-06' },
      { label: 'Webhook Relay pricing', url: 'https://webhookrelay.com/pricing/', consulted: '2026-10-06' },
      { label: 'AppScension Ltd on Companies House', url: 'https://find-and-update.company-information.service.gov.uk/company/10705698', consulted: '2026-10-06' },
    ],
    updated: '2026-10-06',
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-ngrok', label: 'Hook0 vs ngrok' },
      { enSlug: 'hook0-vs-requeue', label: 'Hook0 vs Requeue' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'webhook-playground', label: 'Webhook Tester' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU Webhook Infrastructure' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
      { enSlug: 'hook0-alternatives', label: 'Hook0 Alternatives' },
    ],
  },
};
