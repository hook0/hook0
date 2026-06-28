// Per-page strings for hook0-alternatives (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt, kein --.
// Hook0 = « quelloffen (SSPL-1.0) ». Convoy MPL-2.0 = OSI, also « Open Source » OK für Convoy.
module.exports = {
  pageTitle: 'Hook0-Alternativen (2026), ehrlicher Vergleich | Hook0',
  pageDescription: 'Du suchst Hook0-Alternativen? Vergleich Hook0, Svix, Hookdeck und Convoy Seite an Seite zu Lizenzierung, Selbst-Hosting, Preis und Funktionen.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0-Alternativen',
    titleAccent: 'Ein ehrlicher Vergleich',
    subtitle: 'Du suchst eine Webhook-Plattform? Jemand hat eine Seite « Hook0-Alternativen » publiziert, also hier unsere Sicht der Geschichte. Kein Marketing-Spin, nur Fakten Seite an Seite.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Hook0 vs die Alternativen',
    sub: 'Vier Webhook-Plattformen, eine Tabelle. Urteile selbst.',
    headers: { criteria: 'Kriterium', hook0: 'Hook0', svix: 'Svix', hookdeck: 'Hookdeck', convoy: 'Convoy' },
    rows: [
      { criteria: 'Quellcode', hook0Html: 'Ja (SSPL-1.0, gesamter Quellcode)', svixHtml: 'Teilweise (Open Core, Enterprise geschlossen)', hookdeckHtml: 'Nein (Closed Source)', convoyHtml: 'Ja (MPL-2.0, Open Source)' },
      { criteria: 'Selbst-Hosting', hook0Html: 'Kostenlos (Docker / K8s)', svixHtml: 'Nur Enterprise-Plan', hookdeckHtml: 'Nein', convoyHtml: 'Ja (selbst-verwaltet)' },
      { criteria: 'Kostenloser Tarif', hook0Html: 'Ja, ohne Kreditkarte', svixHtml: 'Ja', hookdeckHtml: 'Ja (100k Events/Monat)', convoyHtml: 'Nur Community-Edition' },
      { criteria: 'Preismodell', hook0Html: 'Event-basiert, transparent', svixHtml: 'Event-basiert + Enterprise-Stufen', hookdeckHtml: 'Event-basiert, nur Cloud', convoyHtml: 'Enterprise-Preise' },
      { criteria: 'HMAC-Signaturen', hook0Html: 'Enthalten (alle Tarife)', svixHtml: 'Enthalten', hookdeckHtml: 'Nur Verifizierung', convoyHtml: 'Enthalten' },
      { criteria: 'Wiederholungslogik', hook0Html: 'Konfigurierbar pro Abonnement (schnelle + langsame Phasen)', svixHtml: 'Automatische Wiederholungen', hookdeckHtml: 'Automatische Wiederholungen', convoyHtml: 'Automatische Wiederholungen' },
      { criteria: 'Finanzierung', hook0Html: '100% bootstrapped', svixHtml: '17 Mio. $ VC-finanziert', hookdeckHtml: '3,5 Mio. $ VC-finanziert', convoyHtml: 'VC-finanziert' },
      { criteria: 'Datenhosting', hook0Html: 'Europa (DSGVO) oder Selbst-Hosting', svixHtml: 'In den USA', hookdeckHtml: 'In den USA', convoyHtml: 'Nur Selbst-Hosting' },
      { criteria: 'Typ', hook0Html: 'Vollständige Webhook-Plattform', svixHtml: 'Webhook-Plattform (Open Core)', hookdeckHtml: 'Webhook-Gateway / Proxy', convoyHtml: 'Webhook-Plattform' },
    ],
  },
  whatTheyLeftOut: {
    eyebrow: 'Das ganze Bild',
    h2: 'Was ihre Vergleichsseite dir nicht sagt',
    sub: 'Hookdeck hat eine Seite « Hook0-Alternativen » publiziert. Wir freuen uns über die Aufmerksamkeit. Hier ist, was sie weggelassen haben.',
    cards: [
      { title: '« Hook0 ist nur HTTPS »', body: 'Ja, und das ist ein Feature, keine Einschränkung. Webhook-Payloads über reines HTTP zu senden bedeutet, dass die Daten deiner Kunden im Klartext durchs Netz wandern. Jedes ernsthafte Produktivsystem nutzt HTTPS. Wir erzwingen es, weil Sicherheit nicht optional ist.', color: 'green' },
      { title: '« Kein veröffentlichtes SLA »', body: 'Hook0 Cloud Enterprise enthält ein massgeschneidertes SLA mit dediziertem Support. Wenn Verfügbarkeitsgarantien wichtig sind, ist das der schnellste Weg, ohne eigene Infrastruktur, ohne Ops-Team. Hook0 ist auch quelloffen (SSPL-1.0), also hast du immer die Option, selbst zu hosten, wenn deine Compliance-Anforderungen es verlangen.', color: 'indigo' },
      { title: '« Preise sind unklar »', body: 'Unsere Preise sind öffentlich und Event-basiert. Kein Verkaufsgespräch nötig. Keine « Kontakt »-Mauer. Cloud startet bei 59 €/Monat, 8x günstiger als Svix für vergleichbare Funktionen. Versuch mal, diese Transparenz von einem VC-finanzierten, geschlossenen Wettbewerber zu bekommen.', color: 'green' },
      { title: 'Was sie nicht erwähnen, die Finanzierung', body: 'Hookdeck hat 3,5 Mio. $ in VC eingesammelt. Svix 17 Mio. $. Convoy ist auch VC-finanziert. Hook0 ist zu 100% bootstrapped. Wenn dein Webhook-Anbieter den Umsatz verzehnfachen muss, um Investoren zu befriedigen, rate mal, wessen Preise steigen. Nicht unsere.', color: 'indigo' },
    ],
  },
  difference: {
    eyebrow: 'Warum Hook0',
    h2: 'Der Hook0-Unterschied',
    cards: [
      { title: 'Nicht nur ein Proxy', body: 'Anders als Hookdeck sendet Hook0 Webhooks für dich, mit Wiederholungen, Signaturen und Abonnenten-Verwaltung. Keine Middleware-Schicht.' },
      { title: 'Keine Enterprise-Paywall', body: 'Anders als Svix wird jedes Feature in jedem Tarif ausgeliefert. Selbst-Hosting steckt nicht hinter einem Verkaufsgespräch.' },
      { title: 'Europäisch, auf DSGVO ausgelegt', body: 'Datenebene in der EU bei Clever Cloud (Frankreich) gehostet. CDN über Cloudflare (USA), im <a href="/de/auftragsverarbeitungsvertrag">DPA</a> und in den <a href="/de/dsgvo-unterauftragsverarbeiter">Unterauftragsverarbeitern</a> offengelegt. Bootstrapped, kein US-VC-Board, das über deine Datenpolitik entscheidet.' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Was sind die besten Hook0-Alternativen?', a: 'Die Haupt-Alternativen zu Hook0 sind Svix (Open Core, VC-finanziert), Hookdeck (Closed-Source-Webhook-Gateway) und Convoy (Open Source, VC-finanziert). Jede löst einen anderen Teil des Webhook-Problems. Hook0 ist die einzige, die vollständig quelloffen (SSPL-1.0), bootstrapped und kostenlos selbst-hostbar ist.' },
      { q: 'Ist Hookdeck besser als Hook0?', a: 'Hookdeck ist ein Webhook-Gateway, es proxyt bestehende Webhooks für Zuverlässigkeit. Hook0 ist eine Webhook-Plattform, es sendet Webhooks für dich mit Wiederholungen, Signaturen und Abonnenten-Verwaltung. Sie lösen verschiedene Probleme. Wenn du Webhooks zu deinem Produkt hinzufügen musst, ist Hook0 das richtige Tool.' },
      { q: 'Soll ich Svix oder Hook0 nutzen?', a: 'Beide sind Webhook-Plattformen, aber sie unterscheiden sich in Lizenzierung und Finanzierung. Svix ist Open Core (Enterprise-Features sind geschlossen) und hat 17 Mio. $ VC eingesammelt. Hook0 ist vollständig quelloffen unter SSPL, bootstrapped, und bietet kostenloses Selbst-Hosting. Wenn dir Anbieter-Unabhängigkeit und langfristige Preisstabilität wichtig sind, ist Hook0 die sicherere Wahl.' },
      { q: 'Was kostet Hook0?', a: 'Hook0 hat einen kostenlosen Tarif ohne Kreditkarte. Hook0 ist auch quelloffen und selbst-hostbar für Compliance-Anforderungen. Hook0 Cloud ergänzt um managed Infrastruktur, EU-Hosting, automatische Updates und Priority-Support. Bezahlte Tarife starten bei 59 €/Monat mit Event-basierter Abrechnung.' },
      { q: 'Skaliert Hook0?', a: 'Ja. Die Architektur von Hook0 unterstützt nur PostgreSQL für Einfachheit oder Pulsar + S3 für hohen Durchsatz. Cloud-Kunden verarbeiten Millionen Events pro Tag. Dieselbe Architektur läuft identisch im Selbst-Hosting.' },
    ],
  },
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'svix-alternatives', label: 'Svix-Alternativen' },
      { enSlug: 'hookdeck-alternatives', label: 'Hookdeck-Alternativen' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy Webhooks' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst-gehostete Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Bester Open-Source-Webhook-Server' },
    ],
  },
};
