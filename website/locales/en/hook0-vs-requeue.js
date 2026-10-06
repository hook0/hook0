// Per-page strings for hook0-vs-requeue (EN base).
//
// Facts captured 2026-10-02 against the Requeue snapshot (getrequeue.com) and the
// Hook0 home page. The page's structuring nuance: Hook0 and Requeue sit on
// OPPOSITE sides of the same reliability problem.
//   - Hook0  = OUTBOUND: you send webhooks to your users; Hook0 owns delivery,
//     retries, HMAC signing, per-attempt logs, replay, subscriber portal.
//   - Requeue = INBOUND dead-letter inbox: it catches the webhooks you RECEIVE
//     (Stripe, Clerk, Resend...) plus failed cron/worker jobs, and replays them
//     to your app.
// Honesty gate (content-seo doctrine #4): the page must NOT claim Hook0 does
// production inbound dead-letter today. Any Requeue attribute we cannot source
// is written "Not stated", never invented. SSPL-1.0 is "open source
// (source-available, not OSI-approved)", never "the only true open source".
module.exports = {
  pageTitle: 'Hook0 vs Requeue: Inbound vs Outbound Webhooks',
  pageDescription: 'Requeue is an inbound dead-letter inbox for webhooks you receive and failed jobs. Hook0 is outbound webhooks-as-a-service: delivery, free retries, HMAC, replay, EU-hosted. Which one fits.',
  pageModified: '2026-10-06',
  breadcrumb: 'Hook0 vs Requeue',
  tldr: {
    h2: 'The Short Answer',
    body: "Hook0 and Requeue solve adjacent problems from opposite directions. Requeue is an inbound dead-letter inbox: it catches the webhooks you receive from providers like Stripe or Clerk, plus failed cron and worker jobs, and lets you replay them to your app. Hook0 is outbound webhooks-as-a-service: it sends webhooks to your own users and owns that delivery, with every attempt logged, payloads you can inspect, configurable free retries, HMAC signatures, and replay from the dashboard or API. Choose Requeue to catch and replay inbound failures on a small team; choose Hook0 to deliver webhooks to your customers at scale, EU-hosted and open source. Hook0 does not offer a production inbound dead-letter today.",
  },
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs Requeue',
    titleAccent: 'Opposite Directions, Same Problem',
    subtitle: "Requeue catches the webhooks you receive and the cron or worker jobs that fail, then replays them into your app. Hook0 does the reverse: it sends webhooks to your users and owns the delivery, with per-attempt logs, free retries, HMAC signing and replay. This page lays out which direction each one is built for, so you can tell at a glance which problem you actually have.",
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  differentiators: {
    eyebrow: 'What Each One Does',
    h2: 'Key Differences',
    cards: [
      { title: 'Catching Inbound vs Sending Outbound', body: "Requeue is a dead-letter inbox for traffic coming <em>in</em>: the webhooks you receive from third parties and the background jobs that fail. Hook0 is the sender. It publishes your events and delivers them to your subscribers, so the failure it manages is a delivery <em>out</em> that did not land. Same reliability problem, opposite ends of the pipe." },
      { title: 'Catch, Inspect, Replay on the Sending Side', body: 'The catch / inspect / replay story Requeue tells for inbound, Hook0 already runs for outbound: every delivery attempt is logged, you can inspect payloads and responses, find out why a delivery failed, and replay events from the dashboard or API. Events and responses are persisted, so a debugging trail is there when you need it.' },
      { title: 'EU Residency and GDPR by Default', body: "Hook0 is published by a French company with no US entity, outside the reach of the US Cloud Act, and its managed cloud runs in the EU by default with HMAC signatures and TLS. Requeue ships a self-hostable core and a hosted API on a waitlist; its public material does not state where the hosted data lives, so EU residency is on you to self-host." },
      { title: 'Open Source, Free Retries, No Per-Endpoint Billing', body: 'Hook0\'s server is open source under SSPL-1.0 (source-available, not OSI-approved) and its 11 client SDKs are MIT. Retries are included for free, HMAC signing is built in, and there is no per-endpoint billing. Requeue also publishes a self-hostable open-source core. Both let you run the software yourself; the fit comes down to which direction of traffic you need to make reliable.' },
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
      { feature: 'Replay', hook0Html: 'From the dashboard or API', requeueHtml: 'One-click replay with payload editing, bulk replay' },
      { feature: 'Per-attempt logs', hook0Html: 'Yes, inspect payloads and responses', requeueHtml: 'Captured failures held in the inbox' },
      { feature: 'Signatures', hook0Html: 'HMAC-SHA256 + TLS', requeueHtml: 'Not stated' },
      { feature: 'Subscriber portal', hook0Html: 'Embeddable', requeueHtml: 'Not applicable (not an outbound sender)' },
      { feature: 'Retries', hook0Html: 'Configurable 2-phase, free', requeueHtml: 'Manual and bulk replay' },
      { feature: 'Hosting', hook0Html: 'Managed EU cloud (Clever Cloud FR) + self-host', requeueHtml: 'Self-hostable core, hosted API on waitlist' },
      { feature: 'Source', hook0Html: 'Server SSPL-1.0, 11 SDKs MIT', requeueHtml: 'Open-source core (self-hostable)' },
      { feature: 'Typical user', hook0Html: 'Teams emitting webhooks to their customers', requeueHtml: 'Indie and small teams (2 to 5)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Hook0 an alternative to Requeue?', a: 'They solve adjacent problems in opposite directions, so it depends on what you need. Requeue catches and replays the webhooks you receive, plus failed cron and worker jobs. Hook0 sends webhooks to your own users and manages that delivery. If you need to deliver webhooks to your customers reliably, Hook0 is the fit. If you need an inbox for failed inbound webhooks today, that is Requeue\'s focus.' },
      { q: 'Does Hook0 do inbound webhook dead-letter?', a: 'Not as a product today. Hook0 is outbound webhooks-as-a-service. The Hook0 CLI can receive webhooks on localhost through a built-in tunnel for local development, but that is a developer tool, not a production inbound dead-letter inbox.' },
      { q: 'Can I replay failed webhooks with Hook0?', a: 'Yes, on the sending side. Every delivery attempt is logged, you can inspect the payload and the response, find out why a delivery failed, and replay events from the dashboard or the API. Retries are configurable in two phases and free.' },
      { q: 'Is Requeue open source?', a: 'Requeue ships a self-hostable open-source core, with a hosted API on a waitlist. Hook0\'s server is open source under SSPL-1.0, which is source-available and not OSI-approved, and its 11 client SDKs are MIT. Both let you self-host; the difference is the direction of traffic each one handles.' },
      { q: 'Which should I choose, Hook0 or Requeue?', a: 'Pick by direction. Sending webhooks to your own customers at scale, with signatures, per-attempt logs, free retries and a subscriber portal: Hook0. Catching and replaying webhooks you receive from third parties, plus failed cron and worker jobs, on a small team: Requeue. They sit on adjacent sides of the same reliability problem.' },
      { q: 'Is Hook0 affiliated with Requeue?', a: 'No. Requeue is a trademark of its respective owner, and Hook0 is independent and not affiliated with or endorsed by Requeue. This page is a factual comparison for teams evaluating both tools.' },
    ],
  },
  comparisonSources: {
    competitors: ['Requeue'],
    scope: "This page compares the role, replay, hosting and source of Hook0 and Requeue. Requeue facts come from its public home page and GitHub repository.",
    sources: [
      { label: 'Requeue home page', url: 'https://getrequeue.com/', consulted: '2026-10-06' },
      { label: 'Requeue on GitHub', url: 'https://github.com/requeue-hq/requeue', consulted: '2026-10-06' },
    ],
    updated: '2026-10-06',
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'migrate-from-webhook-site', label: 'Webhook.site Alternative' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Open-Source Webhooks' },
      { enSlug: 'hook0-alternatives', label: 'Hook0 Alternatives' },
    ],
  },
};
