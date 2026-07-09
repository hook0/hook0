// Per-page strings for hook0-vs-convoy (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// SSPL für Hook0 = « quelloffen (SSPL-1.0) ». Convoy = Elastic License v2.0 (nicht OSI),
// also « quellverfügbar », niemals « Open Source » ohne Einschränkung.
// Fakten aktualisiert 2026-07-08 (Wettbewerbs-Snapshot): Convoy AKTIV (v26.6.2 vom 08.07.2026),
// Cloud ohne öffentliche Preise und ohne Managed-EU-Residenz, Preise 0 $ -> 999 $/Monat flat.
module.exports = {
  pageTitle: 'Hook0 vs Convoy: Webhook-Plattformen im Vergleich | Hook0',
  pageDescription: 'Vergleich Hook0 (Rust, SSPL-1.0, EU-gehostete Cloud ab 59 €/Monat) und Convoy (Go, Elastic License v2.0, 0 bis 999 $/Monat): Features, Lizenzen und Preise Seite an Seite.',
  pageModified: '2026-07-08',
  breadcrumb: 'Hook0 vs. Convoy',
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Convoy',
    titleAccent: 'Gleiches Problem, andere Kompromisse',
    subtitle: 'Beide veröffentlichen ihren vollständigen Quellcode. Beide auf PostgreSQL gebaut. Die echten Unterschiede liegen woanders, Rust vs Go, SSPL-1.0 vs Elastic License v2.0, und eine gestaffelte Preisstruktur gegenüber einer Flatrate von 999 $ im Monat. Diese Seite zerlegt, was wirklich zählt, wenn du eine für die Produktion auswählst.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  differentiators: {
    eyebrow: 'Warum Hook0',
    h2: 'Wichtige Unterschiede',
    cards: [
      { title: 'Ein Zwischentarif vs ein Sprung von 0 auf 999 $', body: 'Die bezahlte Leiter von Convoy hat eine einzige Stufe, der Community-Tarif ist kostenlos, danach kommt Premium für 999 $/Monat flat (JS-Transformationen, RBAC und White-Labeling inklusive). Dazwischen gibt es nichts. Hook0 Cloud startet kostenlos, dann Startup für 59 €/Monat und Pro für 190 €/Monat, ein wachsendes Team steht nie vor einer abrupten Preisklippe ohne Zwischenstufe.' },
      { title: 'Managed EU-Cloud vs Selbst-Hosting für Datenresidenz', body: 'Die Managed Cloud von Hook0 fährt ihre Anwendungs-Datenebene in Frankreich bei Clever Cloud (das US-CDN Cloudflare ist in unserem <a href="/de/auftragsverarbeitungsvertrag" class="underline">DPA</a> offengelegt), auf DSGVO-Konformität ausgelegt. Convoy hat ebenfalls ein Cloud-Angebot, aber keine Managed-Option für EU-Datenresidenz, die Region deiner Webhook-Daten wählen heißt selbst hosten, also übernimmst du auch Monitoring, Backups, Skalierung und Uptime.' },
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
      { feature: 'Managed Cloud', hook0Html: 'Ja (Clever Cloud FR, US-CDN Cloudflare offengelegt)', convoyHtml: 'Ja (keine öffentlichen Preise, keine Managed-EU-Residenz)' },
      { feature: 'Selbst-Hosting', hook0Html: 'Kostenlos (Docker / K8s)', convoyHtml: 'Kostenlos (Community-Tarif)' },
      { feature: 'Bezahlte Tarife', hook0Html: 'Startup 59 €/Monat, Pro 190 €/Monat', convoyHtml: 'Premium 999 $/Monat (flat), Enterprise auf Anfrage' },
      { feature: 'SOC 2', hook0Html: 'Geplant', convoyHtml: 'SOC 2 Type 1' },
      { feature: 'HMAC-Signaturen', hook0Html: 'Ja', convoyHtml: 'Ja' },
      { feature: 'Wiederholungslogik', hook0Html: 'Konfigurierbar 2-phasig (schnell + langsam, smarte Defaults)', convoyHtml: 'Konfigurierbar' },
      { feature: 'Primäres Repo', hook0Html: '<a href="https://github.com/hook0/hook0" class="underline">GitHub</a> + <a href="https://gitlab.com/hook0/hook0" class="underline">GitLab</a>', convoyHtml: '<a href="https://github.com/frain-dev/convoy" class="underline">GitHub</a> (~2,8k Stars)' },
      { feature: 'Finanzierung', hook0Html: '100% bootstrappt', convoyHtml: 'VC-finanziert (YC W22, Frain Technologies)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Convoy Open Source?', a: 'Convoy veröffentlicht seinen vollständigen Quellcode unter der Elastic License v2.0, die keine OSI-anerkannte Open-Source-Lizenz ist, sie verbietet es, Convoy ohne kommerzielle Vereinbarung als Managed Service anzubieten. Hook0 gehört zur selben Familie, vollständiger Quellcode unter SSPL-1.0, ebenfalls nicht OSI-anerkannt, mit einer Einschränkung gegen Cloud-Anbieter, die ihn weiterverkaufen würden. Wenn deine Einkaufsrichtlinie strikt eine OSI-Lizenz verlangt, qualifiziert sich keiner von beiden.' },
      { q: 'Hat Convoy eine Managed Cloud?', a: 'Ja. Convoy bietet eine Cloud-Version an (der Trial umfasst 1 Projekt und 100 Events pro Tag), veröffentlicht aber keine Cloud-Preise, und es gibt keine Managed-Option für EU-Datenresidenz, die Region deiner Webhook-Daten wählen heißt selbst hosten. Die Managed Cloud von Hook0 ist ab dem kostenlosen Tarif EU-gehostet, mit bezahlten Tarifen für 59 € und 190 € pro Monat.' },
      { q: 'Wie vergleichen sich Hook0 und Convoy beim Preis?', a: 'Beim Selbst-Hosting sind beide kostenlos. Für bezahlte Funktionen springt Convoy direkt vom kostenlosen Community-Tarif auf Premium für 999 $/Monat flat, dazwischen gibt es nichts. Hook0 Cloud hat einen kostenlosen Tarif, dann Startup für 59 €/Monat und Pro für 190 €/Monat. Wenn eine pauschale All-inclusive-Rechnung zu deinem Team passt, ist Convoys Premium vorhersehbar. Wenn du klein starten und wachsen willst, deckt Hook0 die Marktmitte ab, die Convoy überspringt.' },
      { q: 'Wie schlagen sich Hook0 und Convoy bei der Performance?', a: 'Hook0 ist in Rust geschrieben, also keine Garbage-Collection-Pausen. Das bedeutet vorhersehbarere Latenz und weniger Speicherverbrauch unter Last. Convoy ist in Go geschrieben, das gut performt, aber GC-Overhead hat. Infrastrukturseitig brauchen beide PostgreSQL, Convoy benötigt zusätzlich Redis.' },
      { q: 'Was macht Convoy besser als Hook0?', a: 'Convoy verarbeitet eingehende und ausgehende Webhooks in einem Produkt, während Hook0 sich auf die ausgehende Zustellung konzentriert. Convoy hat außerdem eine SOC-2-Type-1-Attestierung, mehr GitHub-Stars (~2 800), Fintech-Referenzkunden wie Xendit und PiggyVest sowie einen pauschalen Premium-Tarif für 999 $/Monat, den manche Teams wegen der planbaren Abrechnung bevorzugen.' },
      { q: 'Wird Convoy noch gepflegt?', a: 'Ja. Convoy liefert 2 bis 3 Releases pro Monat (v26.6.2 erschien im Juli 2026), der Blog ist aktiv und das GitHub-Repository hat mehrere regelmäßige Mitwirkende. Jede Behauptung « Convoy ist tot », die du online findest, ist veraltet.' },
    ],
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
    ],
  },
};
