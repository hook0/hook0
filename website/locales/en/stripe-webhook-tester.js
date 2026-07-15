// Per-page strings for stripe-webhook-tester (EN base, pilot page).
// Provider facts verified against docs.stripe.com/webhooks.
module.exports = {
  slug: 'stripe-webhook-tester',
  track: 'stripe-webhook-tester',
  pageTitle: 'Stripe Webhook Tester: The Signed Payload, Header by Header | Hook0',
  pageDescription:
    'Test Stripe webhooks free, no signup. Point a Stripe endpoint at a public URL, then read the raw body and the exact Stripe-Signature header so your whsec_ verification actually matches.',
  keywords:
    'stripe webhook tester, test stripe webhook, stripe webhook testing, stripe webhook test event, verify stripe webhook signature, stripe webhook inspector',
  faqHeading: 'Stripe webhook testing questions',
  hero: {
    eyebrow: 'Stripe Webhook Tester',
    h1Before: 'Test Stripe webhooks',
    h1Accent: 'and read every byte Stripe sends',
    subtitle:
      'Point a Stripe endpoint at a public URL, then inspect the raw body and the <code>Stripe-Signature</code> header in your browser. No signup, nothing to install. The quickest way to see why a signature check passes or fails.',
    ctaPrimary: 'Open the tester',
    ctaSecondary: 'How the playground works',
    trust: 'Free, no account. Works with test-mode and live events.',
  },
  steps: {
    eyebrow: 'How it works',
    h2: 'Test a Stripe webhook in four steps',
    intro:
      'play.hook0.com receives the event for you, so you can see what Stripe actually posted before you write a line of verification code.',
    items: [
      {
        title: 'Get a public URL',
        body: 'Open <a href="https://play.hook0.com" target="_blank" rel="noopener" class="text-indigo-400 hover:text-indigo-300 underline">the tester</a> and it hands you a URL that accepts any HTTP POST. Nothing to install.',
      },
      {
        title: 'Point Stripe at it',
        body: 'Add the URL as an endpoint in the Stripe Dashboard, or forward to it with <code>stripe listen --forward-to</code>. Then fire an event with <code>stripe trigger payment_intent.succeeded</code>.',
      },
      {
        title: 'Read the raw delivery',
        body: 'See the exact body Stripe posted and every header, including <code>Stripe-Signature</code> with its <code>t=</code> timestamp and <code>v1=</code> signature.',
      },
      {
        title: 'Fix your verification',
        body: 'Copy the raw body and the signature into your own handler and confirm the HMAC matches before you wire it to production.',
      },
    ],
  },
  signature: {
    eyebrow: 'Signature',
    h2: 'What the Stripe-Signature header actually contains',
    intro:
      'Stripe signs each event so you can prove it came from Stripe and was not replayed. Getting the signed string wrong is the usual reason a check fails, so here is exactly what to match.',
    facts: [
      { label: 'Signature header', value: '<code>Stripe-Signature</code>, carrying a <code>t=</code> timestamp and a <code>v1=</code> signature.' },
      { label: 'Algorithm', value: 'HMAC-SHA256, hex encoded.' },
      { label: 'What gets signed', value: 'The timestamp, a dot, then the raw request body: <code>t + "." + payload</code>. Sign the raw bytes, not a re-serialized object.' },
      { label: 'Signing secret', value: 'Your endpoint secret, prefixed <code>whsec_</code>. Test mode and live mode each have their own.' },
      { label: 'Replay window', value: 'Stripe libraries reject a timestamp older than five minutes by default. Keep that check on.' },
      { label: 'Send test events', value: '<code>stripe trigger</code>, <code>stripe listen --forward-to</code>, or create an object in the Dashboard.' },
    ],
    note: 'The tester shows the raw body and the <code>Stripe-Signature</code> value side by side, which is all you need to reproduce the HMAC locally.',
  },
  faq: {
    items: [
      {
        q: 'How do I test a Stripe webhook?',
        a: 'Open the tester to get a public URL, add it as an endpoint in the Stripe Dashboard or forward to it with <code>stripe listen</code>, then send an event with <code>stripe trigger</code>. The tester shows the raw body and headers Stripe posted so you can inspect and replay them.',
      },
      {
        q: 'Where do I find my Stripe webhook signing secret?',
        a: 'In the Stripe Dashboard, open the endpoint under Developers, Webhooks and reveal its signing secret. It starts with <code>whsec_</code>. When you run <code>stripe listen</code>, the CLI prints a separate secret for local forwarding.',
      },
      {
        q: 'Why does my Stripe signature verification fail?',
        a: 'Almost always because the signed payload does not match byte for byte. Stripe signs <code>timestamp + "." + raw body</code>, so verify the raw request bytes rather than a parsed and re-serialized object. Reading the exact body and <code>Stripe-Signature</code> header in the tester tells you what to reproduce.',
      },
      {
        q: 'Can I test Stripe webhooks without my own server?',
        a: 'Yes, at least for inspecting deliveries. The tester gives you a public URL that receives events in the browser, so you do not have to expose localhost. You still need a Stripe account to send events, and test mode is free.',
      },
      {
        q: 'Is the webhook tester free?',
        a: 'Yes. <a href="https://play.hook0.com" target="_blank" rel="noopener" class="text-indigo-400 hover:text-indigo-300 underline">play.hook0.com</a> is free and needs no signup. When you are ready to send your own signed webhooks in production, Hook0 Cloud has a free tier and the server is open-source to self-host.',
      },
    ],
  },
};
