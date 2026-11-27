// Per-page strings for webhook-api (FR).
// /humanizer pro + legal-reviewer applied. SSPL = « open source (SSPL-1.0) », jamais « open source » seul.
// Same structure and same checked facts as locales/en/webhook-api.js: change
// a fact there, change it here in the same commit.
module.exports = {
  tldr: {
    h2: 'Qu\'est-ce qu\'une API webhook ?',
    body: 'Une API webhook envoie une requête HTTP vers une URL que tu as enregistrée dès qu\'un event se produit, au lieu d\'attendre que tu la questionnes. Une API REST répond quand le client l\'appelle ; une API webhook appelle le client. Hook0 gère ce côté envoi pour ton SaaS. Ton backend envoie chaque event une seule fois en POST sur /api/v1/event, et Hook0 le signe en HMAC-SHA256, le livre à chaque souscription concernée, relance les livraisons en échec (première relance après 3 secondes environ, les suivantes jusqu\'à 10 heures d\'écart) et garde chaque tentative avec sa réponse pour que tu la rejoues. Si tu as seulement besoin de recevoir les webhooks de Stripe ou GitHub, une passerelle de webhooks entrants convient mieux.',
  },
  pageTitle: 'API webhook : des webhooks signés en un seul appel | Hook0',
  pageDescription: 'Ce qu\'est une API webhook, en quoi elle diffère d\'une API REST, et comment Hook0 envoie des webhooks signés avec relances, logs et rejeu. Gratuit.',
  "pageModified": "2026-11-26",
  "track": "fr-api-webhook",
  "hero": {
    "eyebrow": "API Webhook",
    "titleLine1": "L'API webhook",
    "titleLine2": "pour envoyer tes events à tes clients",
    "subtitle": "Une API webhook permet à ton backend de prévenir les systèmes de tes clients dès qu'un event se produit. Avec Hook0, c'est un POST : Hook0 signe l'event, le livre, relance les échecs et logge chaque tentative. SDK officiels en 11 langages, serveur open source (SSPL-1.0), tier gratuit.",
    "ctaPrimary": "Démarrer gratuitement",
    "ctaSecondary": "Lire la référence API",
    "microcopy": "100 events/jour gratuits. Sans carte bancaire. Open source (SSPL-1.0).",
    "updated": "Mis à jour le 26 novembre 2026"
  },
  "socialProof": true,
  "vsApi": {
    "h2": "Webhook ou API REST : qui lance l'appel",
    "intro": "Les deux passent par HTTP et transportent souvent du JSON. Ce qui change, c'est qui lance la requête et qui gère un destinataire hors ligne.",
    "headers": { "aspect": "Critère", "rest": "API REST (pull)", "webhook": "Webhook (push)" },
    "rows": [
      { "aspect": "Qui lance la requête", "rest": "Le client appelle le serveur", "webhook": "L'émetteur appelle une URL enregistrée par le destinataire" },
      { "aspect": "Arrivée des données", "rest": "Quand le client interroge", "webhook": "Quelques secondes après l'event" },
      { "aspect": "Requêtes inutiles", "rest": "La plupart des interrogations ne ramènent rien de neuf", "webhook": "Une requête par event et par souscription" },
      { "aspect": "Destinataire en panne", "rest": "Le client relance son propre appel", "webhook": "L'émetteur doit mettre en file et relancer" },
      { "aspect": "Authentification", "rest": "Le client envoie un token", "webhook": "Le destinataire vérifie une signature HMAC" },
      { "aspect": "Usage typique", "rest": "Lire ou modifier des données à la demande", "webhook": "Prévenir un autre système qu'il s'est passé quelque chose" }
    ],
    "outro": "La plupart des SaaS proposent une API REST pour lire et écrire les données et des webhooks pour annoncer les changements. Hook0 prend en charge la partie webhook et l'expose elle-même comme une API REST. Le <a href=\"https://documentation.hook0.com/explanation/webhook-vs-polling\" class=\"text-indigo-400 hover:text-indigo-300 underline\">guide webhook vs polling</a> (en anglais) détaille les compromis."
  },
  "codeExample": {
    "eyebrow": "Code",
    "h2": "Envoie ton premier event en 30 secondes",
    "subtitle": "Un seul endpoint. Le payload voyage en chaîne avec son content type, donc tes destinataires reçoivent exactement les octets envoyés.",
    "restCode": "POST https://app.hook0.com/api/v1/event\nAuthorization: Bearer &lt;APPLICATION_TOKEN&gt;\nContent-Type: application/json\n\n{\n  \"application_id\":       \"c0ea6ffa-1972-4435-b434-ec9e93d38f42\",\n  \"event_id\":             \"0192f5b4-7c1e-7d3a-9f2b-4e8a1c6d2b90\",\n  \"event_type\":           \"billing.invoice.paid\",\n  \"payload\":              \"{\\\"invoice_id\\\":\\\"in_8X9aBcDeFgHiJk\\\",\\\"amount_eur\\\":4990}\",\n  \"payload_content_type\": \"application/json\",\n  \"occurred_at\":          \"2026-11-26T09:30:00Z\",\n  \"labels\":               { \"tenant\": \"acme\", \"env\": \"prod\" }\n}\n",
    "pythonCode": "from hook0 import Event, Hook0Client\n\nclient = Hook0Client(\"https://app.hook0.com/api/v1\", APP_ID, TOKEN)\nclient.send_event(Event(\n    event_type=\"billing.invoice.paid\",\n    payload='{\"invoice_id\": \"in_123\"}',\n    payload_content_type=\"application/json\",\n    labels={\"env\": \"prod\"},\n))",
    "nodeCode": "import { Event, Hook0Client } from 'hook0-client';\n\nconst hook0 = new Hook0Client('https://app.hook0.com/api/v1', APP_ID, TOKEN);\nawait hook0.sendEvent(new Event(\n  'billing.invoice.paid',\n  JSON.stringify({ invoice_id: 'in_123' }),\n  'application/json',\n  { env: 'prod' }\n));",
    "docsLabel": "Suivre le démarrage en 5 minutes →",
    "docsHref": "https://documentation.hook0.com/tutorials/getting-started",
    "docsTrack": "fr-api-webhook-docs"
  },
  "capabilities": {
    "eyebrow": "Dans l'API",
    "h2": "Ce que l'API webhook fait pour toi",
    "cards": [
      {
        "title": "Signature HMAC-SHA256",
        "body": "Chaque livraison porte un header <code class=\"text-green-400\">X-Hook0-Signature</code> avec un horodatage et un HMAC-SHA256 du payload. Tes destinataires vérifient les deux, donc une vieille requête rejouée échoue au contrôle d'horodatage."
      },
      {
        "title": "Relances selon un calendrier fixe",
        "body": "Une livraison en échec est relancée après 3 secondes environ, puis 10 secondes, 3 minutes, 30 minutes, et ainsi de suite jusqu'à 10 heures entre deux tentatives, 24 relances par défaut. Une fois épuisées, la dernière tentative reste en échec avec sa réponse, prête à rejouer."
      },
      {
        "title": "Identifiants d'event idempotents",
        "body": "Passe ton propre <code class=\"text-green-400\">event_id</code> (un UUID). Envoyer deux fois le même ID renvoie 409 Conflict au lieu de créer un second event, donc un appel API relancé ne déclenche jamais deux fois en aval."
      },
      {
        "title": "Logs de livraison et rejeu",
        "body": "Headers, body, code HTTP, latence. Stockés par tentative. Rejoue n'importe quel event par ID, depuis le dashboard ou l'API."
      },
      {
        "title": "SDK open source",
        "body": "JavaScript/TypeScript, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, Lua et Zig. Générés depuis la spec OpenAPI, donc le client et l'API restent alignés."
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
        "q": "Qu'est-ce qu'une API webhook ?",
        "a": "Une API webhook envoie un POST HTTP vers une URL enregistrée quand un event se produit, donc le destinataire l'apprend sans interroger. L'API webhook de Hook0 est le côté envoi de cet échange. Ton backend déclenche un event en un appel HTTP, et Hook0 signe le payload en HMAC, le livre à chaque subscriber concerné, relance les échecs et logge chaque tentative. Les SDK officiels couvrent 11 langages."
      },
      {
        "q": "Quelle différence entre un webhook et une API ?",
        "a": "Avec une API REST, le client demande et le serveur répond. Avec un webhook, le serveur appelle le client quand quelque chose change. Le webhook supprime le polling et arrive quelques secondes après l'event, mais l'émetteur doit alors signer les requêtes, les mettre en file et relancer quand le destinataire est en panne. Hook0 prend ce travail côté émetteur à sa charge."
      },
      {
        "q": "Quelle plateforme d'events propose une API et des webhooks ?",
        "a": "Hook0 est une plateforme d'events avec une API REST en entrée et des webhooks en sortie. Tu envoies les events par l'API (ou l'un des 11 SDK), tu déclares types d'events et souscriptions par la même API, et Hook0 livre chaque event en webhook signé à chaque endpoint abonné, avec relances, logs de livraison et rejeu."
      },
      {
        "q": "Comment s'authentifier sur l'API webhook ?",
        "a": "L'authentification utilise un Bearer token (application authentication token) passé dans le header <code class=\"text-green-400\">Authorization</code>. Les tokens sont scopés à une application et se rotent depuis le dashboard à tout moment."
      },
      {
        "q": "L'API webhook inclut-elle les relances et les signatures HMAC ?",
        "a": "Oui. Chaque event envoyé par l'API webhook est signé en HMAC-SHA256 pour que tes destinataires le vérifient. Une livraison en échec est relancée selon un calendrier croissant, de 3 secondes environ à 10 heures entre deux tentatives. Quand un event épuise ses relances, la dernière tentative reste en échec avec la réponse de l'endpoint, et un appel API le rejoue."
      },
      {
        "q": "Quels SDK sont disponibles pour l'API webhook Hook0 ?",
        "a": "Hook0 maintient 11 SDK officiels : JavaScript/TypeScript, Python, Go, Rust, Java, Kotlin, C#, PHP, Ruby, Lua et Zig. Chacun s'installe depuis le registre de paquets de son langage (Zig depuis une archive taguée), envoie des events et vérifie la signature des webhooks. L'API REST est entièrement documentée dans la référence, donc n'importe quel client HTTP fonctionne aussi."
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
