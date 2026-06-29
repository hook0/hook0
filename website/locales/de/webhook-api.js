// Per-page strings for webhook-api (DE).
// /humanizer pro + legal-reviewer applied. DSGVO claims as process; SSPL = quelloffen.
module.exports = {
  "pageTitle": "Webhook-API, ein POST genügt zum Zustellen | Hook0",
  "pageDescription": "Hook0 bietet eine schlanke REST-Webhook-API, ein POST löst ein Event aus, automatische HMAC-Signatur, konfigurierbare Wiederholungsversuche und SDKs für Python und Node.js. Dauerhaft kostenlos.",
  "pageModified": "2026-06-27",
  "track": "de-webhook-api",
  "hero": {
    "eyebrow": "Webhook-API",
    "titleLine1": "Die Webhook-API, die einfach läuft",
    "titleLine2": "in 30 Minuten",
    "subtitle": "Ein POST aus deinem Backend. Hook0 erledigt HMAC, Wiederholungsversuche, DLQ und die Zustellprotokolle. SDKs für Python und Node.js. Quelloffen (SSPL-1.0), kostenloser Tarif, keine Kreditkarte.",
    "ctaPrimary": "Kostenlos starten",
    "ctaSecondary": "API-Referenz lesen",
    "microcopy": "100 Events/Tag kostenlos. Keine Kreditkarte. Quelloffen (SSPL-1.0)."
  },
  "socialProof": true,
  "codeExample": {
    "eyebrow": "Code",
    "h2": "Sende dein erstes Event in 30 Sekunden",
    "subtitle": "Ein Endpoint, ein Payload. Kein SDK nötig, keine Webhook-Konzepte vorab zu lernen.",
    "restCode": "POST https://api.hook0.com/api/v1/event\nAuthorization: Bearer &lt;APPLICATION_AUTH_TOKEN&gt;\nContent-Type: application/json\n\n{\n  \"application_id\": \"c0ea6ffa-1972-4435-b434-ec9e93d38f42\",\n  \"event_type\":     \"invoice.paid\",\n  \"event_id\":       \"evt_Wqb1k73rXprtTm7Qdlr38G\",\n  \"payload\": {\n    \"invoice_id\": \"in_8X9aBcDeFgHiJk\",\n    \"status\":     \"paid\",\n    \"amount_eur\": 4990\n  },\n  \"labels\": { \"tenant\": \"acme\", \"env\": \"prod\" }\n}\n",
    "pythonCode": "hook0 = Hook0(\"AUTH_TOKEN\")\nhook0.message.create(\n  \"app_id\",\n  MessageIn(\n    event_type=\"invoice.paid\",\n    event_id=\"evt_123\",\n    payload={\"status\": \"paid\"}\n  )\n)",
    "nodeCode": "const hook0 = Hook0(\"AUTH_TOKEN\");\nawait hook0.message.create(\"app_id\", {\n  event_type: \"invoice.paid\",\n  event_id:   \"evt_123\",\n  payload:    { status: \"paid\" }\n});",
    "docsLabel": "Zur vollständigen API-Referenz →",
    "docsHref": "https://documentation.hook0.com/reference/",
    "docsTrack": "de-webhook-api-docs"
  },
  "capabilities": {
    "eyebrow": "In der API",
    "h2": "Was die Webhook-API für dich erledigt",
    "cards": [
      {
        "title": "HMAC-SHA256-Signatur",
        "body": "Payloads tragen eine Signatur und einen Zeitstempel. Empfänger prüfen beides. Replay-Angriffe scheitern an der Zeitstempel-Prüfung."
      },
      {
        "title": "Zwei-Phasen-Wiederholungsversuche",
        "body": "Schnelle Wiederholungen in den ersten Minuten für instabile Endpoints. Langsame Wiederholungen über Stunden und Tage bei echten Ausfällen. DLQ, sobald das Budget aufgebraucht ist."
      },
      {
        "title": "Idempotente Event-IDs",
        "body": "Übergib deine eigene <code class=\"text-green-400\">event_id</code>. Hook0 dedupliziert darauf, sodass der API-Aufruf gefahrlos erneut gesendet werden kann, ohne nachgelagert zweimal auszulösen."
      },
      {
        "title": "Zustellprotokolle und Replay",
        "body": "Header, Body, Statuscode, Latenz. Pro Versuch gespeichert. Jedes Event lässt sich per ID erneut zustellen, aus dem Dashboard oder per API."
      },
      {
        "title": "Quelloffene SDKs",
        "body": "Python und Node.js. Aus der OpenAPI-Spezifikation generiert, damit Client und API synchron bleiben."
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
        "q": "Was ist die Webhook-API von Hook0?",
        "a": "Die Webhook-API von Hook0 ist eine REST-Schnittstelle, mit der dein Backend ein Event per einzelnem HTTP-Aufruf auslöst. Hook0 signiert den Payload per HMAC, stellt ihn an jeden passenden Subscriber zu, wiederholt bei Fehlschlägen nach einem konfigurierbaren Zwei-Phasen-Backoff und protokolliert jeden Versuch. SDKs gibt es für Python, Node.js und weitere Sprachen."
      },
      {
        "q": "Wie authentifiziere ich mich an der Webhook-API?",
        "a": "Die Authentifizierung erfolgt über einen Bearer-Token (Application Authentication Token) im Header <code class=\"text-green-400\">Authorization</code>. Tokens sind auf eine Anwendung beschränkt und lassen sich jederzeit aus dem Dashboard rotieren."
      },
      {
        "q": "Enthält die Webhook-API Wiederholungsversuche und HMAC-Signaturen?",
        "a": "Ja. Jedes über die Webhook-API ausgelöste Event wird automatisch per HMAC signiert (damit Empfänger es prüfen können) und bei Zustellfehlern nach einer Zwei-Phasen-Backoff-Strategie wiederholt. Dead-Letter-Queues fangen Events ab, die ihr Wiederholungsbudget aufgebraucht haben."
      },
      {
        "q": "Welche SDKs gibt es für die Webhook-API von Hook0?",
        "a": "Offizielle SDKs umfassen Python und Node.js, dazu Community-Bibliotheken für weitere Sprachen. Die REST-API ist vollständig in der API-Referenz dokumentiert, jeder HTTP-Client funktioniert also."
      },
      {
        "q": "Hat die Webhook-API ein Rate-Limit?",
        "a": "Ja. Die Limits skalieren mit dem Tarif, der kostenlose Tarif erlaubt 100 Events pro Tag, bezahlte Stufen erhöhen sowohl Tagesvolumen als auch Burst-Rate. Selbst gehostete Deployments werden von Hook0 nicht limitiert."
      }
    ]
  },
  "related": {
    "h2": "Weiterführend (auf Englisch)",
    "links": [
      { "label": "Webhook Platform", "href": "/webhook-platform" },
      { "label": "Hook0 vs Svix", "href": "/hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "/hook0-vs-hookdeck" },
      { "label": "Build vs Buy Webhooks", "href": "/build-vs-buy-webhooks" },
      { "label": "Self-Hosted Webhooks", "href": "/self-hosted-webhooks" },
      { "label": "Open-Source Webhooks", "href": "/open-source-webhooks" }
    ]
  }
};
