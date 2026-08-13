// Per-page strings for hookdeck-alternatives (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt, kein --.
// Hook0 = « quelloffen (SSPL-1.0) ». Convoy = Elastic License 2.0, source-available, nicht OSI.
// Hookdeck: Outpost (Zustellung) unter Apache-2.0 und selbst-hostbar, Event Gateway (Ingestion) geschlossen.
module.exports = {
  pageTitle: 'Hookdeck-Alternativen 2026, Webhook-Plattformen | Hook0',
  pageDescription: 'Hookdeck trennt eingehendes Gateway und ausgehende Zustellung in zwei Produkte. Alternativen: Hook0, Svix, Convoy zum Senden und Monitoren von Webhooks.',
  pageModified: '2026-07-16',
  breadcrumb: 'Hookdeck-Alternativen',
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hookdeck-Alternativen',
    titleAccent: 'Hookdeck sind zwei Produkte, du brauchst vielleicht nur eins',
    subtitleHtml: 'Hookdecks Event Gateway empfängt und leitet eingehende Webhooks weiter, das Senden läuft über ein zweites Produkt, Outpost. Wenn du eine einzige Plattform willst, um Webhooks an deine Kunden zu <strong class="text-white">senden</strong> (mit Wiederholungen, Signaturen, Abonnenten-Verwaltung), mit demselben Code in der Managed Cloud und selbst gehostet, dann können diese Alternativen das.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  gatewayVsPlatform: {
    eyebrow: 'Wichtige Unterscheidung',
    h2: 'Gateway vs Plattform, was ist der Unterschied?',
    sub: 'Wähl die falsche Kategorie und du baust am Ende die fehlende Hälfte selbst.',
    cards: [
      { title: 'Webhook-Gateway (Hookdeck Event Gateway)', bodyHtml: 'Ein Gateway sitzt zwischen einem fremden Webhook-Sender und deiner Anwendung. Es empfängt eingehende Webhooks, puffert sie, wiederholt fehlgeschlagene Zustellungen, leitet Events an den richtigen Endpoint. Im Grunde ein Reverse-Proxy für Webhooks. <strong class="text-white">Du bist der Konsument.</strong>', color: 'indigo' },
      { title: 'Webhook-Plattform (Hook0, Svix, Convoy)', bodyHtml: 'Eine Plattform lässt dich Webhooks an deine Kunden senden. Du publizierst Events, die Plattform liefert sie mit Wiederholungen, HMAC-Signaturen und einem Abonnenten-Verwaltungsportal aus. <strong class="text-white">Du bist der Produzent.</strong> Das brauchst du, um Webhooks zu deinem Produkt hinzuzufügen.', color: 'green' },
    ],
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Hookdeck vs die Alternativen',
    sub: 'Fünf Optionen, eine Tabelle. Was am meisten zählt, ist, ob du Webhooks senden, empfangen oder beides musst.',
    headers: { criteria: 'Kriterium', hookdeck: 'Hookdeck', hook0: 'Hook0', svix: 'Svix', convoy: 'Convoy', awsEventbridge: 'AWS EventBridge' },
    rows: [
      { criteria: 'Typ', hookdeckHtml: 'Event Gateway (eingehend) + Outpost (ausgehend)', hook0Html: 'Vollständige Webhook-Plattform', svixHtml: 'Webhook-Plattform (Open Core)', convoyHtml: 'Webhook-Plattform', awsEventbridgeHtml: 'Event-Bus (AWS-Ökosystem)' },
      { criteria: 'Webhooks senden', hookdeckHtml: 'Ja (Outpost, separates Produkt)', hook0Html: 'Ja (Kernfunktion)', svixHtml: 'Ja', convoyHtml: 'Ja', awsEventbridgeHtml: 'Ja (via API Destinations)' },
      { criteria: 'Webhooks empfangen', hookdeckHtml: 'Ja (Kernfunktion)', hook0Html: 'Nein (per Design)', svixHtml: 'Nein', convoyHtml: 'Ja (eingehend + ausgehend)', awsEventbridgeHtml: 'Ja (Event-Ingestion)' },
      { criteria: 'Selbst-Hosting', hookdeckHtml: 'Nur Outpost (Apache-2.0)', hook0Html: 'Kostenlos (Docker / K8s)', svixHtml: 'Nur Enterprise-Plan', convoyHtml: 'Ja (selbst-verwaltet)', awsEventbridgeHtml: 'Nein (nur AWS)' },
      { criteria: 'Quellcode', hookdeckHtml: 'Teilweise (Outpost Apache-2.0, Gateway geschlossen)', hook0Html: 'Ja (SSPL-1.0, gesamter Quellcode)', svixHtml: 'Teilweise (Open Core, Enterprise geschlossen)', convoyHtml: 'Source-available (Elastic License 2.0)', awsEventbridgeHtml: 'Nein (AWS-proprietär)' },
      { criteria: 'Kostenloser Tarif', hookdeckHtml: 'Ja (100k Events/Monat)', hook0Html: 'Ja, ohne Kreditkarte', svixHtml: 'Ja', convoyHtml: 'Nur Community-Edition', awsEventbridgeHtml: 'Pay-per-use (AWS-Abrechnung)' },
      { criteria: 'Datenhosting', hookdeckHtml: 'In Kanada, EU-Region verfügbar', hook0Html: 'Europa (Clever Cloud FR, CDN Cloudflare USA) oder Selbst-Hosting', svixHtml: 'In den USA', convoyHtml: 'Nur Selbst-Hosting', awsEventbridgeHtml: 'Multi-Region (AWS)' },
      { criteria: 'Finanzierung', hookdeckHtml: '3,5 Mio. $ VC-finanziert', hook0Html: '100% bootstrappt', svixHtml: '17 Mio. $ VC-finanziert', convoyHtml: 'VC-finanziert', awsEventbridgeHtml: 'Amazon (börsennotiert)' },
    ],
  },
  whyLookBeyond: {
    eyebrow: 'Warum über Hookdeck hinausschauen',
    h2: 'Wenn Hookdeck nicht reicht',
    sub: 'Hookdeck macht eine Sache gut, eingehende Webhooks empfangen und routen. Aber es gibt klare Fälle, in denen das nicht reicht.',
    cards: [
      { title: 'Du musst Webhooks senden', body: 'Hookdecks Event Gateway sendet keine Webhooks. Das übernimmt Outpost, ein separates Produkt mit eigenen Tarifen. Wenn du eine einzige Plattform willst, die deine Events publiziert und mit Wiederholungen, HMAC-Signaturen und Zustelllogs ausliefert, schau dir Hook0, Svix oder Convoy an.', color: 'green' },
      { title: 'Du willst selbst hosten', body: 'Hookdeck öffnet nur seine Outpost-Komponente, veröffentlicht unter Apache-2.0 und selbst-hostbar. Das Event Gateway bleibt Closed Source und nur Cloud. Hook0 und Convoy lassen sich beide vollständig ohne Lizenzkosten selbst hosten, und bei Hook0 ist der selbst gehostete Code derselbe, der in der Managed Cloud läuft.', color: 'indigo' },
      { title: 'Du brauchst europäisches Datenhosting', body: 'Hookdeck sitzt in Kanada, mit einer EU-Region bei Outpost managed. Die Datenebene von Hook0 Cloud läuft in Frankreich bei Clever Cloud, auf DSGVO-Konformität ausgelegt (CDN Cloudflare USA im <a href="/de/auftragsverarbeitungsvertrag">DPA</a> offengelegt). Wenn du ein EU-Unternehmen bist, das sensible Daten verarbeitet, ist die Wahl unkompliziert.', color: 'green' },
      { title: 'Du willst den Quellcode auditieren', body: 'Outpost steht unter Apache-2.0, Hookdecks Zustellungscode ist also lesbar. Das Event Gateway, das deine Webhooks entgegennimmt, bleibt aber geschlossen. Hook0\'s gesamter Codebase ist quelloffen unter SSPL-1.0 und die Managed Cloud läuft auf genau diesem Code, du kannst jede Zeile lesen und auditieren.', color: 'indigo' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Hookdeck quelloffen?', a: 'Teilweise. Outpost, Hookdecks Zustellungskomponente, steht unter Apache-2.0, ist selbst-hostbar und das Repository ist aktiv. Das Event Gateway, das Webhooks entgegennimmt, bleibt Closed Source und nur Cloud. Hook0 ist unter SSPL-1.0 vollständig quelloffen, die gesamte Plattform ist selbst-hostbar. Convoy steht unter der Elastic License 2.0, source-available wie Hook0s SSPL-1.0, und diese Lizenz verbietet es, Convoy als Managed Service anzubieten.' },
      { q: 'Kann ich Hookdeck selbst hosten?', a: 'Teilweise. Outpost steht unter Apache-2.0 und läuft auf deiner eigenen Infrastruktur, das Event Gateway hat keine Selbst-Hosting-Option. Beachte auch: Bei Outpost managed sind SSO, RBAC und SCIM nicht im Starter-Tarif zu 10 $/Mio. enthalten, sie starten im Growth-Tarif ab 499 $/Monat zusätzlich zu den Event-Kosten. Wenn du deine gesamte Webhook-Infrastruktur auf eigenen Servern betreiben musst, lassen sich Hook0 und Convoy durchgängig selbst hosten, und Hook0 betreibt denselben Code in seiner Managed Cloud, ohne Funktionen hinter einem Enterprise-Tarif.' },
      { q: 'Was ist der Unterschied zwischen einem Webhook-Proxy und einer Webhook-Plattform?', a: 'Ein Webhook-Proxy (wie Hookdecks Event Gateway) sitzt zwischen einem Webhook-Sender und deiner Anwendung. Er empfängt, routet und wiederholt eingehende Webhooks. Eine Webhook-Plattform (wie Hook0 oder Svix) lässt dich Webhooks an deine Kunden senden. Sie kümmert sich um Zustellung, Wiederholungen, Signaturen und Abonnenten-Verwaltung. Wenn du Webhooks zu deinem Produkt hinzufügen willst, brauchst du eine Plattform, keinen Proxy.' },
      { q: 'Was ist die beste Hookdeck-Alternative zum Senden von Webhooks?', a: 'Hook0, wenn du Webhooks senden musst. Du publizierst Events, Hook0 liefert sie an deine Abonnenten mit Wiederholungen, HMAC-Signaturen und einem Verwaltungs-Dashboard. Der Code ist quelloffen (SSPL-1.0), du kannst ihn selbst hosten, das Unternehmen ist bootstrappt und die Cloud läuft in Europa.' },
      { q: "Welche Hookdeck-Alternative ist EU-gehostet und quelloffen?", a: "Hook0. Hookdeck öffnet nur Outpost, seine Zustellungskomponente, unter Apache-2.0; das Event Gateway bleibt geschlossen und rein cloudbasiert. Hook0 betreibt seine Datenebene auf Clever Cloud in Frankreich (innerhalb der EU), ist quelloffen (SSPL-1.0) und läuft selbst gehostet auf Docker oder Kubernetes, sodass Sie den Code lesen oder Webhook-Daten in Ihrem eigenen Netzwerk behalten können. Das vorgelagerte CDN von Hook0 Cloud ist Cloudflare (US), offengelegt in der öffentlichen Unterauftragsverarbeiter-Liste." },
    ],
  },
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Hook0-Alternativen' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst-gehostete Webhooks' },
    ],
  },
};
