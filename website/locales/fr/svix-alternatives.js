// Per-page strings for svix-alternatives (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// Hook0 = « code source ouvert (SSPL-1.0) ». Svix MIT (OSI) = « open source » OK pour Svix.
module.exports = {
  pageTitle: 'Alternatives à Svix 2026 : webhooks comparés | Hook0',
  pageDescription: 'Tu évalues Svix ? Compare Hook0, Hookdeck, Convoy : tarification, auto-hébergement, licences et ce qu\'« open source » veut dire.',
  pageModified: '2026-07-16',
  breadcrumb: 'Alternatives à Svix',
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Tu cherches une alternative à Svix ?',
    titleAccent: 'Plateformes webhook comparées',
    subtitle: 'Svix est une bonne plateforme webhook. Ce n\'est pas la seule. Si tu tiens à des licences vraiment ouvertes, à l\'auto-hébergement gratuit, à la résidence des données en UE ou à un éditeur qui ne va pas faire flamber ses prix après une Series B, cette page décompose tes options.',
    ctaPrimary: 'Démarrer gratuitement avec Hook0',
    ctaSecondary: 'Essayer le Playground',
  },
  whyLookBeyond: {
    eyebrow: 'Pourquoi regarder au-delà de Svix',
    h2: 'Pourquoi les équipes regardent ailleurs',
    cards: [
      { title: 'Les limites de l\'open core', body: 'La base MIT de Svix est de l\'open source authentique. Le hic, les features entreprise comme SSO, analytics avancées et support dédié sont propriétaires. Quand tu scales, tu te cognes au paywall. Si ton équipe a besoin d\'un accès source complet, c\'est un problème.' },
      { title: '17 M$ de VC = pression', body: 'Le venture capital attend un retour. Svix a levé 17 M$, cet argent doit revenir d\'une façon ou d\'une autre, souvent via des hausses de prix, du feature gating ou un rachat. Un éditeur bootstrappé n\'a pas cette pression.' },
      { title: 'Pas d\'hébergement européen', body: 'Svix est basé aux États-Unis et ne propose pas de cloud UE. Si tu es soumis au RGPD ou à des règles de souveraineté des données, c\'est bloquant. Tu peux auto-héberger, mais ça impose leur plan entreprise.' },
    ],
  },
  comparison: {
    eyebrow: 'Comparaison de fonctionnalités',
    h2: 'Svix vs les alternatives',
    sub: 'Cinq plateformes webhook côte à côte. La donnée parle plus fort que les pages marketing.',
    headers: { criteria: 'Critère', svix: 'Svix', hook0: 'Hook0', hookdeck: 'Hookdeck', convoy: 'Convoy', hostedhooks: 'HostedHooks' },
    rows: [
      { criteria: 'Licence', svixHtml: 'MIT (open core, entreprise fermé)', hook0Html: 'SSPL-1.0 (source intégrale disponible)', hookdeckHtml: 'Code fermé', convoyHtml: 'MPL-2.0', hostedhooksHtml: 'Code fermé' },
      { criteria: 'Financement', svixHtml: '17 M$ levés en VC', hook0Html: '100% bootstrappé', hookdeckHtml: '3,5 M$ levés en VC', convoyHtml: 'Financé en VC', hostedhooksHtml: 'Bootstrappé' },
      { criteria: 'Auto-hébergement', svixHtml: 'Plan entreprise uniquement (features complètes)', hook0Html: 'Gratuit (Docker / K8s)', hookdeckHtml: 'Non', convoyHtml: 'Oui (auto-géré)', hostedhooksHtml: 'Non' },
      { criteria: 'Tier gratuit', svixHtml: 'Oui', hook0Html: 'Oui, sans carte bancaire', hookdeckHtml: 'Oui (100k events/mois)', convoyHtml: 'Édition communauté uniquement', hostedhooksHtml: 'Oui (limité)' },
      { criteria: 'Signatures HMAC', svixHtml: 'Incluses', hook0Html: 'Incluses (tous les plans)', hookdeckHtml: 'Vérification seulement', convoyHtml: 'Incluses', hostedhooksHtml: 'Incluses' },
      { criteria: 'Logique de relances', svixHtml: 'Relances automatiques', hook0Html: 'Configurable par abonnement (phases rapide + lente)', hookdeckHtml: 'Relances automatiques', convoyHtml: 'Relances automatiques', hostedhooksHtml: 'Relances automatiques' },
      { criteria: 'Hébergement des données', svixHtml: 'Aux États-Unis', hook0Html: 'Europe (Clever Cloud FR, CDN Cloudflare US) ou auto-hébergement', hookdeckHtml: 'Aux États-Unis', convoyHtml: 'Auto-hébergé uniquement', hostedhooksHtml: 'Aux États-Unis' },
      { criteria: 'Niveau open source', svixHtml: 'Partiel (open core)', hook0Html: 'Total (SSPL, sans add-ons fermés)', hookdeckHtml: 'Aucun', convoyHtml: 'Total (MPL-2.0)', hostedhooksHtml: 'Aucun' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Svix est-il vraiment open source ?', a: 'Partiellement. La base de Svix est sous licence MIT, mais les features entreprise (SSO, analytics avancées, support prioritaire) sont propriétaires et fermées. Ça s\'appelle de l\'open core. Tu peux faire tourner l\'édition communauté, mais les features clés en prod demandent un plan payant. Hook0, à l\'inverse, livre tout sous SSPL-1.0 sans add-ons fermés.' },
      { q: 'Puis-je auto-héberger Svix gratuitement ?', a: 'Tu peux auto-héberger l\'édition communauté sous MIT, mais les features entreprise ne sont pas incluses. Un auto-hébergement complet avec toutes les features exige le plan entreprise de Svix. Hook0 et Convoy proposent tous les deux un auto-hébergement gratuit avec parité fonctionnelle complète.' },
      { q: 'Quelle est la meilleure alternative à Svix pour les startups ?', a: 'Hook0 marche bien pour les startups. Tier gratuit, sans carte bancaire, tarification à l\'event à partir de 59 €/mois, et auto-hébergement gratuit via Docker ou Kubernetes. La société est 100% bootstrappée, donc pas de VC qui pousse à augmenter les prix au prochain trimestre. Convoy vaut le coup d\'œil aussi si la licence MPL-2.0 compte pour toi.' },
      { q: 'Comment la tarification de Svix se compare-t-elle aux alternatives ?', a: 'Svix propose un tier gratuit et des plans payants à l\'event, mais l\'auto-hébergement et les features entreprise demandent une tarification entreprise (contact sales). Hook0 Cloud démarre à 59 €/mois avec une tarification transparente et inclut l\'auto-hébergement gratuit sur tous les plans. Hookdeck est cloud uniquement avec une tarification à l\'event. Convoy est auto-hébergé uniquement, avec une tarification entreprise pour le support. HostedHooks propose des plans payants cloud uniquement.' },
      { q: "Quelle alternative à Svix est à la fois hébergée dans l'UE et à code source ouvert ?", a: "Hook0. Son plan de données tourne sur Clever Cloud en France (dans l'UE) sur chaque offre, et l'intégralité du serveur est à code source ouvert (SSPL-1.0), donc vous pouvez l'auditer ou l'auto-héberger. Svix est hébergé aux États-Unis et en open-core ; beaucoup de services de webhooks hébergés dans l'UE sont fermés et cloud-only. Le CDN en frontal de Hook0 Cloud est Cloudflare (US), divulgué dans la liste publique de sous-traitants." },
    ],
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Alternatives à Hook0' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy webhooks' },
    ],
  },
};
