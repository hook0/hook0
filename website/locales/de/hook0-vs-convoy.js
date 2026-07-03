// Per-page strings for hook0-vs-convoy (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// SSPL für Hook0 = « quelloffen (SSPL-1.0) ». Convoy = MPL-2.0 (OSI), also « Open Source » OK für Convoy.
module.exports = {
  pageTitle: 'Hook0 vs Convoy: Webhook-Plattformen im Vergleich | Hook0',
  pageDescription: 'Vergleich Hook0 (Rust, SSPL-1.0, Managed Cloud) und Convoy (Go, MPL-2.0, nur Selbst-Hosting): Features und Kompromisse Seite an Seite.',
  pageModified: '2026-06-27',
  breadcrumb: 'Hook0 vs. Convoy',
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Convoy',
    titleAccent: 'Gleiches Problem, andere Kompromisse',
    subtitle: 'Beide quelloffen. Beide auf PostgreSQL gebaut. Aber die Gemeinsamkeiten enden hier, Rust vs Go, Managed Cloud vs nur Selbst-Hosting, SSPL-1.0 vs MPL-2.0. Diese Seite zerlegt, was wirklich zählt, wenn du eine für die Produktion auswählst.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  differentiators: {
    eyebrow: 'Warum Hook0',
    h2: 'Wichtige Unterschiede',
    cards: [
      { title: 'Managed Cloud vs nur Selbst-Hosting', body: 'Convoy ist nur selbst-gehostet. Keine Managed Cloud, Punkt. Du betreibst es, du wartest es. Hook0 lässt dir die Wahl, entweder die Managed Cloud (in Europa gehostet) oder kostenloses Selbst-Hosting mit Docker oder Kubernetes.' },
      { title: 'Rust vs Go', body: 'Hook0 ist in Rust geschrieben. Kein Garbage Collector heißt keine GC-Pausen, weniger Speicherverbrauch und vorhersehbarere Latenz unter Last. Convoy ist in Go geschrieben, guter Durchsatz, aber mit Garbage Collection. Bei hohem Volumen zeigt sich der Unterschied in den Tail-Latenzen.' },
      { title: 'SSPL vs MPL-2.0', body: 'Convoy verwendet MPL-2.0. Sehr permissiv, keine Einschränkungen bei der Weitergabe. Hook0 verwendet SSPL-1.0, der gesamte Quellcode ist verfügbar, aber Cloud-Anbieter können ihn nicht als konkurrierenden Service weiterverkaufen. Beide sind quelloffen. Der Unterschied betrifft, was Dritte mit dem Code anstellen dürfen.' },
      { title: 'Europäisches Hosting vs DIY-Infrastruktur', body: 'Die Hook0-Cloud fährt ihre Datenebene in Frankreich bei Clever Cloud (CDN Cloudflare USA im <a href="/de/auftragsverarbeitungsvertrag">DPA</a> offengelegt), auf DSGVO-Konformität ausgelegt. Bei Convoy wählst du deinen Hosting-Standort selbst, aber du übernimmst auch den gesamten Ops-Stack, Monitoring, Backups, Skalierung, Uptime. Es gibt keine Managed-Option, also liegt alles bei dir.' },
    ],
  },
  comparison: {
    eyebrow: 'Funktionsvergleich',
    h2: 'Seite an Seite',
    headers: { feature: 'Funktion', hook0: 'Hook0', convoy: 'Convoy' },
    rows: [
      { feature: 'Lizenz', hook0Html: 'SSPL-1.0 (gesamter Quellcode verfügbar)', convoyHtml: 'MPL-2.0' },
      { feature: 'Sprache', hook0Html: 'Rust', convoyHtml: 'Go' },
      { feature: 'Datenbank', hook0Html: 'Nur PostgreSQL', convoyHtml: 'PostgreSQL + Redis' },
      { feature: 'Managed Cloud', hook0Html: 'Ja (Clever Cloud FR, CDN Cloudflare USA)', convoyHtml: 'Nein' },
      { feature: 'Selbst-Hosting', hook0Html: 'Kostenlos (Docker / K8s)', convoyHtml: 'Ja (einzige Option)' },
      { feature: 'Kostenloser Tarif', hook0Html: 'Ja (Cloud)', convoyHtml: 'N/A (nur Selbst-Hosting)' },
      { feature: 'HMAC-Signaturen', hook0Html: 'Ja', convoyHtml: 'Ja' },
      { feature: 'Wiederholungslogik', hook0Html: 'Konfigurierbar 2-phasig (schnell + langsam, smarte Defaults)', convoyHtml: 'Konfigurierbar' },
      { feature: 'Primäres Repo', hook0Html: '<a href="https://github.com/hook0/hook0" class="underline">GitHub</a> + <a href="https://gitlab.com/hook0/hook0" class="underline">GitLab</a>', convoyHtml: '<a href="https://github.com/frain-dev/convoy" class="underline">GitHub</a> (~2,8k Stars)' },
      { feature: 'Finanzierung', hook0Html: '100% bootstrappt', convoyHtml: 'VC-finanziert (Frain Technologies)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Convoy vollständig quelloffen?', a: 'Ja. Convoy verwendet die MPL-2.0-Lizenz, Hook0 verwendet SSPL-1.0. Beide veröffentlichen ihren vollständigen Quellcode. Der praktische Unterschied liegt in der Weitergabe, MPL-2.0 hat weniger Einschränkungen, während SSPL-1.0 Cloud-Anbieter daran hindert, die Software als konkurrierenden Managed Service anzubieten.' },
      { q: 'Hat Convoy eine Managed Cloud?', a: 'Nein. Convoy ist nur selbst-gehostet, du betreibst und wartest alles selbst. Hook0 hat eine Managed Cloud (in Europa gehostet) und unterstützt auch kostenloses Selbst-Hosting mit Docker oder Kubernetes.' },
      { q: 'Wie schlagen sich Hook0 und Convoy bei der Performance?', a: 'Hook0 ist in Rust geschrieben, also keine Garbage-Collection-Pausen. Das bedeutet vorhersehbarere Latenz und weniger Speicherverbrauch unter Last. Convoy ist in Go geschrieben, das gut performt, aber GC-Overhead hat. Infrastrukturseitig brauchen beide PostgreSQL, Convoy benötigt zusätzlich Redis.' },
      { q: 'Was ist besser für Selbst-Hosting?', a: 'Beide können selbst-gehostet werden, aber bei Convoy ist das deine einzige Option. Hook0 unterstützt Docker Compose und Kubernetes für kostenloses Selbst-Hosting und hat zudem eine Managed Cloud, falls du die Ops-Arbeit lieber überspringst. Ein praktischer Unterschied, Hook0 braucht nur PostgreSQL. Convoy benötigt PostgreSQL und Redis.' },
    ],
  },
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Hook0-Alternativen' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst-gehostete Webhooks' },
    ],
  },
};
