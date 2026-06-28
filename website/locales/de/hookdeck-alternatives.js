// Per-page strings for hookdeck-alternatives (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt, kein --.
// Hook0 = « quelloffen (SSPL-1.0) ». Convoy MPL-2.0 = OSI, also « Open Source » OK für Convoy.
module.exports = {
  pageTitle: 'Hookdeck-Alternativen (2026), Webhook-Plattformen, die mehr können | Hook0',
  pageDescription: 'Hookdeck ist ein Webhook-Proxy, keine Webhook-Plattform. Vergleich der echten Alternativen, Hook0, Svix, Convoy zum Senden, Verwalten und Monitoren von Webhooks.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hookdeck-Alternativen',
    titleAccent: 'Hookdeck ist ein Proxy, du brauchst vielleicht eine Plattform',
    subtitleHtml: 'Hookdeck ist ein Webhook-Gateway, es empfängt und leitet eingehende Webhooks weiter. Wenn du Webhooks an deine Kunden <strong class="text-white">senden</strong> musst (mit Wiederholungen, Signaturen, Abonnenten-Verwaltung), kann Hookdeck das nicht. Diese Alternativen schon.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  gatewayVsPlatform: {
    eyebrow: 'Wichtige Unterscheidung',
    h2: 'Gateway vs Plattform, was ist der Unterschied?',
    sub: 'Wähl die falsche Kategorie und du baust am Ende die fehlende Hälfte selbst.',
    cards: [
      { title: 'Webhook-Gateway (Hookdeck)', bodyHtml: 'Ein Gateway sitzt zwischen einem fremden Webhook-Sender und deiner Anwendung. Es empfängt eingehende Webhooks, puffert sie, wiederholt fehlgeschlagene Zustellungen, leitet Events an den richtigen Endpoint. Im Grunde ein Reverse-Proxy für Webhooks. <strong class="text-white">Du bist der Konsument.</strong>', color: 'indigo' },
      { title: 'Webhook-Plattform (Hook0, Svix, Convoy)', bodyHtml: 'Eine Plattform lässt dich Webhooks an deine Kunden senden. Du publizierst Events, die Plattform liefert sie mit Wiederholungen, HMAC-Signaturen und einem Abonnenten-Verwaltungsportal aus. <strong class="text-white">Du bist der Produzent.</strong> Das brauchst du, um Webhooks zu deinem Produkt hinzuzufügen.', color: 'green' },
    ],
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Hookdeck vs die Alternativen',
    sub: 'Fünf Optionen, eine Tabelle. Was am meisten zählt, ist, ob du Webhooks senden, empfangen oder beides musst.',
    headers: { criteria: 'Kriterium', hookdeck: 'Hookdeck', hook0: 'Hook0', svix: 'Svix', convoy: 'Convoy', awsEventbridge: 'AWS EventBridge' },
    rows: [
      { criteria: 'Typ', hookdeckHtml: 'Webhook-Gateway / Proxy', hook0Html: 'Vollständige Webhook-Plattform', svixHtml: 'Webhook-Plattform (Open Core)', convoyHtml: 'Webhook-Plattform', awsEventbridgeHtml: 'Event-Bus (AWS-Ökosystem)' },
      { criteria: 'Webhooks senden', hookdeckHtml: 'Nein', hook0Html: 'Ja (Kernfunktion)', svixHtml: 'Ja', convoyHtml: 'Ja', awsEventbridgeHtml: 'Ja (via API Destinations)' },
      { criteria: 'Webhooks empfangen', hookdeckHtml: 'Ja (Kernfunktion)', hook0Html: 'Nein (per Design)', svixHtml: 'Nein', convoyHtml: 'Ja (eingehend + ausgehend)', awsEventbridgeHtml: 'Ja (Event-Ingestion)' },
      { criteria: 'Selbst-Hosting', hookdeckHtml: 'Nein', hook0Html: 'Kostenlos (Docker / K8s)', svixHtml: 'Nur Enterprise-Plan', convoyHtml: 'Ja (selbst-verwaltet)', awsEventbridgeHtml: 'Nein (nur AWS)' },
      { criteria: 'Quellcode', hookdeckHtml: 'Nein (Closed Source)', hook0Html: 'Ja (SSPL-1.0, gesamter Quellcode)', svixHtml: 'Teilweise (Open Core, Enterprise geschlossen)', convoyHtml: 'Ja (MPL-2.0)', awsEventbridgeHtml: 'Nein (AWS-proprietär)' },
      { criteria: 'Kostenloser Tarif', hookdeckHtml: 'Ja (100k Events/Monat)', hook0Html: 'Ja, ohne Kreditkarte', svixHtml: 'Ja', convoyHtml: 'Nur Community-Edition', awsEventbridgeHtml: 'Pay-per-use (AWS-Abrechnung)' },
      { criteria: 'Datenhosting', hookdeckHtml: 'In den USA', hook0Html: 'Europa (Clever Cloud FR, CDN Cloudflare USA) oder Selbst-Hosting', svixHtml: 'In den USA', convoyHtml: 'Nur Selbst-Hosting', awsEventbridgeHtml: 'Multi-Region (AWS)' },
      { criteria: 'Finanzierung', hookdeckHtml: '3,5 Mio. $ VC-finanziert', hook0Html: '100% bootstrapped', svixHtml: '17 Mio. $ VC-finanziert', convoyHtml: 'VC-finanziert', awsEventbridgeHtml: 'Amazon (börsennotiert)' },
    ],
  },
  whyLookBeyond: {
    eyebrow: 'Warum über Hookdeck hinausschauen',
    h2: 'Wenn Hookdeck nicht reicht',
    sub: 'Hookdeck macht eine Sache gut, eingehende Webhooks empfangen und routen. Aber es gibt klare Fälle, in denen das nicht reicht.',
    cards: [
      { title: 'Du musst Webhooks senden', body: 'Hookdeck sendet keine Webhooks. Punkt. Wenn dein Produkt Kunden via Webhooks mit Wiederholungen, HMAC-Signaturen und Zustelllogs benachrichtigen muss, brauchst du eine Webhook-Plattform, Hook0, Svix oder Convoy.', color: 'green' },
      { title: 'Du willst selbst hosten', body: 'Hookdeck ist nur Cloud. Es gibt keine Selbst-Hosting-Option. Wenn Compliance- oder Datenresidenz-Regeln eigene Infrastruktur erzwingen, sind Hook0 und Convoy beide kostenlos selbst-hostbar.', color: 'indigo' },
      { title: 'Du brauchst europäisches Datenhosting', body: 'Hookdeck sitzt in den USA. Hook0 Cloud wird in Europa gehostet, auf DSGVO-Konformität ausgelegt. Wenn du ein EU-Unternehmen bist, das sensible Daten verarbeitet, ist die Wahl unkompliziert.', color: 'green' },
      { title: 'Du willst den Quellcode auditieren', body: 'Hookdeck ist Closed Source. Du kannst nicht sehen, wie deine Webhook-Daten verarbeitet werden. Hook0\'s gesamter Codebase ist quelloffen unter SSPL-1.0, du kannst jede Zeile lesen und auditieren.', color: 'indigo' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Hookdeck quelloffen?', a: 'Nein. Hookdeck ist Closed Source und nur Cloud. Du kannst den Code nicht inspizieren, auditieren oder selbst hosten. Wenn dir quelloffen wichtig ist, sind Alternativen wie Hook0 (SSPL-1.0) oder Convoy (MPL-2.0) vollständig quelloffen.' },
      { q: 'Kann ich Hookdeck selbst hosten?', a: 'Nein. Hookdeck bietet keine Selbst-Hosting-Option. Es ist nur Cloud. Wenn du deine Webhook-Infrastruktur auf eigenen Servern betreiben musst aus Compliance-, Datenresidenz- oder Kostengründen, unterstützen Hook0 und Convoy beide Selbst-Hosting.' },
      { q: 'Was ist der Unterschied zwischen einem Webhook-Proxy und einer Webhook-Plattform?', a: 'Ein Webhook-Proxy (wie Hookdeck) sitzt zwischen einem Webhook-Sender und deiner Anwendung. Er empfängt, routet und wiederholt eingehende Webhooks. Eine Webhook-Plattform (wie Hook0 oder Svix) lässt dich Webhooks an deine Kunden senden. Sie kümmert sich um Zustellung, Wiederholungen, Signaturen und Abonnenten-Verwaltung. Wenn du Webhooks zu deinem Produkt hinzufügen willst, brauchst du eine Plattform, keinen Proxy.' },
      { q: 'Was ist die beste Hookdeck-Alternative zum Senden von Webhooks?', a: 'Hook0, wenn du Webhooks senden musst. Du publizierst Events, Hook0 liefert sie an deine Abonnenten mit Wiederholungen, HMAC-Signaturen und einem Verwaltungs-Dashboard. Der Code ist quelloffen (SSPL-1.0), du kannst ihn selbst hosten, das Unternehmen ist bootstrapped und die Cloud läuft in Europa.' },
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
