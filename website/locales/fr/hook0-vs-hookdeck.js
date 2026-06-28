// Per-page strings for hook0-vs-hookdeck (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de middle-dot.
// Hook0 = « code source ouvert (SSPL-1.0) », JAMAIS « open source » (SSPL hors OSI, risque L121-1).
// Hookdeck = closed-source, donc pas de question de licence pour eux.
module.exports = {
  pageTitle: 'Hook0 vs Hookdeck, plateforme webhook vs gateway | Hook0',
  pageDescription: 'Compare Hook0 et Hookdeck, plateforme webhook vs gateway, code source ouvert vs propriétaire, auto-hébergeable vs cloud uniquement. Vois les différences clés.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Hookdeck',
    titleAccent: 'Alternative à code source ouvert',
    subtitle: 'Tu cherches une alternative à Hookdeck ? Hook0 est une plateforme webhook à code source ouvert (SSPL-1.0), hébergée en UE, sans verrouillage fournisseur. Hookdeck est une gateway webhook. Les deux outils ne résolvent pas le même problème. Voilà ce que chacun couvre vraiment.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  platformVsGateway: {
    eyebrow: 'Différence fondamentale',
    h2: 'Plateforme vs gateway',
    intro: 'Hook0 et Hookdeck ne résolvent pas le même problème. L\'un envoie des webhooks, l\'autre les relaie.',
    hook0: {
      title: 'Hook0, plateforme webhook',
      bullets: [
        'Envoie des webhooks vers les endpoints de tes utilisateurs',
        'Gère souscriptions, types d\'events, relances',
        'Signatures HMAC, logs de livraison, gestion des souscriptions',
        'Un appel API pour déclencher un event',
        'Code source ouvert (SSPL-1.0), auto-hébergeable',
      ],
    },
    hookdeck: {
      title: 'Hookdeck, gateway webhook',
      bullets: [
        'Couche proxy entre émetteurs et récepteurs',
        'Ajoute relances et mise en file aux webhooks existants',
        'N\'émet pas et ne génère pas de webhooks',
        'Code propriétaire, cloud uniquement',
        'Pas d\'option d\'auto-hébergement',
      ],
    },
  },
  comparison: {
    eyebrow: 'Comparaison de fonctionnalités',
    h2: 'Côte à côte',
    headers: { feature: 'Fonctionnalité', hook0: 'Hook0', hookdeck: 'Hookdeck' },
    rows: [
      { feature: 'Type', hook0Html: 'Plateforme webhook complète', hookdeckHtml: 'Gateway / proxy webhook' },
      { feature: 'Licence', hook0Html: 'SSPL-1.0 (code source ouvert)', hookdeckHtml: 'Propriétaire (code fermé)' },
      { feature: 'Auto-hébergement', hook0Html: 'Oui (Docker / K8s)', hookdeckHtml: 'Non' },
      { feature: 'Envoi de webhooks', hook0Html: 'Oui (fonction centrale)', hookdeckHtml: 'Non (proxy uniquement)' },
      { feature: 'Gestion des souscripteurs', hook0Html: 'Portail intégré', hookdeckHtml: 'Sans objet' },
      { feature: 'Signatures HMAC', hook0Html: 'Générées automatiquement', hookdeckHtml: 'Vérification uniquement' },
      { feature: 'Gestion des types d\'events', hook0Html: 'Registre complet des types d\'events', hookdeckHtml: 'Non' },
      { feature: 'Tier gratuit', hook0Html: '100 events/jour, hébergé en UE', hookdeckHtml: '100 000 events/mois' },
      { feature: 'Hébergement des données', hook0Html: 'Europe (Clever Cloud FR, CDN Cloudflare US) ou auto-hébergement', hookdeckHtml: 'Basé aux États-Unis' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    lastReviewed: 'Dernière revue, juin 2026.',
    items: [
      { q: 'Quelle est la différence entre Hook0 et Hookdeck ?', a: 'Hook0 est une plateforme webhook, tu envoies des events via API, Hook0 les livre à tes souscripteurs avec relances, signatures et monitoring. Hookdeck est une gateway qui se place entre des émetteurs et des récepteurs de webhooks existants pour ajouter de la fiabilité. Elle n\'émet pas elle-même de webhooks.' },
      { q: 'Hook0 est-il à code source ouvert ?', a: 'Le serveur Hook0 est publié sous SSPL-1.0 et les SDK sous MIT. SSPL est une licence copyleft à source disponible, tu peux inspecter, modifier et auto-héberger toute la plateforme librement. Hookdeck est en code fermé et disponible uniquement en SaaS managé.' },
      { q: 'Puis-je auto-héberger Hook0 ?', a: 'Oui. Hook0 supporte l\'auto-hébergement via Docker Compose ou Kubernetes sans coût. Hookdeck ne propose pas d\'auto-hébergement, c\'est un service cloud uniquement.' },
      { q: 'Lequel choisir ?', a: 'Si tu dois ajouter des webhooks à ton produit (envoyer des events vers les endpoints de tes utilisateurs), prends Hook0. Si tu reçois déjà des webhooks de tiers et qu\'il te faut juste un proxy de fiabilité, Hookdeck peut convenir. Ce sont deux outils pour deux problèmes différents.' },
      { q: 'Hook0 est-il hébergé en UE, contrairement à Hookdeck ?', a: 'Hook0 Cloud est opéré par une société française (FGRibreau SARL), avec le plan de données applicatif hébergé en France chez Clever Cloud. Le CDN Cloudflare (USA) reste exposé au CLOUD Act et est divulgué dans nos <a href="/fr/sous-traitants-rgpd">sous-traitants RGPD</a> et notre <a href="/fr/accord-traitement-donnees">DPA</a> (transfert encadré par SCC 2021 + TIA, EU-US DPF quand applicable). Hookdeck est une société américaine. Tu peux aussi auto-héberger Hook0 pour qu\'aucune donnée webhook ne quitte ton réseau.' },
      { q: 'Hookdeck considère-t-il Hook0 comme une alternative ?', a: 'Hookdeck publie des pages de comparaison qui incluent Hook0, et Svix aussi. Tu peux lire leurs propres évaluations à côté des nôtres.' },
    ],
  },
  deepDive: {
    prefix: 'Tu veux plus de détails ?',
    linkText: 'Lis la comparaison complète avec les schémas d\'architecture dans notre documentation',
    linkHref: 'https://documentation.hook0.com/comparisons/hookdeck-vs-hook0',
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'hookdeck-alternatives', label: 'Alternatives à Hookdeck' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Construire vs acheter ses webhooks' },
    ],
  },
};
