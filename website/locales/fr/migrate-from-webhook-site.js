// Per-page strings for migrate-from-webhook-site (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// SSPL pour Hook0 = « code source ouvert (SSPL-1.0) », JAMAIS « open source » seul.
module.exports = {
  pageTitle: 'Alternative à webhook.site, passe à Hook0 en 30 min | Hook0',
  pageDescription: 'Tu cherches une alternative à webhook.site ? Hook0 est l\'upgrade prod-ready, avec signatures HMAC, relances configurables, portail abonné, code source ouvert (SSPL-1.0). Gratuit à vie.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Alternative à webhook.site',
    titleBefore: 'Au-delà de webhook.site ?',
    titleAccent: 'Passe à Hook0',
    subtitle: 'webhook.site capte du HTTP entrant pour debug. Hook0 envoie tes webhooks vers tes clients, signés HMAC, avec relances, logs de livraison et portail abonné. Métier différent, même domaine. Code source ouvert (SSPL-1.0).',
    ctaPrimary: 'Passer à Hook0',
    ctaSecondary: 'Essayer le Playground',
    ctaNote: '100 events/jour gratuit. Sans carte bancaire. Code source ouvert.',
  },
  vsTable: {
    eyebrow: 'Deux outils adjacents',
    h2: 'Inspecteur entrant vs plateforme sortante',
    sub: 'webhook.site reçoit. Hook0 envoie. Choisir le bon outil dès le départ t\'évite un refactor plus tard.',
    headers: { need: 'Besoin', webhookSite: 'webhook.site', hook0: 'Hook0' },
    rows: [
      { need: 'Inspecter les requêtes entrantes pour debug', webhookSite: 'Oui', webhookSitePositive: true, hook0: 'Oui (play.hook0.com)', hook0Positive: true },
      { need: 'Envoyer des webhooks à tes clients en prod', webhookSite: 'Non', webhookSitePositive: false, hook0: 'Oui', hook0Positive: true },
      { need: 'Signer chaque payload en HMAC', webhookSite: 'Non', webhookSitePositive: false, hook0: 'Oui', hook0Positive: true },
      { need: 'Relances et dead letter queues', webhookSite: 'Non', webhookSitePositive: false, hook0: 'Oui', hook0Positive: true },
      { need: 'Portail abonné pour tes clients', webhookSite: 'Non', webhookSitePositive: false, hook0: 'Oui', hook0Positive: true },
      { need: 'Auto-héberger sur ton infra', webhookSite: 'Non', webhookSitePositive: false, hook0: 'Gratuit (SSPL-1.0)', hook0Positive: true },
      { need: 'Tier gratuit', webhookSite: 'Oui', webhookSitePositive: true, hook0: 'Oui', hook0Positive: true },
    ],
  },
  migration: {
    eyebrow: 'Migration',
    h2: 'De webhook.site à la prod en 30 minutes',
    steps: [
      { index: 'Étape 1', title: 'Crée l\'application', body: 'Inscris-toi, crée une application. Tu reçois un token d\'auth et un application ID dans la foulée. Sans carte bancaire.' },
      { index: 'Étape 2', title: 'Remplace l\'URL', body: 'Remplace l\'URL webhook.site par un appel API Hook0. SDK Python ou Node.js, ou simple HTTP.' },
      { index: 'Étape 3', title: 'Confie-leur le portail', body: 'Mets le portail abonné devant tes clients. Ils enregistrent leurs propres endpoints, font tourner leurs propres clés, consultent leurs propres logs de livraison.' },
    ],
    codeBlock: '// Avant. webhook.site comme receveur de debug.\nfetch("https://webhook.site/abcd-1234", {\n  method: "POST",\n  body: JSON.stringify(payload)\n});\n\n// Après. Hook0 comme plateforme webhook de prod.\nawait hook0.message.create("&lt;application_id&gt;", {\n  event_type: "invoice.paid",\n  event_id:   "evt_Wqb1k73rXprtTm7Qdlr38G",\n  payload\n});\n\n// Pour tes abonnés, payloads signés, relances\n// automatiques, logs de livraison rejouables.\n',
    docsLink: 'Lire le guide de démarrage →',
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions de migration',
    items: [
      { q: 'Hook0 est-il une alternative à webhook.site ?', a: 'Oui. Hook0 est l\'alternative prod-ready quand tu dépasses webhook.site. Là où webhook.site est un inspecteur de requêtes (« quel payload j\'ai reçu ? »), Hook0 est une plateforme webhook qui envoie des events à tes abonnés, les signe en HMAC, relance en cas d\'échec et stocke les logs de livraison. webhook.site sert au debug, Hook0 sert en prod.' },
      { q: 'Comment migrer de webhook.site vers Hook0 ?', a: 'Inscris-toi sur Hook0 (gratuit, sans carte bancaire), crée une application, remplace l\'URL webhook.site dans ton code par un seul appel REST à l\'API Hook0. Tu obtiens livraison signée HMAC, relances, dead letter queues et portail abonné, sans modifier ton code au-delà de l\'endpoint API.' },
      { q: 'Puis-je encore inspecter les payloads webhook bruts avec Hook0 ?', a: 'Oui. Chaque event envoyé via Hook0 est loggé avec la requête complète, la réponse, le code de statut et la latence. Tu peux rejouer n\'importe quel event depuis le dashboard. Pour des tests à la volée sans compte, play.hook0.com te permet de générer des URLs webhook jetables comme le fait webhook.site.' },
      { q: 'Hook0 est-il en code source ouvert contrairement à webhook.site ?', a: 'Oui. Hook0 est entièrement en code source ouvert (SSPL-1.0) et auto-hébergeable via Docker Compose ou Kubernetes. webhook.site est un SaaS au code fermé. Si tu dois garder le trafic sur ta propre infrastructure, Hook0 est la réponse.' },
    ],
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'webhook-platform', label: 'Plateforme webhook' },
      { enSlug: 'webhook-api', label: 'API webhook' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy webhooks' },
    ],
  },
};
