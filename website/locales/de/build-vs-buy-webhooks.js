// Per-page strings for build-vs-buy-webhooks (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// Hook0 = « quelloffen (SSPL-1.0) ».
module.exports = {
  pageTitle: 'Build vs Buy Webhooks: Produktion in 30 Min | Hook0',
  pageDescription: 'Webhooks von Grund auf zu bauen kostet 3+ Sprints. Wiederholungen, Signaturen, Monitoring: nimm Hook0 und liefere in 30 Minuten.',
  pageModified: '2026-06-27',
  breadcrumb: 'Build vs Buy: Webhooks',
  hero: {
    eyebrow: 'Build vs Buy',
    titleBefore: 'Hör auf, Webhooks',
    titleAccent: 'von Grund auf zu bauen',
    subtitle: 'Dein Backlog ist voll mit Features, die deine User wirklich wollen. Wiederholungen, Signaturen, Monitoring, Dead Letter Queues, das sind 3 Sprints oder mehr an Klempnerei. Hook0 ist ein quelloffener (SSPL-1.0) Webhook-Service, der alles davon erledigt. 100 Events/Tag kostenlos, ohne Kreditkarte. Lieferung in 30 Minuten.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
    stats: [
      { value: '3+', label: 'Sprints für eigene Lösung', color: 'green' },
      { value: '30 Min', label: 'Für Hook0-Integration', color: 'indigo' },
      { value: '0 €', label: 'Für den Start (kostenloser Tarif)', color: 'green' },
    ],
  },
  hiddenCosts: {
    eyebrow: 'Die wahren Kosten',
    h2: 'Was du wirklich bauen musst',
    sub: 'Ein HTTP-POST senden ist einfach. Ein produktionsreifes Webhook-System zu bauen ist es nicht.',
    cards: [
      { title: 'Wiederholungslogik', body: 'Zweiphasige Zeitpläne, Jitter, maximale Versuche, Konfiguration pro Abonnement. Du wirst hier Bugs ausliefern. Jeder tut es.' },
      { title: 'Dead Letter Queues', body: 'Was passiert, wenn die Wiederholungen erschöpft sind? Du brauchst DLQ-Speicher, Alerting und manuelles Replay-Werkzeug.' },
      { title: 'HMAC-Signaturen', body: 'Kryptographische Signatur, Schlüsselrotation, Timestamp-Validierung, Schutz vor Replay-Angriffen. Mach einen einzigen Fehler und die Daten deiner Kunden leaken.' },
      { title: 'Zustell-Monitoring', body: 'Dashboards, Zustelllogs, Erfolgs- und Fehlerraten, Latenz-Tracking. Dein erster Kunde wird am ersten Tag fragen « ist mein Webhook angekommen? ».' },
      { title: 'Abonnenten-Verwaltung', body: 'Endpoint-Registrierung, Filterung nach Event-Typ, URL-Validierung, Multi-Abonnement-Support. Allein das ist ein Monat Arbeit, wenn du es richtig machst.' },
      { title: 'Laufende Wartung', body: 'DB-Migrationen, Skalierung, On-Call-Rotationen, Security-Patches. Sechs Monate nach dem Launch wird dafür immer noch jemand um 3 Uhr morgens geweckt.' },
    ],
  },
  comparison: {
    eyebrow: 'Vergleich',
    h2: 'Selbst bauen vs Hook0 nutzen',
    headers: { aspect: 'Aspekt', diy: 'Selbst bauen', hook0: 'Hook0' },
    rows: [
      { aspect: 'Time-to-Production', diyHtml: '3 Sprints oder mehr (6-12 Wochen)', hook0Html: '30 Minuten', diyDim: false },
      { aspect: 'Engineering-Kosten', diyHtml: '2-3 FTE für Monate', hook0Html: 'Ein Entwickler, ein Nachmittag', diyDim: false },
      { aspect: 'Laufende Wartung', diyHtml: 'Kontinuierlich (Bugs, Skalierung, Patches)', hook0Html: 'Von Hook0 verwaltet', diyDim: false },
      { aspect: 'Wiederholungslogik', diyHtml: 'Von Grund auf bauen', hook0Html: 'Eingebaut mit konfigurierbaren 2-phasigen Wiederholungen (schnell + langsam), pro Abonnement anpassbar', diyDim: false },
      { aspect: 'Sicherheit (HMAC)', diyHtml: 'Implementieren und warten', hook0Html: 'Automatisch bei jedem Event', diyDim: false },
      { aspect: 'Monitoring und Logs', diyHtml: 'Dashboards bauen', hook0Html: 'Standardmäßig enthalten', diyDim: false },
      { aspect: 'Abonnement-Verwaltung', diyHtml: 'Komplettes UI bauen', hook0Html: 'Einbettbares Portal enthalten', diyDim: false },
      { aspect: 'Anbieter-Bindung', diyHtml: 'Keine (aber an deinen Code gebunden)', hook0Html: 'Keine (quelloffen, selbst-hostbar)', diyDim: true },
    ],
  },
  integration: {
    eyebrow: 'Integration',
    h2: 'Webhooks in 30 Minuten ausliefern',
    sub: 'Ein API-Aufruf, um ein ausgehendes Event zu publizieren. Hook0 ist Webhook-as-a-Service für event-getriebene Architekturen. Es erledigt den Rest.',
    codeBlock: 'curl -X POST https://app.hook0.com/api/v1/event \\\n  -H "Authorization: Bearer YOUR_API_KEY" \\\n  -H "Content-Type: application/json" \\\n  -d \'{\n    "event_type": "invoice.paid",\n    "payload": {\n      "invoice_id": "inv_123",\n      "amount": 9900,\n      "currency": "eur"\n    }\n  }\'',
    codeFootnote: 'Wiederholungen, HMAC-Signaturen, Zustelllog, Abonnenten-Benachrichtigung, erledigt.',
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Wie lange dauert es, Webhooks von Grund auf zu bauen?', a: 'Plan mindestens 3 Engineering-Sprints ein. Wiederholungslogik, Dead Letter Queues, HMAC-Signaturen, Zustell-Monitoring, Abonnenten-Verwaltung, Endpoint-Health-Checks. Und das ist, bevor dein erster Kunde einen Bug findet.' },
      { q: 'Was sind die versteckten Kosten beim Selbstbau?', a: 'Pflege der Wiederholungs-Queue, Handhabung von Grenzfällen (Timeouts, Weiterleitungen, Zertifikatsfehler), Monitoring-Dashboards, Rate-Limiting, Log-Speicherung, Abonnenten-Onboarding. Nichts davon hört nach v1 auf. Es summiert sich.' },
      { q: 'Wie schnell kann ich Hook0 integrieren?', a: 'Unter 30 Minuten. Ein API-Aufruf, um ein Event zu triggern. SDKs für Python, Node.js und weitere, falls du das bevorzugst.' },
      { q: 'Kann ich von einem Eigenbau-System migrieren?', a: 'Ja. REST-API und SDKs, du kannst beide Systeme während der Migration parallel laufen lassen. Keine Big-Bang-Umstellung nötig.' },
    ],
  },
  deepDiveHtml: 'Mehr Details gefällig? <a href="https://documentation.hook0.com/tutorials/getting-started" class="text-indigo-400 hover:text-indigo-300 underline">Lies den Getting-Started-Guide in der Doku</a>.',
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst-gehostete Webhooks' },
    ],
  },
};
