// Per-page strings for hook0-vs-hookbase (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Middle-Dot.
// Gleiche Struktur und gleiche Leitplanken wie die EN-Basis: Hookbase deckt
// eingehende und ausgehende Webhooks ab, nur verwaltet, auf Cloudflare; Hook0
// deckt die ausgehende Seite ab, in der EU-Cloud oder auf deinen Servern.
// Hookbase ist beim Einstiegspreis günstiger: so schreiben, nicht verstecken.
// Keine Hookbase-Seite nennt eine Hosting-Region oder eine EU-Datenresidenz:
// « nicht angegeben », niemals « nur USA ». Keine Zertifizierungs-Claims.
// Hook0 = « Open Source (SSPL-1.0) » oder « nicht OSI-anerkannt », nie
// « Open Source » ohne Lizenzangabe.
// Fakten zu Hookbase am Datum in comparisonSources erneut geprüft.
module.exports = {
  pageTitle: 'Hook0 vs Hookbase: Webhooks, Open Source und in der EU gehostet',
  pageDescription: 'Hookbase ist ein verwaltetes Webhook-Gateway auf Cloudflare für eingehende und ausgehende Webhooks. Hook0 sendet Webhooks an deine Nutzer, Open Source (SSPL-1.0) und in der EU gehostet.',
  pageModified: '2026-11-24',
  breadcrumb: 'Hook0 vs Hookbase',
  tldr: {
    h2: 'Hook0 oder Hookbase: was passt zu deinem Team?',
    body: "Nimm Hookbase, wenn du vor allem Webhooks von Stripe, GitHub oder Shopify empfängst und ein verwaltetes Gateway willst, das sie prüft, umformt und weiterleitet, mit einem kostenlosen Tarif und bezahlten Tarifen ab 7 $ im Monat. Nimm Hook0, wenn du Webhooks an deine eigenen Kunden sendest und denselben Code in einer EU-Cloud oder auf deinen eigenen Servern betreiben musst. Hook0 wiederholt eine fehlgeschlagene Zustellung bis zu 24 Mal über 8 Tage, in jedem Tarif, signiert jede Anfrage mit HMAC-SHA256 und wird von einem französischen Unternehmen herausgegeben. Hookbase bezeichnet sich selbst als managed-only, also nur als verwalteten Dienst.",
  },
  hero: {
    eyebrow: 'Vergleich',
    titleBefore: 'Hook0 vs Hookbase',
    titleAccent: 'Edge-Gateway oder EU-Plattform zum Senden',
    subtitle: 'Hookbase betreibt ein Webhook-Gateway im Netz von Cloudflare, das Webhooks empfängt, umformt und weiterleitet und auch selbst welche senden kann. Hook0 konzentriert sich darauf, Webhooks an deine Nutzer zu senden, mit kostenlosen Retries, HMAC-Signaturen, Logs pro Versuch und einem Portal für Abonnenten, in einer EU-Cloud oder auf deinen eigenen Servern. Hier steht, wann welches Werkzeug die bessere Wahl ist.',
    ctaPrimary: 'Kostenlos starten',
    ctaSecondary: 'Playground ausprobieren',
  },
  differentiators: {
    eyebrow: 'Was jedes Werkzeug macht',
    h2: 'Die wichtigsten Unterschiede',
    cards: [
      { title: 'Denselben Code selbst hosten, oder verwaltet bleiben', body: "Der Server von Hook0 ist Open Source unter SSPL-1.0 (nicht OSI-anerkannt). Der Code hinter Hook0 Cloud läuft also auch auf deinem Docker- oder Kubernetes-Cluster, und die 11 Client-SDKs stehen unter MIT. Die Vergleichsseite von Hookbase beschreibt Hookbase als <em>managed-only</em>, und keine Hookbase-Seite bietet eine Lizenz oder eine selbst gehostete Edition an." },
      { title: 'Retries, die nicht vom Tarif abhängen', body: 'Hook0 wiederholt eine fehlgeschlagene Zustellung bis zu 24 Mal innerhalb von 8 Tagen, im Abstand von 3 Sekunden bis 10 Stunden, und der Zeitplan ist im kostenlosen Tarif derselbe wie in den bezahlten. Bei Hookbase hängt die Zahl der Versuche vom Tarif ab: 5 im Free-Tarif (etwa 13 Minuten), 11 in Starter, 13 in Pro und 15 in Business (etwa 5 Tage). Siehe <a href="https://documentation.hook0.com/explanation/webhook-retry-logic" class="text-indigo-400 hover:text-indigo-300 underline">den genauen Retry-Zeitplan von Hook0</a> (auf Englisch).' },
      { title: 'Herausgeber in der EU, EU-Cloud als Standard', body: 'Hook0 wird von einem französischen Unternehmen ohne US-Muttergesellschaft herausgegeben, und Hook0 Cloud läuft in der EU, standardmäßig in Frankreich gehostet. Hookbase verarbeitet Events auf Cloudflare Workers an über 300 Standorten und speichert Payloads in Cloudflare R2. Seine öffentlichen Seiten nennen weder eine juristische Person noch eine Hosting-Region, und seine Nutzungsbedingungen unterliegen dem Recht von Delaware.' },
      { title: 'Eine Aufgabe gründlich, oder ein breiteres Gateway', body: 'Hookbase deckt mehr ab: ein Gateway für eingehende Webhooks, das Signaturen von über 70 Anbietern prüft, Payload-Transformationen in 8 Sprachen, Ziele wie Queues und Speicher, lokale Tunnel und einen MCP-Server. Hook0 bleibt bei der ausgehenden Zustellung. Es sendet den Payload, den du veröffentlichst, unverändert, filtert nach Event-Typ und Label, protokolliert jeden Versuch mit Statuscode und Antwort und lässt dich jedes Event aus dem Dashboard oder über die API erneut abspielen.' },
    ],
  },
  comparison: {
    eyebrow: 'Direkt nebeneinander',
    h2: 'Funktionsvergleich',
    headers: { feature: 'Funktion', hook0: 'Hook0', hookbase: 'Hookbase' },
    rows: [
      { feature: 'Richtung der Webhooks', hook0Html: 'Ausgehend (du sendest an deine Nutzer)', hookbaseHtml: 'Eingehend und ausgehend' },
      { feature: 'Kostenloser Tarif', hook0Html: '100 Events pro Tag, ohne Kreditkarte', hookbaseHtml: '1.000 Events pro Monat, harte Grenze' },
      { feature: 'Bezahlte Tarife', hook0Html: 'Ab 59 €/Monat, Mehrverbrauch pro Event abgerechnet', hookbaseHtml: '7 $, 29 $ und 79 $/Monat, oder 1 $ pro 1.000 Events' },
      { feature: 'Automatische Retries', hook0Html: 'Bis zu 24 innerhalb von 8 Tagen, in jedem Tarif', hookbaseHtml: '5 (Free) bis 15 (Business) Versuche' },
      { feature: 'Signaturen', hook0Html: 'HMAC-SHA256 auf jeder Anfrage + TLS', hookbaseHtml: 'Standard-Webhooks-Spezifikation für ausgehende Webhooks' },
      { feature: 'Portal für Abonnenten', hook0Html: 'Einbettbar', hookbaseHtml: 'React-Paket (@hookbase/portal)' },
      { feature: 'Filterung', hook0Html: 'Event-Typen und Labels', hookbaseHtml: 'Event-Typen, Labels, Routen-Filter' },
      { feature: 'Payload-Transformationen', hook0Html: 'Nein, Payload wird wie veröffentlicht gesendet', hookbaseHtml: 'Ja, 8 Sprachen (ab Starter)' },
      { feature: 'Gateway für eingehende Webhooks', hook0Html: 'Nein (der CLI-Tunnel dient der lokalen Entwicklung)', hookbaseHtml: 'Ja, Signaturprüfung für über 70 Anbieter' },
      { feature: 'Replay', hook0Html: 'Pro Event, aus dem Dashboard oder über die API', hookbaseHtml: 'Ja, mit Dead Letter Queue' },
      { feature: 'Hosting', hook0Html: 'Verwaltete EU-Cloud (Clever Cloud, Frankreich) + Selbst-Hosting', hookbaseHtml: 'Cloudflare Workers und R2, Region nicht angegeben' },
      { feature: 'Quellcode', hook0Html: 'Server SSPL-1.0, 11 SDKs MIT', hookbaseHtml: 'Nur verwaltet, keine Lizenz veröffentlicht' },
      { feature: 'Herausgeber', hook0Html: 'Französisches Unternehmen ohne US-Muttergesellschaft', hookbaseHtml: 'Nicht angegeben (Bedingungen nach dem Recht von Delaware)' },
    ],
  },
  fit: {
    h2: 'Wo Hookbase besser passt',
    bullets: [
      'Der Großteil deines Traffics sind Webhooks, die du von Stripe, GitHub, Shopify oder Slack empfängst, und du willst einen Ort, der ihre Signaturen prüft, die Payloads umformt und sie an deine Dienste weiterleitet, an Queues wie SQS oder Pub/Sub oder ab Pro an Objektspeicher wie S3. Hook0 macht keine eingehenden Webhooks.',
      'Du brauchst Payload-Transformationen in JSONata, jq, JavaScript oder Liquid zwischen dem Anbieter und deiner App.',
      'Du suchst einen bezahlten Tarif unter 59 € im Monat. Hookbase Starter kostet 7 $ im Monat für 25.000 eingehende Events.',
      'Du willst deine Webhooks schon heute aus einem KI-Assistenten heraus verwalten, und Hookbase liefert einen MCP-Server, gehostet oder lokal.',
    ],
    linkPrefix: 'Vergleichst du auch andere Gateways für eingehende Webhooks? Siehe',
    link: { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Häufige Fragen',
    items: [
      { q: 'Ist Hook0 eine Alternative zu Hookbase?', a: 'Um Webhooks an deine Kunden zu senden, ja. Beide stellen Events an die Endpoints deiner Abonnenten zu, mit Signaturen, Retries, Logs und einem Portal. Hook0 bringt zusätzlich einen Server unter SSPL-1.0 mit, den du selbst hosten kannst, einen Herausgeber und ein Hosting in der EU und denselben Zeitplan mit 24 Retries in jedem Tarif. Um Webhooks von Dritten zu empfangen und umzuformen, deckt Hookbase etwas ab, das Hook0 nicht abdeckt.' },
      { q: 'Kann ich Hookbase selbst hosten?', a: 'Laut seinen öffentlichen Seiten nicht. Der Vergleich von Hookbase mit Svix beschreibt Hookbase als managed-only, und keine Seite nennt eine Lizenz oder eine selbst gehostete Edition. Der Server von Hook0 ist Open Source unter SSPL-1.0 und läuft auf Docker oder Kubernetes, mit demselben Code wie Hook0 Cloud.' },
      { q: 'Wo wird Hookbase gehostet?', a: 'Hookbase gibt an, Events auf Cloudflare Workers an über 300 Standorten zu verarbeiten und Payloads verschlüsselt in Cloudflare R2 zu speichern. Seine öffentlichen Seiten nennen weder eine Hosting-Region noch eine Option für Datenresidenz in der EU. Hook0 Cloud läuft in der EU, standardmäßig in Frankreich gehostet.' },
      { q: 'Wie oft wiederholt Hookbase einen fehlgeschlagenen Webhook?', a: 'Das hängt vom Tarif ab: 5 Versuche im Free-Tarif (etwa 13 Minuten), 11 in Starter (etwa 28 Stunden), 13 in Pro (etwa 3 Tage) und 15 in Business (etwa 5 Tage), und die Zahl lässt sich nicht einstellen. Hook0 wiederholt bis zu 24 Mal innerhalb von 8 Tagen, in jedem Tarif, im Abstand von 3 Sekunden bis 10 Stunden.' },
      { q: 'Ist Hookbase günstiger als Hook0?', a: 'Beim Einstieg, ja. Hookbase hat einen kostenlosen Tarif mit einer Grenze von 1.000 Events pro Monat und bezahlte Tarife zu 7 $, 29 $ und 79 $ im Monat. Hook0 hat einen kostenlosen Tarif mit 100 Events pro Tag und bezahlte Tarife ab 59 € im Monat, wobei Mehrverbrauch pro Event abgerechnet wird, sodass Traffic über dem Kontingent trotzdem zugestellt wird. Vergleiche mit deinem eigenen Volumen auf den Preisseiten.' },
      { q: 'Formt Hook0 Payloads um wie Hookbase?', a: 'Nein. Hook0 stellt den Payload, den du veröffentlichst, genau so zu, wie du ihn gesendet hast, und wählt die Subscriptions, die ihn erhalten, nach Event-Typ und Labels aus. Wenn du Webhooks von Dritten umformen musst, bevor sie deine App erreichen, ist das eine Funktion von Hookbase.' },
      { q: 'Was soll ich wählen, Hook0 oder Hookbase?', a: 'Entscheide nach der Richtung und danach, wo die Daten liegen müssen. Webhooks von vielen Anbietern auf einem verwalteten Edge-Gateway empfangen und umformen: Hookbase. Webhooks an deine eigenen Kunden senden, mit einem Server unter SSPL-1.0, Hosting in der EU und einem französischen Herausgeber: Hook0.' },
      { q: 'Ist Hook0 mit Hookbase verbunden?', a: 'Nein. Hookbase ist eine Marke ihres jeweiligen Inhabers, und Hook0 ist unabhängig, weder mit Hookbase verbunden noch von Hookbase unterstützt. Diese Seite ist ein sachlicher Vergleich für Teams, die beide Werkzeuge prüfen.' },
    ],
  },
  comparisonSources: {
    competitors: ['Hookbase'],
    scope: 'Diese Seite vergleicht Richtung, Preise, Retries, Signaturen, Portal, Transformationen, Hosting, Quellcode und Herausgeber von Hook0 und Hookbase. Die Angaben zu Hookbase stammen von seiner öffentlichen Website, Preisseite, Produktseiten, Vergleichsseiten, Datenschutzerklärung und Nutzungsbedingungen, alle am angegebenen Datum gelesen. Die Angaben zu Hook0 stammen aus der Hook0-Dokumentation und der Preisseite.',
    sources: [
      { label: 'Startseite von Hookbase', url: 'https://www.hookbase.app/', consulted: '2026-11-24' },
      { label: 'Preise von Hookbase', url: 'https://www.hookbase.app/pricing', consulted: '2026-11-24' },
      { label: 'Retries bei Hookbase', url: 'https://www.hookbase.app/webhook-retries', consulted: '2026-11-24' },
      { label: 'Kundenportal von Hookbase', url: 'https://www.hookbase.app/customer-portal', consulted: '2026-11-24' },
      { label: 'Payload-Transformationen von Hookbase', url: 'https://www.hookbase.app/payload-transforms', consulted: '2026-11-24' },
      { label: 'Hookbase vs Svix (managed-only)', url: 'https://www.hookbase.app/compare/svix', consulted: '2026-11-24' },
      { label: 'Hookbase vs Hookdeck (Cloudflare Workers)', url: 'https://www.hookbase.app/compare/hookdeck', consulted: '2026-11-24' },
      { label: 'Nutzungsbedingungen von Hookbase (anwendbares Recht)', url: 'https://www.hookbase.app/terms', consulted: '2026-11-24' },
      { label: 'Retry-Zeitplan von Hook0', url: 'https://documentation.hook0.com/explanation/webhook-retry-logic', consulted: '2026-11-24' },
    ],
    updated: '2026-11-24',
  },
  related: {
    h2: 'Verwandte Themen',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'hook0-vs-webhook-relay', label: 'Hook0 vs Webhook Relay' },
      { enSlug: 'hook0-vs-requeue', label: 'Hook0 vs Requeue' },
      { enSlug: 'self-hosted-webhooks', label: 'Selbst gehostete Webhooks' },
      { enSlug: 'open-source-webhooks', label: 'Quelloffene Webhooks' },
      { enSlug: 'eu-webhook-infrastructure', label: 'EU-Webhook-Infrastruktur' },
      { enSlug: 'hook0-alternatives', label: 'Hook0-Alternativen' },
    ],
  },
};
