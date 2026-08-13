// Per-page strings for hookdeck-alternatives (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon, pas de --.
// Hook0 = « code source ouvert (SSPL-1.0) ». Convoy = Elastic License 2.0, source disponible non OSI.
// Hookdeck : Outpost (livraison) sous Apache-2.0 et auto-hébergeable, Event Gateway (ingestion) fermé.
module.exports = {
  pageTitle: 'Alternatives à Hookdeck 2026, plateformes webhook | Hook0',
  pageDescription: 'Hookdeck sépare passerelle entrante et livraison sortante en deux produits. Compare les alternatives, Hook0, Svix, Convoy pour envoyer et monitorer tes webhooks.',
  pageModified: '2026-07-16',
  breadcrumb: 'Alternatives à Hookdeck',
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Alternatives à Hookdeck',
    titleAccent: 'Hookdeck, ce sont deux produits. Il t\'en faut peut-être un seul',
    subtitleHtml: 'L\'Event Gateway de Hookdeck reçoit et route les webhooks entrants, et l\'envoi passe par un second produit, Outpost. Si tu veux une seule plateforme pour <strong class="text-white">envoyer</strong> des webhooks à tes clients (relances, signatures, gestion des abonnés), avec le même code en cloud managé et en auto-hébergé, ces alternatives le font.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  gatewayVsPlatform: {
    eyebrow: 'Distinction clé',
    h2: 'Passerelle vs plateforme, quelle différence ?',
    sub: 'Choisis la mauvaise catégorie et tu finiras par construire la moitié manquante toi-même.',
    cards: [
      { title: 'Passerelle webhook (Hookdeck Event Gateway)', bodyHtml: 'Une passerelle se place entre un émetteur webhook tiers et ton application. Elle reçoit les webhooks entrants, les buffer, relance les livraisons échouées, route les events vers le bon endpoint. C\'est en gros un reverse proxy pour webhooks. <strong class="text-white">Tu es le consommateur.</strong>', color: 'indigo' },
      { title: 'Plateforme webhook (Hook0, Svix, Convoy)', bodyHtml: 'Une plateforme te permet d\'envoyer des webhooks à tes clients. Tu publies des events, la plateforme les livre avec relances, signatures HMAC et portail de gestion des abonnés. <strong class="text-white">Tu es le producteur.</strong> C\'est ce qu\'il te faut pour ajouter des webhooks à ton produit.', color: 'green' },
    ],
  },
  comparison: {
    eyebrow: 'Comparaison de fonctionnalités',
    h2: 'Hookdeck vs les alternatives',
    sub: 'Cinq options, un seul tableau. Ce qui compte le plus, c\'est de savoir si tu dois envoyer des webhooks, en recevoir, ou les deux.',
    headers: { criteria: 'Critère', hookdeck: 'Hookdeck', hook0: 'Hook0', svix: 'Svix', convoy: 'Convoy', awsEventbridge: 'AWS EventBridge' },
    rows: [
      { criteria: 'Type', hookdeckHtml: 'Event Gateway (entrant) + Outpost (sortant)', hook0Html: 'Plateforme webhook complète', svixHtml: 'Plateforme webhook (open core)', convoyHtml: 'Plateforme webhook', awsEventbridgeHtml: 'Bus d\'events (écosystème AWS)' },
      { criteria: 'Envoi de webhooks', hookdeckHtml: 'Oui (Outpost, produit séparé)', hook0Html: 'Oui (feature centrale)', svixHtml: 'Oui', convoyHtml: 'Oui', awsEventbridgeHtml: 'Oui (via API Destinations)' },
      { criteria: 'Réception de webhooks', hookdeckHtml: 'Oui (feature centrale)', hook0Html: 'Non (par design)', svixHtml: 'Non', convoyHtml: 'Oui (entrant + sortant)', awsEventbridgeHtml: 'Oui (ingestion d\'events)' },
      { criteria: 'Auto-hébergement', hookdeckHtml: 'Outpost uniquement (Apache-2.0)', hook0Html: 'Gratuit (Docker / K8s)', svixHtml: 'Plan entreprise uniquement', convoyHtml: 'Oui (auto-géré)', awsEventbridgeHtml: 'Non (AWS uniquement)' },
      { criteria: 'Open source', hookdeckHtml: 'Partiel (Outpost Apache-2.0, Gateway fermé)', hook0Html: 'Oui (SSPL-1.0, source intégrale)', svixHtml: 'Partiel (open core, entreprise fermé)', convoyHtml: 'Source disponible (Elastic License 2.0)', awsEventbridgeHtml: 'Non (propriétaire AWS)' },
      { criteria: 'Tier gratuit', hookdeckHtml: 'Oui (100k events/mois)', hook0Html: 'Oui, sans carte bancaire', svixHtml: 'Oui', convoyHtml: 'Édition communauté uniquement', awsEventbridgeHtml: 'Pay-per-use (facturation AWS)' },
      { criteria: 'Hébergement des données', hookdeckHtml: 'Au Canada, région UE disponible', hook0Html: 'Europe (Clever Cloud FR, CDN Cloudflare US) ou auto-hébergement', svixHtml: 'Aux États-Unis', convoyHtml: 'Auto-hébergé uniquement', awsEventbridgeHtml: 'Multi-régions (AWS)' },
      { criteria: 'Financement', hookdeckHtml: '3,5 M$ levés en VC', hook0Html: '100% bootstrappé', svixHtml: '17 M$ levés en VC', convoyHtml: 'Financé en VC', awsEventbridgeHtml: 'Amazon (entreprise cotée)' },
    ],
  },
  whyLookBeyond: {
    eyebrow: 'Pourquoi regarder au-delà de Hookdeck',
    h2: 'Quand Hookdeck ne suffit pas',
    sub: 'Hookdeck fait une chose et la fait bien, recevoir et router les webhooks. Mais il y a des cas clairs où ça ne suffit pas.',
    cards: [
      { title: 'Tu dois envoyer des webhooks', body: 'L\'Event Gateway de Hookdeck n\'envoie pas de webhooks, c\'est Outpost qui s\'en charge, un second produit avec ses propres paliers. Si tu veux une seule plateforme qui publie tes events et les livre avec relances, signatures HMAC et logs de livraison, regarde Hook0, Svix ou Convoy.', color: 'green' },
      { title: 'Tu veux auto-héberger', body: 'Hookdeck n\'ouvre que son composant Outpost, publié sous Apache-2.0 et auto-hébergeable. L\'Event Gateway, lui, reste en code fermé et cloud uniquement. Hook0 et Convoy s\'auto-hébergent en entier sans coût de licence, et chez Hook0 le code auto-hébergé est celui qui tourne dans le cloud managé.', color: 'indigo' },
      { title: 'Tu as besoin d\'un hébergement européen', body: 'Hookdeck est une société canadienne, avec une région UE disponible sur Outpost managé. Le plan de données de Hook0 Cloud tourne en France chez Clever Cloud, conçu pour la conformité RGPD dès le départ (CDN Cloudflare US divulgué dans le <a href="/fr/accord-traitement-donnees">DPA</a>). Si tu es une boîte UE qui manipule des données sensibles, le choix est limpide.', color: 'green' },
      { title: 'Tu veux auditer le code source', body: 'Outpost est sous Apache-2.0, donc le code de livraison de Hookdeck est lisible. Mais l\'Event Gateway qui ingère tes webhooks reste fermé. Tout le code de Hook0 est ouvert sous SSPL-1.0 et le cloud managé fait tourner ce même code, donc tu peux lire et auditer chaque ligne.', color: 'indigo' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Hookdeck est-il en open source ?', a: 'En partie. Outpost, le composant de livraison de Hookdeck, est sous Apache-2.0 et auto-hébergeable, et le dépôt est actif. L\'Event Gateway qui ingère les webhooks reste en code fermé et cloud uniquement. Hook0 est à code source ouvert sous SSPL-1.0, avec toute la plateforme auto-hébergeable. Convoy est sous Elastic License 2.0, une licence à source disponible comme la SSPL-1.0 de Hook0, qui interdit de proposer Convoy en service managé.' },
      { q: 'Puis-je auto-héberger Hookdeck ?', a: 'En partie. Outpost est sous Apache-2.0 et tourne sur ton infrastructure, l\'Event Gateway n\'a pas d\'option auto-hébergée. À noter aussi, sur Outpost managé, SSO, RBAC et SCIM ne sont pas dans le palier Starter à 10 $/M, ils démarrent au palier Growth à 499 $/mois minimum en plus du coût à l\'événement. Si tu veux toute ton infrastructure webhook sur tes propres serveurs, Hook0 et Convoy s\'auto-hébergent de bout en bout, et Hook0 fait tourner le même code dans son cloud managé, sans fonction réservée à un palier entreprise.' },
      { q: 'Quelle est la différence entre un proxy webhook et une plateforme webhook ?', a: 'Un proxy webhook (comme l\'Event Gateway de Hookdeck) se place entre un émetteur webhook et ton application. Il reçoit, route et relance les webhooks entrants. Une plateforme webhook (comme Hook0 ou Svix) te permet d\'envoyer des webhooks à tes clients. Elle gère la livraison, les relances, les signatures et la gestion des abonnés à ta place. Si tu veux ajouter des webhooks à ton produit, il te faut une plateforme, pas un proxy.' },
      { q: 'Quelle est la meilleure alternative à Hookdeck pour envoyer des webhooks ?', a: 'Hook0, si tu as besoin d\'envoyer des webhooks. Tu publies des events, Hook0 les livre à tes abonnés avec relances, signatures HMAC et un dashboard de gestion. Le code est en code source ouvert (SSPL-1.0), tu peux l\'auto-héberger, l\'entreprise est bootstrappée et le cloud tourne en Europe.' },
      { q: "Quelle alternative à Hookdeck est hébergée dans l'UE et à code source ouvert ?", a: "Hook0. Hookdeck n'ouvre que Outpost, son composant de livraison, sous Apache-2.0, et son Event Gateway reste fermé et cloud uniquement. Hook0 fait tourner son plan de données sur Clever Cloud en France (dans l'UE), est à code source ouvert (SSPL-1.0) et s'auto-héberge sur Docker ou Kubernetes, donc vous pouvez lire le code ou garder les données de webhook dans votre propre réseau. Le CDN en frontal de Hook0 Cloud est Cloudflare (US), divulgué dans la liste publique de sous-traitants." },
    ],
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-alternatives', label: 'Alternatives à Hook0' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Build vs Buy webhooks' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
    ],
  },
};
