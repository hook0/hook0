// Per-page strings for hook0-alternatives (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt, kein --.
// Hook0 = « Open Source (SSPL-1.0) ». Convoy = Elastic License 2.0, nicht OSI-anerkannt, also « quelloffen verfügbar » statt « Open Source ».
// Hookdeck: Outpost (Zustellung) ist Apache-2.0 und selbst hostbar; nur das Event Gateway (Ingestion) bleibt geschlossen. Kanadisches Unternehmen.
module.exports = {
  pageTitle: 'Hook0-Alternativen (2026), ehrlicher Vergleich | Hook0',
  pageDescription: 'Du suchst Hook0-Alternativen? Vergleich Hook0, Svix, Hookdeck und Convoy Seite an Seite zu Lizenzierung, Selbst-Hosting, Preis und Funktionen.',
  pageModified: '2026-06-27',
  breadcrumb: 'Hook0-Alternativen',
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
      { criteria: 'Quellcode', hook0Html: 'Ja (SSPL-1.0, gesamter Quellcode)', svixHtml: 'Teilweise (Open Core, Enterprise geschlossen)', hookdeckHtml: 'Teilweise (Outpost Apache-2.0; Event Gateway geschlossen)', convoyHtml: 'Quelloffen verfügbar (Elastic License 2.0)' },
      { criteria: 'Selbst-Hosting', hook0Html: 'Kostenlos (Docker / K8s)', svixHtml: 'Nur Enterprise-Plan', hookdeckHtml: 'Outpost ja; Event Gateway nur Cloud', convoyHtml: 'Ja (selbst-verwaltet)' },
      { criteria: 'Kostenloser Tarif', hook0Html: 'Ja, ohne Kreditkarte', svixHtml: 'Ja', hookdeckHtml: 'Ja (100k Events/Monat)', convoyHtml: 'Nur Community-Edition' },
      { criteria: 'Preismodell', hook0Html: 'Event-basiert, transparent', svixHtml: 'Event-basiert + Enterprise-Stufen', hookdeckHtml: 'Event-basiert im Managed-Betrieb; Outpost selbst gehostet kostenlos', convoyHtml: 'Enterprise-Preise' },
      { criteria: 'HMAC-Signaturen', hook0Html: 'Enthalten (alle Tarife)', svixHtml: 'Enthalten', hookdeckHtml: 'Nur Verifizierung', convoyHtml: 'Enthalten' },
      { criteria: 'Wiederholungslogik', hook0Html: 'Konfigurierbar pro Abonnement (schnelle + langsame Phasen)', svixHtml: 'Automatische Wiederholungen', hookdeckHtml: 'Automatische Wiederholungen', convoyHtml: 'Automatische Wiederholungen' },
      { criteria: 'Finanzierung', hook0Html: '100% bootstrappt', svixHtml: '17 Mio. $ VC-finanziert', hookdeckHtml: '3,5 Mio. $ VC-finanziert', convoyHtml: 'VC-finanziert' },
      { criteria: 'Datenhosting', hook0Html: 'Europa (Clever Cloud FR, CDN Cloudflare USA) oder Selbst-Hosting', svixHtml: 'In den USA', hookdeckHtml: 'In Kanada, EU-Region verfügbar', convoyHtml: 'Nur Selbst-Hosting' },
      { criteria: 'Typ', hook0Html: 'Vollständige Webhook-Plattform', svixHtml: 'Webhook-Plattform (Open Core)', hookdeckHtml: 'Webhook-Gateway + Zustell-Engine Outpost', convoyHtml: 'Webhook-Plattform' },
    ],
  },
  whatTheyLeftOut: {
    eyebrow: 'Das ganze Bild',
    h2: 'Was ihre Vergleichsseite dir nicht sagt',
    sub: 'Hookdeck hat eine Seite « Hook0-Alternativen » publiziert. Wir freuen uns über die Aufmerksamkeit. Hier ist, was sie weggelassen haben.',
    cards: [
      { title: '« Hook0 ist nur HTTPS »', body: 'Ja, und das ist ein Feature, keine Einschränkung. Webhook-Payloads über reines HTTP zu senden bedeutet, dass die Daten deiner Kunden im Klartext durchs Netz wandern. Jedes ernsthafte Produktivsystem nutzt HTTPS. Wir erzwingen es, weil Sicherheit nicht optional ist.', color: 'green' },
      { title: '« Kein veröffentlichtes SLA »', body: 'Hook0 Cloud Enterprise enthält ein massgeschneidertes SLA mit dediziertem Support. Wenn Verfügbarkeitsgarantien wichtig sind, ist das der schnellste Weg, ohne eigene Infrastruktur, ohne Ops-Team. Hook0 ist auch Open Source (SSPL-1.0), also hast du immer die Option, selbst zu hosten, wenn deine Compliance-Anforderungen es verlangen.', color: 'indigo' },
      { title: '« Preise sind unklar »', body: 'Unsere Preise sind öffentlich und Event-basiert. Kein Verkaufsgespräch nötig. Keine « Kontakt »-Mauer. Cloud startet bei 59 €/Monat, 8x günstiger als Svix für vergleichbare Funktionen. Versuch mal, diese Transparenz von einem VC-finanzierten Wettbewerber zu bekommen, dessen echte Zahlen hinter einem Verkaufsgespräch liegen.', color: 'green' },
      { title: 'Was sie nicht erwähnen, die Finanzierung', body: 'Hookdeck hat 3,5 Mio. $ in VC eingesammelt. Svix 17 Mio. $. Convoy ist auch VC-finanziert. Hook0 ist zu 100% bootstrappt. Wenn dein Webhook-Anbieter den Umsatz verzehnfachen muss, um Investoren zu befriedigen, rate mal, wessen Preise steigen. Nicht unsere.', color: 'indigo' },
    ],
  },
  difference: {
    eyebrow: 'Warum Hook0',
    h2: 'Der Hook0-Unterschied',
    cards: [
      { title: 'Eine Plattform, eine Codebasis', body: 'Hookdeck teilt die Aufgabe auf: Das Event Gateway für die Ingestion bleibt geschlossen und nur in der Cloud, während Outpost die ausgehende Zustellung unter Apache-2.0 übernimmt. Hook0 liefert Versand, Wiederholungen, Signaturen und das Abonnenten-Portal als eine Plattform, und der Code, den du selbst hostest, ist der Code, den wir betreiben.' },
      { title: 'Keine Enterprise-Paywall', body: 'Anders als Svix wird jedes Feature in jedem Tarif ausgeliefert. Selbst-Hosting steckt nicht hinter einem Verkaufsgespräch.' },
      { title: 'Europäisch, auf DSGVO ausgelegt', body: 'Datenebene in der EU bei Clever Cloud (Frankreich) gehostet. CDN über Cloudflare (USA), im <a href="/de/auftragsverarbeitungsvertrag">DPA</a> und in den <a href="/de/dsgvo-unterauftragsverarbeiter">Unterauftragsverarbeitern</a> offengelegt. Bootstrapped, kein US-VC-Board, das über deine Datenpolitik entscheidet.' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Was sind die besten Hook0-Alternativen?', a: 'Die Haupt-Alternativen zu Hook0 sind Svix (Open Core, MIT-Kern mit geschlossenen Enterprise-Funktionen, VC-finanziert), Hookdeck (kanadisches Unternehmen: das Event Gateway für die Ingestion bleibt geschlossen und nur in der Cloud, während Outpost, seine Zustell-Engine, Apache-2.0 und selbst hostbar ist) und Convoy (quelloffen verfügbar unter der Elastic License 2.0, wie Hook0s SSPL-1.0, aktiv gepflegt vom Unternehmen frain-dev, VC-finanziert). Jede löst einen anderen Teil des Webhook-Problems. Hook0 ist die einzige, die vollständig Open Source (SSPL-1.0), bootstrappt und kostenlos selbst-hostbar ist.' },
      { q: 'Ist Hookdeck besser als Hook0?', a: 'Hookdeck ist ein Webhook-Gateway, es proxyt bestehende Webhooks für Zuverlässigkeit. Hook0 ist eine Webhook-Plattform, es sendet Webhooks für dich mit Wiederholungen, Signaturen und Abonnenten-Verwaltung. Sie lösen verschiedene Probleme. Wenn du Webhooks zu deinem Produkt hinzufügen musst, ist Hook0 das richtige Tool.' },
      { q: 'Soll ich Svix oder Hook0 nutzen?', a: 'Beide sind Webhook-Plattformen, aber sie unterscheiden sich in Lizenzierung und Finanzierung. Svix ist Open Core (Enterprise-Features sind geschlossen) und hat 17 Mio. $ VC eingesammelt. Hook0 ist vollständig Open Source unter SSPL, bootstrappt, und bietet kostenloses Selbst-Hosting. Wenn dir Anbieter-Unabhängigkeit und langfristige Preisstabilität wichtig sind, ist Hook0 die sicherere Wahl.' },
      { q: 'Was kostet Hook0?', a: 'Hook0 hat einen kostenlosen Tarif ohne Kreditkarte. Hook0 ist auch Open Source und selbst-hostbar für Compliance-Anforderungen. Hook0 Cloud ergänzt um verwaltete Infrastruktur, EU-Hosting, automatische Updates und Priority-Support. Bezahlte Tarife starten bei 59 €/Monat mit Event-basierter Abrechnung.' },
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
