// Per-page strings for hook0-vs-hookdeck (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de middle-dot.
// Hook0 = « code source ouvert (SSPL-1.0) », JAMAIS « open source » (SSPL hors OSI, risque L121-1).
// Hookdeck : Outpost (livraison) sous Apache-2.0 et auto-hébergeable, Event Gateway (ingestion) fermé.
module.exports = {
  pageTitle: 'Hook0 vs Hookdeck, plateforme webhook vs gateway | Hook0',
  pageDescription: 'Compare Hook0 et Hookdeck : une plateforme webhook open source (SSPL-1.0) face à une gateway plus un produit de livraison séparé. Licences, auto-hébergement, tarifs.',
  pageModified: '2026-07-16',
  breadcrumb: 'Hook0 vs Hookdeck',
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Hookdeck',
    titleAccent: 'Alternative à code source ouvert',
    subtitle: 'Tu cherches une alternative à Hookdeck ? Hook0 est une plateforme webhook à code source ouvert (SSPL-1.0), hébergée en UE, sans verrouillage fournisseur. Hookdeck répartit le travail entre son Event Gateway et Outpost, son produit de livraison. Voilà ce que chacun couvre vraiment.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  platformVsGateway: {
    eyebrow: 'Différence fondamentale',
    h2: 'Plateforme vs gateway',
    intro: 'Hook0 envoie des webhooks depuis une seule plateforme. Hookdeck relaie le trafic entrant avec son Event Gateway et vend la livraison sortante à part, sous le nom d\'Outpost.',
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
      title: 'Hookdeck, gateway webhook + Outpost',
      bullets: [
        'Couche proxy entre émetteurs et récepteurs',
        'Ajoute relances et mise en file aux webhooks existants',
        'L\'envoi passe par un produit séparé, Outpost',
        'Outpost sous Apache-2.0, Event Gateway fermé',
        'Auto-hébergement limité à Outpost',
      ],
    },
  },
  comparison: {
    eyebrow: 'Comparaison de fonctionnalités',
    h2: 'Côte à côte',
    headers: { feature: 'Fonctionnalité', hook0: 'Hook0', hookdeck: 'Hookdeck' },
    rows: [
      { feature: 'Type', hook0Html: 'Plateforme webhook complète', hookdeckHtml: 'Event Gateway (entrant) + Outpost (sortant)' },
      { feature: 'Licence', hook0Html: 'SSPL-1.0 (code source ouvert)', hookdeckHtml: 'Outpost Apache-2.0, Event Gateway fermé' },
      { feature: 'Auto-hébergement', hook0Html: 'Oui (Docker / K8s)', hookdeckHtml: 'Outpost uniquement' },
      { feature: 'Envoi de webhooks', hook0Html: 'Oui (fonction centrale)', hookdeckHtml: 'Oui (via Outpost)' },
      { feature: 'Gestion des souscripteurs', hook0Html: 'Portail intégré', hookdeckHtml: 'Sans objet' },
      { feature: 'Signatures HMAC', hook0Html: 'Générées automatiquement', hookdeckHtml: 'Vérification uniquement' },
      { feature: 'Gestion des types d\'events', hook0Html: 'Registre complet des types d\'events', hookdeckHtml: 'Non' },
      { feature: 'Tier gratuit', hook0Html: '100 events/jour, hébergé en UE', hookdeckHtml: '100 000 events/mois' },
      { feature: 'Hébergement des données', hook0Html: 'Europe (Clever Cloud FR, CDN Cloudflare US) ou auto-hébergement', hookdeckHtml: 'Au Canada, région UE disponible' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    lastReviewed: 'Dernière revue, juillet 2026.',
    items: [
      { q: 'Quelle est la différence entre Hook0 et Hookdeck ?', a: 'Hook0 est une plateforme webhook, tu envoies des events via API, Hook0 les livre à tes souscripteurs avec relances, signatures et monitoring. L\'Event Gateway de Hookdeck se place entre des émetteurs et des récepteurs de webhooks existants pour ajouter de la fiabilité. Elle n\'émet pas elle-même de webhooks, c\'est Outpost, le second produit de Hookdeck, qui s\'en charge.' },
      { q: 'Hook0 est-il à code source ouvert ?', a: 'Le serveur Hook0 est publié sous SSPL-1.0 et les SDK sous MIT. SSPL est une licence copyleft à source disponible, tu peux inspecter, modifier et auto-héberger toute la plateforme librement. Hookdeck publie Outpost, son composant de livraison, sous Apache-2.0, et garde son Event Gateway en code fermé et disponible uniquement en SaaS managé.' },
      { q: 'Puis-je auto-héberger Hook0 ?', a: 'Oui. Hook0 supporte l\'auto-hébergement via Docker Compose ou Kubernetes sans coût, et le cloud managé fait tourner ce même code, sans fonction réservée à un palier entreprise. Hookdeck auto-héberge Outpost sous Apache-2.0, son Event Gateway est cloud uniquement, et sur Outpost managé, SSO, RBAC et SCIM démarrent au palier Growth à 499 $/mois minimum en plus du coût à l\'événement.' },
      { q: 'Lequel choisir ?', a: 'Si tu dois ajouter des webhooks à ton produit (envoyer des events vers les endpoints de tes utilisateurs), prends Hook0. Si tu reçois déjà des webhooks de tiers et qu\'il te faut juste un proxy de fiabilité, Hookdeck peut convenir. Ce sont deux outils pour deux problèmes différents.' },
      { q: 'Hook0 est-il hébergé en UE, contrairement à Hookdeck ?', a: "Hook0 Cloud est exploité par une société française (FGRibreau SARL), avec son plan de données sur Clever Cloud en France. Le CDN et la protection anti-DDoS en frontal sont assurés par Cloudflare (US), divulgué dans une liste publique de sous-traitants avec son mécanisme de transfert. Hookdeck est une société canadienne. Et comme Hook0 s'auto-héberge sur le même code, vous pouvez garder les données de webhook entièrement dans votre propre réseau." },
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
      { enSlug: 'webhook-cost-comparison', label: 'Comparatif de coût webhook' },
      { enSlug: 'eu-webhook-infrastructure', label: 'Infrastructure webhook européenne' },
    ],
  },
};
