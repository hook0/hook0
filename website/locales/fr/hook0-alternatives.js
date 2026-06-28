// Per-page strings for hook0-alternatives (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon, pas de --.
// Hook0 = « code source ouvert (SSPL-1.0) ». Convoy MPL-2.0 = OSI, donc « open source » OK pour Convoy.
module.exports = {
  pageTitle: 'Alternatives à Hook0 (2026), comparaison honnête | Hook0',
  pageDescription: 'Tu cherches des alternatives à Hook0 ? Compare Hook0, Svix, Hookdeck et Convoy côte à côte sur les licences, l\'auto-hébergement, la tarification et les fonctionnalités.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Alternatives à Hook0',
    titleAccent: 'Une comparaison honnête',
    subtitle: 'Tu cherches une plateforme webhook ? Quelqu\'un a publié une page « Alternatives à Hook0 », alors voici notre version de l\'histoire. Pas de spin, juste les faits, côte à côte.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  comparison: {
    eyebrow: 'Comparaison de fonctionnalités',
    h2: 'Hook0 vs les alternatives',
    sub: 'Quatre plateformes webhook, un tableau. Juge par toi-même.',
    headers: { criteria: 'Critère', hook0: 'Hook0', svix: 'Svix', hookdeck: 'Hookdeck', convoy: 'Convoy' },
    rows: [
      { criteria: 'Code source', hook0Html: 'Oui (SSPL-1.0, source intégrale)', svixHtml: 'Partiel (open core, entreprise fermé)', hookdeckHtml: 'Non (code fermé)', convoyHtml: 'Oui (MPL-2.0, open source)' },
      { criteria: 'Auto-hébergement', hook0Html: 'Gratuit (Docker / K8s)', svixHtml: 'Plan entreprise uniquement', hookdeckHtml: 'Non', convoyHtml: 'Oui (auto-géré)' },
      { criteria: 'Tier gratuit', hook0Html: 'Oui, sans carte bancaire', svixHtml: 'Oui', hookdeckHtml: 'Oui (100k events/mois)', convoyHtml: 'Édition communauté uniquement' },
      { criteria: 'Modèle tarifaire', hook0Html: 'À l\'event, transparent', svixHtml: 'À l\'event + tarifs entreprise', hookdeckHtml: 'À l\'event, cloud uniquement', convoyHtml: 'Tarification entreprise' },
      { criteria: 'Signatures HMAC', hook0Html: 'Incluses (tous les plans)', svixHtml: 'Incluses', hookdeckHtml: 'Vérification seulement', convoyHtml: 'Incluses' },
      { criteria: 'Logique de relances', hook0Html: 'Configurable par abonnement (phases rapide + lente)', svixHtml: 'Relances automatiques', hookdeckHtml: 'Relances automatiques', convoyHtml: 'Relances automatiques' },
      { criteria: 'Financement', hook0Html: '100% bootstrappé', svixHtml: '17 M$ VC-funded', hookdeckHtml: '3,5 M$ VC-funded', convoyHtml: 'VC-funded' },
      { criteria: 'Hébergement des données', hook0Html: 'Europe (Clever Cloud FR, CDN Cloudflare US) ou auto-hébergement', svixHtml: 'Aux États-Unis', hookdeckHtml: 'Aux États-Unis', convoyHtml: 'Auto-hébergé uniquement' },
      { criteria: 'Type', hook0Html: 'Plateforme webhook complète', svixHtml: 'Plateforme webhook (open core)', hookdeckHtml: 'Passerelle / proxy webhook', convoyHtml: 'Plateforme webhook' },
    ],
  },
  whatTheyLeftOut: {
    eyebrow: 'Le tableau complet',
    h2: 'Ce que leur page de comparaison ne te dit pas',
    sub: 'Hookdeck a publié une page « Alternatives à Hook0 ». On apprécie l\'attention. Voici ce qu\'ils ont laissé de côté.',
    cards: [
      { title: '« Hook0 est HTTPS uniquement »', body: 'Oui, et c\'est une feature, pas une limitation. Envoyer des payloads webhook en HTTP clair, ça fait transiter les données de tes clients en clair. Tout système sérieux en prod utilise HTTPS. On l\'impose parce que la sécurité n\'est pas optionnelle.', color: 'green' },
      { title: '« Pas de SLA publié »', body: 'Hook0 Cloud Enterprise inclut un SLA personnalisé avec support dédié. Si les garanties de disponibilité comptent, c\'est la voie la plus rapide, sans infrastructure à gérer ni équipe ops nécessaire. Hook0 est aussi en code source ouvert (SSPL-1.0), donc tu as toujours l\'option de l\'auto-héberger si la compliance l\'exige.', color: 'indigo' },
      { title: '« La tarification est floue »', body: 'Notre tarification est publique et à l\'event. Pas d\'appel commercial requis. Pas de mur « contactez-nous ». Cloud démarre à 59 €/mois, 8x moins cher que Svix pour des features comparables. Bonne chance pour obtenir cette transparence d\'un concurrent financé en VC et fermé.', color: 'green' },
      { title: 'Ce qu\'ils ne diront pas, le financement', body: 'Hookdeck a levé 3,5 M$ en VC. Svix, 17 M$. Convoy est aussi financé en VC. Hook0 est 100% bootstrappé. Quand ton fournisseur webhook doit faire x10 de revenus pour satisfaire ses investisseurs, devine à qui les prix vont monter ? Pas chez nous.', color: 'indigo' },
    ],
  },
  difference: {
    eyebrow: 'Pourquoi Hook0',
    h2: 'La différence Hook0',
    cards: [
      { title: 'Pas juste un proxy', body: 'Contrairement à Hookdeck, Hook0 envoie les webhooks à ta place, avec relances, signatures et gestion des abonnés. Pas une couche middleware.' },
      { title: 'Pas de paywall entreprise', body: 'Contrairement à Svix, chaque feature est livrée dans chaque plan. L\'auto-hébergement n\'est pas planqué derrière un appel commercial.' },
      { title: 'Européen, conçu pour la conformité RGPD', body: 'Plan de données hébergé en UE chez Clever Cloud (France). CDN via Cloudflare (USA), divulgué dans notre <a href="/fr/accord-traitement-donnees">DPA</a> et nos <a href="/fr/sous-traitants-rgpd">sous-traitants RGPD</a>. Bootstrappé, pas de board de VC américains qui décident de ta politique de données.' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Quelles sont les meilleures alternatives à Hook0 ?', a: 'Les principales alternatives à Hook0 sont Svix (open core, VC-funded), Hookdeck (passerelle webhook en code fermé) et Convoy (open source, VC-funded). Chacune résout une partie différente du problème webhook. Hook0 est la seule à être entièrement en code source ouvert (SSPL-1.0), bootstrappée et auto-hébergeable gratuitement.' },
      { q: 'Hookdeck est-il meilleur que Hook0 ?', a: 'Hookdeck est une passerelle webhook, elle proxy les webhooks existants pour la fiabilité. Hook0 est une plateforme webhook, elle envoie les webhooks à ta place avec relances, signatures et gestion des abonnés. Elles résolvent des problèmes différents. Si tu dois ajouter des webhooks à ton produit, Hook0 est le bon outil.' },
      { q: 'Devrais-je utiliser Svix ou Hook0 ?', a: 'Les deux sont des plateformes webhook, mais elles diffèrent sur la licence et le financement. Svix est en open core (les features entreprise sont fermées) et a levé 17 M$ en VC. Hook0 est en code source ouvert intégral sous SSPL, bootstrappée, et propose l\'auto-hébergement gratuit. Si l\'indépendance vis-à-vis du fournisseur et la stabilité des prix sur le long terme comptent, Hook0 est le pari plus sûr.' },
      { q: 'Combien coûte Hook0 ?', a: 'Hook0 a un tier gratuit sans carte bancaire requise. Hook0 est aussi en code source ouvert et auto-hébergeable pour les besoins de compliance. Hook0 Cloud ajoute une infrastructure managée, l\'hébergement UE, les mises à jour automatiques et le support prioritaire. Les plans payants démarrent à 59 €/mois avec une tarification à l\'event.' },
      { q: 'Hook0 tient-il la charge ?', a: 'Oui. L\'architecture de Hook0 supporte PostgreSQL seul pour la simplicité ou Pulsar + S3 pour le très haut débit. Les clients Cloud traitent des millions d\'events par jour. La même architecture tourne à l\'identique en auto-hébergé.' },
    ],
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'svix-alternatives', label: 'Alternatives à Svix' },
      { enSlug: 'hookdeck-alternatives', label: 'Alternatives à Hookdeck' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy webhooks' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
      { enSlug: 'open-source-webhooks', label: 'Meilleur serveur webhook open source' },
    ],
  },
};
