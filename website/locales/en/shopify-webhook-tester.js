// Per-page strings for shopify-webhook-tester (EN base, pilot page).
// Provider facts verified against shopify.dev webhooks HTTPS verification docs.
module.exports = {
  slug: 'shopify-webhook-tester',
  track: 'shopify-webhook-tester',
  pageTitle: 'Shopify Webhook Tester: Why Base64 Breaks Your HMAC Check | Hook0',
  pageDescription:
    'Test Shopify webhooks free, no signup. Point a webhook at a public URL and inspect the payload, the X-Shopify-Topic header and the base64 X-Shopify-Hmac-Sha256 signature you have to verify.',
  keywords:
    'shopify webhook tester, shopify webhook test, shopify webhook testing, shopify webhook inspector, shopify webhook verification, shopify webhook hmac verification',
  faqHeading: 'Shopify webhook testing questions',
  hero: {
    eyebrow: 'Shopify Webhook Tester',
    h1Before: 'Test Shopify webhooks',
    h1Accent: 'and inspect the raw HMAC',
    subtitle:
      'Point a Shopify webhook at a public URL and read the payload, the <code>X-Shopify-Topic</code> header and the base64 <code>X-Shopify-Hmac-Sha256</code> signature in your browser. No signup, nothing to install.',
    ctaPrimary: 'Open the tester',
    ctaSecondary: 'How the playground works',
    trust: 'Free, no account. Works with app and store webhooks.',
  },
  steps: {
    eyebrow: 'How it works',
    h2: 'Test a Shopify webhook in four steps',
    intro:
      'play.hook0.com receives the delivery for you, so you can see the exact body and the base64 signature before you write your verification.',
    items: [
      {
        title: 'Get a public URL',
        body: 'Open <a href="https://play.hook0.com" target="_blank" rel="noopener" class="text-indigo-400 hover:text-indigo-300 underline">the tester</a> for a URL that accepts any POST. No install, no server to run.',
      },
      {
        title: 'Subscribe a webhook to it',
        body: 'Set that URL as the address for a webhook topic, in the app config or with <code>shopify app webhook trigger</code>, then send a test delivery.',
      },
      {
        title: 'Inspect the delivery',
        body: 'Read the raw body plus the <code>X-Shopify-Topic</code>, <code>X-Shopify-Shop-Domain</code> and <code>X-Shopify-Hmac-Sha256</code> headers exactly as Shopify sent them.',
      },
      {
        title: 'Match the HMAC',
        body: 'Copy the raw body and the signature into your handler and confirm your base64 HMAC matches before going live.',
      },
    ],
  },
  signature: {
    eyebrow: 'Signature',
    h2: 'How Shopify signs a webhook, and the base64 trap',
    intro:
      'Shopify signs each webhook with your app secret so you can verify it is genuine. The catch that breaks most first attempts is the encoding. The digest is base64, not hex.',
    facts: [
      { label: 'Signature header', value: '<code>X-Shopify-Hmac-Sha256</code>.' },
      { label: 'Algorithm', value: 'HMAC-SHA256 over the raw request body, base64 encoded. Not hex, which is the usual mistake.' },
      { label: 'Signing secret', value: 'Your app client secret, the API secret key, rather than a per-webhook value.' },
      { label: 'Routing headers', value: '<code>X-Shopify-Topic</code> names the topic, <code>X-Shopify-Shop-Domain</code> names the store, <code>X-Shopify-Webhook-Id</code> lets you dedupe.' },
      { label: 'Compare safely', value: 'Decode both sides and compare bytes with a constant-time check rather than comparing strings.' },
      { label: 'Send test events', value: '<code>shopify app webhook trigger</code> sends a sample delivery to your URL.' },
    ],
    note: 'Reading the exact body and the base64 <code>X-Shopify-Hmac-Sha256</code> value in the tester shows you precisely what to reproduce.',
  },
  faq: {
    items: [
      {
        q: 'How do I test a Shopify webhook?',
        a: 'Open the tester to get a public URL, subscribe a webhook topic to it in your app config or with <code>shopify app webhook trigger</code>, then send a delivery. The tester shows the raw body and headers Shopify sent so you can inspect and replay them.',
      },
      {
        q: 'Why does my Shopify HMAC verification fail?',
        a: 'The most common reason is encoding: <code>X-Shopify-Hmac-Sha256</code> is base64, not hex. Compute HMAC-SHA256 of the raw body with your app secret, base64 encode it, then compare. Sign the raw bytes, not a re-serialized body.',
      },
      {
        q: 'Which secret signs a Shopify webhook?',
        a: 'Your app client secret, also called the API secret key, signs every webhook. It is not a per-webhook secret, so the same value verifies all topics for that app.',
      },
      {
        q: 'How do I verify the X-Shopify-Hmac-Sha256 header?',
        a: 'Take the raw request body, compute HMAC-SHA256 with your app secret, base64 encode the result and compare it to the header with a constant-time check. Inspecting the exact body and header in the tester tells you what to feed the HMAC.',
      },
      {
        q: 'Is the webhook tester free?',
        a: 'Yes. <a href="https://play.hook0.com" target="_blank" rel="noopener" class="text-indigo-400 hover:text-indigo-300 underline">play.hook0.com</a> is free and needs no signup. When you are ready to send your own signed webhooks, Hook0 Cloud has a free tier and the server is open-source to self-host.',
      },
    ],
  },
};
