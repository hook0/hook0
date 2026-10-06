// Per-page strings for hook0-vs-webhook-relay (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// Fakten erfasst am 2026-10-06 (Startseite und Preise von webhookrelay.com,
// Companies House, Quellcode von Hook0-CLI und -Relay).
// Strukturierende Nuance: entgegengesetzte Richtungen. Webhook Relay empfängt
// Webhooks von Dritten und routet sie (inbound), Hook0 sendet deine an deine
// Nutzer (outbound). Überschneidung nur bei der lokalen Entwicklung (`hook0 listen`).
// Ehrlichkeit: Webhook Relay NENNT eine selbst gehostete Option (über den
// Vertrieb), nie schreiben, es sei nur gehostet. Nicht belegte Attribute stehen
// als « Nicht angegeben ».
module.exports = {
  pageTitle: 'Hook0 vs Webhook Relay: Webhooks senden oder empfangen | Hook0',
  pageDescription: "Webhook Relay empfängt Webhooks von Dritten und leitet sie an localhost oder private Server weiter. Hook0 sendet Webhooks an deine Nutzer, EU-gehostet.",
  pageModified: '2026-10-06',
  breadcrumb: 'Hook0 vs. Webhook Relay',
  tldr: {
    h2: 'Kurz gesagt',
    body: 'Webhook Relay und Hook0 bedienen entgegengesetzte Enden der Webhook-Leitung. Webhook Relay ist ein eingehendes Gateway, das Webhooks von Diensten wie GitHub oder Stripe empfängt, filtert, umformt und an localhost, Server hinter einer Firewall, Kubernetes oder Cloud-Queues weiterleitet. Hook0 ist ausgehendes Webhooks-as-a-Service. Es sendet die Events deines Produkts an die Endpoints deiner Nutzer, mit Protokoll pro Versuch, kostenlosen Retries, HMAC-Signaturen und Replay. Beide treffen sich bei der lokalen Entwicklung, wo auch das Hook0-CLI Webhooks an localhost weiterleitet. Wenn du Webhooks empfängst, nimm Webhook Relay. Wenn du sie sendest, nimm Hook0.',
  },
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Webhook Relay',
    titleAccent: 'Senden oder empfangen',
    subtitle: 'Webhook Relay bringt Webhooks von Dritten in Netze ohne öffentliche IP. Hook0 stellt deine eigenen Events an die Endpoints deiner Kunden zu. Diese Seite zeigt, welche Richtung jedes Tool abdeckt, wo sie sich überschneiden und wann Webhook Relay die richtige Wahl ist.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  differentiators: {
    eyebrow: 'Was jedes Tool macht',
    h2: 'Die wichtigsten Unterschiede',
    cards: [
      { title: 'Eingehendes Gateway oder ausgehende Zustellung', body: 'Webhook Relay steht vor deinen Systemen, nimmt die Webhooks an, die GitHub, Stripe oder Shopify dir schicken, und routet sie dorthin, wo du sie brauchst. Hook0 steht hinter deinem Produkt und veröffentlicht dessen Events an jeden Abonnenten, die Zustellung, die es verwaltet, geht also <em>hinaus</em> zu deinen Nutzern.' },
      { title: 'Beide erreichen localhost', body: 'Webhook Relay leitet über seinen Agenten und seine Tunnel an localhost und an Server ohne öffentliche IP weiter. Das Hook0-CLI deckt den Entwicklungsfall mit <code>hook0 listen 3000</code> ab, einem WebSocket-Tunnel zu deinem lokalen Port, der kein Konto braucht und über ein Relay läuft, das du selbst hosten kannst.' },
      { title: 'EU-Anbieter, EU-Cloud als Standard', body: 'Hook0 wird von einem französischen Unternehmen ohne US-Muttergesellschaft, US-Tochtergesellschaft oder US-Niederlassung herausgegeben, und seine Managed Cloud läuft standardmäßig in der EU. Webhook Relay wird von AppScension Ltd herausgegeben, einem britischen Unternehmen, und gibt an, auf Google Cloud zu laufen. Seine öffentlichen Seiten nennen keine Datenregion.' },
      { title: 'Open Source, heute nutzbar', body: 'Server und Relay von Hook0 sind Open Source unter SSPL-1.0 (quellverfügbar, nicht OSI-geprüft), die Client-SDKs stehen unter MIT. Du kannst den ganzen Stack selbst hosten, ohne jemanden zu fragen. Webhook Relay nennt eine selbst gehostete Option über seinen Vertrieb.' },
    ],
  },
  comparison: {
    eyebrow: 'Im direkten Vergleich',
    h2: 'Was jedes Tool abdeckt',
    headers: { feature: 'Kriterium', hook0: 'Hook0', relay: 'Webhook Relay' },
    rows: [
      { feature: 'Hauptrolle', hook0Html: 'Ausgehendes Webhooks-as-a-Service', relayHtml: 'Eingehendes Webhook-Gateway und Tunnel' },
      { feature: 'Webhook-Richtung', hook0Html: 'Ausgehend (du sendest an deine Nutzer)', relayHtml: 'Eingehend (du empfängst von Dritten)' },
      { feature: 'Ziele', hook0Html: 'Die HTTP-Endpoints deiner Abonnenten', relayHtml: 'Öffentliche URLs, private Server, localhost, Kubernetes, Dienste von AWS, GCP und Azure' },
      { feature: 'Lokale Entwicklung', hook0Html: 'Tunnel <code>hook0 listen</code>, ohne Konto', relayHtml: 'Tunnel ab dem Basic-Tarif' },
      { feature: 'Transformationen', hook0Html: 'Nicht angeboten (Payloads gehen so raus, wie sie veröffentlicht wurden)', relayHtml: 'JavaScript- und Lua-Funktionen, Filter' },
      { feature: 'Signaturen', hook0Html: 'HMAC-SHA256 bei jeder Zustellung', relayHtml: 'Nicht angegeben' },
      { feature: 'Abonnentenportal', hook0Html: 'Einbettbar', relayHtml: 'Nicht zutreffend (kein ausgehender Sender)' },
      { feature: 'Kostenloser Tarif', hook0Html: '100 Events pro Tag', relayHtml: '150 Webhooks pro Monat, keine Tunnel' },
      { feature: 'Anbieter', hook0Html: 'Französisches Unternehmen, ohne US-Gesellschaft', relayHtml: 'AppScension Ltd (Vereinigtes Königreich)' },
      { feature: 'Hosting', hook0Html: 'Managed EU-Cloud (Clever Cloud FR) + Selbsthosting', relayHtml: 'Google Cloud (Region nicht angegeben) + selbst gehostete Option über den Vertrieb' },
      { feature: 'Quellcode', hook0Html: 'Server und Relay SSPL-1.0, SDKs MIT', relayHtml: 'Nicht angegeben' },
    ],
  },
  fit: {
    h2: 'Wann Webhook Relay besser passt',
    body: 'Nimm Webhook Relay, wenn dein Problem auf der Empfängerseite liegt. Typische Fälle sind GitHub-, Stripe- oder Shopify-Webhooks auf einen CI-Server oder einen Kubernetes-Dienst ohne öffentliche IP zu bringen, Payloads vor dem Eintreffen mit einer Funktion umzuformen oder einen eingehenden Webhook auf mehrere interne Ziele und Cloud-Queues zu verteilen. Hook0 formt eingehenden Traffic nicht um, und sein Tunnel ist ein Entwicklungswerkzeug.',
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Hook0 eine Alternative zu Webhook Relay?', a: 'Nur dort, wo sie sich überschneiden. Beide können Webhooks während der Entwicklung an localhost weiterleiten. Darüber hinaus arbeiten sie in entgegengesetzte Richtungen. Webhook Relay empfängt und routet die Webhooks, die du von Dritten bekommst, Hook0 stellt die Webhooks zu, die dein Produkt an seine Nutzer sendet.' },
      { q: 'Kann Hook0 Webhooks an localhost weiterleiten?', a: 'Ja, für die Entwicklung. Starte hook0 listen 3000, dann gibt das CLI eine öffentliche URL auf play.hook0.com aus. Jede Anfrage an diese URL wird über einen WebSocket-Tunnel an localhost:3000 weitergeleitet. Ein Konto ist nicht nötig, und mit --relay-url kannst du dein eigenes Relay betreiben.' },
      { q: 'Formt oder routet Hook0 eingehende Webhooks?', a: 'Nein. Hook0 ist ausgehendes Webhooks-as-a-Service. Es signiert, stellt zu, wiederholt und protokolliert die Events, die du veröffentlichst. Webhooks von Dritten zu filtern, umzuformen und zu routen ist das, wofür Webhook Relay gebaut ist.' },
      { q: 'Wo werden meine Daten gehostet?', a: 'Hook0 Cloud läuft standardmäßig in der EU, und Hook0 wird von einem französischen Unternehmen ohne US-Gesellschaft herausgegeben. Webhook Relay gibt an, auf Google Cloud zu laufen, und nennt auf seinen öffentlichen Seiten keine Region. Beide lassen sich selbst hosten, Hook0 aus seinem Open-Source-Code und Webhook Relay über seinen Vertrieb.' },
      { q: 'Was soll ich wählen, Hook0 oder Webhook Relay?', a: 'Wähle nach Richtung. Wenn du Webhooks mit Signaturen, Protokoll pro Versuch, kostenlosen Retries und Abonnentenportal an deine eigenen Kunden sendest, nimm Hook0. Wenn du Webhooks von Drittdiensten empfängst und in private Netze, CI oder Cloud-Queues routest, nimm Webhook Relay.' },
      { q: 'Ist Hook0 mit Webhook Relay verbunden?', a: 'Nein. Webhook Relay ist eine Marke ihres Inhabers. Hook0 ist unabhängig und weder mit Webhook Relay verbunden noch von Webhook Relay unterstützt. Diese Seite ist ein sachlicher Vergleich für Teams, die beide prüfen.' },
    ],
  },
  comparisonSources: {
    competitors: ['Webhook Relay'],
    scope: 'Diese Seite vergleicht Rolle, Ziele, Hosting und öffentliche Tarife von Hook0 und Webhook Relay. Preise und Kontingente sind die veröffentlichten Angaben der Anbieter. Die Firmenangaben stammen aus dem britischen Register Companies House.',
    sources: [
      { label: 'Webhook Relay Startseite', url: 'https://webhookrelay.com/', consulted: '2026-10-06' },
      { label: 'Webhook Relay Preise', url: 'https://webhookrelay.com/pricing/', consulted: '2026-10-06' },
      { label: 'AppScension Ltd bei Companies House', url: 'https://find-and-update.company-information.service.gov.uk/company/10705698', consulted: '2026-10-06' },
    ],
    updated: '2026-10-06',
  },
  related: {
    h2: 'Verwandte Seiten',
    links: [
      { enSlug: 'hook0-vs-ngrok', label: 'Hook0 vs ngrok' },
      { enSlug: 'hook0-vs-requeue', label: 'Hook0 vs Requeue' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'webhook-playground', label: 'Webhook-Tester' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU-Webhook-Infrastruktur' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst gehostete Webhooks' },
      { enSlug: 'hook0-alternatives', label: 'Hook0-Alternativen' },
    ],
  },
};
