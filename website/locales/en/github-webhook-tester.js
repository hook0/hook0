// Per-page strings for github-webhook-tester (EN base, pilot page).
// Provider facts verified against docs.github.com validating-webhook-deliveries.
module.exports = {
  slug: 'github-webhook-tester',
  track: 'github-webhook-tester',
  pageTitle: 'GitHub Webhook Tester: Inspect Every Delivery and Signature | Hook0',
  pageDescription:
    'Test GitHub webhooks free, no signup. Point a repository webhook at a public URL and inspect the payload, the X-GitHub-Event header and the X-Hub-Signature-256 you have to verify.',
  keywords:
    'github webhook tester, test github webhook, github webhook test, test github webhook locally, verify github webhook signature, github webhook debug',
  faqHeading: 'GitHub webhook testing questions',
  hero: {
    eyebrow: 'GitHub Webhook Tester',
    h1Before: 'Test GitHub webhooks',
    h1Accent: 'and inspect every delivery',
    subtitle:
      'Point a repository or organization webhook at a public URL and read the payload, the <code>X-GitHub-Event</code> type and the <code>X-Hub-Signature-256</code> header in your browser. No signup, nothing to install.',
    ctaPrimary: 'Open the tester',
    ctaSecondary: 'How the playground works',
    trust: 'Free, no account. Works with repository and organization webhooks.',
  },
  steps: {
    eyebrow: 'How it works',
    h2: 'Test a GitHub webhook in four steps',
    intro:
      'play.hook0.com catches the delivery for you, so you can read the exact payload and headers before your handler is anywhere near ready.',
    items: [
      {
        title: 'Get a public URL',
        body: 'Open <a href="https://play.hook0.com" target="_blank" rel="noopener" class="text-indigo-400 hover:text-indigo-300 underline">the tester</a> for a URL that accepts any POST. No install, no server of your own.',
      },
      {
        title: 'Add it to your repo',
        body: 'In your repository, open Settings, Webhooks, Add webhook and paste the URL as the Payload URL. Choose the events you care about and save.',
      },
      {
        title: 'Trigger and inspect',
        body: 'Do the action, such as a push or a new pull request, then read the raw body plus the <code>X-GitHub-Event</code>, <code>X-GitHub-Delivery</code> and <code>X-Hub-Signature-256</code> headers.',
      },
      {
        title: 'Redeliver while you debug',
        body: 'GitHub keeps a Recent Deliveries list on the webhook with a Redeliver button, so you can replay the same payload as you fix your handler.',
      },
    ],
  },
  signature: {
    eyebrow: 'Signature',
    h2: 'How GitHub signs a webhook delivery',
    intro:
      'GitHub signs the delivery with a secret you set on the webhook, so you can confirm the request really came from GitHub. Match the header exactly and you are done.',
    facts: [
      { label: 'Signature header', value: '<code>X-Hub-Signature-256</code>, a value that starts with <code>sha256=</code>.' },
      { label: 'Algorithm', value: 'HMAC-SHA256, hex encoded, computed over the raw request body.' },
      { label: 'Legacy header', value: '<code>X-Hub-Signature</code> carries an older SHA-1 signature. Prefer the 256 version and ignore SHA-1.' },
      { label: 'Shared secret', value: 'The secret you typed into the webhook settings. GitHub does not show it again, so store it when you create the hook.' },
      { label: 'Event and delivery', value: '<code>X-GitHub-Event</code> names the event, <code>X-GitHub-Delivery</code> is a unique id for that delivery.' },
      { label: 'Replay a delivery', value: 'The Recent Deliveries tab on the webhook lets you resend any past payload.' },
    ],
    note: 'Read the raw body and the <code>X-Hub-Signature-256</code> value in the tester, then reproduce the HMAC in your own code to confirm it matches.',
  },
  faq: {
    items: [
      {
        q: 'How do I test a GitHub webhook?',
        a: 'Open the tester to get a public URL, add it as the Payload URL under your repository Settings, Webhooks, then trigger an event. The tester shows the exact body and headers GitHub sent, and GitHub keeps a Recent Deliveries list you can redeliver from.',
      },
      {
        q: 'How do I test a GitHub webhook locally?',
        a: 'Point the webhook at a public URL that receives the payload for you, inspect it, then replay it against your local server. That avoids exposing localhost to the internet while you build the handler.',
      },
      {
        q: 'Which header holds the GitHub webhook signature?',
        a: '<code>X-Hub-Signature-256</code>, a hex HMAC-SHA256 of the raw body prefixed with <code>sha256=</code>. An older <code>X-Hub-Signature</code> header carries a SHA-1 value that you can ignore.',
      },
      {
        q: 'Why is my GitHub signature check failing?',
        a: 'Usually because the body was parsed and re-encoded before hashing. GitHub signs the raw bytes, so compute the HMAC over the exact payload you received. Comparing the header and body in the tester shows you what to sign.',
      },
      {
        q: 'Is the webhook tester free?',
        a: 'Yes. <a href="https://play.hook0.com" target="_blank" rel="noopener" class="text-indigo-400 hover:text-indigo-300 underline">play.hook0.com</a> is free with no signup. When you want to send your own signed webhooks to your users, Hook0 Cloud has a free tier and the server is open-source to self-host.',
      },
    ],
  },
};
