// Per-page strings for webhook-free (FR).
// /humanizer pro appliqué. Tutoiement. Parité stricte avec locales/en/webhook-free.js
// (mêmes sections, mêmes 8 questions FAQ, même tableau).
// Faits vérifiés : offre Cloud gratuite = src/includes/_pricing.ejs (1 développeur,
// 1 application, 10 types d'events, 10 abonnements, 100 events/jour, events en trop
// bloqués, 7 jours de rétention) ; licence SSPL-1.0 (LICENSE.txt) ; play.hook0.com
// sans compte. Cellules concurrents alignées sur svix-alternatives / webhook-service.
// Les réponses faq.items[].a doivent rester identiques au texte visible.
module.exports = {
  "pageTitle": "Webhook gratuit : cloud, auto-hébergé ou testeur | Hook0",
  "pageDescription": "Trois façons d'utiliser Hook0 gratuitement : tester tes webhooks dans le navigateur sans compte, auto-héberger le serveur open source (SSPL-1.0) sans plafond, ou l'offre cloud gratuite sans carte bancaire.",
  "pageModified": "2026-10-08",
  "track": "fr-webhook-gratuit",
  "selfHostHref": "https://documentation.hook0.com/self-hosting/docker-compose",
  "hero": {
    "eyebrow": "Webhooks gratuits",
    "titleLine1": "Des webhooks gratuits, de trois façons",
    "titleLine2": "Teste-en un tout de suite, ou fais tourner le serveur gratuitement",
    "subtitleHtml": "Trois choses sont vraiment gratuites chez Hook0, sans paywall caché. <strong class=\"text-gray-200\">play.hook0.com</strong> inspecte et teste n'importe quel webhook, sans compte. Le <strong class=\"text-gray-200\">serveur open source</strong> (SSPL-1.0) s'auto-héberge sur ton infra, sans plafond de volume. Et <strong class=\"text-gray-200\">Hook0 Cloud</strong> a une offre gratuite : 100 events par jour, sans carte bancaire, sans fonctionnalité bridée. Prends celle dont tu as vraiment besoin.",
    "ctaPlay": "Tester un webhook",
    "ctaSelfHost": "Auto-héberger le serveur gratuitement",
    "microcopy": "Sans compte pour tester. Auto-hébergement sans plafond. Offre cloud gratuite, sans carte bancaire."
  },
  "flow": {
    "title": "Comment Hook0 livre un webhook",
    "desc": "Un event est signé en HMAC, livré, relancé automatiquement après une erreur 503, puis accepté avec une réponse 200.",
    "event": "event",
    "sign": "signe",
    "deliver": "livre",
    "retry": "relance",
    "caption": "Chaque event est signé (HMAC), livré, relancé automatiquement en cas d'échec, puis journalisé, en cloud comme en auto-hébergé."
  },
  "router": {
    "eyebrow": "Par où commencer",
    "h2": "Recevoir ou envoyer ? Choisis ton option gratuite",
    "subtitle": "« Webhook gratuit » recouvre deux besoins différents. Le schéma montre lequel est le tien. Les deux sont gratuits.",
    "diagram": {
      "title": "Recevoir ou envoyer des webhooks avec Hook0",
      "desc": "À gauche, tu reçois et inspectes un webhook qu'un service t'envoie, avec l'outil gratuit play.hook0.com. À droite, tu envoies tes propres webhooks signés en HMAC à tes utilisateurs, avec l'offre Cloud gratuite ou le serveur open source auto-hébergé.",
      "receiveEyebrow": "RECEVOIR ET DÉBOGUER",
      "receiveSub": "play.hook0.com, sans compte",
      "service": "un service",
      "serviceSub": "Stripe, GitHub…",
      "playSub": "inspecter, rejouer",
      "sendEyebrow": "ENVOYER SIGNÉ",
      "sendSub": "Cloud ou auto-hébergé, en prod",
      "hook0Sub": "signe, relance",
      "signed": "signé HMAC",
      "autoRetry": "relance auto sur 5xx",
      "yourUser": "ton client"
    },
    "captionHtml": "<strong class=\"text-gray-300\">À gauche</strong>, tu reçois et inspectes un webhook avec l'outil gratuit <a href=\"https://play.hook0.com\" target=\"_blank\" rel=\"noopener\" class=\"text-indigo-400 hover:text-indigo-300 underline\" onclick=\"trackEvent('CTA', 'Click', 'fr-webhook-gratuit-roles-play')\">play.hook0.com</a>. <strong class=\"text-gray-300\">À droite</strong>, tu envoies tes propres webhooks signés à tes utilisateurs, gratuitement sur le <a href=\"/fr/webhooks-auto-heberges\" class=\"text-indigo-400 hover:text-indigo-300 underline\" onclick=\"trackEvent('CTA', 'Click', 'fr-webhook-gratuit-roles-selfhost')\">serveur auto-hébergé</a> ou sur l'offre Cloud gratuite.",
    "test": {
      "title": "Je veux juste tester un webhook",
      "body": "Génère une URL publique, inspecte les payloads qu'un service t'envoie, rejoue-les. Dans ton navigateur, rien à installer, sans inscription.",
      "cta": "Ouvrir play.hook0.com →",
      "note": "Gratuit pour toujours. Outil public, open source."
    },
    "infra": {
      "title": "Il me faut une infra webhook pour mon produit",
      "body": "Signe, relance, diffuse et surveille les webhooks que tu envoies à tes propres utilisateurs. Auto-héberge le serveur open source, ou démarre sur l'offre cloud gratuite.",
      "ctaSelfHost": "Auto-héberger gratuitement (SSPL)",
      "ctaCloud": "Démarrer gratuitement sur le Cloud",
      "note": "Open source sans plafond, ou 100 events par jour sur le Cloud."
    }
  },
  "ways": {
    "eyebrow": "Trois options gratuites",
    "h2": "Trois façons d'utiliser Hook0 gratuitement, et la limite de chacune",
    "subtitle": "Pas de paywall surprise. Voici exactement ce qui est gratuit et où chaque option s'arrête.",
    "cards": [
      {
        "id": "play",
        "title": "Playground",
        "badge": "Gratuit pour toujours, sans compte",
        "body": "Reçois, inspecte, débogue et rejoue des payloads de webhook dans le navigateur. Le moyen le plus rapide de voir ce qu'un service envoie vraiment.",
        "bestFor": "Idéal pour déboguer une intégration tout de suite.",
        "cta": "Ouvrir le playground →",
        "href": "https://play.hook0.com",
        "external": true
      },
      {
        "id": "selfhost",
        "title": "Auto-hébergé (open source)",
        "badge": "Gratuit pour toujours, volume illimité",
        "body": "Fais tourner le serveur Hook0 complet sur ton infra, sous SSPL-1.0. Même code que le cloud, toutes les fonctions de livraison, tes données restent chez toi. Aucune licence à payer, aucun plafond de volume imposé par Hook0.",
        "bestFor": "Idéal pour la production avec un contrôle total et sans compteur.",
        "cta": "Guide d'auto-hébergement →",
        "href": "https://documentation.hook0.com/self-hosting/docker-compose",
        "external": false
      },
      {
        "id": "cloud",
        "title": "Offre Cloud gratuite",
        "badge": "Gratuit pour toujours, sans carte bancaire",
        "body": "Géré par nous, rien à faire tourner. 1 application, 10 types d'events, 10 abonnements, jusqu'à 100 events par jour, 7 jours de rétention. Toutes les fonctionnalités incluses. Au-delà de 100 events par jour, les events sont bloqués.",
        "bestFor": "Idéal pour un projet perso ou un vrai essai, sans infra à gérer.",
        "cta": "Démarrer gratuitement sur le Cloud →",
        "href": "https://app.hook0.com/register",
        "external": false
      }
    ]
  },
  "compare": {
    "eyebrow": "Comparaison",
    "h2": "Ce que « gratuit » veut dire d'un outil webhook à l'autre",
    "subtitle": "La plupart des offres webhook « gratuites » s'arrêtent à un tier cloud plafonné. Hook0 s'auto-héberge gratuitement avec les mêmes fonctionnalités que son cloud.",
    "headers": {
      "criteria": "Critère",
      "hook0": "Hook0",
      "svix": "Svix",
      "hookdeck": "Hookdeck",
      "webhooksite": "webhook.site"
    },
    "rows": [
      { "criteria": "Offre cloud gratuite", "hook0": "Oui, sans carte bancaire", "svix": "Oui", "hookdeck": "Oui", "webhooksite": "Oui (testeur uniquement)" },
      { "criteria": "Auto-hébergement gratuit", "hook0": "Oui, toutes les fonctionnalités (SSPL)", "svix": "Serveur MIT, certaines fonctions du cloud absentes", "hookdeck": "Outpost uniquement (Apache-2.0)", "webhooksite": "Non" },
      { "criteria": "Envoyer des webhooks à tes utilisateurs", "hook0": "Oui", "svix": "Oui", "hookdeck": "Oui", "webhooksite": "Non (réception uniquement)" },
      { "criteria": "Outil gratuit pour tester des webhooks", "hook0": "Oui (play.hook0.com)", "svix": "Oui", "hookdeck": "Console", "webhooksite": "Oui" },
      { "criteria": "Code source complet disponible", "hook0": "Oui, à 100 % (GitHub et GitLab)", "svix": "Serveur cœur uniquement", "hookdeck": "Outpost uniquement", "webhooksite": "Non" },
      { "criteria": "Où tournent les données", "hook0": "Data plane européen sur chaque offre, ou auto-hébergé", "svix": "Régions US et UE, ou auto-hébergé", "hookdeck": "Régions US, UE et Asie", "webhooksite": "Variable" }
    ],
    "footnote": "Comparaison des offres documentées publiquement, dernier relevé en octobre 2026. Vérifie les offres actuelles de chaque éditeur avant de choisir."
  },
  "limits": {
    "eyebrow": "Sans surprise",
    "h2": "Où s'arrête le « gratuit »",
    "intro": "Mieux vaut le dire franchement qu'un paywall caché. Voici exactement quand tu paies :",
    "itemsHtml": [
      "<strong class=\"text-gray-200\">L'auto-hébergement reste gratuit pour toujours</strong>, sans licence ni plafond de volume. Tu ne paies que ta propre infrastructure.",
      "<strong class=\"text-gray-200\">Les offres Cloud payantes</strong> (Startup à 59 €/mois HT, puis Pro) débloquent plus de volume, plus d'applications et un support dédié. L'offre gratuite a déjà la signature HMAC, les relances, les journaux et le portail, aucune fonctionnalité n'est réservée aux offres payantes.",
      "<strong class=\"text-gray-200\">Tu paies le volume et le support.</strong> Si tu dépasses 100 events par jour sur le cloud, ou s'il te faut un SLA géré, c'est là que commence le payant."
    ],
    "cta": "Voir tous les tarifs →",
    "ctaHref": "/fr/tarifs"
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Questions sur les webhooks gratuits",
    "items": [
      {
        "q": "Hook0 est-il vraiment gratuit ?",
        "a": "Oui, de trois façons. play.hook0.com est un outil gratuit pour tester et inspecter des webhooks dans ton navigateur, sans compte. Le serveur open source s'auto-héberge gratuitement pour toujours, sans plafond de volume imposé par Hook0 (SSPL-1.0). Et Hook0 Cloud a une offre Developer gratuite : 100 events par jour, sans carte bancaire, sans fonctionnalité bridée. Aucune mauvaise surprise au moment de payer."
      },
      {
        "q": "Existe-t-il un serveur webhook open source gratuit ?",
        "a": "Oui. Hook0 est un serveur webhook gratuit et open source sous SSPL-1.0, publié sur GitHub et GitLab. Il fournit des manifestes Docker Compose et Kubernetes et tourne sur PostgreSQL. L'auto-hébergement est gratuit, sans licence à payer ni plafond de volume imposé par Hook0. C'est le même code que le cloud, avec les mêmes fonctionnalités."
      },
      {
        "q": "Quelles sont les limites de l'offre gratuite de Hook0 Cloud ?",
        "a": "L'offre Developer gratuite comprend 1 développeur, 1 application, 10 types d'events, 10 abonnements, jusqu'à 100 events par jour et 7 jours de rétention des données. La signature HMAC, les relances, les journaux de livraison et le portail abonné sont inclus. Aucune carte bancaire n'est demandée et l'offre reste gratuite pour toujours pour les projets perso."
      },
      {
        "q": "Puis-je tester un webhook gratuitement sans compte ?",
        "a": "Oui. play.hook0.com te donne une URL publique pour recevoir, inspecter et rejouer des payloads de webhook directement dans ton navigateur. Rien à installer, aucune inscription. L'outil est gratuit et lui-même open source."
      },
      {
        "q": "L'auto-hébergement de Hook0 est-il gratuit ?",
        "a": "Oui. Hook0 n'impose ni licence à payer ni plafond de volume quand tu auto-héberges. Tu fournis l'infrastructure (Docker ou Kubernetes, plus PostgreSQL) et tu as le support de la communauté sur Discord. La montée en charge gérée, un SLA de disponibilité et les mises à jour gérées sont ce que tu prends en charge toi-même en auto-hébergement."
      },
      {
        "q": "Quelle différence entre play.hook0.com et Hook0 Cloud ?",
        "a": "play.hook0.com est un outil gratuit pour recevoir, inspecter et déboguer les webhooks que d'autres services t'envoient. Hook0 Cloud est l'infrastructure pour envoyer des webhooks signés à tes propres utilisateurs en production, avec relances, dead letter queue et portail abonné. Play sert à déboguer, le Cloud ou l'auto-hébergement servent à livrer."
      },
      {
        "q": "L'offre cloud gratuite expire-t-elle ou demande-t-elle une carte bancaire ?",
        "a": "Non. L'offre Developer gratuite n'a pas de limite de durée et ne demande aucune carte bancaire. Elle reste gratuite pour toujours pour les projets perso."
      },
      {
        "q": "Que se passe-t-il si je dépasse l'offre cloud gratuite ?",
        "a": "Au-delà de 100 events par jour sur l'offre gratuite, les events sont bloqués jusqu'au lendemain. Pour en envoyer plus, tu passes à une offre payante (Startup à 59 €/mois HT, ou Pro). Les offres payantes débloquent seulement plus de volume et de support, aucune fonctionnalité n'y est réservée. Si tu ne veux aucun plafond de volume, auto-héberge le serveur open source gratuitement."
      }
    ]
  },
  "related": {
    "h2": "À lire aussi",
    "links": [
      { "label": "play.hook0.com", "href": "https://play.hook0.com", "external": true },
      { "label": "Testeur webhook", "href": "/fr/testeur-webhook" },
      { "label": "Migrer depuis webhook.site", "href": "/fr/migrer-depuis-webhook-site" },
      { "label": "Webhooks open source", "href": "/fr/webhooks-open-source" },
      { "label": "Webhooks auto-hébergés", "href": "/fr/webhooks-auto-heberges" },
      { "label": "Plateforme webhook", "href": "/fr/plateforme-webhook" },
      { "label": "Service webhook", "href": "/fr/service-webhook" },
      { "label": "Tarifs", "href": "/fr/tarifs" },
      { "label": "Hook0 vs Svix", "href": "/fr/hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "/fr/hook0-vs-hookdeck" }
    ]
  }
};
