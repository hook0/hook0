// Per-page strings for webhook-api (DE).
// /humanizer pro + legal-reviewer applied. DSGVO claims as process; SSPL = « Open Source (SSPL-1.0) », nie « Open Source » allein.
// Same structure and same checked facts as locales/en/webhook-api.js: change
// a fact there, change it here in the same commit.
module.exports = {
  tldr: {
    h2: 'Was ist eine Webhook-API?',
    body: 'Eine Webhook-API schickt eine HTTP-Anfrage an eine von dir registrierte URL, sobald ein Event eintritt, statt darauf zu warten, dass du nachfragst. Eine REST-API antwortet, wenn der Client sie aufruft; eine Webhook-API ruft den Client auf. Hook0 übernimmt diese Senderseite für dein SaaS. Dein Backend sendet jedes Event einmal per POST an /api/v1/event, und Hook0 signiert es mit HMAC-SHA256, stellt es jeder passenden Subscription zu, wiederholt fehlgeschlagene Zustellungen (erster Versuch nach etwa 3 Sekunden, spätere mit bis zu 10 Stunden Abstand) und speichert jeden Versuch samt Antwort, damit du ihn erneut abspielen kannst. Wenn du nur Webhooks von Stripe oder GitHub empfangen willst, passt ein Gateway für eingehende Webhooks besser.',
  },
  pageTitle: 'Webhook-API: signierte Webhooks per REST-Aufruf | Hook0',
  pageDescription: 'Was eine Webhook-API ist, worin sie sich von einer REST-API unterscheidet und wie Hook0 signierte Webhooks mit Retries, Logs und Replay sendet.',
  "pageModified": "2026-11-26",
  "track": "de-webhook-api",
  "hero": {
    "eyebrow": "Webhook-API",
    "titleLine1": "Die Webhook-API",
    "titleLine2": "für Events an deine Kunden",
    "subtitle": "Mit einer Webhook-API benachrichtigt dein Backend die Systeme deiner Kunden, sobald etwas passiert. Bei Hook0 ist das ein POST: Hook0 signiert das Event, stellt es zu, wiederholt Fehlschläge und protokolliert jeden Versuch. Offizielle SDKs in 11 Sprachen, Server Open Source (SSPL-1.0), kostenloser Tarif.",
    "ctaPrimary": "Kostenlos starten",
    "ctaSecondary": "API-Referenz lesen",
    "microcopy": "100 Events/Tag kostenlos. Keine Kreditkarte. Open Source (SSPL-1.0).",
    "updated": "Aktualisiert am 26. November 2026"
  },
  "socialProof": true,
  "vsApi": {
    "h2": "Webhook oder REST-API: wer ruft wen auf",
    "intro": "Beide laufen über HTTP und transportieren meist JSON. Der Unterschied liegt darin, wer die Anfrage startet und wer sich um einen ausgefallenen Empfänger kümmert.",
    "headers": { "aspect": "Kriterium", "rest": "REST-API (Pull)", "webhook": "Webhook (Push)" },
    "rows": [
      { "aspect": "Wer startet die Anfrage", "rest": "Der Client ruft den Server auf", "webhook": "Der Sender ruft eine vom Empfänger registrierte URL auf" },
      { "aspect": "Wann Daten ankommen", "rest": "Sobald der Client abfragt", "webhook": "Sekunden nach dem Event" },
      { "aspect": "Unnötige Anfragen", "rest": "Die meisten Abfragen liefern nichts Neues", "webhook": "Eine Anfrage pro Event und Subscription" },
      { "aspect": "Empfänger ausgefallen", "rest": "Der Client wiederholt seinen eigenen Aufruf", "webhook": "Der Sender muss puffern und wiederholen" },
      { "aspect": "Authentifizierung", "rest": "Der Client sendet ein Token", "webhook": "Der Empfänger prüft eine HMAC-Signatur" },
      { "aspect": "Typischer Einsatz", "rest": "Daten bei Bedarf lesen oder ändern", "webhook": "Einem anderen System melden, dass etwas passiert ist" }
    ],
    "outro": "Die meisten SaaS-Produkte bieten eine REST-API zum Lesen und Schreiben von Daten und dazu Webhooks, die Änderungen melden. Hook0 übernimmt den Webhook-Teil und stellt ihn selbst als REST-API bereit. Der <a href=\"https://documentation.hook0.com/explanation/webhook-vs-polling\" class=\"text-indigo-400 hover:text-indigo-300 underline\">Leitfaden Webhook vs. Polling</a> (auf Englisch) erklärt die Abwägungen im Detail."
  },
  "codeExample": {
    "eyebrow": "Code",
    "h2": "Sende dein erstes Event in 30 Sekunden",
    "subtitle": "Ein Endpoint. Der Payload reist als String mit seinem Content-Type, damit Empfänger genau die Bytes erhalten, die du gesendet hast.",
    "restCode": "POST https://app.hook0.com/api/v1/event\nAuthorization: Bearer &lt;APPLICATION_TOKEN&gt;\nContent-Type: application/json\n\n{\n  \"application_id\":       \"c0ea6ffa-1972-4435-b434-ec9e93d38f42\",\n  \"event_id\":             \"0192f5b4-7c1e-7d3a-9f2b-4e8a1c6d2b90\",\n  \"event_type\":           \"billing.invoice.paid\",\n  \"payload\":              \"{\\\"invoice_id\\\":\\\"in_8X9aBcDeFgHiJk\\\",\\\"amount_eur\\\":4990}\",\n  \"payload_content_type\": \"application/json\",\n  \"occurred_at\":          \"2026-11-26T09:30:00Z\",\n  \"labels\":               { \"tenant\": \"acme\", \"env\": \"prod\" }\n}\n",
    "pythonCode": "from hook0 import Event, Hook0Client\n\nclient = Hook0Client(\"https://app.hook0.com/api/v1\", APP_ID, TOKEN)\nclient.send_event(Event(\n    event_type=\"billing.invoice.paid\",\n    payload='{\"invoice_id\": \"in_123\"}',\n    payload_content_type=\"application/json\",\n    labels={\"env\": \"prod\"},\n))",
    "nodeCode": "import { Event, Hook0Client } from 'hook0-client';\n\nconst hook0 = new Hook0Client('https://app.hook0.com/api/v1', APP_ID, TOKEN);\nawait hook0.sendEvent(new Event(\n  'billing.invoice.paid',\n  JSON.stringify({ invoice_id: 'in_123' }),\n  'application/json',\n  { env: 'prod' }\n));",
    "docsLabel": "Zum 5-Minuten-Schnellstart →",
    "docsHref": "https://documentation.hook0.com/tutorials/getting-started",
    "docsTrack": "de-webhook-api-docs"
  },
  "capabilities": {
    "eyebrow": "In der API",
    "h2": "Was die Webhook-API für dich erledigt",
    "cards": [
      {
        "title": "HMAC-SHA256-Signatur",
        "body": "Jede Zustellung trägt einen Header <code class=\"text-green-400\">X-Hook0-Signature</code> mit Zeitstempel und einem HMAC-SHA256 des Payloads. Empfänger prüfen beides, sodass eine alte, erneut gesendete Anfrage an der Zeitstempel-Prüfung scheitert."
      },
      {
        "title": "Wiederholungen nach festem Zeitplan",
        "body": "Eine fehlgeschlagene Zustellung wird nach etwa 3 Sekunden wiederholt, dann nach 10 Sekunden, 3 Minuten, 30 Minuten und so weiter bis zu 10 Stunden Abstand, standardmäßig 24 Wiederholungen. Danach bleibt der letzte Versuch samt Antwort fehlgeschlagen und lässt sich erneut abspielen."
      },
      {
        "title": "Idempotente Event-IDs",
        "body": "Übergib deine eigene <code class=\"text-green-400\">event_id</code> (eine UUID). Dieselbe ID zweimal zu senden liefert 409 Conflict statt eines zweiten Events, sodass ein wiederholter API-Aufruf nachgelagert nie zweimal auslöst."
      },
      {
        "title": "Zustellprotokolle und Replay",
        "body": "Header, Body, Statuscode, Latenz. Pro Versuch gespeichert. Jedes Event lässt sich per ID erneut zustellen, aus dem Dashboard oder per API."
      },
      {
        "title": "Open-Source-SDKs",
        "body": "JavaScript/TypeScript, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, Lua und Zig. Aus der OpenAPI-Spezifikation generiert, damit Client und API synchron bleiben."
      },
      {
        "title": "Kostenloser Tarif, ohne Schranken",
        "body": "100 Events pro Tag, keine Kreditkarte. Bezahlte Pläne erhöhen das Volumen. Jede Funktion auf dieser Seite ist im kostenlosen Tarif enthalten."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Fragen zur Webhook-API",
    "items": [
      {
        "q": "Was ist eine Webhook-API?",
        "a": "Eine Webhook-API sendet einen HTTP-POST an eine registrierte URL, wenn ein Event eintritt, sodass der Empfänger davon erfährt, ohne abzufragen. Die Webhook-API von Hook0 ist die Senderseite dieses Austauschs. Dein Backend löst ein Event mit einem HTTP-Aufruf aus, und Hook0 signiert den Payload per HMAC, stellt ihn jedem passenden Subscriber zu, wiederholt Fehlschläge und protokolliert jeden Versuch. Offizielle SDKs gibt es in 11 Sprachen."
      },
      {
        "q": "Was ist der Unterschied zwischen einem Webhook und einer API?",
        "a": "Bei einer REST-API fragt der Client und der Server antwortet. Bei einem Webhook ruft der Server den Client auf, wenn sich etwas ändert. Webhooks ersetzen das Polling und kommen Sekunden nach dem Event an, aber der Sender muss dann Anfragen signieren, puffern und wiederholen, wenn der Empfänger ausfällt. Hook0 nimmt dir diese Arbeit auf der Senderseite ab."
      },
      {
        "q": "Welche Event-Plattform bietet eine API und Webhooks?",
        "a": "Hook0 ist eine Event-Plattform mit einer REST-API als Eingang und Webhooks als Ausgang. Du sendest Events über die API (oder eines der 11 SDKs), legst Event-Typen und Subscriptions über dieselbe API an, und Hook0 stellt jedes Event als signierten Webhook an jeden abonnierten Endpoint zu, mit Wiederholungen, Zustellprotokollen und Replay."
      },
      {
        "q": "Wie authentifiziere ich mich an der Webhook-API?",
        "a": "Die Authentifizierung nutzt ein Bearer-Token (Application Authentication Token) im Header <code class=\"text-green-400\">Authorization</code>. Tokens sind auf eine Anwendung beschränkt und lassen sich jederzeit im Dashboard rotieren."
      },
      {
        "q": "Enthält die Webhook-API Wiederholungsversuche und HMAC-Signaturen?",
        "a": "Ja. Jedes über die Webhook-API gesendete Event wird mit HMAC-SHA256 signiert, damit Empfänger es prüfen können. Eine fehlgeschlagene Zustellung wird nach einem wachsenden Zeitplan wiederholt, von etwa 3 Sekunden bis zu 10 Stunden zwischen zwei Versuchen. Sind alle Wiederholungen aufgebraucht, bleibt der letzte Versuch samt Antwort des Endpoints fehlgeschlagen, und ein API-Aufruf stellt das Event erneut zu."
      },
      {
        "q": "Welche SDKs gibt es für die Webhook-API von Hook0?",
        "a": "Hook0 pflegt 11 offizielle SDKs: JavaScript/TypeScript, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, Lua und Zig. Jedes wird über die Paket-Registry seiner Sprache installiert (Zig über ein getaggtes Archiv), sendet Events und prüft Webhook-Signaturen. Die REST-API ist in der API-Referenz vollständig dokumentiert, daher funktioniert auch jeder HTTP-Client."
      },
      {
        "q": "Hat die Webhook-API ein Rate-Limit?",
        "a": "Ja. Die Limits wachsen mit dem Tarif. Der kostenlose Tarif erlaubt 100 Events pro Tag, bezahlte Tarife erhöhen das Tagesvolumen und die Burst-Rate. Selbst gehostete Installationen werden von Hook0 nicht begrenzt."
      }
    ]
  },
  "related": {
    "h2": "Weiterführend (auf Englisch)",
    "links": [
      { "label": "API Reference", "href": "https://documentation.hook0.com/openapi/intro" },
      { "label": "Retry Schedule", "href": "https://documentation.hook0.com/explanation/webhook-retry-logic" },
      { "label": "Webhook vs Polling", "href": "https://documentation.hook0.com/explanation/webhook-vs-polling" },
      { "label": "Webhook Platform", "href": "/webhook-platform" },
      { "label": "Hook0 vs Svix", "href": "/hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "/hook0-vs-hookdeck" },
      { "label": "Build vs Buy Webhooks", "href": "/build-vs-buy-webhooks" },
      { "label": "Self-Hosted Webhooks", "href": "/self-hosted-webhooks" },
      { "label": "Open-Source Webhooks", "href": "/open-source-webhooks" },
      { "label": "How Webhooks Work", "href": "/how-webhooks-work" }
    ]
  }
};
