// Per-page strings for hook0-vs-requeue (EN base).
//
// Facts re-captured 2026-10-22 from the Requeue home page, its GitHub README and
// docs/faq.md (sources listed in comparisonSources). The page's structuring
// nuance: Hook0 and Requeue sit on OPPOSITE sides of the same reliability problem.
//   - Hook0  = OUTBOUND: you send webhooks to your users; Hook0 owns delivery,
//     retries, HMAC signing, per-attempt logs, replay, subscriber portal.
//   - Requeue = INBOUND dead-letter inbox: it catches the webhooks you RECEIVE
//     (Stripe, Clerk, Resend...) plus failed cron/worker jobs, and replays them
//     to your app. Its relay URL forwards a provider's webhook once and keeps a
//     copy when your app fails.
// Honesty gate (content-seo doctrine #4): the page must NOT claim Hook0 does
// production inbound dead-letter today. Any Requeue attribute we cannot source
// is written "Not stated", never invented (hosted data region, pricing).
// SSPL-1.0 is "open source (source-available, not OSI-approved)", never "the
// only true open source". Requeue's own FAQ says it does not replace outbound
// webhooks-as-a-service: quote that, do not paraphrase it into a weakness.
module.exports = {
  pageTitle: 'Hook0 vs Requeue: Inbound vs Outbound Webhooks',
  pageDescription: 'Requeue catches webhooks you receive and failed jobs, then replays them. Hook0 sends webhooks to your users with retries, HMAC and replay, EU-hosted.',
  pageModified: '2026-10-22',
  breadcrumb: 'Hook0 vs Requeue',
  tldr: {
    h2: 'The Short Answer',
    body: "Hook0 and Requeue solve adjacent problems from opposite directions. Requeue is an inbound dead-letter inbox. You point a Stripe or Clerk webhook at its relay URL, or post failed cron and worker jobs to it, and it keeps whatever your app failed to process so you can inspect it and replay it. Hook0 is outbound webhooks-as-a-service. It sends webhooks to your own users and owns that delivery, with every attempt logged, up to 24 free retries within 8 days, HMAC signatures, and replay from the dashboard or API. Choose Requeue to catch inbound failures on a small team; choose Hook0 to deliver webhooks to your customers, EU-hosted and open source. Hook0 does not offer a production inbound dead-letter today, and Requeue's own FAQ says it does not replace outbound webhooks-as-a-service.",
  },
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Requeue',
    titleAccent: 'Opposite Directions, Same Problem',
    subtitle: "Requeue catches the webhooks you receive and the cron or worker jobs that fail, then replays them into your app. Hook0 does the reverse and sends webhooks to your users and owns the delivery, with per-attempt logs, free retries, HMAC signing and replay. This page lays out which direction each one is built for, so you can tell at a glance which problem you actually have.",
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  differentiators: {
    eyebrow: 'What Each One Does',
    h2: 'Key Differences',
    cards: [
      { title: 'Catching Inbound vs Sending Outbound', body: "Requeue sits in front of traffic coming <em>in</em>. Its relay URL takes a provider's webhook, forwards the raw body and signature headers to your app with a 10-second timeout, and stores the request if your app errors or times out. Hook0 is the sender. It publishes your events and delivers them to your subscribers, so the failure it manages is a delivery <em>out</em> that did not land." },
      { title: 'Catch, Inspect, Replay on the Sending Side', body: 'The catch / inspect / replay story Requeue tells for inbound, Hook0 already runs for outbound. Every delivery attempt is logged with its status code and response, Hook0 retries up to 24 times within 8 days by default, and when the last retry fails the attempt stays in <code>failed</code> so you can replay the event from the dashboard or the API. See <a href="https://documentation.hook0.com/explanation/webhook-dead-letter-queue" class="text-indigo-400 hover:text-indigo-300 underline">what happens to a webhook after its last retry</a>.' },
      { title: 'EU Residency and GDPR by Default', body: "Hook0 is published by a French company with no US parent, subsidiary or establishment, and its managed cloud runs in the EU by default with HMAC signatures and TLS. Requeue's core runs on Cloudflare Workers and Cloudflare D1, and its public material does not state where the hosted instance stores data. If you need EU residency with Requeue, you deploy the Worker on your own Cloudflare account and choose the location there." },
      { title: 'Open Source, Free Retries, Public Pricing', body: 'Hook0\'s server is open source under SSPL-1.0 (source-available, not OSI-approved) and its 11 client SDKs are MIT. Retries are free, HMAC signing is built in, there is no per-endpoint billing, and pricing is public with a free Developer plan. Requeue\'s core and JavaScript SDK are MIT, version 0.1.0 as of October 2026, and hosted keys come from a waitlist with no public pricing yet.' },
    ],
  },
  comparison: {
    eyebrow: 'Side by Side',
    h2: 'What Each One Handles',
    headers: { feature: 'Feature', hook0: 'Hook0', requeue: 'Requeue' },
    rows: [
      { feature: 'Primary role', hook0Html: 'Outbound webhooks-as-a-service', requeueHtml: 'Inbound dead-letter inbox' },
      { feature: 'Webhook direction', hook0Html: 'Outbound (you send to your users)', requeueHtml: 'Inbound (you receive from third parties)' },
      { feature: 'What it captures', hook0Html: 'Your failed deliveries to subscribers', requeueHtml: 'Incoming webhooks + failed cron / worker jobs' },
      { feature: 'Inbound relay URL', hook0Html: 'No (the CLI tunnel is for local development)', requeueHtml: 'Yes, forwards once to your app and keeps failures' },
      { feature: 'Replay', hook0Html: 'Per event, from the dashboard or API', requeueHtml: 'One click, payload edit per delivery, bulk replay up to 50 events' },
      { feature: 'Per-attempt logs', hook0Html: 'Yes, status code and response body', requeueHtml: 'Failed events with headers, payload and reason' },
      { feature: 'Signatures', hook0Html: 'HMAC-SHA256 on every request + TLS', requeueHtml: 'HMAC-SHA256 on replays when the endpoint has a secret' },
      { feature: 'Automatic retries', hook0Html: 'Up to 24 within 8 days (3 s to 10 h apart), free', requeueHtml: 'Queued replays: 1, 2, 4, 8, 16 min, then marked failed' },
      { feature: 'Subscriber portal', hook0Html: 'Embeddable', requeueHtml: 'Not applicable (not an outbound sender)' },
      { feature: 'Hosting', hook0Html: 'Managed EU cloud (Clever Cloud, France) + self-host', requeueHtml: 'Cloudflare Workers + D1; hosted region not stated' },
      { feature: 'Source', hook0Html: 'Server SSPL-1.0, 11 SDKs MIT', requeueHtml: 'Core and JS SDK MIT (v0.1.0)' },
      { feature: 'Pricing', hook0Html: 'Public, free Developer plan', requeueHtml: 'Not stated (hosted keys by waitlist)' },
      { feature: 'Typical user', hook0Html: 'Teams emitting webhooks to their customers', requeueHtml: 'Indie and small teams (2 to 5)' },
    ],
  },
  fit: {
    h2: 'Where Requeue Fits Better',
    bullets: [
      'Stripe, Clerk or Resend send you webhooks and you want a copy of every request your app failed to process, ready to replay once the bug is fixed. That is Requeue\'s relay URL, and Hook0 does not do it.',
      'Your nightly crons and queue workers drop payloads after their last retry, and you want them all in one inbox with the error reason attached.',
      'You are a solo founder or a team of two to five, already on Cloudflare, and you are fine running a 0.1.0 MIT Worker yourself.',
    ],
    linkPrefix: 'Need an inbound gateway with its own retry pipeline instead of an inbox? See',
    link: { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Hook0 an alternative to Requeue?', a: 'They solve adjacent problems in opposite directions, so it depends on what you need. Requeue catches and replays the webhooks you receive, plus failed cron and worker jobs. Hook0 sends webhooks to your own users and manages that delivery. If you need to deliver webhooks to your customers reliably, Hook0 is the fit. If you need an inbox for failed inbound webhooks, that is Requeue\'s focus.' },
      { q: 'Does Requeue send webhooks to my customers?', a: 'No. Requeue\'s FAQ says it does not transform or fan out events and does not replace outbound webhooks-as-a-service. Its relay forwards one provider\'s webhook to one target URL in your app. Sending events to many customer endpoints, with signatures, retries and a portal where customers manage their subscriptions, is what Hook0 does.' },
      { q: 'Does Hook0 do inbound webhook dead-letter?', a: 'Not as a product today. Hook0 is outbound webhooks-as-a-service. The Hook0 CLI can receive webhooks on localhost through a built-in tunnel and that tunnel is meant for local development only.' },
      { q: 'Can I replay failed webhooks with Hook0?', a: 'Yes, on the sending side. Every delivery attempt is logged with its status code and response. Hook0 retries a failed delivery up to 24 times within 8 days by default, and once retries are exhausted the last attempt stays in failed status. You then replay the event from the dashboard or with POST /events/{event_id}/replay, which sends it again to every matching subscription.' },
      { q: 'Where does Requeue run?', a: 'According to its README, the Requeue core is a Cloudflare Worker that stores events in Cloudflare D1, and the hosted API runs at api.getrequeue.com. Its public material does not state the hosted data region. Hook0 Cloud runs in the EU, hosted in France by default, and the Hook0 server can be self-hosted on Docker or Kubernetes.' },
      { q: 'Is Requeue open source?', a: 'Yes. The Requeue core API and its JavaScript SDK are published under the MIT licence on GitHub, at version 0.1.0 as of October 2026. Hook0\'s server is open source under SSPL-1.0, which is source-available and not OSI-approved, and its 11 client SDKs are MIT. Both let you self-host; the difference is the direction of traffic each one handles.' },
      { q: 'Which should I choose, Hook0 or Requeue?', a: 'Pick by direction. Sending webhooks to your own customers, with signatures, per-attempt logs, free retries and a subscriber portal: Hook0. Catching and replaying webhooks you receive from third parties, plus failed cron and worker jobs, on a small team: Requeue. Some teams need both, one for each side of the pipe.' },
      { q: 'Is Hook0 affiliated with Requeue?', a: 'No. Requeue is a trademark of its respective owner, and Hook0 is independent and not affiliated with or endorsed by Requeue. This page is a factual comparison for teams evaluating both tools.' },
    ],
  },
  comparisonSources: {
    competitors: ['Requeue'],
    scope: "This page compares the role, relay, replay, retries, signatures, hosting, licence and pricing of Hook0 and Requeue. Requeue facts come from its public home page, the README and FAQ of its GitHub repository, and its JavaScript SDK repository, all read on the date shown. Hook0 facts come from the Hook0 documentation and pricing page.",
    sources: [
      { label: 'Requeue home page', url: 'https://getrequeue.com/', consulted: '2026-10-22', archive: 'https://web.archive.org/web/20261022060503/https://getrequeue.com/' },
      { label: 'Requeue core README on GitHub', url: 'https://github.com/requeue-hq/requeue', consulted: '2026-10-22', archive: 'https://web.archive.org/web/20261022060541/https://github.com/requeue-hq/requeue' },
      { label: 'Requeue FAQ on GitHub', url: 'https://github.com/requeue-hq/requeue/blob/main/docs/faq.md', consulted: '2026-10-22', archive: 'https://web.archive.org/web/20261022060523/https://github.com/requeue-hq/requeue/blob/main/docs/faq.md' },
      { label: 'Requeue JavaScript SDK on GitHub', url: 'https://github.com/requeue-hq/requeue-sdk-js', consulted: '2026-10-22' },
      { label: 'Hook0 retry schedule', url: 'https://documentation.hook0.com/explanation/webhook-retry-logic', consulted: '2026-10-22' },
    ],
    updated: '2026-10-22',
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-webhook-relay', label: 'Hook0 vs Webhook Relay' },
      { enSlug: 'hook0-vs-ngrok', label: 'Hook0 vs ngrok' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Open-Source Webhooks' },
      { enSlug: 'migrate-from-webhook-site', label: 'Webhook.site Alternative' },
      { enSlug: 'hook0-alternatives', label: 'Hook0 Alternatives' },
    ],
  },
};
