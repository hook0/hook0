// Per-page strings for hook0-vs-hookdeck (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Middle-Dot.
// Hook0 = « quelloffen (SSPL-1.0) », NIEMALS « Open Source » (SSPL nicht OSI, UWG §5 Risiko).
// Hookdeck: Outpost (Zustellung) unter Apache-2.0 und selbst-hostbar, Event Gateway (Ingestion) geschlossen.
// Keine absoluten DSGVO-Claims im Body. Keine Behauptung « kein US-Konzern im Stack ».
module.exports = {
  pageTitle: 'Hook0 vs Hookdeck, Webhook-Plattform vs Gateway | Hook0',
  pageDescription: 'Vergleich Hook0 und Hookdeck: eine Open-Source-Webhook-Plattform (SSPL-1.0) gegen ein Gateway plus separates Zustellprodukt. Lizenzen, Selbst-Hosting und Tarife.',
  pageModified: '2026-07-16',
  breadcrumb: 'Hook0 vs. Hookdeck',
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Hookdeck',
    titleAccent: 'Quelloffene Webhook-Alternative',
    subtitle: 'Suchst du eine Alternative zu Hookdeck? Hook0 ist eine Webhook-Plattform, quelloffen (SSPL-1.0), in der EU gehostet, ohne Anbieter-Lock-in. Hookdeck teilt die Arbeit zwischen seinem Event Gateway und Outpost auf, seinem Zustellprodukt. Hier siehst du, was jede Lösung wirklich abdeckt.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  platformVsGateway: {
    eyebrow: 'Grundlegender Unterschied',
    h2: 'Plattform vs Gateway',
    intro: 'Hook0 sendet Webhooks aus einer einzigen Plattform. Hookdeck leitet eingehenden Verkehr über sein Event Gateway weiter und verkauft die ausgehende Zustellung separat als Outpost.',
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
      title: 'Hookdeck, Webhook-Gateway + Outpost',
      bullets: [
        'Proxy-Schicht zwischen Sendern und Empfängern',
        'Fügt bestehenden Webhooks Wiederholungen und Queueing hinzu',
        'Das Senden läuft über ein separates Produkt, Outpost',
        'Outpost unter Apache-2.0, Event Gateway geschlossen',
        'Selbst-Hosting nur für Outpost',
      ],
    },
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Seite an Seite',
    headers: { feature: 'Funktion', hook0: 'Hook0', hookdeck: 'Hookdeck' },
    rows: [
      { feature: 'Typ', hook0Html: 'Vollständige Webhook-Plattform', hookdeckHtml: 'Event Gateway (eingehend) + Outpost (ausgehend)' },
      { feature: 'Lizenz', hook0Html: 'SSPL-1.0 (quelloffen)', hookdeckHtml: 'Outpost Apache-2.0, Event Gateway geschlossen' },
      { feature: 'Selbst-Hosting', hook0Html: 'Ja (Docker / K8s)', hookdeckHtml: 'Nur Outpost' },
      { feature: 'Webhooks senden', hook0Html: 'Ja (Kernfunktion)', hookdeckHtml: 'Ja (über Outpost)' },
      { feature: 'Subscriber-Verwaltung', hook0Html: 'Integriertes Portal', hookdeckHtml: 'Nicht zutreffend' },
      { feature: 'HMAC-Signaturen', hook0Html: 'Automatisch erzeugt', hookdeckHtml: 'Nur Verifikation' },
      { feature: 'Event-Typ-Verwaltung', hook0Html: 'Vollständige Event-Typ-Registry', hookdeckHtml: 'Nein' },
      { feature: 'Kostenloser Tarif', hook0Html: '100 Events/Tag, EU-gehostet', hookdeckHtml: '100.000 Events/Monat' },
      { feature: 'Datenhosting', hook0Html: 'Europa (Clever Cloud FR, CDN Cloudflare USA) oder Selbst-Hosting', hookdeckHtml: 'In Kanada, EU-Region verfügbar' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    lastReviewed: 'Zuletzt geprüft Juli 2026.',
    items: [
      { q: 'Was ist der Unterschied zwischen Hook0 und Hookdeck?', a: 'Hook0 ist eine Webhook-Plattform, du sendest Events per API, Hook0 stellt sie mit Wiederholungen, Signaturen und Monitoring an deine Subscriber zu. Hookdecks Event Gateway sitzt zwischen bestehenden Webhook-Sendern und -Empfängern und ergänzt Zuverlässigkeit. Es sendet selbst keine Webhooks, das übernimmt Outpost, Hookdecks zweites Produkt.' },
      { q: 'Ist Hook0 quelloffen?', a: 'Der Hook0-Server wird unter SSPL-1.0 veröffentlicht und die SDKs unter MIT. SSPL ist eine Copyleft-Lizenz mit verfügbarem Quellcode, du darfst die gesamte Plattform frei prüfen, ändern und selbst hosten. Hookdeck veröffentlicht Outpost, seine Zustellungskomponente, unter Apache-2.0 und hält sein Event Gateway geschlossen und nur als verwalteten Dienst verfügbar.' },
      { q: 'Kann ich Hook0 selbst hosten?', a: 'Ja. Hook0 unterstützt Selbst-Hosting per Docker Compose oder Kubernetes ohne Kosten, und die Managed Cloud läuft auf genau diesem Code, ohne Funktionen hinter einem Enterprise-Tarif. Hookdeck lässt Outpost unter Apache-2.0 selbst hosten, sein Event Gateway ist ein reiner Cloud-Dienst, und bei Outpost managed starten SSO, RBAC und SCIM im Growth-Tarif ab 499 $/Monat zusätzlich zu den Event-Kosten.' },
      { q: 'Welches soll ich wählen?', a: 'Wenn du Webhooks zu deinem Produkt hinzufügen musst (Events an die Endpoints deiner Nutzer senden), nimm Hook0. Wenn du bereits Webhooks von Dritten empfängst und nur einen Zuverlässigkeits-Proxy brauchst, kann Hookdeck passen. Das sind zwei Werkzeuge für zwei verschiedene Probleme.' },
      { q: 'Ist Hook0 in der EU gehostet, anders als Hookdeck?', a: "Hook0 Cloud wird von einem französischen Unternehmen (FGRibreau SARL) betrieben, mit seiner Datenebene auf Clever Cloud in Frankreich. Die vorgelagerte CDN- und DDoS-Schicht stellt Cloudflare (US), offengelegt in einer öffentlichen Unterauftragsverarbeiter-Liste samt Übermittlungsmechanismus. Hookdeck ist ein kanadisches Unternehmen. Und weil Hook0 mit demselben Code selbst gehostet werden kann, können Sie Webhook-Daten vollständig in Ihrem eigenen Netzwerk halten." },
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
