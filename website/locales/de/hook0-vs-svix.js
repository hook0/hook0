// Per-page strings for hook0-vs-svix (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Middle-Dot.
// Hook0 = « quelloffen (SSPL-1.0) », NIEMALS « Open Source » (SSPL nicht OSI, UWG §5 Risiko).
// Svix Kern MIT = OSI, also « Open Source » OK für Svix.
// Keine absoluten DSGVO-Claims im Body. Keine Behauptung « kein US-Konzern im Stack ».
module.exports = {
  pageTitle: 'Hook0 vs Svix: Webhook-Plattformen im Vergleich | Hook0',
  pageDescription: 'Vergleich Hook0 und Svix: SSPL-1.0 vs Open-Core, bootstrappt vs VC, EU vs US, Selbst-Hosting in jedem Tarif. Ehrlich vergleichen.',
  pageModified: '2026-07-16',
  breadcrumb: 'Hook0 vs. Svix',
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Svix',
    titleAccent: 'Webhook-Plattformen im Vergleich',
    subtitle: 'Suchst du eine Alternative zu Svix? Beide sind Webhook-Plattformen, aber sie unterscheiden sich bei Lizenz, Finanzierungsmodell, Hosting und dem, was « Open Source » in der Praxis wirklich heißt. Hook0 ist quelloffen (SSPL-1.0), bootstrappt, in der EU gehostet, ohne Anbieter-Lock-in.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  differentiators: {
    eyebrow: 'Warum Hook0',
    h2: 'Wichtige Unterschiede',
    cards: [
      { title: 'Quellcode verfügbar, keine geschlossenen Add-ons', body: 'Der Hook0-Server wird unter SSPL-1.0 veröffentlicht, die SDKs unter MIT. Du bekommst die gesamte Plattform, du liest sie, änderst sie, hostest sie selbst. Der Svix-Kern ist MIT, aber die Enterprise-Funktionen (SSO, erweiterte Analytics, dedizierter Support) bleiben im geschlossenen Bereich der bezahlten Tarife.' },
      { title: 'Bootstrapped seit Tag eins', body: 'Svix ist VC-finanziert. Investoren erwarten Rendite, das erzeugt Druck, Preise zu erhöhen oder übernommen zu werden. Hook0 ist 100% bootstrappt. Kein Board zu zufriedenstellen, kein Mandat für Wachstum um jeden Preis.' },
      { title: 'Kein Anbieter-Lock-in', body: 'Hook0 Cloud betreibt denselben quelloffenen Code, den du lesen und prüfen kannst. Falls du es irgendwann brauchst, exportierst du und betreibst es selbst (kostenlos, Docker oder Kubernetes), du sitzt also nie in einer proprietären Plattform fest. Svix beschränkt das Selbst-Hosting auf Enterprise-Kunden.' },
      { title: 'EU-Datenebene, US-Edge offengelegt', body: "Die Datenebene von Hook0 läuft auf Clever Cloud in Frankreich, betrieben von einem französischen Unternehmen. Das vorgelagerte CDN ist Cloudflare (US), offengelegt in unserer öffentlichen Unterauftragsverarbeiter-Liste samt Übermittlungsmechanismus. Svix ist in den USA ansässig. Und weil derselbe Code selbst gehostet werden kann, können Sie Hook0 in Ihrem eigenen Netzwerk betreiben, wo keine Webhook-Daten es verlassen." },
    ],
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Seite an Seite',
    headers: { feature: 'Funktion', hook0: 'Hook0', svix: 'Svix' },
    rows: [
      { feature: 'Lizenz', hook0Html: 'SSPL-1.0 (gesamter Quellcode verfügbar)', svixHtml: 'MIT (Open-Core, Enterprise geschlossen)' },
      { feature: 'Finanzierung', hook0Html: '100% bootstrappt', svixHtml: 'VC-finanziert' },
      { feature: 'Selbst-Hosting', hook0Html: 'Kostenlos (Docker / K8s)', svixHtml: 'Nur Enterprise-Tarif' },
      { feature: 'Kostenloser Tarif', hook0Html: 'Ja, ohne Kreditkarte', svixHtml: 'Ja' },
      { feature: 'HMAC-Signaturen', hook0Html: 'Enthalten (alle Tarife)', svixHtml: 'Enthalten' },
      { feature: 'Wiederholungslogik', hook0Html: 'Konfigurierbar pro Subscription (schnelle + langsame Phasen, smarte Defaults)', svixHtml: 'Automatische Wiederholungen' },
      { feature: 'Datenhosting', hook0Html: 'Europa (Clever Cloud FR, CDN Cloudflare USA)', svixHtml: 'Sitz in den USA' },
      { feature: 'Subscription-Management', hook0Html: 'Enthalten', svixHtml: 'App Portal (bezahlte Tarife)' },
      { feature: 'Risiko von Anbieter-Lock-in', hook0Html: 'Keines (gesamter Quellcode, selbst-hostbar)', svixHtml: 'Moderat (Enterprise-Funktionen geschlossen)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    lastReviewed: 'Zuletzt geprüft Juli 2026.',
    items: [
      { q: 'Ist Hook0 quelloffen wie Svix?', a: 'Der Hook0-Server wird unter SSPL-1.0 veröffentlicht und die Client-SDKs unter MIT, ohne proprietäre Enterprise-Stufe. SSPL ist eine Copyleft-Lizenz mit verfügbarem Quellcode, du darfst die gesamte Plattform frei lesen, ändern und selbst hosten. Der Svix-Kern ist MIT, aber mehrere Enterprise-Funktionen sind geschlossen und nur in bezahlten Tarifen verfügbar.' },
      { q: 'Wie verhält sich der kostenlose Tarif von Hook0 zu dem von Svix?', a: 'Der kostenlose Tarif von Hook0 bleibt für immer kostenlos, ohne Kreditkarte, 100 Events pro Tag, HMAC-Signaturen und Zustellungs-Monitoring, in der EU gehostet. Bezahlte Tarife wachsen mit deinem Volumen auf derselben verwalteten Infrastruktur, jede Funktion ist enthalten, keine Enterprise-Paywall. Svix behält mehrere Funktionen den bezahlten Tarifen vor.' },
      { q: 'Unterstützt Hook0 Standard Webhooks?', a: 'Standard Webhooks ist eine Spezifikation, die von Svix verfasst wurde. Hook0 signiert jede Payload mit HMAC-SHA256 und dokumentiert das Schema. Der Support für Standard Webhooks ist geplant.' },
      { q: 'Kann ich Hook0 für regulierte oder compliance-sensitive Workloads verwenden?', a: "Ja, auch wenn formale Zertifizierungen noch ausstehen. Hook0 Cloud betreibt seine Datenebene auf Clever Cloud in Frankreich, betrieben von einem französischen Unternehmen, wobei das Cloudflare-CDN (US) samt Übermittlungsmechanismus in einer öffentlichen Unterauftragsverarbeiter-Liste offengelegt ist. Da der vollständige Server-Quellcode offen ist (SSPL-1.0), können Sie genau prüfen, wie Daten verarbeitet werden, und Sie können selbst hosten, sodass keine Webhook-Daten Ihr eigenes Netzwerk verlassen. Prüfberichte Dritter wie SOC 2, HIPAA und PCI-DSS sind geplant." },
      { q: "Wo wird Hook0 gehostet, und wie geht es mit der US-Datenübermittlung um?", a: "Hook0 Cloud wird von einem französischen Unternehmen (FGRibreau SARL) betrieben, mit seiner Datenebene auf Clever Cloud in Frankreich. Die vorgelagerte CDN- und DDoS-Schicht stellt Cloudflare (US), das wir in einer öffentlichen Unterauftragsverarbeiter-Liste offenlegen, abgesichert durch die Standardvertragsklauseln von 2021 und ein Transfer Impact Assessment, statt eine fehlende US-Exposition zu behaupten. Svix und Hookdeck sind US-Unternehmen. Wenn Webhook-Daten vollständig in Ihrem eigenen Netzwerk bleiben sollen, hosten Sie Hook0 mit demselben Code selbst." },
      { q: 'Kann ich Hook0 kostenlos selbst hosten?', a: 'Ja. Derselbe quelloffene Code läuft kostenlos auf Docker Compose oder Kubernetes, was dich davor bewahrt, jemals eingesperrt zu sein. Die meisten Teams starten auf Hook0 Cloud (managed, EU-gehostet, kostenloser Tarif) und behalten Selbst-Hosting als Ausstiegsoption. Svix bietet Selbst-Hosting nur in seinem Enterprise-Tarif.' },
      { q: 'Ist Hook0 bootstrappt?', a: 'Ja. Hook0 ist 100% bootstrappt, ohne jede VC-Finanzierung. Svix ist VC-finanziert. Bootstrapped heißt, dass Hook0 seinen Nutzern verpflichtet ist, nicht Investoren, die einen Exit suchen.' },
      { q: 'Betrachten Svix und Hookdeck Hook0 als Konkurrenten?', a: 'Svix und Hookdeck veröffentlichen beide Vergleichsseiten, die Hook0 einschließen. Du kannst ihre eigenen Einschätzungen neben unserer lesen.' },
    ],
  },
  deepDive: {
    prefix: 'Willst du mehr Details?',
    linkText: 'Lies den Funktion-für-Funktion-Vergleich in unserer Dokumentation',
    linkHref: 'https://documentation.hook0.com/comparisons/svix-vs-hook0',
  },
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'svix-alternatives', label: 'Svix-Alternativen' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Selbst bauen vs kaufen bei Webhooks' },
      { enSlug: 'webhook-cost-comparison', label: 'Webhook-Kostenvergleich (auf Englisch)' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU-Webhook-Infrastruktur (auf Englisch)' },
    ],
  },
};
