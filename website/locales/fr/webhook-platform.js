// Per-page strings for webhook-platform (FR).
// /humanizer pro + legal-reviewer applied.
module.exports = {
  pageTitle: 'Plateforme webhooks en Europe, alignée RGPD | Hook0',
  pageDescription: 'Plateforme webhooks à code source ouvert (SSPL-1.0) : HMAC, relances, file des échecs, portail abonnés. Données en France (Clever Cloud).',
  "pageModified": "2026-06-25",
  "track": "fr-webhook-plateforme",
  "hero": {
    "eyebrow": "Plateforme de webhooks",
    "titleLine1": "Des webhooks qui arrivent",
    "titleLine2": "fiables, vérifiables, en Europe",
    "subtitle": "Relances automatiques, signatures HMAC et logs complets. Les données applicatives sont traitées en France (Clever Cloud). Code source ouvert (SSPL-1.0), auto-hébergeable. Éditeur sans maison-mère américaine.",
    "ctaPrimary": "Commencer gratuitement",
    "ctaSecondary": "Tester le Playground",
    "microcopy": "100 events/jour gratuits. Sans carte bancaire. Code source ouvert (SSPL-1.0)."
  },
  "socialProof": false,
  "capabilities": {
    "eyebrow": "Fonctionnalités",
    "h2": "Aucun webhook perdu",
    "subtitle": "Une livraison de webhooks prête pour la production, c'est environ trois sprints d'infrastructure. Tu les économises.",
    "cards": [
      {
        "title": "Signatures HMAC",
        "body": "Chaque payload est signé pour que tes destinataires vérifient qu'une requête vient bien de toi. Les horodatages couvrent les attaques par rejeu."
      },
      {
        "title": "Relances en deux phases",
        "body": "Relances rapides pour les coupures brèves, lentes sur plusieurs jours pour les vraies pannes. Réglables par souscription, avec une dead-letter queue au bout."
      },
      {
        "title": "Journaux de livraison",
        "body": "Requête, réponse, code HTTP, latence. Rejoue n'importe quel event depuis le dashboard. Ton support arrête de deviner."
      },
      {
        "title": "Portail abonnés",
        "body": "Une interface prête à l'emploi où tes utilisateurs gèrent leurs endpoints, secrets et filtres d'events. Fini les tickets pour faire tourner une clé."
      },
      {
        "title": "SSPL, pas d'open-core",
        "body": "Le même code en cloud et en auto-hébergement. Docker Compose ou Kubernetes. Aucune édition entreprise qui cache les fonctions utiles."
      },
      {
        "title": "Hébergé en Europe",
        "body": "Données applicatives en France chez Clever Cloud, conçu pour la conformité RGPD. CDN Cloudflare US divulgué dans le <a href=\"/fr/accord-traitement-donnees\">DPA</a>. Important quand tes clients regardent où atterrissent leurs events."
      }
    ]
  },
  "howItWorks": {
    "eyebrow": "Code",
    "h2": "Un appel API pour livrer un event",
    "code": "// Déclenche un event depuis n'importe où dans ton backend.\nawait hook0.message.create(\"&lt;application_id&gt;\", {\n  event_type: \"invoice.paid\",\n  event_id:   \"evt_Wqb1k73rXprtTm7Qdlr38G\",\n  payload: {\n    invoice_id: \"in_8X9aBcDeFgHiJk\",\n    status:     \"paid\",\n    amount_eur: 4990\n  }\n});\n\n// Ce que Hook0 fait après cet appel :\n// signe le payload en HMAC, diffuse à chaque subscriber\n// concerné, relance les livraisons en échec selon un plan\n// en deux phases, et conserve requête et réponse pour le rejeu.\n",
    "docsLabel": "Lire le guide de démarrage (en anglais) →",
    "docsHref": "https://documentation.hook0.com/docs/getting-started",
    "docsTrack": "fr-webhook-plateforme-docs"
  },
  "dataResidency": {
    "eyebrow": "Résidence des données & RGPD",
    "h2": "Tes données restent en UE, expliqué honnêtement",
    "paras": [
      {
        "html": "Les données applicatives de Hook0 Cloud sont traitées exclusivement dans l'UE, en France chez <a href=\"https://www.clever-cloud.com\" target=\"_blank\" rel=\"noopener\" class=\"text-indigo-400 hover:text-indigo-300 underline\">Clever Cloud</a>. L'éditeur est FGRibreau SARL, une société française sans maison-mère américaine."
      },
      {
        "html": "Comme CDN, nous utilisons Cloudflare, un groupe américain soumis par principe au CLOUD&nbsp;Act américain. Quelles données transitent par le CDN et sur quelle base juridique : c'est précisé dans notre accord de traitement (DPA). Nous ne prétendons pas être « 100&nbsp;% souverains ». Nous disons où sont tes données et qui les traite. Un DPA avec la liste complète des sous-traitants est disponible sur demande."
      },
      {
        "html": "<strong>Pour les environnements régulés :</strong> une livraison de webhooks opérée dans l'UE, avec une chaîne de sous-traitants claire, soutient tes obligations sur le risque fournisseur (NIS2) et les prestataires informatiques tiers (DORA). Hook0 n'est pas un prestataire certifié de ces cadres, mais une brique documentable et établie dans l'UE. L'auto-hébergement te donne en plus un contrôle maximal.",
        "emphasis": true
      }
    ]
  },
  "openSource": {
    "eyebrow": "Code source ouvert",
    "h2": "Tu gardes le contrôle",
    "html": "Hook0 est à code source ouvert sous licence <a href=\"https://spdx.org/licenses/SSPL-1.0.html\" target=\"_blank\" rel=\"noopener\" class=\"text-indigo-400 hover:text-indigo-300 underline\">SSPL-1.0</a> (Server Side Public License). Tu peux faire tourner la même plateforme sur ta propre infrastructure dans l'UE, quand tu veux. C'est ta porte de sortie garantie. Le code source est consultable à tout moment sur <a href=\"{{GITHUB}}\" target=\"_blank\" rel=\"noopener\" class=\"text-indigo-400 hover:text-indigo-300 underline\">GitHub</a>."
  },
  "buildVsBuy": {
    "eyebrow": "Faire soi-même vs Hook0",
    "h2": "Ce que coûte vraiment le fait-maison",
    "head": [
      "Critère",
      "Le faire soi-même",
      "Hook0"
    ],
    "rows": [
      [
        "Temps jusqu'au premier webhook",
        "3+ sprints",
        "30 minutes"
      ],
      [
        "Logique de relance",
        "Tu conçois le backoff et la DLQ",
        "Deux phases, configurable"
      ],
      [
        "Signatures HMAC",
        "À implémenter et documenter",
        "Inclus, documenté"
      ],
      [
        "Interface abonnés",
        "Encore un sprint",
        "Portail prêt à l'emploi"
      ],
      [
        "Rejeu et débogage",
        "Tes propres dashboards",
        "Logs et rejeu intégrés"
      ],
      [
        "Maintenance continue",
        "Pour toujours, ton équipe",
        "La nôtre"
      ],
      [
        "Risque de lock-in",
        "Aucun, mais tout repose sur toi",
        "Aucun. Code source complet, auto-hébergeable."
      ]
    ],
    "footHtml": "Les tarifs transparents sont sur notre <a href=\"/pricing\" class=\"text-indigo-400 hover:text-indigo-300 underline\">page de prix</a> (en anglais)."
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Questions sur la plateforme de webhooks",
    "items": [
      {
        "q": "Qu'est-ce qu'une plateforme de webhooks ?",
        "a": "Une plateforme de webhooks, c'est l'infrastructure de production entre ton application et les endpoints de tes clients. Elle signe les payloads en HMAC, relance les livraisons en échec avec un backoff configurable, conserve les journaux de livraison et expose un portail abonnés où tes utilisateurs gèrent leurs propres endpoints. Hook0 livre tout ça en service managé ou en stack Docker auto-hébergée."
      },
      {
        "q": "Pourquoi utiliser une plateforme plutôt que la construire ?",
        "a": "Un système de webhooks prêt pour la production, c'est plus de trois sprints d'ingénierie pour les relances, les dead-letter queues, les signatures HMAC, le monitoring et une interface abonnés, plus la maintenance permanente. Avec une plateforme comme Hook0, tu es en production en 30 minutes et tu récupères ce temps d'ingénierie pour ton produit."
      },
      {
        "q": "Où sont hébergées mes données ? Hook0 est-il conçu pour la conformité RGPD ?",
        "a": "Les données applicatives de Hook0 Cloud sont traitées en France chez Clever Cloud. L'éditeur est une SARL française sans maison-mère américaine. Comme CDN, nous utilisons Cloudflare ; c'est précisé, avec la base juridique du transfert, dans notre accord de traitement (DPA). Hook0 est conçu pour la conformité RGPD ; un DPA avec la liste complète des sous-traitants est disponible sur demande. Pour un contrôle maximal, tu peux auto-héberger Hook0 dans l'UE."
      },
      {
        "q": "Hook0 est-il gratuit ?",
        "a": "Oui. Hook0 a une offre gratuite à vie, sans carte bancaire. Tu peux aussi auto-héberger gratuitement le même code sur ton infrastructure, sous licence SSPL-1.0. Les offres payantes ne débloquent que le volume et le support dédié ; aucune fonction n'est verrouillée."
      },
      {
        "q": "Puis-je auto-héberger Hook0 ?",
        "a": "Oui. Toute la plateforme est à code source ouvert sous SSPL-1.0 et fournie avec Docker Compose et des manifestes Kubernetes. L'auto-hébergement est gratuit, sans édition entreprise, et l'édition auto-hébergée a les mêmes fonctionnalités que le cloud."
      }
    ]
  },
  "related": {
    "h2": "Pour aller plus loin (en anglais)",
    "links": [
      {
        "label": "Webhook Platform",
        "href": "/webhook-platform"
      },
      {
        "label": "Open-Source Webhooks",
        "href": "/open-source-webhooks"
      },
      {
        "label": "Self-Hosted Webhooks",
        "href": "/self-hosted-webhooks"
      },
      {
        "label": "Security & Compliance",
        "href": "/security"
      },
      {
        "label": "Tarifs",
        "href": "/pricing"
      }
    ]
  },
  "cta": {
    "h2": "Tu as mieux à construire",
    "subtitle": "Arrête de bâtir de l'infrastructure de webhooks. Livre des fonctionnalités. En quelques minutes tu es en route, et tu peux migrer vers l'auto-hébergement quand tu veux.",
    "ctaPrimary": "Commencer gratuitement",
    "ctaSecondary": "Guide de démarrage",
    "ctaSecondaryHref": "https://documentation.hook0.com/docs/getting-started",
    "track": "fr-webhook-plateforme-cta-register",
    "badges": [
      "Sans carte bancaire",
      "En route en 5 minutes",
      "Code source ouvert (SSPL-1.0), migrable à tout moment"
    ]
  }
};
