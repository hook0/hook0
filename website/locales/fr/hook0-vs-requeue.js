// Per-page strings for hook0-vs-requeue (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// Hook0 SSPL-1.0 = « open source (source disponible, non approuvé OSI) », jamais
// « le seul vrai open source ». Faits re-capturés le 2026-10-22 (page d'accueil,
// README et FAQ GitHub de Requeue, listés dans comparisonSources).
// Nuance qui structure la page : Hook0 et Requeue sont sur des sens opposés du même
// problème. Hook0 émet (outbound), Requeue attrape l'entrant (inbound dead-letter,
// y compris via son URL de relais).
// Honnêteté : ne jamais prétendre que Hook0 fait l'inbound dead-letter en prod aujourd'hui.
// Tout attribut Requeue non sourcé est écrit « Non précisé », jamais inventé
// (région d'hébergement, tarifs).
module.exports = {
  pageTitle: 'Hook0 vs Requeue : webhooks entrants vs sortants | Hook0',
  pageDescription: 'Requeue attrape les webhooks reçus et les jobs en échec pour les rejouer. Hook0 envoie tes webhooks avec relances, HMAC et replay, hébergé en UE.',
  pageModified: '2026-10-22',
  breadcrumb: 'Hook0 vs Requeue',
  tldr: {
    h2: 'En bref',
    body: "Hook0 et Requeue traitent deux faces du même problème, dans des sens opposés. Requeue est une inbox dead-letter entrante. Tu pointes un webhook Stripe ou Clerk vers son URL de relais, ou tu lui envoies tes jobs cron et worker en échec, et il garde tout ce que ton app n'a pas su traiter pour que tu l'inspectes et le rejoues. Hook0 est du webhooks-as-a-service sortant. Il envoie les webhooks à tes propres utilisateurs et pilote cette livraison, avec chaque tentative journalisée, jusqu'à 24 relances gratuites sur 8 jours, des signatures HMAC et un replay depuis le dashboard ou l'API. Choisis Requeue pour attraper les échecs entrants dans une petite équipe, choisis Hook0 pour livrer des webhooks à tes clients, hébergé en UE et open source. Hook0 ne propose pas d'inbox dead-letter entrante en production aujourd'hui, et la FAQ de Requeue précise qu'il ne remplace pas le webhooks-as-a-service sortant.",
  },
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Requeue',
    titleAccent: 'Sens opposés, même problème',
    subtitle: "Requeue attrape les webhooks que tu reçois et les jobs cron ou worker qui échouent, puis les rejoue dans ton app. Hook0 fait l'inverse et envoie les webhooks à tes utilisateurs en pilotant la livraison, avec journal par tentative, relances gratuites, signature HMAC et replay. Cette page pose le sens pour lequel chacun est conçu, pour que tu voies d'un coup d'œil quel problème tu as vraiment.",
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  differentiators: {
    eyebrow: 'Ce que fait chacun',
    h2: 'Différences clés',
    cards: [
      { title: "Attraper l'entrant vs envoyer le sortant", body: "Requeue se place devant le trafic qui <em>arrive</em>. Son URL de relais reçoit le webhook d'un fournisseur, transmet le corps brut et les en-têtes de signature à ton app avec un timeout de 10 secondes, et garde la requête si ton app renvoie une erreur ou ne répond pas. Hook0 est l'émetteur. Il publie tes événements et les livre à tes abonnés, donc l'échec qu'il gère est une livraison <em>sortante</em> qui n'a pas abouti." },
      { title: 'Attraper, inspecter, rejouer, côté émission', body: "La story attraper / inspecter / rejouer que Requeue raconte pour l'entrant, Hook0 la fait déjà pour le sortant. Chaque tentative de livraison est journalisée avec son code HTTP et sa réponse, Hook0 relance jusqu'à 24 fois sur 8 jours par défaut, et quand la dernière relance échoue, la tentative reste en <code>failed</code> pour que tu rejoues l'événement depuis le dashboard ou l'API. Voir <a href=\"https://documentation.hook0.com/explanation/webhook-dead-letter-queue\" class=\"text-indigo-400 hover:text-indigo-300 underline\">ce que devient un webhook après sa dernière relance</a> (en anglais)." },
      { title: 'Résidence UE et RGPD par défaut', body: "Hook0 est édité par une société française sans maison mère, filiale ni établissement aux États-Unis, et son cloud managé tourne en UE par défaut avec signatures HMAC et TLS. Le core de Requeue tourne sur Cloudflare Workers et Cloudflare D1, et son matériel public ne précise pas où l'instance hébergée stocke les données. Pour une résidence UE avec Requeue, tu déploies le Worker sur ton propre compte Cloudflare et tu choisis l'emplacement toi-même." },
      { title: 'Open source, relances gratuites, tarifs publics', body: "Le serveur de Hook0 est open source sous SSPL-1.0 (source disponible, non approuvé OSI) et ses 11 SDK clients sont en MIT. Les relances sont gratuites, la signature HMAC est intégrée, il n'y a pas de facturation par endpoint, et les tarifs sont publics avec un plan Developer gratuit. Le core et le SDK JavaScript de Requeue sont en MIT, en version 0.1.0 en octobre 2026, et les clés hébergées passent par une liste d'attente, sans tarif public pour l'instant." },
    ],
  },
  comparison: {
    eyebrow: 'Côte à côte',
    h2: 'Ce que gère chacun',
    headers: { feature: 'Fonctionnalité', hook0: 'Hook0', requeue: 'Requeue' },
    rows: [
      { feature: 'Rôle principal', hook0Html: 'Webhooks-as-a-service sortant', requeueHtml: 'Inbox dead-letter entrante' },
      { feature: 'Sens des webhooks', hook0Html: 'Sortant (tu envoies à tes utilisateurs)', requeueHtml: 'Entrant (tu reçois de tiers)' },
      { feature: 'Ce qui est capturé', hook0Html: 'Tes livraisons en échec vers les abonnés', requeueHtml: 'Webhooks entrants + jobs cron / worker en échec' },
      { feature: 'URL de relais entrante', hook0Html: 'Non (le tunnel du CLI sert au développement local)', requeueHtml: 'Oui, transmet une fois à ton app et garde les échecs' },
      { feature: 'Replay', hook0Html: "Par événement, depuis le dashboard ou l'API", requeueHtml: "En un clic, payload modifiable par livraison, replay en masse jusqu'à 50 événements" },
      { feature: 'Journal par tentative', hook0Html: 'Oui, code HTTP et corps de réponse', requeueHtml: 'Événements en échec avec en-têtes, payload et raison' },
      { feature: 'Signatures', hook0Html: 'HMAC-SHA256 sur chaque requête + TLS', requeueHtml: "HMAC-SHA256 sur les replays quand l'endpoint a un secret" },
      { feature: 'Relances automatiques', hook0Html: "Jusqu'à 24 sur 8 jours (de 3 s à 10 h d'écart), gratuites", requeueHtml: 'Replays en file : 1, 2, 4, 8, 16 min, puis marqués en échec' },
      { feature: 'Portail abonnés', hook0Html: 'Embarquable', requeueHtml: "Sans objet (pas un émetteur sortant)" },
      { feature: 'Hébergement', hook0Html: 'Cloud managé UE (Clever Cloud, France) + auto-hébergement', requeueHtml: 'Cloudflare Workers + D1, région hébergée non précisée' },
      { feature: 'Source', hook0Html: 'Serveur SSPL-1.0, 11 SDK MIT', requeueHtml: 'Core et SDK JS en MIT (v0.1.0)' },
      { feature: 'Tarifs', hook0Html: 'Publics, plan Developer gratuit', requeueHtml: "Non précisé (clés hébergées sur liste d'attente)" },
      { feature: 'Utilisateur type', hook0Html: 'Équipes qui émettent des webhooks vers leurs clients', requeueHtml: 'Indés et petites équipes (2 à 5)' },
    ],
  },
  fit: {
    h2: 'Quand Requeue convient mieux',
    bullets: [
      "Stripe, Clerk ou Resend t'envoient des webhooks et tu veux une copie de chaque requête que ton app n'a pas su traiter, prête à rejouer une fois le bug corrigé. C'est l'URL de relais de Requeue, et Hook0 ne le fait pas.",
      'Tes crons de nuit et tes workers de file perdent leurs payloads après la dernière relance, et tu veux tout retrouver dans une seule inbox avec la raison de l\'échec.',
      "Tu es fondateur solo ou dans une équipe de deux à cinq, déjà sur Cloudflare, et faire tourner toi-même un Worker MIT en version 0.1.0 te convient.",
    ],
    linkPrefix: "Il te faut plutôt une passerelle entrante avec son propre pipeline de relances ? Consulte",
    link: { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Hook0 est-il une alternative à Requeue ?', a: "Ils traitent des problèmes adjacents dans des sens opposés, donc ça dépend de ton besoin. Requeue attrape et rejoue les webhooks que tu reçois, plus les jobs cron et worker en échec. Hook0 envoie les webhooks à tes propres utilisateurs et pilote cette livraison. Si tu dois livrer des webhooks à tes clients de façon fiable, Hook0 est le bon choix. Si tu as besoin d'une inbox pour les webhooks entrants en échec, c'est le terrain de Requeue." },
      { q: 'Requeue envoie-t-il des webhooks à mes clients ?', a: "Non. La FAQ de Requeue indique qu'il ne transforme ni ne diffuse les événements et qu'il ne remplace pas le webhooks-as-a-service sortant. Son relais transmet le webhook d'un fournisseur vers une seule URL cible dans ton app. Envoyer des événements vers les endpoints de nombreux clients, avec signatures, relances et un portail où chaque client gère ses abonnements, c'est le rôle de Hook0." },
      { q: 'Hook0 fait-il du dead-letter de webhooks entrants ?', a: "Pas en tant que produit aujourd'hui. Hook0 est du webhooks-as-a-service sortant. Le CLI Hook0 peut recevoir des webhooks sur localhost via un tunnel intégré, et ce tunnel sert uniquement au développement local." },
      { q: 'Puis-je rejouer des webhooks en échec avec Hook0 ?', a: "Oui, côté émission. Chaque tentative de livraison est journalisée avec son code HTTP et sa réponse. Hook0 relance une livraison en échec jusqu'à 24 fois sur 8 jours par défaut, et une fois les relances épuisées, la dernière tentative reste en statut failed. Tu rejoues alors l'événement depuis le dashboard ou avec POST /events/{event_id}/replay, qui le renvoie à chaque abonnement correspondant." },
      { q: 'Où tourne Requeue ?', a: "D'après son README, le core de Requeue est un Worker Cloudflare qui stocke les événements dans Cloudflare D1, et l'API hébergée répond sur api.getrequeue.com. Son matériel public ne précise pas la région d'hébergement des données. Hook0 Cloud tourne en UE, hébergé en France par défaut, et le serveur Hook0 s'auto-héberge sur Docker ou Kubernetes." },
      { q: 'Requeue est-il open source ?', a: "Oui. L'API core de Requeue et son SDK JavaScript sont publiés sous licence MIT sur GitHub, en version 0.1.0 en octobre 2026. Le serveur de Hook0 est open source sous SSPL-1.0, qui est source disponible et non approuvé par l'OSI, et ses 11 SDK clients sont en MIT. Les deux te laissent auto-héberger, la différence tient au sens de trafic que chacun gère." },
      { q: 'Lequel choisir, Hook0 ou Requeue ?', a: "Choisis selon le sens. Envoyer des webhooks à tes propres clients, avec signatures, journal par tentative, relances gratuites et portail abonnés, c'est Hook0. Attraper et rejouer les webhooks que tu reçois de tiers, plus les jobs cron et worker en échec, dans une petite équipe, c'est Requeue. Certaines équipes ont besoin des deux, un pour chaque bout du tuyau." },
      { q: 'Hook0 est-il affilié à Requeue ?', a: "Non. Requeue est une marque de son détenteur respectif, et Hook0 est indépendant, ni affilié ni soutenu par Requeue. Cette page est une comparaison factuelle pour les équipes qui évaluent les deux outils." },
    ],
  },
  comparisonSources: {
    competitors: ['Requeue'],
    scope: "Cette page compare le rôle, le relais, le replay, les relances, les signatures, l'hébergement, la licence et les tarifs de Hook0 et de Requeue. Les faits sur Requeue viennent de sa page d'accueil publique, du README et de la FAQ de son dépôt GitHub, et du dépôt de son SDK JavaScript, tous lus à la date indiquée. Les faits sur Hook0 viennent de sa documentation et de sa page de tarifs.",
    sources: [
      { label: "Page d'accueil de Requeue", url: 'https://getrequeue.com/', consulted: '2026-10-22', archive: 'https://web.archive.org/web/20261022060503/https://getrequeue.com/' },
      { label: 'README du core Requeue sur GitHub', url: 'https://github.com/requeue-hq/requeue', consulted: '2026-10-22', archive: 'https://web.archive.org/web/20261022060541/https://github.com/requeue-hq/requeue' },
      { label: 'FAQ de Requeue sur GitHub', url: 'https://github.com/requeue-hq/requeue/blob/main/docs/faq.md', consulted: '2026-10-22', archive: 'https://web.archive.org/web/20261022060523/https://github.com/requeue-hq/requeue/blob/main/docs/faq.md' },
      { label: 'SDK JavaScript de Requeue sur GitHub', url: 'https://github.com/requeue-hq/requeue-sdk-js', consulted: '2026-10-22' },
      { label: 'Calendrier de relances de Hook0', url: 'https://documentation.hook0.com/explanation/webhook-retry-logic', consulted: '2026-10-22' },
    ],
    updated: '2026-10-22',
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-webhook-relay', label: 'Hook0 vs Webhook Relay' },
      { enSlug: 'hook0-vs-ngrok', label: 'Hook0 vs ngrok' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
      { enSlug: 'open-source-webhooks', label: 'Webhooks open source' },
      { enSlug: 'migrate-from-webhook-site', label: 'Alternative à Webhook.site' },
      { enSlug: 'hook0-alternatives', label: 'Alternatives à Hook0' },
    ],
  },
};
