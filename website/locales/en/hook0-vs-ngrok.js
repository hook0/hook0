// Per-page strings for hook0-vs-ngrok (EN base).
//
// Facts captured 2026-10-06: ngrok from its public pricing page, its Share
// Localhost quickstart and its verify-webhook traffic-policy doc; Hook0 from the
// CLI source (cli/src/commands/listen.rs), the relay server (play/) and the CLI
// reference page. The page's structuring nuance: the two overlap on ONE job,
// receiving webhooks on localhost while you develop.
//   - ngrok = general-purpose ingress (HTTP/S, TCP, TLS endpoints, custom
//     domains, traffic policy). Webhooks are one use case among many.
//   - Hook0 = outbound webhooks-as-a-service. `hook0 listen` is a developer
//     tunnel built into its CLI, backed by an open-source relay you can host.
// Honesty gate (content-seo doctrine #4): the page must NOT present Hook0 as a
// general ingress or production tunnel. ngrok's install path is simpler (brew,
// apt, Windows store) while the Hook0 CLI installs through Cargo today; say so.
// If ngrok's plans change, update comparisonSources dates with the rows.
module.exports = {
  pageTitle: 'Hook0 vs ngrok for Webhooks: Local Tunnel Compared',
  pageDescription: "ngrok exposes any local service. The Hook0 CLI forwards webhooks to localhost with no account, through an open-source relay you can self-host.",
  pageModified: '2026-10-06',
  breadcrumb: 'Hook0 vs ngrok',
  tldr: {
    h2: 'The Short Answer',
    body: "ngrok and Hook0 overlap on one job, which is getting webhooks onto your laptop while you build. ngrok is a general-purpose ingress that exposes any local HTTP, TCP or TLS service, with custom domains and traffic policies on paid plans. Hook0 is outbound webhooks-as-a-service, and its CLI ships hook0 listen, a WebSocket tunnel that forwards webhooks to localhost without an account. The relay behind it is open source and you can run your own. Use ngrok when you need to expose more than webhooks. Use Hook0 when the webhooks you are testing are the ones your product will send to its users.",
  },
  hero: {
    eyebrow: 'Comparison',
    titleBefore: 'Hook0 vs ngrok',
    titleAccent: 'Receiving Webhooks on Localhost',
    subtitle: "ngrok puts any local service on a public URL. The Hook0 CLI does one narrower thing. It forwards webhooks to your local server through a relay you can self-host. This page compares the two on that job, and says where ngrok is the better pick.",
    ctaPrimary: 'Start Free',
    ctaSecondary: 'Read the CLI Docs',
  },
  differentiators: {
    eyebrow: 'What Each One Does',
    h2: 'Key Differences',
    cards: [
      { title: 'General Ingress or Webhook Tunnel', body: "ngrok exposes local HTTP, TCP and TLS services, so it covers demos, APIs, SSH and webhooks alike. <code>hook0 listen 3000</code> covers webhooks only. It opens a WebSocket to the relay, prints a public URL, and forwards each incoming request to your local port." },
      { title: 'No Account to Start', body: "The ngrok agent needs an account and an authtoken before the first tunnel. <code>hook0 listen</code> generates a token locally and connects straight to the public relay, so you get a URL with no signup. Pass <code>--token</code> to keep the same URL across restarts." },
      { title: 'A Relay You Can Host Yourself', body: "The Hook0 relay server is open source under SSPL-1.0 and ships with a Helm chart in the Hook0 repository. Point the CLI at your own instance with <code>--relay-url</code>, so test payloads stay on infrastructure you run. ngrok is a hosted service." },
      { title: 'Built for the Sending Side', body: "Hook0's main product delivers webhooks to your users, with per-attempt logs, free retries, HMAC signatures and replay. The same CLI sends events, manages subscriptions and replays failures, so you can test the full loop from <code>hook0 event send</code> to your local handler." },
    ],
  },
  comparison: {
    eyebrow: 'Side by Side',
    h2: 'Receiving Webhooks Locally',
    headers: { feature: 'Feature', hook0: 'Hook0 CLI', ngrok: 'ngrok' },
    rows: [
      { feature: 'Primary role', hook0Html: 'Outbound webhooks-as-a-service, with a dev tunnel in the CLI', ngrokHtml: 'General-purpose ingress and tunnels' },
      { feature: 'Protocols exposed', hook0Html: 'HTTP webhooks forwarded to a local URL', ngrokHtml: 'HTTP/S and TCP; TLS on Pay-as-you-go' },
      { feature: 'Account needed', hook0Html: 'No, for <code>hook0 listen</code>', ngrokHtml: 'Yes, account and authtoken' },
      { feature: 'Stable URL', hook0Html: 'Same token gives the same URL (<code>--token</code>)', ngrokHtml: 'One development domain on every plan; custom domains on Pay-as-you-go' },
      { feature: 'Free plan', hook0Html: 'Public relay at no cost', ngrokHtml: '1 GB data transfer, 20,000 HTTP/S requests, up to 3 endpoints' },
      { feature: 'Request inspection', hook0Html: 'Terminal dashboard (TUI) with headers, body and status', ngrokHtml: 'Traffic Inspector, 24 h retention on Free' },
      { feature: 'Webhook signature checks', hook0Html: 'HMAC-SHA256 signing on the sending side', ngrokHtml: 'verify-webhook traffic-policy action for many providers' },
      { feature: 'Forwarding guard', hook0Html: 'Localhost only unless <code>--allow-external</code>', ngrokHtml: 'Not applicable (you choose the upstream)' },
      { feature: 'Self-host the relay', hook0Html: 'Yes, open source (SSPL-1.0) with a Helm chart', ngrokHtml: 'No, hosted service' },
      { feature: 'Install', hook0Html: '<code>cargo install</code> (needs Rust)', ngrokHtml: 'Homebrew, apt, Windows store or direct download' },
    ],
  },
  fit: {
    h2: 'Where ngrok Fits Better',
    body: "Pick ngrok when you need to expose something other than webhooks, such as a demo app, an API for a teammate, a TCP service, or an endpoint behind a custom domain with traffic policies. It also installs in one command on most systems, while the Hook0 CLI currently installs through Cargo.",
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Common Questions',
    items: [
      { q: 'Is Hook0 an ngrok alternative?', a: 'For one job, yes: receiving webhooks on localhost during development. hook0 listen forwards webhooks to your local server through a WebSocket relay, with no account. For anything beyond webhooks (TCP services, demos, custom domains, traffic policies), ngrok is the broader tool.' },
      { q: 'How do I receive webhooks on localhost with Hook0?', a: 'Install the CLI, then run hook0 listen 3000. It prints a public URL on play.hook0.com. Paste that URL into the provider or into a Hook0 subscription, and every request is forwarded to localhost:3000 and shown in the terminal dashboard.' },
      { q: 'Do I need an account to use hook0 listen?', a: 'No. The CLI generates a token on your machine and connects to the public relay. You need a Hook0 account only to send events through the Hook0 API, for example with hook0 event send.' },
      { q: 'Can I self-host the Hook0 relay?', a: 'Yes. The relay server is in the Hook0 repository under SSPL-1.0, with a Helm chart. Run your instance, then point the CLI at it with --relay-url. Add --insecure if it uses a self-signed certificate.' },
      { q: 'Can I keep the same webhook URL between sessions?', a: 'Yes. The URL comes from the token, so hook0 listen 3000 --token my-stable-token gives you the same URL each time. On ngrok, every plan includes one development domain.' },
      { q: 'Is Hook0 affiliated with ngrok?', a: 'No. ngrok is a trademark of its owner. Hook0 is independent and is not affiliated with or endorsed by ngrok. This page is a factual comparison for developers evaluating both.' },
    ],
  },
  comparisonSources: {
    competitors: ['ngrok'],
    scope: 'This page compares how each tool receives webhooks on a local machine during development. ngrok figures come from its public pricing page and documentation. Hook0 figures come from the Hook0 CLI and relay source code and the CLI reference.',
    sources: [
      { label: 'ngrok pricing', url: 'https://ngrok.com/pricing', consulted: '2026-10-06' },
      { label: 'ngrok Share Localhost quickstart', url: 'https://ngrok.com/docs/share-localhost/quickstart', consulted: '2026-10-06' },
      { label: 'ngrok verify-webhook action', url: 'https://ngrok.com/docs/traffic-policy/actions/verify-webhook/', consulted: '2026-10-06' },
      { label: 'Hook0 CLI reference', url: 'https://documentation.hook0.com/reference/cli', consulted: '2026-10-06' },
    ],
    updated: '2026-10-06',
  },
  related: {
    h2: 'Related',
    links: [
      { enSlug: 'hook0-vs-webhook-relay', label: 'Hook0 vs Webhook Relay' },
      { enSlug: 'webhook-playground', label: 'Webhook Tester' },
      { enSlug: 'migrate-from-webhook-site', label: 'Webhook.site Alternative' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'self-hosted-webhooks', label: 'Self-Hosted Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Open-Source Webhooks' },
    ],
  },
};
