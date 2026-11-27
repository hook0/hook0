// Per-page strings for open-source-webhooks (FR).
// /humanizer pro + legal-reviewer applied.
// Hook0 lui-même = « open source (SSPL-1.0) », la parenthèse est obligatoire,
// JAMAIS « open source » seul (SSPL non approuvée par l'OSI, risque L121-1
// C. conso). Le titre/slug
// « open-source-webhooks » reste un terme CATÉGORIE/SEO qui couvre l'écosystème
// (Svix open-core, Convoy Elastic License 2.0, Hook0 SSPL), la règle s'applique aux claims
// sur Hook0 lui-même dans le corps.
// Souveraineté : CDN Cloudflare (USA) divulgué, data plane Clever Cloud (France).
// JAMAIS « 100 % souverain / no data sharing / CLOUD Act free ».
module.exports = {
  "tldr": {
    "h2": "Quel serveur de webhooks open source choisir ?",
    "body": "Prends Hook0 si tu veux que le code que tu auto-héberges soit celui qui tourne dans le cloud managé. Il est open source (SSPL-1.0), tourne sur Docker ou Kubernetes avec toutes ses fonctions, et Hook0 Cloud est hébergé dans l'UE à partir de 59 €/mois. Les autres projets se distinguent surtout par la licence et par l'endroit où vit le portail client. Svix a un cœur MIT, et son App Portal est documenté pour le service hébergé. Convoy est sous Elastic License 2.0 et range les Portal Links dans une offre auto-hébergée payante. Hookdeck Outpost (Apache-2.0) inclut un portail par tenant, mais demande PostgreSQL, Redis et une file de messages. WebhookX (Apache-2.0) réserve le multi-tenant à une licence commerciale et ne documente aucun cloud managé."
  },
  pageTitle: 'Webhooks open source (SSPL-1.0) : Hook0',
  pageDescription: 'Serveurs de webhooks open source comparés : Hook0 (SSPL-1.0), Svix, Convoy, Hookdeck Outpost et WebhookX. Licence, stack, portail, cloud.',
  "pageModified": "2026-11-25",
  "track": "fr-oss-webhooks",
  "hero": {
    "eyebrow": "Open source",
    "titleLine1": "Le meilleur serveur",
    "titleLine2": "webhook à code ouvert",
    "subtitle": "Hook0 est open source sous SSPL-1.0, audite chaque ligne, auto-héberge pour la conformité, ou prends Hook0 Cloud pour l'infra managée, les mises à jour automatiques et l'hébergement applicatif en France (CDN Cloudflare US divulgué). Bootstrappé, sans astuce open-core.",
    "ctaPrimary": "Démarrer gratuitement sur le Cloud",
    "ctaPrimaryTrack": "fr-oss-webhooks-hero-cloud-signup",
    "ctaSecondary": "Tester le Playground",
    "ctaSecondaryHref": "https://play.hook0.com",
    "ctaSecondaryTrack": "fr-oss-webhooks-hero-playground",
    "trustIndicators": [
      "Open source SSPL-1.0",
      "Auto-hébergement dispo (Docker / K8s)",
      "Bootstrappé, sans VC"
    ]
  },
  "socialProof": true,
  "whyOss": {
    "eyebrow": "Pourquoi le code ouvert",
    "h2": "Pourquoi ton serveur webhook devrait être open source",
    "cards": [
      {
        "icon": "audit",
        "title": "Auditer chaque ligne de code",
        "body": "Les webhooks transportent des payloads sensibles. Avec du open source, ton équipe sécurité audite précisément comment la donnée est manipulée, signée et livrée. Pas de boîte noire."
      },
      {
        "icon": "lock",
        "title": "Pas de verrou éditeur",
        "body": "Si l'éditeur disparaît, monte ses prix ou pivote, tu gardes le code. Forke, maintiens, ou migre à ton rythme. Ton infrastructure webhook t'appartient."
      },
      {
        "icon": "selfhost",
        "title": "Auto-hébergement partout",
        "body": "Déploie sur tes serveurs, ton propre cloud, ou des réseaux air-gapped. Le code ouvert, c'est toi qui choisis où vivent les données, pas l'éditeur."
      },
      {
        "icon": "community",
        "title": "Communauté et contributions",
        "body": "Signale des bugs, envoie des PR, demande des fonctionnalités. Les projets à code ouvert alignent les intérêts, le produit s'améliore parce que les utilisateurs le façonnent directement."
      }
    ]
  },
  "projects": {
    "eyebrow": "Projets",
    "h2": "Cinq serveurs de webhooks open source côte à côte",
    "intro": "Chaque ligne vient du dépôt public, de la documentation ou de la page de prix du projet, consultés les 24 et 25 novembre 2026. Les sources sont listées en bas de page.",
    "columns": {
      "project": "Projet",
      "license": "Licence",
      "stack": "Stack",
      "selfHost": "Auto-hébergement",
      "portal": "Portail client",
      "managed": "Cloud managé"
    },
    "rows": [
      {
        "project": "Hook0",
        "license": "SSPL-1.0",
        "stack": "Rust, PostgreSQL (Pulsar + S3 en option)",
        "selfHost": "Docker Compose, Kubernetes",
        "portal": "API et tableau de bord des abonnements",
        "managed": "Hook0 Cloud, hébergé dans l'UE, offre gratuite puis à partir de 59 €/mois"
      },
      {
        "project": "Svix",
        "license": "Cœur MIT, fonctions enterprise fermées",
        "stack": "Rust, PostgreSQL, Redis en option",
        "selfHost": "Docker Compose ou un seul conteneur",
        "portal": "App Portal, documenté pour le service hébergé",
        "managed": "Svix Cloud, gratuit jusqu'à 50 000 messages/mois"
      },
      {
        "project": "Convoy",
        "license": "Elastic License 2.0",
        "stack": "Go, PostgreSQL, Redis",
        "selfHost": "Docker Compose, Helm",
        "portal": "Portal Links, dans l'offre auto-hébergée Premium payante",
        "managed": "Convoy Cloud"
      },
      {
        "project": "Hookdeck Outpost",
        "license": "Apache-2.0",
        "stack": "Go, PostgreSQL, Redis, RabbitMQ ou SQS",
        "selfHost": "Docker Compose, Helm, Railway",
        "portal": "Portail par tenant inclus",
        "managed": "Hookdeck, 10 $ par million d'événements, les 50 000 premiers offerts"
      },
      {
        "project": "WebhookX",
        "license": "Apache-2.0 ; multi-tenant, gestion des secrets et connect-auth sous licence commerciale",
        "stack": "Go, PostgreSQL 13+, Redis 6.2+",
        "selfHost": "Docker Compose ou binaire",
        "portal": "Aucun documenté",
        "managed": "Aucun documenté"
      }
    ]
  },
  "comparison": {
    "eyebrow": "Licences",
    "h2": "Modèles de licence webhook comparés",
    "columns": {
      "criteria": "Critère",
      "sspl": "SSPL (Hook0)",
      "openCore": "Open-core (Svix)",
      "mit": "MIT (permissif)",
      "proprietary": "Propriétaire (SaaS fermé)"
    },
    "rows": [
      {
        "criteria": "Code source disponible",
        "sspl": "Oui, toute la base de code sur GitHub et GitLab",
        "openCore": "Partiel (cœur uniquement)",
        "mit": "Oui, sur GitHub",
        "proprietary": "Non"
      },
      {
        "criteria": "Audit du code",
        "sspl": "Chaque ligne, y compris l'infra",
        "openCore": "Cœur seulement, l'enterprise reste fermé",
        "mit": "Oui",
        "proprietary": "Non"
      },
      {
        "criteria": "Auto-hébergement possible",
        "sspl": "Oui, gratuit (Docker / K8s)",
        "openCore": "Oui, serveur cœur MIT ; on-prem supporté en Enterprise",
        "mit": "Oui, gratuit",
        "proprietary": "Non"
      },
      {
        "criteria": "Parité fonctionnelle (cloud = auto-hébergé)",
        "sspl": "Même base de code, toutes les fonctions",
        "openCore": "Éditions distinctes, fonctions cloisonnées",
        "mit": "Le cloud est un produit séparé",
        "proprietary": "Sans objet (cloud uniquement)"
      },
      {
        "criteria": "Risque de verrou éditeur",
        "sspl": "Faible, fork à tout moment, PostgreSQL standard",
        "openCore": "Moyen, les fonctions enterprise se perdent si tu pars",
        "mit": "Faible, MIT autorise le fork",
        "proprietary": "Élevé, pas de source, pas d'auto-hébergement"
      },
      {
        "criteria": "Souveraineté des données",
        "sspl": "Contrôle total (auto-hébergé ou Cloud UE)",
        "openCore": "Cloud US ou UE, ou auto-hébergement du cœur",
        "mit": "Auto-hébergement uniquement",
        "proprietary": "Cloud US, pas d'option d'auto-hébergement"
      },
      {
        "criteria": "Contributions communautaires",
        "sspl": "PR ouvertes sur toute la base de code",
        "openCore": "PR sur le cœur uniquement",
        "mit": "PR ouvertes",
        "proprietary": "Pas d'accès communauté"
      },
      {
        "criteria": "Restrictions de licence",
        "sspl": "Interdit de revendre en service managé",
        "openCore": "Les fonctions enterprise demandent une licence payante",
        "mit": "Aucune (permissif)",
        "proprietary": "Usage soumis aux conditions de l'éditeur"
      }
    ]
  },
  "differentiators": {
    "eyebrow": "Différenciation Hook0",
    "h2": "Ce qui distingue Hook0",
    "cards": [
      {
        "icon": "audit",
        "title": "Auditer chaque ligne",
        "body": "Les webhooks transportent des payloads sensibles. Tes équipes sécurité et conformité relisent toute la base de code, API, worker, schéma de base, avant la mise en production. Pas de boîte noire propriétaire."
      },
      {
        "icon": "lock",
        "title": "Pas de verrou éditeur",
        "body": "Migre quand tu veux. Pas d'API propriétaire, pas de format de données propriétaire. Hook0 stocke tout dans du PostgreSQL standard. Si tu pars, tes données et la connaissance de ton infra partent avec toi."
      },
      {
        "icon": "cloud",
        "title": "Le Cloud quand tu veux",
        "body": "Démarre sur Hook0 Cloud pour la mise en production la plus rapide. Bascule en auto-hébergé plus tard pour la conformité ou la souveraineté des données, ou l'inverse. Même base de code, aucun effort de migration."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Questions courantes",
    "items": [
      {
        "q": "Hook0 est-il open source ?",
        "a": "Oui. Hook0 est open source sous la licence SSPL-1.0. Chaque ligne de code est sur GitHub et GitLab. Il n'y a pas d'édition enterprise propriétaire."
      },
      {
        "q": "Quelle licence Hook0 utilise-t-il ?",
        "a": "SSPL-1.0 (Server Side Public License). Tu peux auto-héberger, modifier et auditer le code librement. La seule restriction, c'est de proposer Hook0 en service managé à des tiers sans ouvrir le code de ta stack."
      },
      {
        "q": "Qu'est-ce que l'auto-hébergement de Hook0 demande ?",
        "a": "Auto-héberger Hook0 demande Docker Compose ou Kubernetes et une base de données PostgreSQL. Tu gères ton infra, le scaling, les sauvegardes, les mises à jour et la supervision. Le binaire auto-hébergé est compilé depuis la même base de code que Hook0 Cloud, aucune fonction n'est retirée. Hook0 Cloud s'occupe de tout ça pour toi si tu préfères la voie managée."
      },
      {
        "q": "Quels sont les risques des outils webhook open-core ?",
        "a": "Les outils webhook open-core découpent leur base de code en une édition communautaire gratuite et une édition enterprise payante. Le risque : les fonctions sur lesquelles tu t'appuies aujourd'hui (SSO, supervision avancée, support de l'auto-hébergement) peuvent passer derrière le paywall à n'importe quel moment. Tu ne peux pas auditer les parties à code fermé pour la sécurité. Et si tu auto-héberges, tu fais tourner une version amputée. Hook0 évite ça, toute la base de code est disponible sous SSPL, sans édition enterprise."
      },
      {
        "q": "Hook0 est-il vraiment gratuit en auto-hébergement ?",
        "a": "Oui. Hook0 est open source et auto-hébergeable sans coût de licence. Hook0 Cloud ajoute l'infra managée, les mises à jour automatiques, l'hébergement UE, le support prioritaire et un SLA, pour que tu te concentres sur ton produit plutôt que sur l'exploitation d'une infra webhook. Démarre sur le tier gratuit du Cloud (100 events/jour, sans carte bancaire)."
      },
      {
        "q": "Existe-t-il une plateforme de webhooks open source qui soit aussi hébergée dans l'UE ?",
        "a": "Oui. Hook0 est open source (SSPL-1.0), et Hook0 Cloud fait tourner son plan de données sur Clever Cloud en France (dans l'UE) sur chaque offre. Beaucoup de services de webhooks hébergés dans l'UE sont fermés et cloud-only, donc vous ne pouvez ni lire le code ni le faire tourner vous-même. Avec Hook0, vous pouvez auditer le code, l'auto-héberger ou utiliser le cloud UE ; le CDN en frontal du cloud est Cloudflare (US), divulgué dans notre liste publique de sous-traitants."
      },
      {
        "q": "WebhookX est-il une bonne alternative open source pour envoyer des webhooks ?",
        "a": "WebhookX est une passerelle de webhooks en Go, sous Apache-2.0, qui reçoit, valide, transforme et livre des événements. Elle demande PostgreSQL 13+ et Redis 6.2+, et sa dernière version au 25 novembre 2026 est la v1.2.0. Le multi-tenant, la gestion des secrets et le plugin connect-auth demandent une licence commerciale, et son README ne documente ni cloud managé ni portail client. Si tu envoies des webhooks à beaucoup de clients et que tu veux isoler chaque tenant sans licence à part, Hook0 l'inclut dans le code SSPL-1.0."
      },
      {
        "q": "Qu'est-ce que Hookdeck Outpost, et que vaut-il face à Hook0 ?",
        "a": "Outpost est le moteur d'envoi de Hookdeck : Go, Apache-2.0, multi-tenant, avec un portail par tenant. Auto-hébergé, il tourne sur PostgreSQL, Redis et une file RabbitMQ ou SQS ; en version managée, Hookdeck facture 10 $ par million d'événements après les 50 000 premiers. Hook0 tourne sur PostgreSQL seul dans la plupart des cas, avec Pulsar et S3 seulement pour les gros débits, et son cloud managé est hébergé dans l'UE."
      }
    ]
  },
  "comparisonSources": {
    "competitors": [
      "Svix",
      "Convoy",
      "Hookdeck Outpost",
      "WebhookX"
    ],
    "scope": "Cette page compare des serveurs de webhooks open source ou à source disponible sur la licence, la stack, l'auto-hébergement, le portail client et l'offre managée. Les faits viennent du dépôt public, de la documentation et de la page de prix de chaque projet.",
    "sources": [
      {
        "label": "Code source, licence et dépendances de Svix sur GitHub",
        "url": "https://github.com/svix/svix-webhooks",
        "consulted": "2026-11-25"
      },
      {
        "label": "Documentation de l'App Portal de Svix",
        "url": "https://docs.svix.com/app-portal",
        "consulted": "2026-11-25"
      },
      {
        "label": "Tarifs de Svix",
        "url": "https://www.svix.com/pricing/",
        "consulted": "2026-11-24"
      },
      {
        "label": "Code source et licence de Convoy sur GitHub",
        "url": "https://github.com/frain-dev/convoy",
        "consulted": "2026-11-25"
      },
      {
        "label": "Guide d'installation Kubernetes de Convoy",
        "url": "https://getconvoy.io/docs/deployment/install-convoy/kubernetes",
        "consulted": "2026-11-25"
      },
      {
        "label": "Tarifs de Convoy",
        "url": "https://www.getconvoy.io/pricing",
        "consulted": "2026-11-25"
      },
      {
        "label": "Code source et licence de Hookdeck Outpost sur GitHub",
        "url": "https://github.com/hookdeck/outpost",
        "consulted": "2026-11-25"
      },
      {
        "label": "Tarifs de Hookdeck Outpost",
        "url": "https://hookdeck.com/outpost/pricing",
        "consulted": "2026-11-25"
      },
      {
        "label": "Code source et licence de WebhookX sur GitHub",
        "url": "https://github.com/webhookx-io/webhookx",
        "consulted": "2026-11-25"
      },
      {
        "label": "Notes de version WebhookX v1.2.0",
        "url": "https://github.com/webhookx-io/webhookx/releases/tag/v1.2.0",
        "consulted": "2026-11-25"
      }
    ],
    "updated": "2026-11-25"
  },
  "related": {
    "h2": "Pour aller plus loin (en anglais)",
    "links": [
      { "label": "Self-Hosted Webhooks", "href": "/self-hosted-webhooks" },
      { "label": "Hook0 vs Svix", "href": "/hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "/hook0-vs-hookdeck" },
      { "label": "Hook0 vs Convoy", "href": "/hook0-vs-convoy" },
      { "label": "Build vs Buy Webhooks", "href": "/build-vs-buy-webhooks" },
      { "label": "Hook0 Alternatives", "href": "/hook0-alternatives" }
    ]
  }
};
