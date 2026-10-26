// Per-page strings for hook0-vs-requeue (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// Hook0 SSPL-1.0 = « Open Source (quellverfügbar, nicht OSI-geprüft) », niemals
// « das einzige echte Open Source ». Fakten neu erfasst am 2026-10-22 (Startseite,
// README und FAQ von Requeue auf GitHub, aufgeführt in comparisonSources).
// Strukturierende Nuance: Hook0 und Requeue liegen auf entgegengesetzten Seiten desselben
// Problems. Hook0 sendet (outbound), Requeue fängt Eingehendes ab (Inbound-Dead-Letter,
// auch über seine Relay-URL).
// Ehrlichkeit: nie behaupten, Hook0 mache heute produktives Inbound-Dead-Letter.
// Jedes nicht belegte Requeue-Attribut steht als « Nicht angegeben », niemals erfunden
// (Hosting-Region, Preise).
module.exports = {
  pageTitle: 'Hook0 vs Requeue: eingehende vs ausgehende Webhooks | Hook0',
  pageDescription: 'Requeue fängt empfangene Webhooks und fehlgeschlagene Jobs ab. Hook0 sendet Webhooks an deine Nutzer, mit Retries, HMAC und Replay, EU-gehostet.',
  pageModified: '2026-10-22',
  breadcrumb: 'Hook0 vs. Requeue',
  tldr: {
    h2: 'Kurz gesagt',
    body: "Hook0 und Requeue lösen zwei Seiten desselben Problems, in entgegengesetzten Richtungen. Requeue ist eine eingehende Dead-Letter-Inbox. Du richtest einen Stripe- oder Clerk-Webhook auf seine Relay-URL oder schickst ihm deine fehlgeschlagenen Cron- und Worker-Jobs, und es behält alles, was deine App nicht verarbeiten konnte, damit du es prüfen und erneut abspielen kannst. Hook0 ist ausgehendes Webhooks-as-a-Service. Es sendet Webhooks an deine eigenen Nutzer und steuert diese Zustellung, mit Protokoll für jeden Versuch, bis zu 24 kostenlosen Retries innerhalb von 8 Tagen, HMAC-Signaturen und Replay über Dashboard oder API. Nimm Requeue, um eingehende Fehler in einem kleinen Team abzufangen, nimm Hook0, um Webhooks an deine Kunden zuzustellen, EU-gehostet und Open Source. Hook0 bietet heute keine produktive eingehende Dead-Letter-Inbox, und die FAQ von Requeue sagt, dass es ausgehendes Webhooks-as-a-Service nicht ersetzt.",
  },
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Requeue',
    titleAccent: 'Entgegengesetzte Richtungen, gleiches Problem',
    subtitle: "Requeue fängt die Webhooks ab, die du empfängst, und die Cron- oder Worker-Jobs, die fehlschlagen, und spielt sie dann in deine App zurück. Hook0 macht das Gegenteil und sendet die Webhooks an deine Nutzer, inklusive Zustellsteuerung, Protokoll pro Versuch, kostenlosen Retries, HMAC-Signatur und Replay. Diese Seite zeigt, für welche Richtung jedes Tool gebaut ist, damit du auf einen Blick siehst, welches Problem du wirklich hast.",
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  differentiators: {
    eyebrow: 'Was jedes Tool macht',
    h2: 'Zentrale Unterschiede',
    cards: [
      { title: 'Eingehendes abfangen vs ausgehendes senden', body: "Requeue sitzt vor dem Traffic, der <em>hereinkommt</em>. Seine Relay-URL nimmt den Webhook eines Anbieters an, leitet den rohen Body und die Signatur-Header mit 10 Sekunden Timeout an deine App weiter und speichert die Anfrage, wenn deine App einen Fehler liefert oder nicht antwortet. Hook0 ist der Sender. Es veröffentlicht deine Events und stellt sie deinen Abonnenten zu, der Fehler, den es verwaltet, ist also eine <em>ausgehende</em> Zustellung, die nicht angekommen ist." },
      { title: 'Abfangen, inspizieren, zurückspielen, auf der Sendeseite', body: "Die Story abfangen / inspizieren / zurückspielen, die Requeue für Eingehendes erzählt, macht Hook0 bereits für Ausgehendes. Jeder Zustellversuch wird mit Statuscode und Antwort protokolliert, Hook0 versucht es standardmäßig bis zu 24 Mal innerhalb von 8 Tagen erneut, und wenn der letzte Retry scheitert, bleibt der Versuch auf <code>failed</code>, sodass du das Event über Dashboard oder API erneut abspielst. Siehe <a href=\"https://documentation.hook0.com/explanation/webhook-dead-letter-queue\" class=\"text-indigo-400 hover:text-indigo-300 underline\">was nach dem letzten Retry mit einem Webhook passiert</a> (auf Englisch)." },
      { title: 'EU-Residenz und DSGVO standardmäßig', body: "Hook0 wird von einem französischen Unternehmen ohne US-Muttergesellschaft, US-Tochtergesellschaft oder US-Niederlassung herausgegeben, und seine managed Cloud läuft standardmäßig in der EU mit HMAC-Signaturen und TLS. Der Core von Requeue läuft auf Cloudflare Workers und Cloudflare D1, und sein öffentliches Material nennt nicht, wo die gehostete Instanz Daten speichert. Für EU-Residenz mit Requeue deployst du den Worker in deinem eigenen Cloudflare-Konto und wählst den Standort selbst." },
      { title: 'Open Source, kostenlose Retries, öffentliche Preise', body: "Der Server von Hook0 ist Open Source unter SSPL-1.0 (quellverfügbar, nicht OSI-geprüft) und seine 11 Client-SDKs sind MIT. Retries sind kostenlos, die HMAC-Signatur ist eingebaut, es gibt keine Abrechnung pro Endpoint, und die Preise sind öffentlich, mit einem kostenlosen Developer-Plan. Core und JavaScript-SDK von Requeue stehen unter MIT, Version 0.1.0 im Oktober 2026, und gehostete Keys gibt es über eine Warteliste, bisher ohne öffentliche Preise." },
    ],
  },
  comparison: {
    eyebrow: 'Seite an Seite',
    h2: 'Was jedes Tool abdeckt',
    headers: { feature: 'Funktion', hook0: 'Hook0', requeue: 'Requeue' },
    rows: [
      { feature: 'Hauptrolle', hook0Html: 'Ausgehendes Webhooks-as-a-Service', requeueHtml: 'Eingehende Dead-Letter-Inbox' },
      { feature: 'Webhook-Richtung', hook0Html: 'Ausgehend (du sendest an deine Nutzer)', requeueHtml: 'Eingehend (du empfängst von Dritten)' },
      { feature: 'Was erfasst wird', hook0Html: 'Deine fehlgeschlagenen Zustellungen an Abonnenten', requeueHtml: 'Eingehende Webhooks + fehlgeschlagene Cron- / Worker-Jobs' },
      { feature: 'Eingehende Relay-URL', hook0Html: 'Nein (der CLI-Tunnel ist für lokale Entwicklung)', requeueHtml: 'Ja, leitet einmal an deine App weiter und behält Fehler' },
      { feature: 'Replay', hook0Html: 'Pro Event, über Dashboard oder API', requeueHtml: 'Ein Klick, Payload pro Zustellung änderbar, Bulk-Replay bis 50 Events' },
      { feature: 'Protokoll pro Versuch', hook0Html: 'Ja, Statuscode und Antwort-Body', requeueHtml: 'Fehlgeschlagene Events mit Headern, Payload und Grund' },
      { feature: 'Signaturen', hook0Html: 'HMAC-SHA256 bei jeder Anfrage + TLS', requeueHtml: 'HMAC-SHA256 bei Replays, wenn der Endpoint ein Secret hat' },
      { feature: 'Automatische Retries', hook0Html: 'Bis zu 24 innerhalb von 8 Tagen (3 s bis 10 h Abstand), kostenlos', requeueHtml: 'Replays in der Queue: 1, 2, 4, 8, 16 Min., dann als fehlgeschlagen markiert' },
      { feature: 'Abonnenten-Portal', hook0Html: 'Einbettbar', requeueHtml: 'Nicht zutreffend (kein ausgehender Sender)' },
      { feature: 'Hosting', hook0Html: 'Managed EU-Cloud (Clever Cloud, Frankreich) + Self-Hosting', requeueHtml: 'Cloudflare Workers + D1, Hosting-Region nicht angegeben' },
      { feature: 'Quellcode', hook0Html: 'Server SSPL-1.0, 11 SDKs MIT', requeueHtml: 'Core und JS-SDK unter MIT (v0.1.0)' },
      { feature: 'Preise', hook0Html: 'Öffentlich, kostenloser Developer-Plan', requeueHtml: 'Nicht angegeben (gehostete Keys über Warteliste)' },
      { feature: 'Typischer Nutzer', hook0Html: 'Teams, die Webhooks an ihre Kunden senden', requeueHtml: 'Indies und kleine Teams (2 bis 5)' },
    ],
  },
  fit: {
    h2: 'Wo Requeue besser passt',
    bullets: [
      'Stripe, Clerk oder Resend schicken dir Webhooks, und du willst eine Kopie jeder Anfrage, die deine App nicht verarbeiten konnte, bereit zum erneuten Abspielen, sobald der Bug behoben ist. Das ist die Relay-URL von Requeue, und Hook0 macht das nicht.',
      'Deine nächtlichen Crons und Queue-Worker verlieren ihre Payloads nach dem letzten Retry, und du willst sie alle in einer Inbox sehen, mit dem Fehlergrund dazu.',
      'Du bist Solo-Gründer oder in einem Team von zwei bis fünf Leuten, schon auf Cloudflare, und es passt dir, einen MIT-Worker in Version 0.1.0 selbst zu betreiben.',
    ],
    linkPrefix: 'Brauchst du eher ein eingehendes Gateway mit eigener Retry-Pipeline? Sieh dir',
    link: { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck an' },
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Hook0 eine Alternative zu Requeue?', a: 'Sie lösen benachbarte Probleme in entgegengesetzten Richtungen, es hängt also von deinem Bedarf ab. Requeue fängt die Webhooks ab, die du empfängst, und spielt sie erneut ab, dazu fehlgeschlagene Cron- und Worker-Jobs. Hook0 sendet Webhooks an deine eigenen Nutzer und steuert diese Zustellung. Wenn du Webhooks zuverlässig an deine Kunden zustellen musst, passt Hook0. Wenn du eine Inbox für fehlgeschlagene eingehende Webhooks brauchst, ist das das Feld von Requeue.' },
      { q: 'Sendet Requeue Webhooks an meine Kunden?', a: 'Nein. Die FAQ von Requeue sagt, dass es Events weder transformiert noch verteilt und ausgehendes Webhooks-as-a-Service nicht ersetzt. Sein Relay leitet den Webhook eines Anbieters an eine einzige Ziel-URL in deiner App weiter. Events an die Endpoints vieler Kunden zu senden, mit Signaturen, Retries und einem Portal, in dem jeder Kunde seine Abonnements verwaltet, ist die Aufgabe von Hook0.' },
      { q: 'Macht Hook0 Dead-Letter für eingehende Webhooks?', a: 'Heute nicht als Produkt. Hook0 ist ausgehendes Webhooks-as-a-Service. Das Hook0-CLI kann Webhooks über einen eingebauten Tunnel auf localhost empfangen, und dieser Tunnel ist nur für die lokale Entwicklung gedacht.' },
      { q: 'Kann ich fehlgeschlagene Webhooks mit Hook0 erneut abspielen?', a: 'Ja, auf der Sendeseite. Jeder Zustellversuch wird mit Statuscode und Antwort protokolliert. Hook0 versucht eine fehlgeschlagene Zustellung standardmäßig bis zu 24 Mal innerhalb von 8 Tagen erneut, und wenn die Retries aufgebraucht sind, bleibt der letzte Versuch im Status failed. Dann spielst du das Event über das Dashboard oder mit POST /events/{event_id}/replay erneut ab, das es an jedes passende Abonnement sendet.' },
      { q: 'Wo läuft Requeue?', a: 'Laut README ist der Core von Requeue ein Cloudflare Worker, der Events in Cloudflare D1 speichert, und die gehostete API antwortet unter api.getrequeue.com. Sein öffentliches Material nennt die Hosting-Region der Daten nicht. Hook0 Cloud läuft in der EU, standardmäßig in Frankreich gehostet, und der Hook0-Server lässt sich auf Docker oder Kubernetes selbst hosten.' },
      { q: 'Ist Requeue Open Source?', a: 'Ja. Die Core-API von Requeue und sein JavaScript-SDK stehen unter MIT-Lizenz auf GitHub, in Version 0.1.0 im Oktober 2026. Der Server von Hook0 ist Open Source unter SSPL-1.0, also quellverfügbar und nicht OSI-geprüft, und seine 11 Client-SDKs sind MIT. Beide lassen dich selbst hosten, der Unterschied liegt in der Traffic-Richtung, die jedes Tool abdeckt.' },
      { q: 'Was soll ich wählen, Hook0 oder Requeue?', a: 'Wähle nach Richtung. Webhooks an deine eigenen Kunden senden, mit Signaturen, Protokoll pro Versuch, kostenlosen Retries und Abonnenten-Portal: Hook0. Webhooks abfangen und erneut abspielen, die du von Dritten empfängst, dazu fehlgeschlagene Cron- und Worker-Jobs, in einem kleinen Team: Requeue. Manche Teams brauchen beides, eins für jedes Ende der Leitung.' },
      { q: 'Ist Hook0 mit Requeue verbunden?', a: 'Nein. Requeue ist eine Marke ihres jeweiligen Inhabers, und Hook0 ist unabhängig, weder verbunden mit noch unterstützt von Requeue. Diese Seite ist ein sachlicher Vergleich für Teams, die beide Tools bewerten.' },
    ],
  },
  comparisonSources: {
    competitors: ['Requeue'],
    scope: 'Diese Seite vergleicht Rolle, Relay, Replay, Retries, Signaturen, Hosting, Lizenz und Preise von Hook0 und Requeue. Die Fakten zu Requeue stammen von seiner öffentlichen Startseite, aus README und FAQ seines GitHub-Repositorys und aus dem Repository seines JavaScript-SDKs, alle am angegebenen Datum gelesen. Die Fakten zu Hook0 stammen aus der Hook0-Dokumentation und der Preisseite.',
    sources: [
      { label: 'Startseite von Requeue', url: 'https://getrequeue.com/', consulted: '2026-10-22', archive: 'https://web.archive.org/web/20261022060503/https://getrequeue.com/' },
      { label: 'README des Requeue-Cores auf GitHub', url: 'https://github.com/requeue-hq/requeue', consulted: '2026-10-22', archive: 'https://web.archive.org/web/20261022060541/https://github.com/requeue-hq/requeue' },
      { label: 'FAQ von Requeue auf GitHub', url: 'https://github.com/requeue-hq/requeue/blob/main/docs/faq.md', consulted: '2026-10-22', archive: 'https://web.archive.org/web/20261022060523/https://github.com/requeue-hq/requeue/blob/main/docs/faq.md' },
      { label: 'JavaScript-SDK von Requeue auf GitHub', url: 'https://github.com/requeue-hq/requeue-sdk-js', consulted: '2026-10-22' },
      { label: 'Retry-Zeitplan von Hook0', url: 'https://documentation.hook0.com/explanation/webhook-retry-logic', consulted: '2026-10-22' },
    ],
    updated: '2026-10-22',
  },
  related: {
    h2: 'Zum selben Thema',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-webhook-relay', label: 'Hook0 vs Webhook Relay' },
      { enSlug: 'hook0-vs-ngrok', label: 'Hook0 vs ngrok' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst gehostete Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Quelloffene Webhooks' },
      { enSlug: 'migrate-from-webhook-site', label: 'Webhook.site-Alternative' },
      { enSlug: 'hook0-alternatives', label: 'Hook0-Alternativen' },
    ],
  },
};
