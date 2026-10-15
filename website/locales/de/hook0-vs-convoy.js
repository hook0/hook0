// Per-page strings for hook0-vs-convoy (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// SSPL für Hook0 = « Open Source (SSPL-1.0) ». Convoy = Elastic License v2.0 (nicht OSI),
// also « quellverfügbar », niemals « Open Source » ohne Einschränkung.
// Fakten aktualisiert 2026-09-29 (Wettbewerbs-Snapshot): Convoy AKTIV (v26.8.0 vom 28.09.2026,
// tägliche Commits, ~2.875 Stars). Preise geprüft am 2026-10-15: Convoy Cloud Pro 99 $/Monat flat
// (25 Events/s), Premium 499 $/Monat, US- oder EU-Region bei der Anmeldung; selbst gehostetes Premium 999 $/Monat.
module.exports = {
  pageTitle: 'Hook0 vs Convoy: Webhook-Plattformen im Vergleich | Hook0',
  pageDescription: 'Vergleich Hook0 (Rust, SSPL-1.0, EU-gehostete Cloud ab 59 €/Monat) und Convoy (Go, Elastic License v2.0, Cloud ab 99 $/Monat): Features, Lizenzen und Preise Seite an Seite.',
  pageModified: '2026-10-15',
  breadcrumb: 'Hook0 vs. Convoy',
  tldr: {
    h2: 'Kurz gesagt',
    body: "Hook0 und Convoy veröffentlichen beide ihren vollständigen Quellcode und laufen auf PostgreSQL, die echten Kompromisse liegen also woanders. Hook0 ist in Rust unter SSPL-1.0 geschrieben, mit einer managed EU-Cloud ab 59 €/Monat und einer Pro-Stufe bei 190 €. Convoy ist in Go unter der Elastic License v2.0 geschrieben. Seine Cloud hat einen 14-tägigen Test, danach Pro für pauschal 99 $/Monat bei 25 Events pro Sekunde und Premium für 499 $/Monat, in einer US- oder EU-Region; die selbst gehostete Premium-Lizenz kostet 999 $/Monat. Wähle Convoy für Inbound und Outbound in einem Produkt und Pauschalpreise nach Durchsatz; wähle Hook0 für eine Rust-Datenebene, einen dauerhaft kostenlosen Tarif, einen französischen Betreiber und einen selbst gehosteten Server mit allen Funktionen.",
  },
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Convoy',
    titleAccent: 'Gleiches Problem, andere Kompromisse',
    subtitle: 'Beide veröffentlichen ihren vollständigen Quellcode. Beide auf PostgreSQL gebaut. Die echten Unterschiede liegen woanders, Rust vs Go, SSPL-1.0 vs Elastic License v2.0, und tägliche Event-Kontingente gegenüber Pauschaltarifen nach Durchsatz. Diese Seite zerlegt, was wirklich zählt, wenn du eine für die Produktion auswählst.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  differentiators: {
    eyebrow: 'Warum Hook0',
    h2: 'Wichtige Unterschiede',
    cards: [
      { title: 'Event-Kontingente vs Durchsatz-Tarife', body: 'Convoy berechnet seine Cloud nach Durchsatz: Pro kostet pauschal 99 $/Monat für 25 Events pro Sekunde, Premium 499 $/Monat mit individuellen Limits, und nach dem 14-tägigen Test bleibt nichts kostenlos. Selbst gehostet kosten die Premium-Funktionen (JS-Transformationen, RBAC, White-Labeling) 999 $/Monat. Hook0 Cloud rechnet nach Tagesvolumen ab: kostenlos bis 100 Events pro Tag, Startup für 59 €/Monat, Pro für 190 €/Monat bei 100.000 Events pro Tag, und Mehrverbrauch wird pro Event berechnet, statt die Zustellung zu blockieren. Der selbst gehostete Hook0-Server enthält alle Funktionen ohne Lizenzkosten.' },
      { title: 'Eine EU-Region vs ein französischer Betreiber', body: 'Die Managed Cloud von Hook0 fährt ihre Anwendungs-Datenebene in Frankreich bei Clever Cloud (das US-CDN Cloudflare ist in unserem <a href="/de/auftragsverarbeitungsvertrag" class="underline">DPA</a> offengelegt), auf DSGVO-Konformität ausgelegt, und Hook0 wird von einem französischen Unternehmen betrieben. Bei Convoy Cloud wählst du bei der Anmeldung eine US- oder EU-Region, Optionen zur Datenresidenz stehen im Enterprise-Tarif; die Firmenadresse von Convoy liegt in San Francisco. Wo deine Daten liegen und welches Recht für den Anbieter gilt, sind zwei getrennte Fragen.' },
      { title: 'Rust vs Go', body: 'Hook0 ist in Rust geschrieben. Kein Garbage Collector heißt keine GC-Pausen, weniger Speicherverbrauch und vorhersehbarere Latenz unter Last. Convoy ist in Go geschrieben, guter Durchsatz, aber mit Garbage Collection. Bei hohem Volumen zeigt sich der Unterschied in den Tail-Latenzen.' },
      { title: 'SSPL-1.0 vs Elastic License v2.0', body: 'Convoy verwendet die Elastic License v2.0, der gesamte Quellcode ist verfügbar, aber Convoy als Managed Service anzubieten erfordert eine kommerzielle Vereinbarung. Hook0 verwendet SSPL-1.0, der gesamte Quellcode ist verfügbar, aber Cloud-Anbieter können ihn nicht als konkurrierenden Service weiterverkaufen. Beide sind quellverfügbare Lizenzen und keine von beiden ist OSI-anerkannt. Der praktische Unterschied liegt darin, welche Aktivität eingeschränkt wird, nicht darin, wie viel Code du lesen kannst.' },
    ],
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Seite an Seite',
    headers: { feature: 'Funktion', hook0: 'Hook0', convoy: 'Convoy' },
    rows: [
      { feature: 'Lizenz', hook0Html: 'SSPL-1.0 (gesamter Quellcode verfügbar, nicht OSI-anerkannt)', convoyHtml: 'Elastic License v2.0 (gesamter Quellcode verfügbar, nicht OSI-anerkannt)' },
      { feature: 'Sprache', hook0Html: 'Rust', convoyHtml: 'Go' },
      { feature: 'Datenbank', hook0Html: 'Nur PostgreSQL', convoyHtml: 'PostgreSQL + Redis' },
      { feature: 'Webhook-Richtung', hook0Html: 'Ausgehend (Versand)', convoyHtml: 'Ausgehend + eingehend' },
      { feature: 'Managed Cloud', hook0Html: 'Ja (Clever Cloud FR, US-CDN Cloudflare offengelegt)', convoyHtml: 'Ja (US- oder EU-Region, bei der Anmeldung gewählt)' },
      { feature: 'Selbst-Hosting', hook0Html: 'Kostenlos (Docker / K8s)', convoyHtml: 'Kostenlos (Community-Tarif)' },
      { feature: 'Bezahlte Tarife', hook0Html: 'Startup 59 €/Monat, Pro 190 €/Monat', convoyHtml: 'Cloud Pro 99 $/Monat, Cloud Premium 499 $/Monat, selbst gehostetes Premium 999 $/Monat, Enterprise auf Anfrage' },
      { feature: 'SOC 2', hook0Html: 'Geplant', convoyHtml: 'SOC 2 Type 1' },
      { feature: 'HMAC-Signaturen', hook0Html: 'Ja', convoyHtml: 'Ja' },
      { feature: 'Wiederholungslogik', hook0Html: 'Konfigurierbar 2-phasig (schnell + langsam, smarte Defaults)', convoyHtml: 'Konfigurierbar' },
      { feature: 'Primäres Repo', hook0Html: '<a href="https://github.com/hook0/hook0" class="underline">GitHub</a> + <a href="https://gitlab.com/hook0/hook0" class="underline">GitLab</a>', convoyHtml: '<a href="https://github.com/frain-dev/convoy" class="underline">GitHub</a> (~2,9k Stars)' },
      { feature: 'Finanzierung', hook0Html: '100% bootstrappt', convoyHtml: 'VC-finanziert (YC W22, Frain Technologies)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Convoy Open Source?', a: 'Convoy veröffentlicht seinen vollständigen Quellcode unter der Elastic License v2.0, die keine OSI-anerkannte Open-Source-Lizenz ist, sie verbietet es, Convoy ohne kommerzielle Vereinbarung als Managed Service anzubieten. Hook0 gehört zur selben Familie, vollständiger Quellcode unter SSPL-1.0, ebenfalls nicht OSI-anerkannt, mit einer Einschränkung gegen Cloud-Anbieter, die ihn weiterverkaufen würden. Wenn deine Einkaufsrichtlinie strikt eine OSI-Lizenz verlangt, qualifiziert sich keiner von beiden.' },
      { q: 'Hat Convoy eine Managed Cloud?', a: 'Ja. Convoy Cloud hat einen 14-tägigen Test (1 Projekt, 100 Events pro Tag), danach Pro für pauschal 99 $/Monat bei 25 Events pro Sekunde und Premium für 499 $/Monat mit individuellen Limits. Die US- oder EU-Region wählst du bei der Anmeldung. Die Managed Cloud von Hook0 ist ab dem kostenlosen Tarif EU-gehostet, mit bezahlten Tarifen für 59 € und 190 € pro Monat.' },
      { q: 'Wie vergleichen sich Hook0 und Convoy beim Preis?', a: 'Selbst gehostet sind beide Server kostenlos, aber Convoys Premium-Funktionen kosten 999 $/Monat, während der Hook0-Server alle Funktionen enthält. In der Cloud berechnet Convoy einen Pauschalpreis pro Durchsatzstufe: 99 $/Monat für 25 Events pro Sekunde, 499 $/Monat für individuelle Limits. Hook0 Cloud ist bis 100 Events pro Tag kostenlos, dann 59 €/Monat für 30.000 Events pro Tag und 190 €/Monat für 100.000, mit Mehrverbrauch pro Event. Bei hohem, gleichmäßigem Volumen kann Convoy Pro günstiger sein; bei kleinem oder wachsendem Volumen und für einen kostenlosen Tarif ohne Ablaufdatum ist Hook0 günstiger.' },
      { q: 'Wie schlagen sich Hook0 und Convoy bei der Performance?', a: 'Hook0 ist in Rust geschrieben, also keine Garbage-Collection-Pausen. Das bedeutet vorhersehbarere Latenz und weniger Speicherverbrauch unter Last. Convoy ist in Go geschrieben, das gut performt, aber GC-Overhead hat. Infrastrukturseitig brauchen beide PostgreSQL, Convoy benötigt zusätzlich Redis.' },
      { q: 'Was macht Convoy besser als Hook0?', a: 'Convoy verarbeitet eingehende und ausgehende Webhooks in einem Produkt, während Hook0 sich auf die ausgehende Zustellung konzentriert. Convoy hat außerdem eine SOC-2-Type-1-Attestierung, mehr GitHub-Stars (~2.875), öffentlich genannte Fintech-Referenzkunden sowie pauschale Cloud-Tarife nach Durchsatz ab 99 $/Monat, die manche Teams wegen der planbaren Abrechnung bevorzugen.' },
      { q: 'Wird Convoy noch gepflegt?', a: 'Ja. Convoy liefert 2 bis 3 Releases pro Monat (v26.8.0 erschien im September 2026), der Blog ist aktiv und das GitHub-Repository hat mehrere regelmäßige Mitwirkende. Jede Behauptung « Convoy ist tot », die du online findest, ist veraltet.' },
    ],
  },
  comparisonSources: {
    competitors: ["Convoy"],
    scope: "Diese Seite vergleicht die öffentlichen Tarife, die Lizenz und den Wartungsstand von Hook0 und Convoy. Die Preise sind die veröffentlichten Angaben der Anbieter, und der Wartungsstand stammt aus Convoys öffentlicher GitHub-Aktivität.",
    sources: [
      { label: "Convoy Releases auf GitHub", url: "https://github.com/frain-dev/convoy/releases", consulted: "2026-09-29" },
      { label: "Convoy Preise", url: "https://www.getconvoy.io/pricing", consulted: "2026-10-15", archive: "https://web.archive.org/web/20260604143101/https://www.getconvoy.io/pricing" },
      { label: "Convoy Lizenz auf GitHub", url: "https://github.com/frain-dev/convoy/blob/main/LICENSE", consulted: "2026-10-04" },
    ],
    updated: "2026-10-15",
  },
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Hook0-Alternativen' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst-gehostete Webhooks' },
      { enSlug: 'webhook-cost-comparison', label: 'Webhook-Kostenvergleich (auf Englisch)' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU-Webhook-Infrastruktur (auf Englisch)' },
      { enSlug: 'sovereign-webhooks', label: 'Souveräne Webhooks' },
    ],
  },
};
