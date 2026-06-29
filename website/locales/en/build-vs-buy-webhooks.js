// Per-page strings for build-vs-buy-webhooks (EN base).
// VERBATIM extraction from the legacy inline template: do not humanize.
module.exports = {
  pageTitle: 'Build vs Buy Webhooks: Ship in 30 Min | Hook0',
  pageDescription: 'Building webhooks from scratch costs 3+ sprints. Retries, signatures, monitoring, dead letter queues: or use Hook0 and ship in 30 minutes.',
  hero: {
    eyebrow: 'Build vs Buy',
    titleBefore: 'Stop Building Webhooks',
    titleAccent: 'From Scratch',
    subtitle: 'You have a backlog full of features your users actually want. Retries, signatures, monitoring, dead letter queues: that is 3+ sprints of plumbing. Hook0 is an open-source webhook service that handles all of it. 100 events/day free, no credit card required. Ship in 30 minutes.',
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
    stats: [
      { value: '3+', label: 'Sprints to build in-house', color: 'green' },
      { value: '30 min', label: 'To integrate Hook0', color: 'indigo' },
      { value: '$0', label: 'To start (free tier)', color: 'green' },
    ],
  },
  hiddenCosts: {
    eyebrow: 'The Real Cost',
    h2: 'What You Actually Have to Build',
    sub: 'Sending an HTTP POST is easy. Building a production-grade webhook system is not.',
    cards: [
      { title: 'Retry Logic', body: 'Two-phase schedules, jitter, max attempts, per-subscription configuration. You will ship bugs here. Everyone does.' },
      { title: 'Dead Letter Queues', body: 'What happens when retries are exhausted? You need DLQ storage, alerting, and manual replay tooling.' },
      { title: 'HMAC Signatures', body: "Cryptographic signing, key rotation, timestamp validation, replay attack prevention. Get any of this wrong and your customers' data leaks." },
      { title: 'Delivery Monitoring', body: 'Dashboards, delivery logs, success/failure rates, latency tracking. Your first customer will ask "did my webhook go through?" on day one.' },
      { title: 'Subscriber Management', body: 'Endpoint registration, event type filtering, URL validation, multi-subscription support. This alone is a month of work if you do it right.' },
      { title: 'Ongoing Maintenance', body: 'Database migrations, scaling, on-call rotations, security patches. Six months after launch, someone still gets paged for it at 3am.' },
    ],
  },
  comparison: {
    eyebrow: 'Comparison',
    h2: 'Build In-House vs Use Hook0',
    headers: { aspect: 'Aspect', diy: 'Build In-House', hook0: 'Hook0' },
    rows: [
      { aspect: 'Time to production', diyHtml: '3+ sprints (6-12 weeks)', hook0Html: '30 minutes', diyDim: false },
      { aspect: 'Engineering cost', diyHtml: '2-3 FTE for months', hook0Html: 'One developer, one afternoon', diyDim: false },
      { aspect: 'Ongoing maintenance', diyHtml: 'Continuous (bugs, scaling, patches)', hook0Html: 'Managed by Hook0', diyDim: false },
      { aspect: 'Retry logic', diyHtml: 'Build from scratch', hook0Html: 'Built-in with configurable two-phase retries (fast + slow), customizable per subscription', diyDim: false },
      { aspect: 'Security (HMAC)', diyHtml: 'Implement and maintain', hook0Html: 'Automatic on every event', diyDim: false },
      { aspect: 'Monitoring & logs', diyHtml: 'Build dashboards', hook0Html: 'Included out of the box', diyDim: false },
      { aspect: 'Subscription management', diyHtml: 'Build a whole UI', hook0Html: 'Embeddable portal included', diyDim: false },
      { aspect: 'Vendor lock-in', diyHtml: 'None (but locked to your code)', hook0Html: 'None (open-source, self-hostable)', diyDim: true },
    ],
  },
  integration: {
    eyebrow: 'Integration',
    h2: 'Ship Webhooks in 30 Minutes',
    sub: 'One API call to publish an outbound event. Hook0 is webhooks as a service for event-driven architectures. It handles the rest.',
    codeBlock: 'curl -X POST https://app.hook0.com/api/v1/event \\\n  -H "Authorization: Bearer YOUR_API_KEY" \\\n  -H "Content-Type: application/json" \\\n  -d \'{\n    "event_type": "invoice.paid",\n    "payload": {\n      "invoice_id": "inv_123",\n      "amount": 9900,\n      "currency": "eur"\n    }\n  }\'',
    codeFootnote: 'Retries, HMAC signatures, delivery logging, subscriber notification: handled.',
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'How long does it take to build webhooks from scratch?', a: 'Plan for 3+ engineering sprints minimum. Retry logic, dead letter queues, HMAC signatures, delivery monitoring, subscriber management, endpoint health checking. And that is before your first customer finds a bug.' },
      { q: 'What is the hidden cost of building your own?', a: 'Retry queue maintenance, edge case handling (timeouts, redirects, certificate errors), monitoring dashboards, rate limiting, log storage, subscriber onboarding. None of this stops after v1 ships. It compounds.' },
      { q: 'How quickly can I integrate Hook0?', a: 'Under 30 minutes. One API call to trigger an event. SDKs for Python, Node.js, and others if you prefer.' },
      { q: 'Can I migrate from a homegrown system?', a: 'Yes. REST API and SDKs, so you can run both systems in parallel during migration. No big bang cutover required.' },
    ],
  },
  deepDiveHtml: 'Want more detail? <a href="https://documentation.hook0.com/tutorials/getting-started" class="text-indigo-400 hover:text-indigo-300 underline">Read the getting started guide in our docs</a>.',
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
    ],
  },
};
