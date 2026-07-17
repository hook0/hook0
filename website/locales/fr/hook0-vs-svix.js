// Per-page strings for hook0-vs-svix (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de middle-dot.
// Hook0 = « code source ouvert (SSPL-1.0) », JAMAIS « open source » (SSPL hors OSI, risque L121-1).
// Svix core MIT = OSI, donc « open source » OK pour Svix.
module.exports = {
  pageTitle: 'Hook0 vs Svix : plateformes webhook comparées | Hook0',
  pageDescription: 'Compare Hook0 et Svix : SSPL-1.0 vs open-core, bootstrappé vs VC, UE vs US, auto-hébergement sur tous les plans. Honnête.',
  pageModified: '2026-07-16',
  breadcrumb: 'Hook0 vs Svix',
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Svix',
    titleAccent: 'Comparaison des plateformes webhook',
    subtitle: 'Tu cherches une alternative à Svix ? Tous deux sont des plateformes webhook, mais ils divergent sur la licence, le modèle de financement, l\'hébergement et ce que veut vraiment dire « open source » en pratique. Hook0 est code source ouvert (SSPL-1.0), bootstrappé, hébergé en UE, sans verrouillage fournisseur.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  differentiators: {
    eyebrow: 'Pourquoi Hook0',
    h2: 'Différences clés',
    cards: [
      { title: 'Code source disponible, pas d\'add-ons fermés', body: 'Le serveur Hook0 est publié sous SSPL-1.0, les SDK sous MIT. Tu récupères la plateforme entière, tu la lis, tu la modifies, tu l\'auto-héberges. Le cœur de Svix est en MIT, mais les fonctions enterprise (SSO, analytics avancées, support dédié) restent fermées sur les plans payants.' },
      { title: 'Bootstrappé depuis le premier jour', body: 'Svix est financé par des VC. Les investisseurs attendent un retour, ce qui crée de la pression pour augmenter les prix ou se faire racheter. Hook0 est 100% bootstrappé. Pas de board à satisfaire, pas de mandat de croissance à tout prix.' },
      { title: 'Aucun verrouillage fournisseur', body: 'Hook0 Cloud fait tourner le même code à source ouverte que tu peux lire et auditer. Si tu en as besoin un jour, tu exportes et tu l\'exécutes toi-même (gratuit, Docker ou Kubernetes), donc tu n\'es jamais piégé dans une plateforme propriétaire. Svix réserve l\'auto-hébergement à ses clients enterprise.' },
      { title: 'Plan de données UE, edge US divulgué', body: "Le plan de données de Hook0 tourne sur Clever Cloud en France, exploité par une société française. Le CDN en frontal est Cloudflare (US), divulgué dans notre liste publique de sous-traitants avec son mécanisme de transfert. Svix est basé aux États-Unis. Et comme le même code s'auto-héberge, vous pouvez faire tourner Hook0 dans votre propre réseau, où aucune donnée de webhook ne le quitte." },
    ],
  },
  comparison: {
    eyebrow: 'Comparaison de fonctionnalités',
    h2: 'Côte à côte',
    headers: { feature: 'Fonctionnalité', hook0: 'Hook0', svix: 'Svix' },
    rows: [
      { feature: 'Licence', hook0Html: 'SSPL-1.0 (source intégrale disponible)', svixHtml: 'MIT (open-core, enterprise fermé)' },
      { feature: 'Financement', hook0Html: '100% bootstrappé', svixHtml: 'Financement VC' },
      { feature: 'Auto-hébergement', hook0Html: 'Gratuit (Docker / K8s)', svixHtml: 'Plan enterprise uniquement' },
      { feature: 'Tier gratuit', hook0Html: 'Oui, sans carte bancaire', svixHtml: 'Oui' },
      { feature: 'Signatures HMAC', hook0Html: 'Inclus (tous les plans)', svixHtml: 'Inclus' },
      { feature: 'Logique de relances', hook0Html: 'Configurable par souscription (phases rapide + lent, defaults intelligents)', svixHtml: 'Relances automatiques' },
      { feature: 'Hébergement des données', hook0Html: 'Europe (Clever Cloud FR, CDN Cloudflare US)', svixHtml: 'Basé aux États-Unis' },
      { feature: 'Gestion des souscriptions', hook0Html: 'Inclus', svixHtml: 'App Portal (plans payants)' },
      { feature: 'Risque de verrouillage fournisseur', hook0Html: 'Aucun (source intégrale, auto-hébergeable)', svixHtml: 'Modéré (fonctions enterprise fermées)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    lastReviewed: 'Dernière revue, juillet 2026.',
    items: [
      { q: 'Hook0 est-il en code source ouvert comme Svix ?', a: 'Le serveur Hook0 est publié sous SSPL-1.0 et les SDK clients sous MIT, sans palier enterprise propriétaire. SSPL est une licence copyleft à source disponible, tu peux lire, modifier et auto-héberger toute la plateforme librement. Le cœur de Svix est MIT, mais plusieurs fonctions enterprise sont fermées et réservées aux plans payants.' },
      { q: 'Comment le tier gratuit de Hook0 se compare-t-il à celui de Svix ?', a: 'Le tier gratuit de Hook0 reste gratuit à vie sans carte bancaire, 100 events par jour, signatures HMAC et monitoring de livraison, hébergé en UE. Les plans payants montent avec ton volume sur la même infrastructure managée, toutes les fonctions sont incluses, pas de paywall enterprise. Svix réserve plusieurs fonctions aux plans payants.' },
      { q: 'Hook0 supporte-t-il Standard Webhooks ?', a: 'Standard Webhooks est une spécification rédigée par Svix. Hook0 signe chaque payload en HMAC-SHA256 et documente le schéma. Le support de Standard Webhooks est prévu.' },
      { q: 'Puis-je utiliser Hook0 pour des charges réglementées ou sensibles à la conformité ?', a: "Oui, même si les attestations formelles sont encore en cours. Hook0 Cloud fait tourner son plan de données sur Clever Cloud en France, exploité par une société française, avec le CDN Cloudflare (US) divulgué dans une liste publique de sous-traitants et son mécanisme de transfert. Comme l'intégralité du code serveur est ouverte (SSPL-1.0), vous pouvez auditer exactement comment les données sont traitées, et vous pouvez auto-héberger pour qu'aucune donnée de webhook ne quitte votre propre réseau. Des attestations tierces comme SOC 2, HIPAA et PCI-DSS sont prévues." },
      { q: "Où Hook0 est-il hébergé, et comment gère-t-il l'exposition aux transferts vers les États-Unis ?", a: "Hook0 Cloud est exploité par une société française (FGRibreau SARL), avec son plan de données sur Clever Cloud en France. Le CDN et la protection anti-DDoS en frontal sont assurés par Cloudflare (US), que nous divulguons dans une liste publique de sous-traitants, encadrée par les Clauses Contractuelles Types 2021 et une analyse d'impact des transferts (TIA), plutôt que de prétendre à une exposition américaine nulle. Svix et Hookdeck sont des sociétés américaines. Si vous avez besoin que les données de webhook restent entièrement dans votre propre réseau, auto-hébergez Hook0 sur le même code." },
      { q: 'Puis-je auto-héberger Hook0 gratuitement ?', a: 'Oui. Le même code à source ouverte tourne gratuitement sur Docker Compose ou Kubernetes, ce qui te garantit de ne jamais être verrouillé. La plupart des équipes démarrent sur Hook0 Cloud (managé, hébergé en UE, tier gratuit) et gardent l\'auto-hébergement comme option de sortie. Svix ne propose l\'auto-hébergement que sur son plan enterprise.' },
      { q: 'Hook0 est-il bootstrappé ?', a: 'Oui. Hook0 est 100% bootstrappé sans aucun financement VC. Svix est financé par des VC. Bootstrappé veut dire que Hook0 rend des comptes à ses utilisateurs, pas à des investisseurs qui cherchent une sortie.' },
      { q: 'Svix et Hookdeck considèrent-ils Hook0 comme un concurrent ?', a: 'Svix et Hookdeck publient chacun des pages de comparaison qui incluent Hook0. Tu peux lire leurs propres évaluations à côté des nôtres.' },
    ],
  },
  deepDive: {
    prefix: 'Tu veux plus de détails ?',
    linkText: 'Lis la comparaison fonctionnalité par fonctionnalité dans notre documentation',
    linkHref: 'https://documentation.hook0.com/comparisons/svix-vs-hook0',
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'svix-alternatives', label: 'Alternatives à Svix' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Construire vs acheter ses webhooks' },
      { enSlug: 'webhook-cost-comparison', label: 'Comparatif de coût webhook' },
      { enSlug: 'eu-webhook-infrastructure', label: 'Infrastructure webhook européenne' },
    ],
  },
};
