// Per-page strings for hook0-vs-hookdeck (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Middle-Dot.
// Hook0 = « quelloffen (SSPL-1.0) », NIEMALS « Open Source » (SSPL nicht OSI, UWG §5 Risiko).
// Hookdeck = closed-source, also keine Lizenzfrage für sie.
// Keine absoluten DSGVO-Claims im Body. Keine Behauptung « kein US-Konzern im Stack ».
module.exports = {
  pageTitle: 'Hook0 vs Hookdeck, Webhook-Plattform vs Gateway | Hook0',
  pageDescription: 'Vergleich Hook0 und Hookdeck, Webhook-Plattform vs Gateway, quelloffen vs proprietär, selbst-hostbar vs cloudgebunden. Schau dir die wichtigen Unterschiede an.',
  pageModified: '2026-06-27',
  breadcrumb: 'Hook0 vs. Hookdeck',
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Hookdeck',
    titleAccent: 'Quelloffene Webhook-Alternative',
    subtitle: 'Suchst du eine Alternative zu Hookdeck? Hook0 ist eine Webhook-Plattform, quelloffen (SSPL-1.0), in der EU gehostet, ohne Anbieter-Lock-in. Hookdeck ist ein Webhook-Gateway. Sie lösen verschiedene Probleme. Hier siehst du, was jede Lösung wirklich abdeckt.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  platformVsGateway: {
    eyebrow: 'Grundlegender Unterschied',
    h2: 'Plattform vs Gateway',
    intro: 'Hook0 und Hookdeck lösen verschiedene Probleme. Das eine sendet Webhooks, das andere leitet sie weiter.',
    hook0: {
      title: 'Hook0, Webhook-Plattform',
      bullets: [
        'Sendet Webhooks an die Endpoints deiner Nutzer',
        'Verwaltet Subscriptions, Event-Typen, Wiederholungen',
        'HMAC-Signaturen, Zustellungslogs, Subscription-Verwaltung',
        'Ein API-Aufruf, um ein Event auszulösen',
        'Quelloffen (SSPL-1.0), selbst-hostbar',
      ],
    },
    hookdeck: {
      title: 'Hookdeck, Webhook-Gateway',
      bullets: [
        'Proxy-Schicht zwischen Sendern und Empfängern',
        'Fügt bestehenden Webhooks Wiederholungen und Queueing hinzu',
        'Erzeugt und sendet keine Webhooks',
        'Proprietär, nur Cloud',
        'Keine Option zum Selbst-Hosting',
      ],
    },
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Seite an Seite',
    headers: { feature: 'Funktion', hook0: 'Hook0', hookdeck: 'Hookdeck' },
    rows: [
      { feature: 'Typ', hook0Html: 'Vollständige Webhook-Plattform', hookdeckHtml: 'Webhook-Gateway / Proxy' },
      { feature: 'Lizenz', hook0Html: 'SSPL-1.0 (quelloffen)', hookdeckHtml: 'Proprietär (geschlossen)' },
      { feature: 'Selbst-Hosting', hook0Html: 'Ja (Docker / K8s)', hookdeckHtml: 'Nein' },
      { feature: 'Webhooks senden', hook0Html: 'Ja (Kernfunktion)', hookdeckHtml: 'Nein (nur Proxy)' },
      { feature: 'Subscriber-Verwaltung', hook0Html: 'Integriertes Portal', hookdeckHtml: 'Nicht zutreffend' },
      { feature: 'HMAC-Signaturen', hook0Html: 'Automatisch erzeugt', hookdeckHtml: 'Nur Verifikation' },
      { feature: 'Event-Typ-Verwaltung', hook0Html: 'Vollständige Event-Typ-Registry', hookdeckHtml: 'Nein' },
      { feature: 'Kostenloser Tarif', hook0Html: '100 Events/Tag, EU-gehostet', hookdeckHtml: '100.000 Events/Monat' },
      { feature: 'Datenhosting', hook0Html: 'Europa (Clever Cloud FR, CDN Cloudflare USA) oder Selbst-Hosting', hookdeckHtml: 'Sitz in den USA' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    lastReviewed: 'Zuletzt geprüft Juni 2026.',
    items: [
      { q: 'Was ist der Unterschied zwischen Hook0 und Hookdeck?', a: 'Hook0 ist eine Webhook-Plattform, du sendest Events per API, Hook0 stellt sie mit Wiederholungen, Signaturen und Monitoring an deine Subscriber zu. Hookdeck ist ein Gateway, das zwischen bestehenden Webhook-Sendern und -Empfängern sitzt und Zuverlässigkeit ergänzt. Es sendet selbst keine Webhooks.' },
      { q: 'Ist Hook0 quelloffen?', a: 'Der Hook0-Server wird unter SSPL-1.0 veröffentlicht und die SDKs unter MIT. SSPL ist eine Copyleft-Lizenz mit verfügbarem Quellcode, du darfst die gesamte Plattform frei prüfen, ändern und selbst hosten. Hookdeck ist geschlossen und nur als verwalteter SaaS-Dienst verfügbar.' },
      { q: 'Kann ich Hook0 selbst hosten?', a: 'Ja. Hook0 unterstützt Selbst-Hosting per Docker Compose oder Kubernetes ohne Kosten. Hookdeck bietet kein Selbst-Hosting, es ist ein reiner Cloud-Dienst.' },
      { q: 'Welches soll ich wählen?', a: 'Wenn du Webhooks zu deinem Produkt hinzufügen musst (Events an die Endpoints deiner Nutzer senden), nimm Hook0. Wenn du bereits Webhooks von Dritten empfängst und nur einen Zuverlässigkeits-Proxy brauchst, kann Hookdeck passen. Das sind zwei Werkzeuge für zwei verschiedene Probleme.' },
      { q: 'Ist Hook0 in der EU gehostet, anders als Hookdeck?', a: 'Hook0 Cloud wird von einem französischen Unternehmen (FGRibreau SARL) betrieben, die Anwendungs-Datenebene läuft in Frankreich bei Clever Cloud. Das CDN Cloudflare (USA) bleibt dem CLOUD Act ausgesetzt und ist in unseren <a href="/de/dsgvo-unterauftragsverarbeiter">DSGVO-Unterauftragsverarbeitern</a> und im <a href="/de/auftragsverarbeitungsvertrag">DPA</a> offengelegt (Transfer geregelt durch SCC 2021 + TIA, EU-US DPF wo anwendbar). Hookdeck ist ein US-Unternehmen. Du kannst Hook0 auch selbst hosten, damit keine Webhook-Daten dein Netzwerk verlassen.' },
      { q: 'Betrachtet Hookdeck Hook0 als Alternative?', a: 'Hookdeck veröffentlicht Vergleichsseiten, die Hook0 einschließen, und Svix tut das auch. Du kannst ihre eigenen Einschätzungen neben unserer lesen.' },
    ],
  },
  deepDive: {
    prefix: 'Willst du mehr Details?',
    linkText: 'Lies den vollständigen Vergleich mit Architektur-Diagrammen in unserer Dokumentation',
    linkHref: 'https://documentation.hook0.com/comparisons/hookdeck-vs-hook0',
  },
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'hookdeck-alternatives', label: 'Hookdeck-Alternativen' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Selbst bauen vs kaufen bei Webhooks' },
      { enSlug: 'webhook-cost-comparison', label: 'Webhook-Kostenvergleich (auf Englisch)' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU-Webhook-Infrastruktur (auf Englisch)' },
    ],
  },
};
