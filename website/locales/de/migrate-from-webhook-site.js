// Per-page strings for migrate-from-webhook-site (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// SSPL für Hook0 = « quelloffen (SSPL-1.0) », NIE « Open Source » allein.
module.exports = {
  pageTitle: 'webhook.site-Alternative: in 30 Min zu Hook0 | Hook0',
  pageDescription: 'Production-Upgrade von webhook.site: HMAC-Signaturen, konfigurierbare Wiederholungen, Subscriber-Portal, quelloffen (SSPL-1.0).',
  pageModified: '2026-06-27',
  breadcrumb: 'Von webhook.site migrieren',
  hero: {
    eyebrow: 'webhook.site-Alternative',
    titleBefore: 'Über webhook.site hinaus?',
    titleAccent: 'Wechsel zu Hook0',
    subtitle: 'webhook.site fängt eingehendes HTTP zum Debuggen ab. Hook0 sendet deine Webhooks an deine Kunden, mit HMAC-Signaturen, Wiederholungen, Zustelllogs und Abonnenten-Portal. Anderer Job, gleiche Domäne. Quelloffen (SSPL-1.0).',
    ctaPrimary: 'Zu Hook0 wechseln',
    ctaSecondary: 'Playground ausprobieren',
    ctaNote: '100 Events/Tag kostenlos. Ohne Kreditkarte. Quelloffen.',
  },
  vsTable: {
    eyebrow: 'Zwei verwandte Werkzeuge',
    h2: 'Eingehender Inspektor vs ausgehende Plattform',
    sub: 'webhook.site empfängt. Hook0 sendet. Die richtige Wahl zu Beginn erspart dir später ein Refactoring.',
    headers: { need: 'Bedarf', webhookSite: 'webhook.site', hook0: 'Hook0' },
    rows: [
      { need: 'Eingehende Anfragen zum Debuggen prüfen', webhookSite: 'Ja', webhookSitePositive: true, hook0: 'Ja (play.hook0.com)', hook0Positive: true },
      { need: 'Webhooks an Kunden in Produktion senden', webhookSite: 'Nein', webhookSitePositive: false, hook0: 'Ja', hook0Positive: true },
      { need: 'Jeden Payload mit HMAC signieren', webhookSite: 'Nein', webhookSitePositive: false, hook0: 'Ja', hook0Positive: true },
      { need: 'Wiederholungen und Dead Letter Queues', webhookSite: 'Nein', webhookSitePositive: false, hook0: 'Ja', hook0Positive: true },
      { need: 'Abonnenten-Portal für deine Kunden', webhookSite: 'Nein', webhookSitePositive: false, hook0: 'Ja', hook0Positive: true },
      { need: 'Selbst-Hosting auf deiner Infrastruktur', webhookSite: 'Nein', webhookSitePositive: false, hook0: 'Kostenlos (SSPL-1.0)', hook0Positive: true },
      { need: 'Kostenloser Tarif', webhookSite: 'Ja', webhookSitePositive: true, hook0: 'Ja', hook0Positive: true },
    ],
  },
  migration: {
    eyebrow: 'Migration',
    h2: 'Von webhook.site zur Produktion in 30 Minuten',
    steps: [
      { index: 'Schritt 1', title: 'Anwendung anlegen', body: 'Registriere dich, lege eine Anwendung an. Du bekommst sofort ein Auth-Token und eine Application-ID. Ohne Kreditkarte.' },
      { index: 'Schritt 2', title: 'URL austauschen', body: 'Ersetze die webhook.site-URL durch einen Hook0-API-Aufruf. Python- oder Node.js-SDK, oder einfaches HTTP.' },
      { index: 'Schritt 3', title: 'Übergib das Portal', body: 'Stell das Abonnenten-Portal vor deine Kunden. Sie tragen ihre eigenen Endpoints ein, rotieren ihre eigenen Schlüssel, lesen ihre eigenen Zustelllogs.' },
    ],
    codeBlock: '// Vorher. webhook.site als Debug-Empfänger:\nfetch("https://webhook.site/abcd-1234", {\n  method: "POST",\n  body: JSON.stringify(payload)\n});\n\n// Nachher. Hook0 als Produktions-Webhook-Plattform:\nawait hook0.message.create("&lt;application_id&gt;", {\n  event_type: "invoice.paid",\n  event_id:   "evt_Wqb1k73rXprtTm7Qdlr38G",\n  payload\n});\n\n// Was sich für deine Abonnenten ändert: signierte Payloads,\n// automatische Wiederholungen, Zustelllogs, die sie selbst replayen können.\n',
    docsLink: 'Zum Getting-Started-Guide →',
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Migrationsfragen',
    items: [
      { q: 'Ist Hook0 eine Alternative zu webhook.site?', a: 'Ja. Hook0 ist die produktionsreife Alternative, wenn du webhook.site entwachsen bist. Während webhook.site ein Request-Inspektor ist (« welchen Payload habe ich erhalten? »), ist Hook0 eine Webhook-Plattform: sie sendet Events an deine Abonnenten, signiert sie mit HMAC, wiederholt bei Fehlern und speichert Zustelllogs. webhook.site nutzt du zum Debuggen, Hook0 in der Produktion.' },
      { q: 'Wie migriere ich von webhook.site zu Hook0?', a: 'Registriere dich bei Hook0 (kostenlos, ohne Kreditkarte), lege eine Anwendung an, ersetze die webhook.site-URL in deinem Code durch einen einzigen Hook0-REST-API-Aufruf. Du erhältst HMAC-signierte Zustellung, Wiederholungen, Dead Letter Queues und Abonnenten-Portal, ohne Code-Änderung über den API-Endpunkt hinaus.' },
      { q: 'Kann ich rohe Webhook-Payloads weiterhin mit Hook0 prüfen?', a: 'Ja. Jedes über Hook0 gesendete Event wird mit vollständigem Request, Response, Statuscode und Latenz geloggt. Du kannst jedes Event aus dem Dashboard replayen. Für Ad-hoc-Tests ohne Account erzeugst du auf play.hook0.com Wegwerf-Webhook-URLs wie bei webhook.site.' },
      { q: 'Ist Hook0 quelloffen, anders als webhook.site?', a: 'Ja. Hook0 ist vollständig quelloffen unter SSPL-1.0 und selbst-hostbar auf Docker Compose oder Kubernetes. webhook.site ist ein Closed-Source-SaaS. Wenn du den Traffic auf deiner eigenen Infrastruktur halten musst, ist Hook0 die Antwort.' },
    ],
  },
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'webhook-platform', label: 'Webhook-Plattform' },
      { enSlug: 'webhook-api', label: 'Webhook-API' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
    ],
  },
};
