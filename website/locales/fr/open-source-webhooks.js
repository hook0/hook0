// Per-page strings for open-source-webhooks (FR).
// /humanizer pro + legal-reviewer applied.
// Hook0 lui-même = « code source ouvert (SSPL-1.0) », JAMAIS « Open Source »
// (SSPL rejetée par l'OSI, risque L121-1 C. conso). Le titre/slug
// « open-source-webhooks » reste un terme CATÉGORIE/SEO qui couvre l'écosystème
// (Svix open-core, Convoy MIT, Hook0 SSPL), la règle s'applique aux claims
// sur Hook0 lui-même dans le corps.
// Souveraineté : CDN Cloudflare (USA) divulgué, data plane Clever Cloud (France).
// JAMAIS « 100 % souverain / no data sharing / CLOUD Act free ».
module.exports = {
  "pageTitle": "Meilleur serveur webhook open source (2026) | Hook0",
  "pageDescription": "Comparatif des serveurs webhook open source : Hook0 (SSPL, complet), Svix (open-core), Convoy (MIT). Cloud à partir de 59 €/mois, ou auto-hébergement pour la conformité.",
  "pageModified": "2026-06-27",
  "track": "fr-oss-webhooks",
  "hero": {
    "eyebrow": "Code source ouvert",
    "titleLine1": "Le meilleur serveur",
    "titleLine2": "webhook à code ouvert",
    "subtitle": "Hook0 est entièrement à code source ouvert sous SSPL-1.0, audite chaque ligne, auto-héberge pour la conformité, ou prends Hook0 Cloud pour l'infra managée, les mises à jour automatiques et l'hébergement applicatif en France (CDN Cloudflare US divulgué). Bootstrappé, sans astuce open-core.",
    "ctaPrimary": "Démarrer gratuitement sur le Cloud",
    "ctaPrimaryTrack": "fr-oss-webhooks-hero-cloud-signup",
    "ctaSecondary": "Tester le Playground",
    "ctaSecondaryHref": "https://play.hook0.com",
    "ctaSecondaryTrack": "fr-oss-webhooks-hero-playground",
    "trustIndicators": [
      "Code source ouvert SSPL-1.0",
      "Auto-hébergement dispo (Docker / K8s)",
      "Bootstrappé, sans VC"
    ]
  },
  "socialProof": true,
  "whyOss": {
    "eyebrow": "Pourquoi le code ouvert",
    "h2": "Pourquoi ton serveur webhook devrait être à code source ouvert",
    "cards": [
      {
        "icon": "audit",
        "title": "Auditer chaque ligne de code",
        "body": "Les webhooks transportent des payloads sensibles. Avec du code source ouvert, ton équipe sécurité audite précisément comment la donnée est manipulée, signée et livrée. Pas de boîte noire."
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
  "comparison": {
    "eyebrow": "Licences",
    "h2": "Modèles de licence webhook comparés",
    "columns": {
      "criteria": "Critère",
      "sspl": "SSPL (Hook0)",
      "openCore": "Open-core (Svix)",
      "mit": "MIT (Convoy)",
      "proprietary": "Propriétaire (Hookdeck)"
    },
    "rows": [
      {
        "criteria": "Code source disponible",
        "sspl": "Oui, 100 % sur GitHub et GitLab",
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
        "openCore": "Plan enterprise uniquement",
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
        "openCore": "Cloud US ou auto-hébergement enterprise",
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
        "q": "Hook0 est-il à code source ouvert ?",
        "a": "Oui. Hook0 est entièrement à code source ouvert sous la licence SSPL-1.0. Chaque ligne de code est sur GitHub et GitLab. Il n'y a pas d'édition enterprise propriétaire."
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
        "a": "Oui. Hook0 est à code source ouvert et auto-hébergeable sans coût de licence. Hook0 Cloud ajoute l'infra managée, les mises à jour automatiques, l'hébergement UE, le support prioritaire et un SLA, pour que tu te concentres sur ton produit plutôt que sur l'exploitation d'une infra webhook. Démarre sur le tier gratuit du Cloud (100 events/jour, sans carte bancaire)."
      }
    ]
  },
  "related": {
    "h2": "Pour aller plus loin (en anglais)",
    "links": [
      { "label": "Self-Hosted Webhooks", "href": "/self-hosted-webhooks" },
      { "label": "Hook0 vs Svix", "href": "/hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "/hook0-vs-hookdeck" },
      { "label": "Build vs Buy Webhooks", "href": "/build-vs-buy-webhooks" },
      { "label": "Hook0 Alternatives", "href": "/hook0-alternatives" }
    ]
  }
};
