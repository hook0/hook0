// Per-page strings for hook0-vs-requeue (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// Hook0 SSPL-1.0 = « Open Source (quellverfügbar, nicht OSI-geprüft) », niemals
// « das einzige echte Open Source ». Fakten erfasst am 2026-10-02 (getrequeue.com + Hook0-Home).
// Strukturierende Nuance: Hook0 und Requeue liegen auf entgegengesetzten Seiten desselben
// Problems. Hook0 sendet (outbound), Requeue fängt Eingehendes ab (Inbound-Dead-Letter).
// Ehrlichkeit: nie behaupten, Hook0 mache heute produktives Inbound-Dead-Letter.
// Jedes nicht belegte Requeue-Attribut steht als « Nicht angegeben », niemals erfunden.
module.exports = {
  pageTitle: 'Hook0 vs Requeue: eingehende vs ausgehende Webhooks | Hook0',
  pageDescription: 'Requeue ist eine Dead-Letter-Inbox für empfangene Webhooks und fehlgeschlagene Jobs. Hook0 macht das Gegenteil: managed ausgehende Webhooks, kostenlose Retries, HMAC, Replay, EU-gehostet. Was passt.',
  pageModified: '2026-10-06',
  breadcrumb: 'Hook0 vs. Requeue',
  tldr: {
    h2: 'Kurz gesagt',
    body: "Hook0 und Requeue lösen zwei Seiten desselben Problems, in entgegengesetzten Richtungen. Requeue ist eine eingehende Dead-Letter-Inbox, es fängt die Webhooks ab, die du von Anbietern wie Stripe oder Clerk empfängst, dazu die Cron- und Worker-Jobs, die fehlschlagen, und lässt dich sie in deine App zurückspielen. Hook0 ist ausgehendes Webhooks-as-a-Service, es sendet die Webhooks an deine eigenen Nutzer und steuert diese Zustellung, mit jedem protokollierten Versuch, inspizierbaren Payloads, konfigurierbaren und kostenlosen Retries, HMAC-Signaturen und Replay aus dem Dashboard oder der API. Wähle Requeue, um eingehende Fehler in einem kleinen Team abzufangen und zurückzuspielen, wähle Hook0, um Webhooks an deine Kunden im großen Maßstab zuzustellen, EU-gehostet und Open Source. Hook0 bietet heute keine produktive eingehende Dead-Letter-Inbox.",
  },
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Requeue',
    titleAccent: 'Entgegengesetzte Richtungen, gleiches Problem',
    subtitle: "Requeue fängt die Webhooks ab, die du empfängst, und die Cron- oder Worker-Jobs, die fehlschlagen, und spielt sie dann in deine App zurück. Hook0 macht das Gegenteil, es sendet die Webhooks an deine Nutzer und steuert die Zustellung, mit Protokoll pro Versuch, kostenlosen Retries, HMAC-Signatur und Replay. Diese Seite legt offen, für welche Richtung jedes Tool gebaut ist, damit du auf einen Blick siehst, welches Problem du wirklich hast.",
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  differentiators: {
    eyebrow: 'Was jedes Tool macht',
    h2: 'Zentrale Unterschiede',
    cards: [
      { title: 'Eingehendes abfangen vs ausgehendes senden', body: "Requeue ist eine Dead-Letter-Inbox für Traffic, der <em>hereinkommt</em>, die Webhooks, die du von Dritten empfängst, und die Hintergrundjobs, die fehlschlagen. Hook0 ist der Sender. Es veröffentlicht deine Events und stellt sie deinen Abonnenten zu, der Fehler, den es verwaltet, ist also eine <em>ausgehende</em> Zustellung, die nicht angekommen ist. Gleiches Zuverlässigkeitsproblem, entgegengesetzte Enden der Leitung." },
      { title: 'Abfangen, inspizieren, zurückspielen, auf der Sendeseite', body: "Die Story abfangen / inspizieren / zurückspielen, die Requeue für Eingehendes erzählt, macht Hook0 bereits für Ausgehendes, jeder Zustellversuch wird protokolliert, du inspizierst Payloads und Antworten, du erfährst, warum eine Zustellung fehlgeschlagen ist, und du spielst Events aus dem Dashboard oder der API zurück. Events und Antworten werden persistiert, die Debug-Spur ist also da, wenn du sie brauchst." },
      { title: 'EU-Residenz und DSGVO standardmäßig', body: "Hook0 wird von einem französischen Unternehmen ohne US-Muttergesellschaft, US-Tochtergesellschaft oder US-Niederlassung herausgegeben, und seine managed Cloud läuft standardmäßig in der EU mit HMAC-Signaturen und TLS. Requeue bietet einen selbst hostbaren Core und eine gehostete API auf Warteliste, sein öffentliches Material gibt nicht an, wo die gehosteten Daten liegen, EU-Residenz läuft also über dein eigenes Hosting." },
      { title: 'Open Source, kostenlose Retries, keine Abrechnung pro Endpoint', body: "Der Server von Hook0 ist Open Source unter SSPL-1.0 (quellverfügbar, nicht OSI-geprüft) und seine 11 Client-SDKs sind MIT. Retries sind kostenlos enthalten, die HMAC-Signatur ist eingebaut, und es gibt keine Abrechnung pro Endpoint. Requeue veröffentlicht ebenfalls einen selbst hostbaren Open-Source-Core. Beide lassen dich die Software selbst betreiben, die Wahl hängt an der Traffic-Richtung, die du zuverlässig machen musst." },
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
      { feature: 'Replay', hook0Html: 'Aus dem Dashboard oder der API', requeueHtml: 'Replay mit einem Klick samt Payload-Bearbeitung, Bulk-Replay' },
      { feature: 'Protokoll pro Versuch', hook0Html: 'Ja, Inspektion von Payloads und Antworten', requeueHtml: 'Erfasste Fehler in der Inbox gehalten' },
      { feature: 'Signaturen', hook0Html: 'HMAC-SHA256 + TLS', requeueHtml: 'Nicht angegeben' },
      { feature: 'Abonnenten-Portal', hook0Html: 'Einbettbar', requeueHtml: 'Nicht zutreffend (kein ausgehender Sender)' },
      { feature: 'Retries', hook0Html: 'Konfigurierbar 2-phasig, kostenlos', requeueHtml: 'Manuelles und Bulk-Replay' },
      { feature: 'Hosting', hook0Html: 'Managed EU-Cloud (Clever Cloud FR) + Self-Hosting', requeueHtml: 'Selbst hostbarer Core, gehostete API auf Warteliste' },
      { feature: 'Quelle', hook0Html: 'Server SSPL-1.0, 11 SDKs MIT', requeueHtml: 'Open-Source-Core (selbst hostbar)' },
      { feature: 'Typischer Nutzer', hook0Html: 'Teams, die Webhooks an ihre Kunden senden', requeueHtml: 'Indies und kleine Teams (2 bis 5)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Hook0 eine Alternative zu Requeue?', a: "Sie lösen benachbarte Probleme in entgegengesetzten Richtungen, es hängt also von deinem Bedarf ab. Requeue fängt die Webhooks ab, die du empfängst, und spielt sie zurück, dazu fehlgeschlagene Cron- und Worker-Jobs. Hook0 sendet die Webhooks an deine eigenen Nutzer und steuert diese Zustellung. Wenn du Webhooks zuverlässig an deine Kunden zustellen musst, ist Hook0 die passende Wahl. Wenn du heute eine Inbox für fehlgeschlagene eingehende Webhooks brauchst, ist das das Terrain von Requeue." },
      { q: 'Macht Hook0 Dead-Letter für eingehende Webhooks?', a: "Heute nicht als Produkt. Hook0 ist ausgehendes Webhooks-as-a-Service. Das Hook0-CLI kann Webhooks über einen eingebauten Tunnel auf localhost empfangen, für die lokale Entwicklung, aber das ist ein Dev-Tool, keine produktive eingehende Dead-Letter-Inbox." },
      { q: 'Kann ich mit Hook0 fehlgeschlagene Webhooks zurückspielen?', a: "Ja, auf der Sendeseite. Jeder Zustellversuch wird protokolliert, du inspizierst das Payload und die Antwort, du erfährst, warum eine Zustellung fehlgeschlagen ist, und du spielst Events aus dem Dashboard oder der API zurück. Retries sind in zwei Phasen konfigurierbar und kostenlos." },
      { q: 'Ist Requeue Open Source?', a: "Requeue bietet einen selbst hostbaren Open-Source-Core, mit einer gehosteten API auf Warteliste. Der Server von Hook0 ist Open Source unter SSPL-1.0, das quellverfügbar und nicht OSI-geprüft ist, und seine 11 Client-SDKs sind MIT. Beide lassen dich selbst hosten, der Unterschied ist die Traffic-Richtung, die jedes Tool abdeckt." },
      { q: 'Welches soll ich wählen, Hook0 oder Requeue?', a: "Wähle nach Richtung. Webhooks im großen Maßstab an deine eigenen Kunden senden, mit Signaturen, Protokoll pro Versuch, kostenlosen Retries und Abonnenten-Portal, das ist Hook0. Webhooks abfangen und zurückspielen, die du von Dritten empfängst, dazu fehlgeschlagene Cron- und Worker-Jobs, in einem kleinen Team, das ist Requeue. Sie liegen auf benachbarten Seiten desselben Zuverlässigkeitsproblems." },
      { q: 'Ist Hook0 mit Requeue verbunden?', a: "Nein. Requeue ist eine Marke seines jeweiligen Inhabers, und Hook0 ist unabhängig, weder verbunden mit noch unterstützt von Requeue. Diese Seite ist ein sachlicher Vergleich für Teams, die beide Tools bewerten." },
    ],
  },
  comparisonSources: {
    competitors: ['Requeue'],
    scope: "Diese Seite vergleicht Rolle, Replay, Hosting und Quellcode von Hook0 und Requeue. Die Angaben zu Requeue stammen von seiner öffentlichen Startseite und seinem GitHub-Repository.",
    sources: [
      { label: 'Requeue Startseite', url: 'https://getrequeue.com/', consulted: '2026-10-06' },
      { label: 'Requeue auf GitHub', url: 'https://github.com/requeue-hq/requeue', consulted: '2026-10-06' },
    ],
    updated: '2026-10-06',
  },
  related: {
    h2: 'Zum selben Thema',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'migrate-from-webhook-site', label: 'Webhook.site-Alternative' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst gehostete Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Quelloffene Webhooks' },
      { enSlug: 'hook0-alternatives', label: 'Hook0-Alternativen' },
    ],
  },
};
