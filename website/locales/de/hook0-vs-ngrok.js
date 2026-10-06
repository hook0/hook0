// Per-page strings for hook0-vs-ngrok (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// Fakten erfasst am 2026-10-06 (ngrok-Preise und -Doku, Quellcode von Hook0-CLI und -Relay).
// Strukturierende Nuance: Beide Tools überschneiden sich nur bei einem Einsatz,
// Webhooks während der Entwicklung lokal empfangen. ngrok ist ein allgemeiner
// Ingress, `hook0 listen` ist ein Dev-Tunnel im Hook0-CLI.
// Ehrlichkeit: Hook0 nie als allgemeinen Ingress oder Produktionstunnel darstellen.
// ngrok ist einfacher zu installieren (brew, apt, Windows Store), das Hook0-CLI
// läuft heute über Cargo. Das steht so auf der Seite.
module.exports = {
  pageTitle: 'Hook0 vs ngrok für Webhooks: lokaler Tunnel im Vergleich | Hook0',
  pageDescription: "ngrok macht jeden lokalen Dienst erreichbar. Das Hook0-CLI leitet Webhooks ohne Konto an localhost weiter, über ein selbst hostbares Open-Source-Relay.",
  pageModified: '2026-10-06',
  breadcrumb: 'Hook0 vs. ngrok',
  tldr: {
    h2: 'Kurz gesagt',
    body: "ngrok und Hook0 überschneiden sich bei genau einem Einsatz, nämlich Webhooks während der Entwicklung auf deinen Rechner zu holen. ngrok ist ein allgemeiner Ingress, der jeden lokalen HTTP-, TCP- oder TLS-Dienst erreichbar macht, mit eigenen Domains und Traffic Policies in den bezahlten Tarifen. Hook0 ist ausgehendes Webhooks-as-a-Service, und sein CLI bringt hook0 listen mit, einen WebSocket-Tunnel, der Webhooks ohne Konto an localhost weiterleitet. Das Relay dahinter ist Open Source, du kannst dein eigenes betreiben. Nimm ngrok, wenn du mehr als Webhooks erreichbar machen musst. Nimm Hook0, wenn die Webhooks, die du testest, die sind, die dein Produkt später an seine Nutzer sendet.",
  },
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs ngrok',
    titleAccent: 'Webhooks auf localhost empfangen',
    subtitle: 'ngrok bringt jeden lokalen Dienst auf eine öffentliche URL. Das Hook0-CLI macht etwas Engeres. Es leitet Webhooks über ein Relay, das du selbst hosten kannst, an deinen lokalen Server weiter. Diese Seite vergleicht beide für diesen Einsatz und sagt, wann ngrok die bessere Wahl ist.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'CLI-Doku lesen',
  },
  differentiators: {
    eyebrow: 'Was jedes Tool macht',
    h2: 'Die wichtigsten Unterschiede',
    cards: [
      { title: 'Allgemeiner Ingress oder Webhook-Tunnel', body: 'ngrok macht lokale HTTP-, TCP- und TLS-Dienste erreichbar und deckt damit Demos, APIs, SSH und Webhooks gleichermaßen ab. <code>hook0 listen 3000</code> deckt nur Webhooks ab. Es öffnet einen WebSocket zum Relay, gibt eine öffentliche URL aus und leitet jede eingehende Anfrage an deinen lokalen Port weiter.' },
      { title: 'Kein Konto zum Start', body: 'Der ngrok-Agent braucht vor dem ersten Tunnel ein Konto und einen Authtoken. <code>hook0 listen</code> erzeugt lokal ein Token und verbindet sich direkt mit dem öffentlichen Relay, du bekommst also eine URL ohne Registrierung. Mit <code>--token</code> behältst du dieselbe URL über Neustarts hinweg.' },
      { title: 'Ein Relay, das du selbst hostest', body: 'Der Hook0-Relay-Server ist Open Source unter SSPL-1.0 und liegt mit einem Helm-Chart im Hook0-Repository. Richte das CLI mit <code>--relay-url</code> auf deine eigene Instanz, dann bleiben Test-Payloads auf Infrastruktur, die du betreibst. ngrok ist ein gehosteter Dienst.' },
      { title: 'Für die Senderseite gebaut', body: 'Das Kernprodukt von Hook0 stellt Webhooks an deine Nutzer zu, mit Protokoll pro Versuch, kostenlosen Retries, HMAC-Signaturen und Replay. Dasselbe CLI sendet Events, verwaltet Subscriptions und spielt Fehlschläge erneut ab, so testest du die ganze Schleife von <code>hook0 event send</code> bis zu deinem lokalen Handler.' },
    ],
  },
  comparison: {
    eyebrow: 'Im direkten Vergleich',
    h2: 'Webhooks lokal empfangen',
    headers: { feature: 'Kriterium', hook0: 'Hook0-CLI', ngrok: 'ngrok' },
    rows: [
      { feature: 'Hauptrolle', hook0Html: 'Ausgehendes Webhooks-as-a-Service, mit Dev-Tunnel im CLI', ngrokHtml: 'Allgemeiner Ingress und Tunnel' },
      { feature: 'Freigegebene Protokolle', hook0Html: 'HTTP-Webhooks, an eine lokale URL weitergeleitet', ngrokHtml: 'HTTP/S und TCP, TLS im Pay-as-you-go-Tarif' },
      { feature: 'Konto nötig', hook0Html: 'Nein, für <code>hook0 listen</code>', ngrokHtml: 'Ja, Konto und Authtoken' },
      { feature: 'Stabile URL', hook0Html: 'Gleiches Token, gleiche URL (<code>--token</code>)', ngrokHtml: 'Eine Development-Domain in jedem Tarif, eigene Domains ab Pay-as-you-go' },
      { feature: 'Kostenloser Tarif', hook0Html: 'Öffentliches Relay ohne Kosten', ngrokHtml: '1 GB Datentransfer, 20.000 HTTP/S-Anfragen, bis zu 3 Endpoints' },
      { feature: 'Anfragen einsehen', hook0Html: 'Terminal-Dashboard (TUI) mit Headern, Body und Status', ngrokHtml: 'Traffic Inspector, 24 h Aufbewahrung im Free-Tarif' },
      { feature: 'Signaturprüfung', hook0Html: 'HMAC-SHA256-Signatur auf der Senderseite', ngrokHtml: 'Traffic-Policy-Aktion verify-webhook für viele Anbieter' },
      { feature: 'Schutz bei der Weiterleitung', hook0Html: 'Nur localhost, außer mit <code>--allow-external</code>', ngrokHtml: 'Nicht zutreffend (du wählst das Ziel)' },
      { feature: 'Relay selbst hosten', hook0Html: 'Ja, Open Source (SSPL-1.0) mit Helm-Chart', ngrokHtml: 'Nein, gehosteter Dienst' },
      { feature: 'Installation', hook0Html: '<code>cargo install</code> (Rust nötig)', ngrokHtml: 'Homebrew, apt, Windows Store oder direkter Download' },
    ],
  },
  fit: {
    h2: 'Wann ngrok besser passt',
    body: 'Nimm ngrok, wenn du etwas anderes als Webhooks erreichbar machen musst, etwa eine Demo-App, eine API für Kollegen, einen TCP-Dienst oder einen Endpoint hinter einer eigenen Domain mit Traffic Policies. Es lässt sich außerdem auf den meisten Systemen mit einem Befehl installieren, während das Hook0-CLI heute über Cargo installiert wird.',
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Hook0 eine Alternative zu ngrok?', a: 'Für einen Einsatz ja, nämlich Webhooks während der Entwicklung auf localhost zu empfangen. hook0 listen leitet Webhooks über ein WebSocket-Relay an deinen lokalen Server weiter, ohne Konto. Für alles darüber hinaus (TCP-Dienste, Demos, eigene Domains, Traffic Policies) ist ngrok das breitere Tool.' },
      { q: 'Wie empfange ich Webhooks auf localhost mit Hook0?', a: 'Installiere das CLI und starte hook0 listen 3000. Es gibt eine öffentliche URL auf play.hook0.com aus. Trag diese URL beim Anbieter oder in einer Hook0-Subscription ein, dann wird jede Anfrage an localhost:3000 weitergeleitet und im Terminal-Dashboard angezeigt.' },
      { q: 'Brauche ich ein Konto für hook0 listen?', a: 'Nein. Das CLI erzeugt ein Token auf deinem Rechner und verbindet sich mit dem öffentlichen Relay. Ein Hook0-Konto brauchst du nur, um Events über die Hook0-API zu senden, zum Beispiel mit hook0 event send.' },
      { q: 'Kann ich das Hook0-Relay selbst hosten?', a: 'Ja. Der Relay-Server liegt unter SSPL-1.0 mit einem Helm-Chart im Hook0-Repository. Starte deine Instanz und richte das CLI mit --relay-url darauf. Füge --insecure hinzu, wenn sie ein selbstsigniertes Zertifikat nutzt.' },
      { q: 'Kann ich dieselbe Webhook-URL über mehrere Sitzungen behalten?', a: 'Ja. Die URL hängt am Token, also liefert hook0 listen 3000 --token mein-stabiles-token jedes Mal dieselbe URL. Bei ngrok enthält jeder Tarif eine Development-Domain.' },
      { q: 'Ist Hook0 mit ngrok verbunden?', a: 'Nein. ngrok ist eine Marke ihres Inhabers. Hook0 ist unabhängig und weder mit ngrok verbunden noch von ngrok unterstützt. Diese Seite ist ein sachlicher Vergleich für Entwickler, die beide prüfen.' },
    ],
  },
  comparisonSources: {
    competitors: ['ngrok'],
    scope: 'Diese Seite vergleicht, wie beide Tools Webhooks während der Entwicklung auf einem lokalen Rechner empfangen. Die Angaben zu ngrok stammen von seiner öffentlichen Preisseite und Dokumentation. Die Angaben zu Hook0 stammen aus dem Quellcode von CLI und Relay sowie aus der CLI-Referenz.',
    sources: [
      { label: 'ngrok Preise', url: 'https://ngrok.com/pricing', consulted: '2026-10-06' },
      { label: 'ngrok Share-Localhost-Quickstart', url: 'https://ngrok.com/docs/share-localhost/quickstart', consulted: '2026-10-06' },
      { label: 'ngrok Aktion verify-webhook', url: 'https://ngrok.com/docs/traffic-policy/actions/verify-webhook/', consulted: '2026-10-06' },
      { label: 'Hook0-CLI-Referenz', url: 'https://documentation.hook0.com/reference/cli', consulted: '2026-10-06' },
    ],
    updated: '2026-10-06',
  },
  related: {
    h2: 'Verwandte Seiten',
    links: [
      { enSlug: 'hook0-vs-webhook-relay', label: 'Hook0 vs Webhook Relay' },
      { enSlug: 'webhook-playground', label: 'Webhook-Tester' },
      { enSlug: 'migrate-from-webhook-site', label: 'Alternative zu Webhook.site' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst gehostete Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Open-Source-Webhooks' },
    ],
  },
};
