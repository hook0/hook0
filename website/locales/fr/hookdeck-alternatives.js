// Per-page strings for hookdeck-alternatives (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon, pas de --.
// Hook0 = « code source ouvert (SSPL-1.0) ». Convoy MPL-2.0 = OSI, donc « open source » OK pour Convoy.
module.exports = {
  pageTitle: 'Alternatives à Hookdeck (2026), plateformes webhook qui en font plus | Hook0',
  pageDescription: 'Hookdeck est un proxy webhook, pas une plateforme webhook. Compare les vraies alternatives, Hook0, Svix, Convoy pour envoyer, gérer et monitorer tes webhooks.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Alternatives à Hookdeck',
    titleAccent: 'Hookdeck est un proxy, il te faut peut-être une plateforme',
    subtitleHtml: 'Hookdeck est une passerelle webhook, il reçoit et route les webhooks entrants. Si tu dois <strong class="text-white">envoyer</strong> des webhooks à tes clients (avec relances, signatures, gestion des abonnés), Hookdeck ne le fait pas. Ces alternatives, si.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  gatewayVsPlatform: {
    eyebrow: 'Distinction clé',
    h2: 'Passerelle vs plateforme, quelle différence ?',
    sub: 'Choisis la mauvaise catégorie et tu finiras par construire la moitié manquante toi-même.',
    cards: [
      { title: 'Passerelle webhook (Hookdeck)', bodyHtml: 'Une passerelle se place entre un émetteur webhook tiers et ton application. Elle reçoit les webhooks entrants, les buffer, relance les livraisons échouées, route les events vers le bon endpoint. C\'est en gros un reverse proxy pour webhooks. <strong class="text-white">Tu es le consommateur.</strong>', color: 'indigo' },
      { title: 'Plateforme webhook (Hook0, Svix, Convoy)', bodyHtml: 'Une plateforme te permet d\'envoyer des webhooks à tes clients. Tu publies des events, la plateforme les livre avec relances, signatures HMAC et portail de gestion des abonnés. <strong class="text-white">Tu es le producteur.</strong> C\'est ce qu\'il te faut pour ajouter des webhooks à ton produit.', color: 'green' },
    ],
  },
  comparison: {
    eyebrow: 'Comparaison de fonctionnalités',
    h2: 'Hookdeck vs les alternatives',
    sub: 'Cinq options, un seul tableau. Ce qui compte le plus, c\'est de savoir si tu dois envoyer des webhooks, en recevoir, ou les deux.',
    headers: { criteria: 'Critère', hookdeck: 'Hookdeck', hook0: 'Hook0', svix: 'Svix', convoy: 'Convoy', awsEventbridge: 'AWS EventBridge' },
    rows: [
      { criteria: 'Type', hookdeckHtml: 'Passerelle / proxy webhook', hook0Html: 'Plateforme webhook complète', svixHtml: 'Plateforme webhook (open core)', convoyHtml: 'Plateforme webhook', awsEventbridgeHtml: 'Bus d\'events (écosystème AWS)' },
      { criteria: 'Envoi de webhooks', hookdeckHtml: 'Non', hook0Html: 'Oui (feature centrale)', svixHtml: 'Oui', convoyHtml: 'Oui', awsEventbridgeHtml: 'Oui (via API Destinations)' },
      { criteria: 'Réception de webhooks', hookdeckHtml: 'Oui (feature centrale)', hook0Html: 'Non (par design)', svixHtml: 'Non', convoyHtml: 'Oui (entrant + sortant)', awsEventbridgeHtml: 'Oui (ingestion d\'events)' },
      { criteria: 'Auto-hébergement', hookdeckHtml: 'Non', hook0Html: 'Gratuit (Docker / K8s)', svixHtml: 'Plan entreprise uniquement', convoyHtml: 'Oui (auto-géré)', awsEventbridgeHtml: 'Non (AWS uniquement)' },
      { criteria: 'Open source', hookdeckHtml: 'Non (code fermé)', hook0Html: 'Oui (SSPL-1.0, source intégrale)', svixHtml: 'Partiel (open core, entreprise fermé)', convoyHtml: 'Oui (MPL-2.0)', awsEventbridgeHtml: 'Non (propriétaire AWS)' },
      { criteria: 'Tier gratuit', hookdeckHtml: 'Oui (100k events/mois)', hook0Html: 'Oui, sans carte bancaire', svixHtml: 'Oui', convoyHtml: 'Édition communauté uniquement', awsEventbridgeHtml: 'Pay-per-use (facturation AWS)' },
      { criteria: 'Hébergement des données', hookdeckHtml: 'Aux États-Unis', hook0Html: 'Europe (Clever Cloud FR, CDN Cloudflare US) ou auto-hébergement', svixHtml: 'Aux États-Unis', convoyHtml: 'Auto-hébergé uniquement', awsEventbridgeHtml: 'Multi-régions (AWS)' },
      { criteria: 'Financement', hookdeckHtml: '3,5 M$ VC-funded', hook0Html: '100% bootstrappé', svixHtml: '17 M$ VC-funded', convoyHtml: 'VC-funded', awsEventbridgeHtml: 'Amazon (entreprise cotée)' },
    ],
  },
  whyLookBeyond: {
    eyebrow: 'Pourquoi regarder au-delà de Hookdeck',
    h2: 'Quand Hookdeck ne suffit pas',
    sub: 'Hookdeck fait une chose et la fait bien, recevoir et router les webhooks. Mais il y a des cas clairs où ça ne suffit pas.',
    cards: [
      { title: 'Tu dois envoyer des webhooks', body: 'Hookdeck n\'envoie pas de webhooks. Point. Si ton produit doit notifier ses clients via webhooks avec relances, signatures HMAC et logs de livraison, il te faut une plateforme webhook, Hook0, Svix ou Convoy.', color: 'green' },
      { title: 'Tu veux auto-héberger', body: 'Hookdeck est cloud uniquement. Pas d\'option auto-hébergement. Si la compliance ou les règles de souveraineté des données t\'imposent ton infrastructure, Hook0 et Convoy sont tous deux auto-hébergeables sans coût.', color: 'indigo' },
      { title: 'Tu as besoin d\'un hébergement européen', body: 'Hookdeck est basé aux États-Unis. Hook0 Cloud est hébergé en Europe, conçu pour le RGPD dès le départ. Si tu es une boîte UE qui manipule des données sensibles, le choix est limpide.', color: 'green' },
      { title: 'Tu veux auditer le code source', body: 'Hookdeck est en code fermé. Tu ne peux pas voir comment tes données webhook sont traitées. Tout le code de Hook0 est ouvert sous SSPL-1.0, donc tu peux lire et auditer chaque ligne.', color: 'indigo' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Hookdeck est-il en open source ?', a: 'Non. Hookdeck est en code fermé et cloud uniquement. Tu ne peux pas inspecter le code, l\'auditer ni l\'auto-héberger. Convoy (MPL-2.0) est en open source au sens OSI. Hook0 est à code source ouvert sous SSPL-1.0, tu peux inspecter, modifier et auto-héberger toute la plateforme.' },
      { q: 'Puis-je auto-héberger Hookdeck ?', a: 'Non. Hookdeck ne propose pas d\'option auto-hébergée. C\'est cloud uniquement. Si tu dois faire tourner ton infrastructure webhook sur tes propres serveurs pour la compliance, la souveraineté des données ou des raisons de coût, Hook0 et Convoy supportent tous deux l\'auto-hébergement.' },
      { q: 'Quelle est la différence entre un proxy webhook et une plateforme webhook ?', a: 'Un proxy webhook (comme Hookdeck) se place entre un émetteur webhook et ton application. Il reçoit, route et relance les webhooks entrants. Une plateforme webhook (comme Hook0 ou Svix) te permet d\'envoyer des webhooks à tes clients. Elle gère la livraison, les relances, les signatures et la gestion des abonnés à ta place. Si tu veux ajouter des webhooks à ton produit, il te faut une plateforme, pas un proxy.' },
      { q: 'Quelle est la meilleure alternative à Hookdeck pour envoyer des webhooks ?', a: 'Hook0, si tu as besoin d\'envoyer des webhooks. Tu publies des events, Hook0 les livre à tes abonnés avec relances, signatures HMAC et un dashboard de gestion. Le code est en code source ouvert (SSPL-1.0), tu peux l\'auto-héberger, l\'entreprise est bootstrappée et le cloud tourne en Europe.' },
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
