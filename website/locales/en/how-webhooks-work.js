// Per-page strings for how-webhooks-work (EN base).
// New educational/GEO page (HOO-106 / GL issue #62): explain webhooks by
// rebuilding GitLab's from scratch. Hooky voice, humanizer pro pass applied.
module.exports = {
  pageTitle: 'How Webhooks Work: Rebuild GitLab Webhooks | Hook0',
  pageDescription:
    "Understand webhooks by rebuilding GitLab's from scratch: the HTTP POST, the X-Gitlab-Token header, and what real delivery needs beyond the happy path.",
  breadcrumb: 'How webhooks work',
  pageType: 'article',
  pageModified: '2026-09-03',
  pageSchema: {
    '@context': 'https://schema.org',
    '@type': 'TechArticle',
    headline: 'How Webhooks Work: Rebuild GitLab Webhooks From Scratch',
    description:
      "A webhook is an HTTP POST your server sends when something happens. Rebuild GitLab's webhooks to see the moving parts, then see what production delivery adds.",
    inLanguage: 'en',
    datePublished: '2026-09-03',
    dateModified: '2026-09-03',
    author: { '@type': 'Organization', name: 'Hook0', url: 'https://www.hook0.com' },
    publisher: {
      '@type': 'Organization',
      name: 'Hook0',
      url: 'https://www.hook0.com',
      logo: { '@type': 'ImageObject', url: 'https://www.hook0.com/img/hook0-social-card.png' },
    },
    mainEntityOfPage: { '@type': 'WebPage', '@id': 'https://www.hook0.com/how-webhooks-work' },
    about: [
      { '@type': 'Thing', name: 'Webhook' },
      { '@type': 'Thing', name: 'GitLab webhooks' },
      { '@type': 'Thing', name: 'HTTP callback' },
    ],
  },
  hero: {
    eyebrow: 'Webhooks, explained',
    titleBefore: 'How Webhooks Work',
    titleAccent: 'Rebuild GitLab Webhooks to Find Out',
    subtitle:
      "A webhook is one HTTP POST your server sends when something happens. That is the whole idea. The fastest way to really get it is to rebuild a webhook system you already use every day, so this page rebuilds GitLab's push webhook in about 40 lines, then shows what a production system adds on top.",
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Try the Playground',
  },
  concept: {
    eyebrow: 'The idea',
    h2: 'A Webhook Is a Reverse API Call',
    sub: 'With a normal API, your code calls someone else and waits. With a webhook, they call you.',
    body: "You register a URL with a provider. When an event fires on their side, a new commit, a paid invoice, a closed merge request, they send an HTTP POST to that URL with a JSON body describing what happened. No polling, no waiting, no cron job hammering an endpoint every 30 seconds asking \"anything new yet?\". The event pushes itself to you the moment it exists. GitLab, Stripe, GitHub, and Shopify all work this way. Under the hood it is the same three moves every time: an event happens, the provider builds a payload, the provider POSTs it to your URL.",
  },
  rebuild: {
    eyebrow: 'Rebuild it',
    h2: "Rebuild GitLab's Push Webhook",
    sub: "GitLab's own webhooks are a good teacher because the mechanics are visible and small. Here is the sender, the part GitLab runs when you push.",
    steps: [
      {
        n: '1',
        title: 'An event happens',
        body: 'A developer pushes commits. GitLab now has a fact to broadcast: repository X received a push on branch Y. Your job as the sender is to turn that fact into an HTTP request.',
      },
      {
        n: '2',
        title: 'Build the payload',
        body: 'Serialize the event to JSON: the ref that changed, the commits, the author, the project. GitLab uses one shape per event type, so a receiver can branch on the event name and trust the fields that follow.',
      },
      {
        n: '3',
        title: 'Sign and POST it',
        body: 'GitLab sets two headers a receiver checks: X-Gitlab-Event names the event ("Push Hook"), and X-Gitlab-Token carries the shared secret you configured. Then it POSTs the JSON to your subscriber URL. That request is the webhook.',
      },
    ],
    senderTitle: 'The sender (what GitLab runs)',
    senderCode:
      "// Fire a webhook when a push happens. This is the whole core.\nasync function sendPushWebhook(subscriber, push) {\n  const payload = {\n    object_kind: 'push',\n    ref: push.ref,                 // refs/heads/main\n    checkout_sha: push.sha,\n    user_username: push.author,\n    project: { name: push.project, web_url: push.url },\n    commits: push.commits,\n  };\n\n  await fetch(subscriber.url, {\n    method: 'POST',\n    headers: {\n      'Content-Type': 'application/json',\n      'X-Gitlab-Event': 'Push Hook',\n      'X-Gitlab-Token': subscriber.secretToken,\n    },\n    body: JSON.stringify(payload),\n  });\n}",
    senderFootnote:
      "GitLab authenticates with a plain shared token in X-Gitlab-Token, so the receiver compares strings. GitHub and Stripe instead sign the body with HMAC-SHA256 and send the digest in a header (X-Hub-Signature-256 for GitHub), which is stronger because the signature is bound to the exact bytes of the payload.",
    receiverTitle: 'The receiver (what you run)',
    receiverCode:
      "// Verify the token, then act on the event.\napp.post('/webhooks/gitlab', (req, res) => {\n  const token = req.header('X-Gitlab-Token');\n  if (token !== process.env.WEBHOOK_SECRET) {\n    return res.status(401).send('bad token');\n  }\n\n  const event = req.header('X-Gitlab-Event'); // 'Push Hook'\n  if (event === 'Push Hook') {\n    deployBranch(req.body.ref, req.body.checkout_sha);\n  }\n\n  res.status(200).send('ok'); // ack fast, work async\n});",
    receiverFootnote:
      'That is a working webhook end to end. It is also where the easy part ends: the code above assumes the network never fails, the receiver is always up, and nobody replays an old request.',
  },
  production: {
    eyebrow: 'The hard 20%',
    h2: 'What Production Delivery Adds',
    sub: 'The 40-line version works on a whiteboard. Real traffic breaks it in ways you only see at 3am.',
    cards: [
      {
        title: 'Retries with backoff',
        body: "The receiver returns a 500 or times out. Do you drop the event? Retry immediately and hammer a service that is already struggling? You need a retry schedule with increasing delays, a cap on attempts, and jitter so every failed delivery does not retry in lockstep.",
      },
      {
        title: 'Signatures done right',
        body: "A shared token in a header leaks the moment it lands in a log. HMAC over the raw body plus a timestamp lets the receiver verify the bytes came from you and are recent, which kills replay attacks. Now you own key storage and rotation.",
      },
      {
        title: 'Idempotency',
        body: 'A retry means the same event can arrive twice. Send a stable event id so the receiver can dedupe, or you will double-charge a card and double-deploy a branch.',
      },
      {
        title: 'Delivery visibility',
        body: 'Your first integrator will ask "did my webhook fire?" on day one. You need a log of every attempt, the response code, the latency, and a button to replay a failed delivery. Building that dashboard is its own project.',
      },
      {
        title: 'Ordering and speed',
        body: 'Push events can arrive out of order, and a slow subscriber must not block every other delivery. That means a queue, per-subscriber concurrency, and a fast path that acks before the heavy work runs.',
      },
      {
        title: 'Subscriber management',
        body: 'Endpoint registration, URL validation, event-type filtering, disabling dead endpoints, and a portal your users can self-serve. None of this is the fun part, and all of it is required.',
      },
    ],
  },
  comparison: {
    eyebrow: 'The gap',
    h2: 'Prototype vs Production',
    headers: { aspect: 'Concern', diy: '40-line prototype', hook0: 'Production system' },
    rows: [
      { aspect: 'Delivery on failure', diy: 'Event is lost', hook0: 'Retried on a backoff schedule' },
      { aspect: 'Auth', diy: 'Plain shared token', hook0: 'HMAC signature plus timestamp' },
      { aspect: 'Duplicate events', diy: 'Not handled', hook0: 'Stable event id for dedupe' },
      { aspect: 'Debugging', diy: 'Read server logs by hand', hook0: 'Delivery log with replay' },
      { aspect: 'Slow subscriber', diy: 'Blocks the sender', hook0: 'Queued, per-subscriber concurrency' },
      { aspect: 'Onboarding a subscriber', diy: 'Edit code, redeploy', hook0: 'Self-serve portal and API' },
    ],
  },
  hook0Tie: {
    eyebrow: 'Skip the plumbing',
    h2: 'Hook0 Is the Production Half, as a Service',
    body: "You just saw the 40 lines that fire a webhook, and the six hard problems that turn those 40 lines into a six-month project. Hook0 is an open-source webhook platform that owns the hard half: you POST an event once to the API, and Hook0 signs it, retries it on a configurable schedule, logs every attempt, and gives your subscribers a portal. Self-host the SSPL code or use the EU-hosted cloud. 100 events per day are free, no credit card.",
    codeTitle: 'Send an event, Hook0 delivers it',
    code:
      "curl -X POST https://app.hook0.com/api/v1/event \\\n  -H \"Authorization: Bearer YOUR_API_KEY\" \\\n  -H \"Content-Type: application/json\" \\\n  -d '{\n    \"event_type\": \"push.received\",\n    \"payload\": { \"ref\": \"refs/heads/main\", \"sha\": \"a1b2c3\" }\n  }'",
    codeFootnote: 'Retries, HMAC signatures, delivery logs, and subscriber notification are handled for you.',
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Webhook Questions, Answered',
    items: [
      {
        q: 'What is a webhook in simple terms?',
        a: 'A webhook is an automated HTTP POST that one server sends to another when a specific event happens. Instead of your app repeatedly asking an API "anything new?", the provider pushes the event to a URL you registered, the moment it occurs.',
      },
      {
        q: 'How do GitLab webhooks work?',
        a: 'When an event fires (a push, a merge request, a pipeline change), GitLab sends a POST to your configured URL with a JSON body. It sets X-Gitlab-Event to name the event and X-Gitlab-Token to carry the secret you set, so your endpoint can verify the request and branch on the event type.',
      },
      {
        q: 'What is the difference between a webhook and an API?',
        a: 'Direction. With an API, your code calls the provider and waits for a response. With a webhook, the provider calls your code when something happens. An API is pull, a webhook is push. Most integrations use both.',
      },
      {
        q: 'How do you secure a webhook endpoint?',
        a: 'Verify every request before you act on it. GitLab uses a shared token; GitHub and Stripe sign the raw body with HMAC-SHA256 and send the digest in a header, which also defends against replay when paired with a timestamp. Always serve the endpoint over HTTPS and return a fast 2xx to acknowledge receipt.',
      },
      {
        q: 'Why not just build webhooks yourself?',
        a: 'Sending the POST is 40 lines. Retries with backoff, HMAC signatures with key rotation, idempotency, delivery logs, queuing, and a subscriber portal are the other six months. A webhook platform like Hook0 gives you that half as a service, open-source and self-hostable if you want to keep it in-house.',
      },
    ],
  },
  related: {
    h2: 'Keep reading',
    links: [
      { enSlug: 'webhook-platform', label: 'What a Webhook Platform Does' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
      { enSlug: 'webhook-playground', label: 'Test Webhooks in the Playground' },
    ],
  },
};
