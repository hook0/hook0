// Per-page strings for how-webhooks-work (EN base).
// Educational/GEO page (HOO-106, re-angled HOO-647 / GL issue #62): teach
// webhooks by rebuilding GitLab's push webhook, then demonstrate Hook0's power
// with the multitenant case: if GitLab.com ran all its webhooks on Hook0, one
// label-routed event would reach project, group, and instance subscribers with
// no custom fan-out code. Hooky voice, humanizer pro pass applied.
module.exports = {
  pageTitle: 'How Webhooks Work: Rebuild GitLab, Scale to Multitenant | Hook0',
  pageDescription:
    "Rebuild GitLab's webhook, then see how Hook0 labels route one event to project, group, and instance subscribers, the way GitLab.com would deliver at scale.",
  breadcrumb: 'How webhooks work',
  pageType: 'article',
  pageModified: '2026-09-10',
  pageSchema: {
    '@context': 'https://schema.org',
    '@type': 'TechArticle',
    headline: 'How Webhooks Work: From One GitLab Webhook to Multitenant Delivery',
    description:
      "A webhook is an HTTP POST your server sends when something happens. Rebuild GitLab's push webhook to see the moving parts, then see how one label-routed event on Hook0 reaches project, group, and instance subscribers at once.",
    inLanguage: 'en',
    datePublished: '2026-09-03',
    dateModified: '2026-09-10',
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
      { '@type': 'Thing', name: 'Multitenant webhooks' },
      { '@type': 'Thing', name: 'Webhook routing' },
    ],
  },
  hero: {
    eyebrow: 'Webhooks, explained',
    titleBefore: 'How Webhooks Work',
    titleAccent: 'Rebuild GitLab Webhooks, Then Scale Them',
    subtitle:
      "A webhook is one HTTP POST your server sends when something happens. That is the whole idea. This page rebuilds GitLab's push webhook in about 40 lines so you can see every moving part, then scales it up to what GitLab.com actually faces. If GitLab.com ran all its webhooks on Hook0, one label-routed event would reach the right subscribers at the repository, group, and instance level, with no custom fan-out code.",
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
  scale: {
    eyebrow: 'Now scale it',
    h2: 'One Webhook Is Easy. GitLab.com Has Millions.',
    sub: 'The 40-line version fires for one repository. GitLab.com runs webhooks for millions of projects, and it lets you attach them at three levels of the org tree.',
    levels: [
      {
        title: 'Project (repository) webhooks',
        body: 'Attach a webhook to a single repository and it fires for events in that project only. This is the level most integrations use, and the one the code above rebuilds.',
      },
      {
        title: 'Group webhooks',
        body: 'Attach a webhook to a group or subgroup and it fires for every project inside it, including projects added later. One endpoint covers a whole namespace without touching each repository.',
      },
      {
        title: 'System hooks (instance)',
        body: 'An instance admin registers a system hook that fires across the entire GitLab install: pushes, new projects, new users. It sits above every group and every project.',
      },
    ],
    body: "One push to one repository can need three deliveries at once: the project's own webhook, its parent group's webhook, and an instance system hook. In GitLab's source these are distinct hook types (project, group, and system) that share one delivery path. Build that yourself and every event walks the org tree, looks up the hooks at each level, and enqueues a job per hook, multiplied by millions of tenants. That routing layer is the real cost of multitenant webhooks.",
  },
  multitenant: {
    eyebrow: 'The Hook0 way',
    h2: 'GitLab.com on Hook0: Publish Once, Route by Label',
    sub: 'Hook0 turns that routing layer into labels. You publish each event once, tagged with its position in the org tree. Subscribers register at the level they care about, and the label match delivers to all of them.',
    publishTitle: 'Publish the push once, tagged with its full path',
    publishCode:
      "curl -X POST https://app.hook0.com/api/v1/event \\\n  -H \"Authorization: Bearer $HOOK0_TOKEN\" \\\n  -H \"Content-Type: application/json\" \\\n  -d '{\n    \"event_type\": \"gitlab.push\",\n    \"payload\": { \"ref\": \"refs/heads/main\", \"checkout_sha\": \"a1b2c3\" },\n    \"labels\": {\n      \"instance\": \"gitlab.com\",\n      \"namespace_id\": \"9970\",\n      \"project_id\": \"278964\",\n      \"plan\": \"ultimate\"\n    }\n  }'",
    publishFootnote:
      'Labels are up to ten key/value pairs on the event. Here they carry the full path: the instance, the group (namespace_id), the repository (project_id), and the plan.',
    routingIntro:
      'A subscription receives an event when the event carries every label the subscription lists. Fewer labels means a broader subscription, so the same push reaches all three levels at once.',
    routingHeaders: { subscriber: 'Subscriber', filter: 'Label filter', receives: 'What it receives' },
    routingRows: [
      { subscriber: 'A repository CI hook', filter: 'project_id: "278964"', receives: 'Events from that one repository' },
      { subscriber: 'A group audit sink', filter: 'namespace_id: "9970"', receives: 'Every event from every project in the group' },
      { subscriber: 'An instance system hook', filter: 'instance: "gitlab.com"', receives: 'Every event on the instance' },
    ],
    payoffTitle: 'Why this is the whole point',
    payoffBody:
      'The label match does the hierarchy for you. There is no per-tenant subscription table to keep in sync, no fan-out worker, and no routing code. Each subscriber sees only the events whose labels it asked for, so tenants stay isolated by construction. And every delivery gets the production half from the section above: retries with backoff, HMAC signatures, a delivery log, and replay, per subscriber.',
    docLinkLabel: 'Read the multi-tenant routing guide',
    docLinkHref: 'https://documentation.hook0.com/how-to-guides/multi-tenant-architecture',
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
      { aspect: 'Multitenant routing', diy: 'Custom fan-out per org level', hook0: 'Publish once, match by label' },
      { aspect: 'Onboarding a subscriber', diy: 'Edit code, redeploy', hook0: 'Self-serve portal and API' },
    ],
  },
  hook0Tie: {
    eyebrow: 'Skip the plumbing',
    h2: 'Hook0 Is the Production Half, as a Service',
    body: "You just saw the 40 lines that fire a webhook, the six problems that turn them into a six-month project, and how labels route one event across a whole org tree. Hook0 is an open-source webhook platform that owns all of it: you POST an event once with its labels, and Hook0 signs it, retries it on a schedule you set, logs every attempt, matches it to the right subscribers, and gives them a portal. Self-host the SSPL code or use the EU-hosted cloud. 100 events per day are free, no credit card.",
    codeTitle: 'Send an event, Hook0 delivers it',
    code:
      "curl -X POST https://app.hook0.com/api/v1/event \\\n  -H \"Authorization: Bearer YOUR_API_KEY\" \\\n  -H \"Content-Type: application/json\" \\\n  -d '{\n    \"event_type\": \"push.received\",\n    \"payload\": { \"ref\": \"refs/heads/main\", \"sha\": \"a1b2c3\" },\n    \"labels\": { \"project_id\": \"278964\" }\n  }'",
    codeFootnote: 'Retries, HMAC signatures, delivery logs, label routing, and subscriber notification are handled for you.',
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
        q: 'What is the difference between project and group webhooks in GitLab?',
        a: 'A project webhook fires for events in one repository. A group webhook fires for every project inside a group or subgroup, including projects added later. GitLab also has instance-level system hooks for admins. The three levels let a single push reach subscribers at the repository, group, and instance at once.',
      },
      {
        q: 'How do you build multitenant webhooks?',
        a: 'Tag every event with labels that describe the tenant, then let subscriptions filter on those labels. On Hook0 an event carries up to ten key/value labels, and a subscription receives it only when the event contains all the labels the subscription lists. That gives you per-tenant isolation and hierarchical routing without a custom fan-out layer.',
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
        a: 'Sending the POST is 40 lines. Retries with backoff, HMAC signatures with key rotation, idempotency, delivery logs, queuing, multitenant routing, and a subscriber portal are the other six months. A webhook platform like Hook0 gives you that half as a service, open-source and self-hostable if you want to keep it in-house.',
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
