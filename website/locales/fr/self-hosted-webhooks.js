// Per-page strings for self-hosted-webhooks (FR).
// /humanizer pro + legal-reviewer applied.
// Hook0 lui-même = « code source ouvert (SSPL-1.0) », JAMAIS « Open Source »
// (SSPL rejetée par l'OSI, risque L121-1 C. conso).
// Souveraineté : CDN Cloudflare (USA) divulgué, data plane Clever Cloud (France).
// JAMAIS « 100 % souverain / no data sharing / CLOUD Act free ».
module.exports = {
  pageTitle: 'Webhooks auto-hébergés : déploie Hook0 chez toi | Hook0',
  pageDescription: 'Auto-héberge Hook0 sur ton infra. Code source ouvert (SSPL-1.0), Docker, Kubernetes, support Postgres et S3.',
  "pageModified": "2026-07-16",
  "track": "fr-self-hosted",
  "hero": {
    "eyebrow": "Auto-hébergé",
    "titleLine1": "Plateforme webhook",
    "titleLine2": "auto-hébergée",
    "subtitle": "Déploie tes webhooks sur ton infra avec la même base de code que notre Cloud. Tes payloads webhook ne quittent jamais ton réseau. Docker Compose ou Kubernetes. Code source ouvert sous SSPL-1.0, sans verrou éditeur.",
    "ctaPrimary": "Démarrer gratuitement",
    "ctaPrimaryTrack": "fr-self-hosted-hero-register",
    "ctaSecondary": "Guide d'installation",
    "ctaSecondaryHref": "https://documentation.hook0.com/self-hosting/docker-compose",
    "ctaSecondaryTrack": "fr-self-hosted-hero-docs",
    "trustIndicators": [
      "Même code que le Cloud",
      "Licence SSPL-1.0",
      "Aucune télémétrie"
    ]
  },
  "socialProof": true,
  "whySelfHost": {
    "eyebrow": "Pourquoi auto-héberger",
    "h2": "Tes données, ton infrastructure",
    "cards": [
      {
        "icon": "shield",
        "title": "Souveraineté des données",
        "body": "Des webhooks on-premise où les payloads restent dans ton périmètre. Point. Aucun tiers ne voit tes données. Pensé pour les CISO en santé, finance, secteur public, et la conformité RGPD."
      },
      {
        "icon": "code",
        "title": "Code source ouvert SSPL-1.0",
        "body": "Sous licence SSPL-1.0. Pas d'astuce open-core, pas de fonctions cloisonnées. Chaque ligne est sur GitHub et GitLab. Tu peux l'auditer, le forker, ou envoyer une PR."
      },
      {
        "icon": "server",
        "title": "Docker et Kubernetes",
        "body": "Docker Compose pour le dev et les petits déploiements. Helm chart pour la production Kubernetes. Les deux tournent direct."
      },
      {
        "icon": "sync",
        "title": "Même code, mêmes fonctions",
        "body": "Une seule base de code. Le binaire que tu déploies est compilé depuis le même dépôt que notre Cloud. Relances, signatures, supervision, gestion des abonnements, rien n'est retiré."
      }
    ]
  },
  "deployment": {
    "eyebrow": "Déploiement",
    "h2": "Deux façons de déployer",
    "options": [
      {
        "kind": "docker",
        "title": "Docker Compose",
        "body": "Bien pour le dev, les tests et la production à petite échelle. Trois commandes, tout démarre.",
        "code": "git clone https://github.com/hook0/hook0.git<br>cd hook0<br>docker compose up -d",
        "docsHref": "https://documentation.hook0.com/self-hosting/docker-compose",
        "docsLabel": "Guide Docker Compose",
        "docsTrack": "fr-self-hosted-docker-docs"
      },
      {
        "kind": "kubernetes",
        "title": "Kubernetes",
        "body": "Pour la production. Scaling horizontal, health checks, rolling updates via Helm.",
        "code": "helm repo add hook0 https://charts.hook0.com<br>helm install hook0 hook0/hook0",
        "docsHref": "https://documentation.hook0.com/self-hosting/kubernetes",
        "docsLabel": "Guide Kubernetes",
        "docsTrack": "fr-self-hosted-k8s-docs"
      }
    ]
  },
  "whoSelfHosts": {
    "eyebrow": "Cas d'usage",
    "h2": "Qui auto-héberge Hook0 ?",
    "cards": [
      {
        "icon": "industry",
        "title": "Secteurs régulés",
        "body": "Santé, finance, secteur public. Quand ta conformité dit « pas de SaaS externe pour cette donnée », tu as quand même besoin de webhooks."
      },
      {
        "icon": "globe",
        "title": "Souveraineté des données",
        "body": "Entreprises européennes sous RGPD, ou tout acteur qui doit prouver exactement où la donnée est traitée et stockée."
      },
      {
        "icon": "lock",
        "title": "Réseaux air-gapped",
        "body": "Pas d'internet ? Pas de problème. Hook0 ne fait aucun phone-home, aucune télémétrie, aucune dépendance externe."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Questions courantes",
    "items": [
      {
        "q": "La version auto-hébergée est-elle identique au Cloud ?",
        "a": "Oui. Une seule base de code, pas d'« édition communautaire ». Ce qui tourne sur notre Cloud est ce que tu déploies sur le tien."
      },
      {
        "q": "De quelle infra ai-je besoin ?",
        "a": "Docker Compose pour les installations simples, Kubernetes (Helm) pour la production. PostgreSQL pour le stockage. Un seul nœud encaisse des milliers d'events par minute."
      },
      {
        "q": "Mes données quittent-elles mon réseau ?",
        "a": "Non. En auto-hébergement, tout reste sur votre propre infrastructure, sans télémétrie, sans phone-home et sans appel externe. C'est le seul cas où les données de webhook ne quittent jamais votre réseau. Si vous préférez un service géré, Hook0 Cloud garde son plan de données en France sur Clever Cloud (UE) ; le CDN et la protection anti-DDoS en frontal sont assurés par Cloudflare (US), divulgué dans notre liste publique de sous-traitants."
      },
      {
        "q": "Du support pour les déploiements auto-hébergés ?",
        "a": "Oui. Le support commercial couvre l'aide à l'installation, la revue de configuration et les correctifs prioritaires."
      },
      {
        "q": "Puis-je essayer Hook0 avant d'auto-héberger ?",
        "a": "Oui. Notre Cloud propose un tier gratuit à 100 events par jour, sans carte bancaire. Essaie d'abord, puis déploie on-premise quand tu es prêt."
      }
    ]
  },
  "deepDive": {
    "prefix": "Tu veux plus de détails ?",
    "linkLabel": "Lis le guide complet d'auto-hébergement dans la doc",
    "linkHref": "https://documentation.hook0.com/self-hosting/docker-compose",
    "suffix": "."
  },
  "related": {
    "h2": "Pour aller plus loin (en anglais)",
    "links": [
      { "enSlug": "hook0-vs-svix", "label": "Hook0 vs Svix" },
      { "enSlug": "hook0-vs-hookdeck", "label": "Hook0 vs Hookdeck" },
      { "enSlug": "build-vs-buy-webhooks", "label": "Build vs Buy Webhooks" },
      { "enSlug": "webhook-cost-comparison", "label": "Comparatif de coût webhook" },
      { "enSlug": "eu-webhook-infrastructure", "label": "Infrastructure webhook européenne" }
    ]
  }
};
