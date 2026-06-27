// Per-page strings for hook0-vs-svix (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de middle-dot.
// Hook0 = « code source ouvert (SSPL-1.0) », JAMAIS « open source » (SSPL hors OSI, risque L121-1).
// Svix core MIT = OSI, donc « open source » OK pour Svix.
module.exports = {
  pageTitle: 'Hook0 vs Svix, comparaison des plateformes webhook | Hook0',
  pageDescription: 'Compare Hook0 et Svix, code source ouvert SSPL-1.0 vs open-core, bootstrappé vs financement VC, hébergement UE vs US, auto-hébergement sur tous les plans. Une comparaison honnête côte à côte.',
  pageModified: '2026-06-27',
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
      { title: 'Hébergement UE, hors du CLOUD Act', body: 'Hook0 Cloud tourne sur infrastructure française (Clever Cloud) et est opéré par une société française, donc hors juridiction du CLOUD Act américain. Tes payloads webhook restent en UE. Svix est basé aux États-Unis. Tu peux aussi auto-héberger pour qu\'aucune donnée webhook ne quitte ton réseau.' },
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
      { feature: 'Hébergement des données', hook0Html: 'Europe (RGPD)', svixHtml: 'Basé aux États-Unis' },
      { feature: 'Gestion des souscriptions', hook0Html: 'Inclus', svixHtml: 'App Portal (plans payants)' },
      { feature: 'Risque de verrouillage fournisseur', hook0Html: 'Aucun (source intégrale, auto-hébergeable)', svixHtml: 'Modéré (fonctions enterprise fermées)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    lastReviewed: 'Dernière revue, juin 2026.',
    items: [
      { q: 'Hook0 est-il en code source ouvert comme Svix ?', a: 'Le serveur Hook0 est publié sous SSPL-1.0 et les SDK clients sous MIT, sans palier enterprise propriétaire. SSPL est une licence copyleft à source disponible, tu peux lire, modifier et auto-héberger toute la plateforme librement. Le cœur de Svix est MIT, mais plusieurs fonctions enterprise sont fermées et réservées aux plans payants.' },
      { q: 'Comment le tier gratuit de Hook0 se compare-t-il à celui de Svix ?', a: 'Le tier gratuit de Hook0 reste gratuit à vie sans carte bancaire, 100 events par jour, signatures HMAC et monitoring de livraison, hébergé en UE. Les plans payants montent avec ton volume sur la même infrastructure managée, toutes les fonctions sont incluses, pas de paywall enterprise. Svix réserve plusieurs fonctions aux plans payants.' },
      { q: 'Hook0 supporte-t-il Standard Webhooks ?', a: 'Standard Webhooks est une spécification rédigée par Svix. Hook0 signe chaque payload en HMAC-SHA256 et documente le schéma. Le support de Standard Webhooks est prévu.' },
      { q: 'Puis-je utiliser Hook0 pour des charges réglementées ou sensibles à la conformité ?', a: 'Oui. Hook0 Cloud garde tes données webhook en UE, sur infrastructure française opérée par une société française, hors juridiction du CLOUD Act américain, ce qui est la première exigence de la plupart des équipes sensibles à la conformité. Comme le code source intégral du serveur est ouvert (SSPL-1.0), tu peux auditer précisément la façon dont les données sont gérées et tu n\'es jamais verrouillé. Les attestations tierces formelles comme SOC 2, HIPAA et PCI-DSS sont prévues.' },
      { q: 'Hook0 est-il hébergé en UE et hors du CLOUD Act américain ?', a: 'Hook0 Cloud est opéré par une société française (FGRibreau SARL) sur infrastructure française (Clever Cloud), donc hors juridiction du CLOUD Act américain. Tes payloads webhook, qui transportent souvent des données client, restent en UE. Svix et Hookdeck sont des sociétés américaines. Tu peux aussi auto-héberger Hook0 pour qu\'aucune donnée webhook ne quitte ton réseau.' },
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
    ],
  },
};
