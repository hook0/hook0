// Per-page strings for webhook-api (FR).
// /humanizer pro + legal-reviewer applied. SSPL = code source ouvert (SSPL-1.0).
module.exports = {
  "pageTitle": "API Webhook, un POST suffit pour livrer un event | Hook0",
  "pageDescription": "Hook0 expose une API REST sobre, un POST pour déclencher un event, signature HMAC automatique, relances configurables et SDK Python et Node.js. Gratuit, sans carte bancaire.",
  "pageModified": "2026-06-27",
  "track": "fr-api-webhook",
  "hero": {
    "eyebrow": "API Webhook",
    "titleLine1": "L'API webhook qui marche",
    "titleLine2": "en 30 minutes",
    "subtitle": "Un POST depuis ton backend. Hook0 gère HMAC, relances, DLQ et les logs de livraison. SDK Python et Node.js. Code source ouvert (SSPL-1.0), tier gratuit, sans carte bancaire.",
    "ctaPrimary": "Démarrer gratuitement",
    "ctaSecondary": "Lire la référence API",
    "microcopy": "100 events/jour gratuits. Sans carte bancaire. Code source ouvert (SSPL-1.0)."
  },
  "socialProof": true,
  "codeExample": {
    "eyebrow": "Code",
    "h2": "Envoie ton premier event en 30 secondes",
    "subtitle": "Un endpoint, un payload. Sans SDK requis, sans concept webhook à apprendre avant.",
    "restCode": "POST https://api.hook0.com/api/v1/event\nAuthorization: Bearer &lt;APPLICATION_AUTH_TOKEN&gt;\nContent-Type: application/json\n\n{\n  \"application_id\": \"c0ea6ffa-1972-4435-b434-ec9e93d38f42\",\n  \"event_type\":     \"invoice.paid\",\n  \"event_id\":       \"evt_Wqb1k73rXprtTm7Qdlr38G\",\n  \"payload\": {\n    \"invoice_id\": \"in_8X9aBcDeFgHiJk\",\n    \"status\":     \"paid\",\n    \"amount_eur\": 4990\n  },\n  \"labels\": { \"tenant\": \"acme\", \"env\": \"prod\" }\n}\n",
    "pythonCode": "hook0 = Hook0(\"AUTH_TOKEN\")\nhook0.message.create(\n  \"app_id\",\n  MessageIn(\n    event_type=\"invoice.paid\",\n    event_id=\"evt_123\",\n    payload={\"status\": \"paid\"}\n  )\n)",
    "nodeCode": "const hook0 = Hook0(\"AUTH_TOKEN\");\nawait hook0.message.create(\"app_id\", {\n  event_type: \"invoice.paid\",\n  event_id:   \"evt_123\",\n  payload:    { status: \"paid\" }\n});",
    "docsLabel": "Parcourir la référence API complète →",
    "docsHref": "https://documentation.hook0.com/reference/",
    "docsTrack": "fr-api-webhook-docs"
  },
  "capabilities": {
    "eyebrow": "Dans l'API",
    "h2": "Ce que l'API webhook fait pour toi",
    "cards": [
      {
        "title": "Signature HMAC-SHA256",
        "body": "Les payloads transportent une signature et un horodatage. Tes destinataires vérifient les deux. Les attaques par rejeu échouent au contrôle d'horodatage."
      },
      {
        "title": "Relances en deux phases",
        "body": "Relances rapides dans les premières minutes pour les endpoints fragiles. Relances lentes sur des heures et des jours pour les vraies pannes. DLQ quand le budget de relance est épuisé."
      },
      {
        "title": "Identifiants d'event idempotents",
        "body": "Passe ton propre <code class=\"text-green-400\">event_id</code>. Hook0 déduplique dessus, donc l'appel API se rejoue sans déclencher deux fois en aval."
      },
      {
        "title": "Logs de livraison et rejeu",
        "body": "Headers, body, code HTTP, latence. Stockés par tentative. Rejoue n'importe quel event par ID, depuis le dashboard ou l'API."
      },
      {
        "title": "SDK à code source ouvert",
        "body": "Python et Node.js. Générés depuis la spec OpenAPI, donc le client et l'API restent alignés."
      },
      {
        "title": "Tier gratuit, sans verrou",
        "body": "100 events par jour, sans carte bancaire. Les offres payantes montent le volume. Chaque fonction de cette page est dans le tier gratuit."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Questions sur l'API webhook",
    "items": [
      {
        "q": "Qu'est-ce que l'API webhook de Hook0 ?",
        "a": "L'API webhook de Hook0 est une interface REST qui permet à ton backend de déclencher un event avec un seul appel HTTP. Hook0 signe ensuite le payload en HMAC, livre à chaque subscriber concerné, relance en cas d'échec avec un backoff configurable en deux phases et logge chaque tentative. Des SDK sont disponibles pour Python, Node.js et d'autres langages."
      },
      {
        "q": "Comment s'authentifier sur l'API webhook ?",
        "a": "L'authentification utilise un Bearer token (application authentication token) passé dans le header <code class=\"text-green-400\">Authorization</code>. Les tokens sont scopés à une application et se rotent depuis le dashboard à tout moment."
      },
      {
        "q": "L'API webhook inclut-elle les relances et les signatures HMAC ?",
        "a": "Oui. Chaque event déclenché via l'API webhook est signé automatiquement en HMAC (pour que tes destinataires le vérifient) et relancé selon une stratégie de backoff en deux phases en cas d'échec. Les dead-letter queues capturent les events qui épuisent leur budget de relance."
      },
      {
        "q": "Quels SDK sont disponibles pour l'API webhook Hook0 ?",
        "a": "Les SDK officiels incluent Python et Node.js, avec des librairies communautaires pour d'autres langages. L'API REST est entièrement documentée dans la référence, donc n'importe quel client HTTP fonctionne."
      },
      {
        "q": "L'API webhook a-t-elle un rate limit ?",
        "a": "Oui. Les limites de débit suivent l'offre, le tier gratuit autorise 100 events par jour, les offres payantes augmentent le volume quotidien et la bande passante en burst. Les déploiements auto-hébergés ne sont pas limités par Hook0."
      }
    ]
  },
  "related": {
    "h2": "Pour aller plus loin (en anglais)",
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
