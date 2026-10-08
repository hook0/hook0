// Per-page strings for svix-alternatives (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// Hook0 = « Open Source (SSPL-1.0) ». Svix MIT (OSI) = « Open Source » OK für Svix.
module.exports = {
  pageTitle: 'Svix-Alternativen 2026, Webhook-Plattformen | Hook0',
  pageDescription: 'Du evaluierst Svix? Vergleich Hook0, Hookdeck, Convoy: Preis, Selbst-Hosting, Lizenzierung und was « Open Source » bedeutet.',
  pageModified: '2026-10-08',
  breadcrumb: 'Svix-Alternativen',
  tldr: {
    h2: 'Kurz gesagt',
    body: 'Die beste Svix-Alternative hängt davon ab, was du brauchst, aber für wirklich offene Lizenzierung, EU-Datenresidenz und kostenloses Selbst-Hosting passt Hook0. Es läuft unter SSPL-1.0 ohne geschlossene Add-ons, hält seine Datenebene auf Clever Cloud in Frankreich und rechnet pro Event ab 59 €/Monat ab. Convoy und Hookdeck decken jeweils einen Teil davon ab, und die Tabelle unten zeigt, wo jede Option steht.',
  },
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Du suchst eine Svix-Alternative?',
    titleAccent: 'Webhook-Plattformen im Vergleich',
    subtitle: 'Svix ist eine gute Webhook-Plattform, und nicht die einzige Option. Wenn dir eine SSPL-1.0-Lizenz wichtig ist, die du kostenlos selbst hosten kannst, dazu EU-Datenresidenz oder eine öffentliche, transparente Preisgestaltung, schlüsselt diese Seite die Alternativen auf.',
    ctaPrimary: 'Kostenlos mit Hook0 starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  whyLookBeyond: {
    eyebrow: 'Warum über Svix hinausschauen',
    h2: 'Warum Teams woanders schauen',
    cards: [
      { title: 'Grenzen des Open Core', body: 'Die MIT-Basis von Svix ist echtes Open Source. Der Haken, Enterprise-Features wie SSO, erweiterte Analytics und dedizierter Support sind proprietär. Wenn du skalierst, läufst du gegen die Paywall. Wenn dein Team vollen Quellzugriff braucht, ist das ein Problem.' },
      { title: '17 Mio. $ VC vs bootstrappt', body: 'Svix hat 17 Mio. $ Wagniskapital aufgenommen; Hook0 ist zu 100% bootstrappt und nimmt kein externes Kapital. Beide Modelle folgen unterschiedlichen Interessengruppen, Investoren gegenüber Kunden, was bei der Wahl eines langfristigen Anbieters zu bedenken ist.' },
      { title: 'Eine EU-Region ist nicht alles', body: 'Die verwaltete Cloud von Svix bietet EU- und US-Regionen, und der MIT-Server läuft auch auf deiner eigenen Infrastruktur. Eine EWR-Auftragsverarbeitungsvereinbarung gibt es in den Tarifen Professional und Enterprise. Hook0 betreibt seine Datenebene in jedem Tarif, auch im kostenlosen, bei Clever Cloud in Frankreich und veröffentlicht seine Liste der Unterauftragsverarbeiter. Für eine DSGVO- oder Souveränitätsprüfung zählt, wer die Infrastruktur betreibt, nicht nur, wo sie steht.' },
    ],
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Svix vs die Alternativen',
    sub: 'Fünf Webhook-Plattformen Seite an Seite. Daten sprechen lauter als Marketing-Seiten.',
    headers: { criteria: 'Kriterium', svix: 'Svix', hook0: 'Hook0', hookdeck: 'Hookdeck', convoy: 'Convoy', hostedhooks: 'HostedHooks' },
    rows: [
      { criteria: 'Lizenz', svixHtml: 'MIT (Open Core, Enterprise geschlossen)', hook0Html: 'SSPL-1.0 (gesamter Quellcode verfügbar)', hookdeckHtml: 'Teilweise (Outpost Apache-2.0; Event Gateway geschlossen)', convoyHtml: 'Quelloffen verfügbar (Elastic License 2.0)', hostedhooksHtml: 'Closed Source' },
      { criteria: 'Finanzierung', svixHtml: '17 Mio. $ VC-finanziert', hook0Html: '100% bootstrappt', hookdeckHtml: '3,5 Mio. $ VC-finanziert', convoyHtml: 'VC-finanziert', hostedhooksHtml: 'Bootstrapped' },
      { criteria: 'Selbst-Hosting', svixHtml: 'MIT-Server, selbst betrieben; On-Prem-Support in Enterprise', hook0Html: 'Kostenlos (Docker / K8s)', hookdeckHtml: 'Nur Outpost (Event Gateway nur Cloud)', convoyHtml: 'Ja (selbst-verwaltet)', hostedhooksHtml: 'Nein' },
      { criteria: 'Kostenloser Tarif', svixHtml: 'Ja', hook0Html: 'Ja, ohne Kreditkarte', hookdeckHtml: 'Ja (10.000 Events/Monat)', convoyHtml: 'Nur Community-Edition', hostedhooksHtml: 'Ja (begrenzt)' },
      { criteria: 'HMAC-Signaturen', svixHtml: 'Enthalten', hook0Html: 'Enthalten (alle Tarife)', hookdeckHtml: 'Nur Verifizierung', convoyHtml: 'Enthalten', hostedhooksHtml: 'Enthalten' },
      { criteria: 'Wiederholungslogik', svixHtml: 'Automatische Wiederholungen', hook0Html: 'Konfigurierbar pro Abonnement (schnelle + langsame Phasen)', hookdeckHtml: 'Automatische Wiederholungen', convoyHtml: 'Automatische Wiederholungen', hostedhooksHtml: 'Automatische Wiederholungen' },
      { criteria: 'Datenhosting', svixHtml: 'EU- und US-Regionen oder Selbst-Hosting', hook0Html: 'Europa (Clever Cloud FR, CDN Cloudflare USA) oder Selbst-Hosting', hookdeckHtml: 'In den USA', convoyHtml: 'Nur Selbst-Hosting', hostedhooksHtml: 'In den USA' },
      { criteria: 'Open-Source-Level', svixHtml: 'Teilweise (Open Core)', hook0Html: 'Vollständig (SSPL, keine geschlossenen Add-ons)', hookdeckHtml: 'Teilweise (nur Outpost)', convoyHtml: 'Quelloffen verfügbar (Elastic License 2.0)', hostedhooksHtml: 'Keiner' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Svix wirklich Open Source?', a: 'Teilweise. Die Basis von Svix steht unter MIT-Lizenz, aber Enterprise-Features (SSO, erweiterte Analytics, Priority-Support) sind proprietär und geschlossen. Das nennt sich Open Core. Du kannst die Community-Edition betreiben, aber zentrale Produktionsfeatures erfordern einen kostenpflichtigen Tarif. Hook0 dagegen liefert alles unter SSPL-1.0 aus, ohne geschlossene Add-ons.' },
      { q: 'Kann ich Svix kostenlos selbst hosten?', a: 'Ja, mit Einschränkungen. Der Svix-Server steht unter MIT-Lizenz, und sein README beschreibt ein Docker- und Docker-Compose-Setup, das jeder betreiben kann. Dasselbe README hält fest, dass einige Funktionen des gehosteten Dienstes noch nicht im Open-Source-Server enthalten sind, und unterstütztes On-Prem-Deployment gehört zum Enterprise-Tarif. Hook0 lässt sich kostenlos selbst hosten, mit denselben Funktionen wie seine Cloud.' },
      { q: 'Was ist die beste Svix-Alternative für Startups?', a: 'Hook0 funktioniert gut für Startups. Kostenloser Tarif, ohne Kreditkarte, Event-basierter Preis ab 59 €/Monat und kostenloses Selbst-Hosting via Docker oder Kubernetes. Das Unternehmen ist zu 100% bootstrappt, also kein VC, der nächstes Quartal auf höhere Preise drängt. Convoy ist auch einen Blick wert, steht aber unter der Elastic License 2.0, quelloffen verfügbar, wie Hook0s SSPL-1.0.' },
      { q: 'Wie schneiden die Preise von Svix gegen die Alternativen ab?', a: 'Svix bietet einen kostenlosen Tarif, Basic ab 20 $/Monat und Professional ab 490 $/Monat. Unterstütztes On-Prem-Deployment, SAML/OIDC-SSO und eigene Regionen gehören zum Enterprise-Tarif, Preis auf Anfrage. Hook0 Cloud startet bei 59 €/Monat mit transparenten Preisen und enthält Selbst-Hosting kostenlos in jedem Tarif. Hookdeck rechnet pro Event ab, mit dem Event Gateway nur in der Cloud und Outpost selbst hostbar. Convoy ist nur selbst-gehostet, mit Enterprise-Pricing für Support. HostedHooks bietet nur kostenpflichtige Cloud-Pläne.' },
      { q: "Welche Svix-Alternative ist zugleich EU-gehostet und Open Source?", a: "Hook0. Seine Datenebene läuft in jedem Tarif auf Clever Cloud in Frankreich (innerhalb der EU), und der vollständige Server ist Open Source (SSPL-1.0), sodass Sie ihn prüfen oder selbst hosten können. Svix ist open-core und bietet EU- und US-Regionen in seiner verwalteten Cloud; viele in der EU gehostete Webhook-Dienste sind proprietär und rein cloudbasiert. Das vorgelagerte CDN von Hook0 Cloud ist Cloudflare (US), offengelegt in der öffentlichen Unterauftragsverarbeiter-Liste." },
      { q: 'Gibt es eine kostenlose, quelloffene Svix-Alternative?', a: 'Ja. Hook0 ist quelloffen unter SSPL-1.0 und kostenlos selbst hostbar auf Docker oder Kubernetes, mit einem kostenlosen Cloud-Tarif ohne Kreditkarte. Convoy ist quelloffen verfügbar unter der Elastic License 2.0 und ebenfalls selbst hostbar, aber Hook0 bietet zusätzlich eine EU-gehostete Cloud, falls du es nicht selbst betreiben willst.' },
      { q: 'Wie migriere ich von Svix zu Hook0?', a: 'Hook0 nutzt dieselben Webhook-Bausteine wie Svix, nämlich eine Event-API, Abonnements pro Endpoint, HMAC-Signaturen und automatische Wiederholungen. Du richtest deine Producer auf die Hook0-API, legst deine Abonnements neu an und stellst deine Signaturprüfung auf das Hook0-Secret um. Weil der Server quelloffen ist, kannst du den gesamten Ablauf auf einer selbst gehosteten Instanz proben, bevor du Produktionstraffic umziehst.' },
      { q: 'Ist Hook0 im Vergleich zu Svix produktionsreif?', a: 'Hook0 liefert die Zustellgarantien, die du von einer Webhook-Plattform erwartest, nämlich HMAC-signierte Payloads, konfigurierbare Wiederholungen in schnellen und langsamen Phasen und Zustellungs-Monitoring in jedem Tarif. Weil der Server unter SSPL-1.0 quelloffen ist, kannst du den genauen Zustell- und Wiederholungscode lesen, statt einer Behauptung zu vertrauen, und ihn für volle Kontrolle selbst betreiben. Seine EU-Datenebene läuft auf Clever Cloud in Frankreich.' },
    ],
  },
  comparisonSources: {
    competitors: ["Svix", "Hookdeck", "Convoy", "HostedHooks"],
    scope: "Diese Seite vergleicht Plattformen für den Webhook-Versand anhand ihrer öffentlichen Tarife, Lizenzen und ihres Hostings. Die Preise sind die veröffentlichten Angaben der Anbieter, und die Funktionen entsprechen ihrer öffentlichen Dokumentation.",
    sources: [
      { label: "Svix Preise", url: "https://www.svix.com/pricing/", consulted: "2026-10-08" },
      { label: "Svix Quellcode und Lizenz auf GitHub", url: "https://github.com/svix/svix-webhooks", consulted: "2026-10-08" },
      { label: "Hookdeck Preise", url: "https://hookdeck.com/pricing", consulted: "2026-10-04", archive: "https://web.archive.org/web/20260914151910/https://hookdeck.com/pricing" },
      { label: "Hookdeck Outpost Quellcode und Lizenz auf GitHub", url: "https://github.com/hookdeck/outpost", consulted: "2026-09-29" },
      { label: "Convoy Releases auf GitHub", url: "https://github.com/frain-dev/convoy/releases", consulted: "2026-09-29" },
      { label: "Convoy Preise", url: "https://www.getconvoy.io/pricing", consulted: "2026-10-04", archive: "https://web.archive.org/web/20260604143101/https://www.getconvoy.io/pricing" },
      { label: "HostedHooks Preise", url: "https://www.hostedhooks.com/pricing", consulted: "2026-09-29" },
    ],
    updated: "2026-10-08",
  },
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Hook0-Alternativen' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
    ],
  },
};
