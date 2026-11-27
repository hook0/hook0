// Per-page strings for hook0-vs-hookbase (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// Faits capturés le 2026-11-24 sur hookbase.app (accueil, tarifs, relances,
// portail, transforms, pages /compare/svix et /compare/hookdeck, conditions),
// listés dans comparisonSources. Même structure et mêmes garde-fous que la
// version EN : Hookbase couvre l'entrant et le sortant en managé sur Cloudflare,
// Hook0 couvre le sortant, en cloud UE ou sur tes serveurs. Prix d'entrée de
// Hookbase plus bas, dit tel quel. Aucune région d'hébergement Hookbase publiée :
// « Non précisée », jamais « US uniquement ». Aucune affirmation de certification.
module.exports = {
  pageTitle: 'Hook0 vs Hookbase : webhooks open source hébergés en UE',
  pageDescription: "Hookbase est une passerelle webhook managée sur Cloudflare, en entrant et en sortant. Hook0 envoie tes webhooks à tes utilisateurs, open source et hébergé en UE.",
  pageModified: '2026-11-24',
  breadcrumb: 'Hook0 vs Hookbase',
  tldr: {
    h2: 'Hook0 ou Hookbase : lequel convient à ton équipe ?',
    body: "Prends Hookbase si tu reçois surtout des webhooks de Stripe, GitHub ou Shopify et que tu veux une passerelle managée qui les vérifie, les transforme et les route, avec un plan gratuit et des plans payants dès 7 $ par mois. Prends Hook0 si tu envoies des webhooks à tes propres clients et que tu dois faire tourner le même code dans un cloud UE ou sur tes serveurs. Hook0 relance une livraison en échec jusqu'à 24 fois sur 8 jours sur tous les plans, signe chaque requête en HMAC-SHA256 et est édité par une société française. Hookbase se présente comme uniquement managé.",
  },
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Hookbase',
    titleAccent: 'Passerelle edge ou plateforme d’envoi européenne',
    subtitle: "Hookbase fait tourner une passerelle webhook sur le réseau de Cloudflare qui reçoit, transforme et route les webhooks, et sait aussi en envoyer. Hook0 se concentre sur l'envoi de webhooks à tes utilisateurs, avec relances gratuites, signatures HMAC, journal par tentative et portail abonnés, dans un cloud UE ou sur tes serveurs. Voici dans quel cas chacun est le meilleur choix.",
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  differentiators: {
    eyebrow: 'Ce que fait chacun',
    h2: 'Différences clés',
    cards: [
      { title: 'Le même code auto-hébergé, ou du managé', body: "Le serveur de Hook0 est open source sous SSPL-1.0 (non approuvé OSI), donc le code qui fait tourner Hook0 Cloud tourne aussi sur ton cluster Docker ou Kubernetes, et ses 11 SDK clients sont en MIT. La page de comparaison de Hookbase le décrit comme <em>managed-only</em>, et aucune page Hookbase ne propose de licence ni d'édition auto-hébergée." },
      { title: 'Des relances qui ne dépendent pas du plan', body: "Hook0 relance une livraison en échec jusqu'à 24 fois sur 8 jours, de 3 secondes à 10 heures d'écart, avec le même calendrier sur le plan gratuit et sur les plans payants. Hookbase fixe le nombre de tentatives selon le plan : 5 en Free (environ 13 minutes), 11 en Starter, 13 en Pro et 15 en Business (environ 5 jours). Voir <a href=\"https://documentation.hook0.com/explanation/webhook-retry-logic\" class=\"text-indigo-400 hover:text-indigo-300 underline\">le calendrier exact des relances Hook0</a> (en anglais)." },
      { title: 'Éditeur européen, cloud UE par défaut', body: "Hook0 est édité par une société française sans maison mère aux États-Unis, et Hook0 Cloud tourne en UE, hébergé en France par défaut. Hookbase traite les événements sur Cloudflare Workers dans plus de 300 emplacements et stocke les payloads dans Cloudflare R2. Ses pages publiques ne nomment ni entité juridique ni région d'hébergement, et ses conditions relèvent du droit du Delaware." },
      { title: 'Un seul métier, ou une passerelle plus large', body: "Hookbase couvre plus de terrain : une passerelle entrante qui vérifie les signatures de plus de 70 fournisseurs, des transforms de payload en 8 langages, des destinations de file et de stockage, des tunnels locaux et un serveur MCP. Hook0 reste sur la livraison sortante. Il envoie tel quel le payload que tu publies, filtre par type d'événement et par label, journalise chaque tentative avec son code HTTP et sa réponse, et te laisse rejouer n'importe quel événement depuis le dashboard ou l'API." },
    ],
  },
  comparison: {
    eyebrow: 'Côte à côte',
    h2: 'Comparatif des fonctionnalités',
    headers: { feature: 'Fonctionnalité', hook0: 'Hook0', hookbase: 'Hookbase' },
    rows: [
      { feature: 'Sens des webhooks', hook0Html: 'Sortant (tu envoies à tes utilisateurs)', hookbaseHtml: 'Entrant et sortant' },
      { feature: 'Plan gratuit', hook0Html: '100 événements par jour, sans carte bancaire', hookbaseHtml: '1 000 événements par mois, plafond strict' },
      { feature: 'Plans payants', hook0Html: 'Dès 59 €/mois, dépassement facturé à l’événement', hookbaseHtml: '7 $, 29 $ et 79 $/mois, ou 1 $ les 1 000 événements' },
      { feature: 'Relances automatiques', hook0Html: "Jusqu'à 24 sur 8 jours, sur tous les plans", hookbaseHtml: '5 (Free) à 15 (Business) tentatives' },
      { feature: 'Signatures', hook0Html: 'HMAC-SHA256 sur chaque requête + TLS', hookbaseHtml: 'Spécification Standard Webhooks en sortant' },
      { feature: 'Portail abonnés', hook0Html: 'Embarquable', hookbaseHtml: 'Package React (@hookbase/portal)' },
      { feature: 'Filtrage', hook0Html: "Types d'événements et labels", hookbaseHtml: "Types d'événements, labels, filtres de route" },
      { feature: 'Transforms de payload', hook0Html: 'Non, payload envoyé tel que publié', hookbaseHtml: 'Oui, 8 langages (dès Starter)' },
      { feature: 'Passerelle entrante', hook0Html: 'Non (le tunnel du CLI sert au développement local)', hookbaseHtml: 'Oui, vérification de plus de 70 fournisseurs' },
      { feature: 'Replay', hook0Html: "Par événement, depuis le dashboard ou l'API", hookbaseHtml: 'Oui, avec une dead letter queue' },
      { feature: 'Hébergement', hook0Html: 'Cloud managé UE (Clever Cloud, France) + auto-hébergement', hookbaseHtml: 'Cloudflare Workers et R2, région non précisée' },
      { feature: 'Code source', hook0Html: 'Serveur SSPL-1.0, 11 SDK MIT', hookbaseHtml: 'Uniquement managé, aucune licence publiée' },
      { feature: 'Éditeur', hook0Html: 'Société française, sans maison mère aux États-Unis', hookbaseHtml: 'Non précisé (conditions sous droit du Delaware)' },
    ],
  },
  fit: {
    h2: 'Quand Hookbase convient mieux',
    bullets: [
      "L'essentiel de ton trafic, ce sont des webhooks que tu reçois de Stripe, GitHub, Shopify ou Slack, et tu veux un seul endroit qui vérifie leurs signatures, remodèle les payloads et les route vers tes services, vers des files comme SQS ou Pub/Sub, ou vers un stockage objet comme S3 dès le plan Pro. Hook0 ne fait pas d'entrant.",
      'Tu as besoin de transforms de payload écrits en JSONata, jq, JavaScript ou Liquid entre le fournisseur et ton app.',
      'Tu cherches un plan payant sous 59 € par mois, et Hookbase Starter coûte 7 $ par mois pour 25 000 événements entrants.',
      'Tu veux piloter tes webhooks depuis un assistant IA dès aujourd\'hui, et Hookbase fournit un serveur MCP, hébergé ou local.',
    ],
    linkPrefix: "Tu compares aussi d'autres passerelles entrantes ? Consulte",
    link: { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Hook0 est-il une alternative à Hookbase ?', a: "Pour envoyer des webhooks à tes clients, oui. Les deux livrent des événements aux endpoints de tes abonnés avec signatures, relances, journal et portail. Hook0 ajoute un serveur open source que tu peux auto-héberger, un éditeur et un hébergement européens, et le même calendrier de 24 relances sur tous les plans. Pour recevoir et transformer les webhooks de tiers, Hookbase couvre un terrain que Hook0 ne couvre pas." },
      { q: 'Peut-on auto-héberger Hookbase ?', a: "Pas d'après ses pages publiques. La comparaison de Hookbase avec Svix le décrit comme managed-only, et aucune page ne mentionne de licence ni d'édition auto-hébergée. Le serveur de Hook0 est open source sous SSPL-1.0 et tourne sur Docker ou Kubernetes, avec le même code que Hook0 Cloud." },
      { q: 'Où Hookbase est-il hébergé ?', a: "Hookbase indique traiter les événements sur Cloudflare Workers dans plus de 300 emplacements et stocker les payloads chiffrés au repos dans Cloudflare R2. Ses pages publiques ne précisent ni région d'hébergement ni option de résidence des données en UE. Hook0 Cloud tourne en UE, hébergé en France par défaut." },
      { q: 'Combien de fois Hookbase relance-t-il un webhook en échec ?', a: "Ça dépend du plan : 5 tentatives en Free (environ 13 minutes), 11 en Starter (environ 28 heures), 13 en Pro (environ 3 jours) et 15 en Business (environ 5 jours), sans réglage possible. Hook0 relance jusqu'à 24 fois sur 8 jours sur tous les plans, de 3 secondes à 10 heures d'écart." },
      { q: 'Hookbase est-il moins cher que Hook0 ?', a: "En entrée de gamme, oui. Hookbase a un plan gratuit plafonné à 1 000 événements par mois et des plans payants à 7 $, 29 $ et 79 $ par mois. Hook0 a un plan gratuit de 100 événements par jour et des plans payants dès 59 € par mois, avec un dépassement facturé à l'événement, donc le trafic au-delà du quota est quand même livré. Compare à ton propre volume sur les pages de tarifs." },
      { q: 'Hook0 transforme-t-il les payloads comme Hookbase ?', a: "Non. Hook0 livre le payload que tu publies exactement tel que tu l'as envoyé, et choisit les abonnements qui le reçoivent par type d'événement et par labels. Si tu dois remodeler les webhooks de tiers avant qu'ils n'arrivent dans ton app, c'est une fonction de Hookbase." },
      { q: 'Lequel choisir, Hook0 ou Hookbase ?', a: "Choisis selon le sens du trafic et l'endroit où les données doivent vivre. Recevoir et transformer les webhooks de nombreux fournisseurs sur une passerelle edge managée, c'est Hookbase. Envoyer des webhooks à tes propres clients, avec un serveur open source, un hébergement en UE et un éditeur français, c'est Hook0." },
      { q: 'Hook0 est-il affilié à Hookbase ?', a: "Non. Hookbase est une marque de son détenteur respectif, et Hook0 est indépendant, ni affilié ni soutenu par Hookbase. Cette page est une comparaison factuelle pour les équipes qui évaluent les deux outils." },
    ],
  },
  comparisonSources: {
    competitors: ['Hookbase'],
    scope: "Cette page compare le sens du trafic, les tarifs, les relances, les signatures, le portail, les transforms, l'hébergement, le code source et l'éditeur de Hook0 et de Hookbase. Les faits sur Hookbase viennent de son site public, de sa page de tarifs, de ses pages produit et de comparaison, de sa politique de confidentialité et de ses conditions, tous lus à la date indiquée. Les faits sur Hook0 viennent de sa documentation et de sa page de tarifs.",
    sources: [
      { label: "Page d'accueil de Hookbase", url: 'https://www.hookbase.app/', consulted: '2026-11-24' },
      { label: 'Tarifs de Hookbase', url: 'https://www.hookbase.app/pricing', consulted: '2026-11-24' },
      { label: 'Relances de Hookbase', url: 'https://www.hookbase.app/webhook-retries', consulted: '2026-11-24' },
      { label: 'Portail client de Hookbase', url: 'https://www.hookbase.app/customer-portal', consulted: '2026-11-24' },
      { label: 'Transforms de payload de Hookbase', url: 'https://www.hookbase.app/payload-transforms', consulted: '2026-11-24' },
      { label: 'Hookbase vs Svix (managed-only)', url: 'https://www.hookbase.app/compare/svix', consulted: '2026-11-24' },
      { label: 'Hookbase vs Hookdeck (Cloudflare Workers)', url: 'https://www.hookbase.app/compare/hookdeck', consulted: '2026-11-24' },
      { label: 'Conditions de Hookbase (droit applicable)', url: 'https://www.hookbase.app/terms', consulted: '2026-11-24' },
      { label: 'Calendrier de relances de Hook0', url: 'https://documentation.hook0.com/explanation/webhook-retry-logic', consulted: '2026-11-24' },
    ],
    updated: '2026-11-24',
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'hook0-vs-webhook-relay', label: 'Hook0 vs Webhook Relay' },
      { enSlug: 'hook0-vs-requeue', label: 'Hook0 vs Requeue' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
      { enSlug: 'open-source-webhooks', label: 'Webhooks open source' },
      { enSlug: 'eu-webhook-infrastructure', label: 'Infrastructure webhook européenne' },
      { enSlug: 'hook0-alternatives', label: 'Alternatives à Hook0' },
    ],
  },
};
