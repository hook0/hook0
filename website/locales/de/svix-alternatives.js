// Per-page strings for svix-alternatives (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// Hook0 = « quelloffen (SSPL-1.0) ». Svix MIT (OSI) = « Open Source » OK für Svix.
module.exports = {
  pageTitle: 'Svix-Alternativen (2026), Webhook-Plattformen im Vergleich | Hook0',
  pageDescription: 'Du evaluierst Svix? Vergleich Hook0, Hookdeck, Convoy und mehr. Seite an Seite zu Preis, Selbst-Hosting, Lizenzierung und was « Open Source » in der Praxis bedeutet.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Du suchst eine Svix-Alternative?',
    titleAccent: 'Webhook-Plattformen im Vergleich',
    subtitle: 'Svix ist eine gute Webhook-Plattform. Sie ist nicht die einzige. Wenn dir wirklich offene Lizenzierung, kostenloses Selbst-Hosting, EU-Datenresidenz oder ein Anbieter wichtig ist, der nach einer Series B nicht die Preise hochzieht, schlüsselt diese Seite deine Optionen auf.',
    ctaPrimary: 'Kostenlos mit Hook0 starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  whyLookBeyond: {
    eyebrow: 'Warum über Svix hinausschauen',
    h2: 'Warum Teams woanders schauen',
    cards: [
      { title: 'Grenzen des Open Core', body: 'Die MIT-Basis von Svix ist echtes Open Source. Der Haken, Enterprise-Features wie SSO, erweiterte Analytics und dedizierter Support sind proprietär. Wenn du skalierst, läufst du gegen die Paywall. Wenn dein Team vollen Quellzugriff braucht, ist das ein Problem.' },
      { title: '17 Mio. $ VC = Druck', body: 'Wagniskapital erwartet Rendite. Svix hat 17 Mio. $ eingesammelt, das Geld muss irgendwie zurückfließen, meist über Preiserhöhungen, Feature-Gating oder eine Übernahme. Ein bootstrapper Anbieter hat diesen Druck nicht.' },
      { title: 'Kein europäisches Hosting', body: 'Svix sitzt in den USA und bietet keine EU-Cloud-Option. Wenn du der DSGVO oder Datenresidenz-Regeln unterliegst, ist das ein Blocker. Du könntest selbst hosten, aber das setzt ihren Enterprise-Plan voraus.' },
    ],
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Svix vs die Alternativen',
    sub: 'Fünf Webhook-Plattformen Seite an Seite. Daten sprechen lauter als Marketing-Seiten.',
    headers: { criteria: 'Kriterium', svix: 'Svix', hook0: 'Hook0', hookdeck: 'Hookdeck', convoy: 'Convoy', hostedhooks: 'HostedHooks' },
    rows: [
      { criteria: 'Lizenz', svixHtml: 'MIT (Open Core, Enterprise geschlossen)', hook0Html: 'SSPL-1.0 (gesamter Quellcode verfügbar)', hookdeckHtml: 'Closed Source', convoyHtml: 'MPL-2.0', hostedhooksHtml: 'Closed Source' },
      { criteria: 'Finanzierung', svixHtml: '17 Mio. $ VC-finanziert', hook0Html: '100% bootstrappt', hookdeckHtml: '3,5 Mio. $ VC-finanziert', convoyHtml: 'VC-finanziert', hostedhooksHtml: 'Bootstrapped' },
      { criteria: 'Selbst-Hosting', svixHtml: 'Nur Enterprise-Plan (volle Features)', hook0Html: 'Kostenlos (Docker / K8s)', hookdeckHtml: 'Nein', convoyHtml: 'Ja (selbst-verwaltet)', hostedhooksHtml: 'Nein' },
      { criteria: 'Kostenloser Tarif', svixHtml: 'Ja', hook0Html: 'Ja, ohne Kreditkarte', hookdeckHtml: 'Ja (100k Events/Monat)', convoyHtml: 'Nur Community-Edition', hostedhooksHtml: 'Ja (begrenzt)' },
      { criteria: 'HMAC-Signaturen', svixHtml: 'Enthalten', hook0Html: 'Enthalten (alle Tarife)', hookdeckHtml: 'Nur Verifizierung', convoyHtml: 'Enthalten', hostedhooksHtml: 'Enthalten' },
      { criteria: 'Wiederholungslogik', svixHtml: 'Automatische Wiederholungen', hook0Html: 'Konfigurierbar pro Abonnement (schnelle + langsame Phasen)', hookdeckHtml: 'Automatische Wiederholungen', convoyHtml: 'Automatische Wiederholungen', hostedhooksHtml: 'Automatische Wiederholungen' },
      { criteria: 'Datenhosting', svixHtml: 'In den USA', hook0Html: 'Europa (Clever Cloud FR, CDN Cloudflare USA) oder Selbst-Hosting', hookdeckHtml: 'In den USA', convoyHtml: 'Nur Selbst-Hosting', hostedhooksHtml: 'In den USA' },
      { criteria: 'Open-Source-Level', svixHtml: 'Teilweise (Open Core)', hook0Html: 'Vollständig (SSPL, keine geschlossenen Add-ons)', hookdeckHtml: 'Keiner', convoyHtml: 'Vollständig (MPL-2.0)', hostedhooksHtml: 'Keiner' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Svix wirklich Open Source?', a: 'Teilweise. Die Basis von Svix steht unter MIT-Lizenz, aber Enterprise-Features (SSO, erweiterte Analytics, Priority-Support) sind proprietär und geschlossen. Das nennt sich Open Core. Du kannst die Community-Edition betreiben, aber zentrale Produktionsfeatures erfordern einen kostenpflichtigen Tarif. Hook0 dagegen liefert alles unter SSPL-1.0 aus, ohne geschlossene Add-ons.' },
      { q: 'Kann ich Svix kostenlos selbst hosten?', a: 'Du kannst die MIT-lizenzierte Community-Edition selbst hosten, aber Enterprise-Features sind nicht enthalten. Vollständiges Selbst-Hosting mit allen Features erfordert den Enterprise-Plan von Svix. Hook0 und Convoy bieten beide kostenloses Selbst-Hosting mit voller Feature-Parität.' },
      { q: 'Was ist die beste Svix-Alternative für Startups?', a: 'Hook0 funktioniert gut für Startups. Kostenloser Tarif, ohne Kreditkarte, Event-basierter Preis ab 59 €/Monat und kostenloses Selbst-Hosting via Docker oder Kubernetes. Das Unternehmen ist zu 100% bootstrappt, also kein VC, der nächstes Quartal auf höhere Preise drängt. Convoy ist auch einen Blick wert, falls dir die MPL-2.0-Lizenz wichtig ist.' },
      { q: 'Wie schneiden die Preise von Svix gegen die Alternativen ab?', a: 'Svix bietet einen kostenlosen Tarif und Event-basierte Bezahlpläne, aber Selbst-Hosting und Enterprise-Features erfordern Enterprise-Pricing (Kontakt zum Vertrieb). Hook0 Cloud startet bei 59 €/Monat mit transparenten Preisen und enthält Selbst-Hosting kostenlos in jedem Tarif. Hookdeck ist nur Cloud mit Event-basiertem Pricing. Convoy ist nur selbst-gehostet, mit Enterprise-Pricing für Support. HostedHooks bietet nur kostenpflichtige Cloud-Pläne.' },
    ],
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
