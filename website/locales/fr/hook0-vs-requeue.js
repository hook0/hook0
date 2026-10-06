// Per-page strings for hook0-vs-requeue (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// Hook0 SSPL-1.0 = « open source (source disponible, non approuvé OSI) », jamais
// « le seul vrai open source ». Faits capturés le 2026-10-02 (getrequeue.com + home Hook0).
// Nuance qui structure la page : Hook0 et Requeue sont sur des sens opposés du même
// problème. Hook0 émet (outbound), Requeue attrape l'entrant (inbound dead-letter).
// Honnêteté : ne jamais prétendre que Hook0 fait l'inbound dead-letter en prod aujourd'hui.
// Tout attribut Requeue non sourcé est écrit « Non précisé », jamais inventé.
module.exports = {
  pageTitle: 'Hook0 vs Requeue : webhooks entrants vs sortants | Hook0',
  pageDescription: "Requeue est une inbox dead-letter pour les webhooks que tu reçois et les jobs en échec. Hook0 fait l'inverse, émission de webhooks managée, relances gratuites, HMAC, replay, hébergé en UE. Lequel choisir.",
  pageModified: '2026-10-02',
  breadcrumb: 'Hook0 vs Requeue',
  tldr: {
    h2: 'En bref',
    body: "Hook0 et Requeue traitent deux faces du même problème, dans des sens opposés. Requeue est une inbox dead-letter entrante, il attrape les webhooks que tu reçois de fournisseurs comme Stripe ou Clerk, plus les jobs cron et worker qui échouent, et te laisse les rejouer vers ton app. Hook0 est du webhooks-as-a-service sortant, il envoie les webhooks à tes propres utilisateurs et pilote cette livraison, avec chaque tentative journalisée, des payloads inspectables, des relances configurables et gratuites, des signatures HMAC et un replay depuis le dashboard ou l'API. Choisis Requeue pour attraper et rejouer les échecs entrants dans une petite équipe, choisis Hook0 pour livrer des webhooks à tes clients à l'échelle, hébergé en UE et open source. Hook0 ne propose pas d'inbox dead-letter entrante en production aujourd'hui.",
  },
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Requeue',
    titleAccent: 'Sens opposés, même problème',
    subtitle: "Requeue attrape les webhooks que tu reçois et les jobs cron ou worker qui échouent, puis les rejoue dans ton app. Hook0 fait l'inverse, il envoie les webhooks à tes utilisateurs et pilote la livraison, avec journal par tentative, relances gratuites, signature HMAC et replay. Cette page pose le sens pour lequel chacun est conçu, pour que tu voies d'un coup d'œil quel problème tu as vraiment.",
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  differentiators: {
    eyebrow: 'Ce que fait chacun',
    h2: 'Différences clés',
    cards: [
      { title: "Attraper l'entrant vs envoyer le sortant", body: "Requeue est une inbox dead-letter pour le trafic qui arrive, les webhooks que tu reçois de tiers et les tâches de fond qui échouent. Hook0 est l'émetteur. Il publie tes événements et les livre à tes abonnés, donc l'échec qu'il gère est une livraison <em>sortante</em> qui n'a pas abouti. Même problème de fiabilité, deux bouts opposés du tuyau." },
      { title: 'Attraper, inspecter, rejouer, côté émission', body: "La story attraper / inspecter / rejouer que Requeue raconte pour l'entrant, Hook0 la fait déjà pour le sortant, chaque tentative de livraison est journalisée, tu inspectes les payloads et les réponses, tu sais pourquoi une livraison a échoué, et tu rejoues les événements depuis le dashboard ou l'API. Les événements et les réponses sont persistés, donc la trace de debug est là quand tu en as besoin." },
      { title: 'Résidence UE et RGPD par défaut', body: "Hook0 est édité par une société française sans entité US, hors de portée du Cloud Act américain, et son cloud managé tourne en UE par défaut avec signatures HMAC et TLS. Requeue propose un core auto-hébergeable et une API hébergée en liste d'attente, son matériel public ne précise pas où vivent les données hébergées, donc la résidence UE passe par l'auto-hébergement de ton côté." },
      { title: 'Open source, relances gratuites, pas de facturation par endpoint', body: "Le serveur de Hook0 est open source sous SSPL-1.0 (source disponible, non approuvé OSI) et ses 11 SDK clients sont en MIT. Les relances sont incluses gratuitement, la signature HMAC est intégrée, et il n'y a pas de facturation par endpoint. Requeue publie lui aussi un core open source auto-hébergeable. Les deux te laissent faire tourner le logiciel toi-même, le choix tient au sens de trafic que tu dois fiabiliser." },
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
      { feature: 'Replay', hook0Html: "Depuis le dashboard ou l'API", requeueHtml: 'Replay en un clic avec édition du payload, replay en masse' },
      { feature: 'Journal par tentative', hook0Html: 'Oui, inspection des payloads et réponses', requeueHtml: "Échecs capturés gardés dans l'inbox" },
      { feature: 'Signatures', hook0Html: 'HMAC-SHA256 + TLS', requeueHtml: 'Non précisé' },
      { feature: 'Portail abonnés', hook0Html: 'Embarquable', requeueHtml: "Sans objet (pas un émetteur sortant)" },
      { feature: 'Relances', hook0Html: 'Configurables 2-phases, gratuites', requeueHtml: 'Replay manuel et en masse' },
      { feature: 'Hébergement', hook0Html: 'Cloud managé UE (Clever Cloud FR) + auto-hébergement', requeueHtml: "Core auto-hébergeable, API hébergée en liste d'attente" },
      { feature: 'Source', hook0Html: 'Serveur SSPL-1.0, 11 SDK MIT', requeueHtml: 'Core open source (auto-hébergeable)' },
      { feature: 'Utilisateur type', hook0Html: 'Équipes qui émettent des webhooks vers leurs clients', requeueHtml: 'Indés et petites équipes (2 à 5)' },
    ],
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Hook0 est-il une alternative à Requeue ?', a: "Ils traitent des problèmes adjacents dans des sens opposés, donc ça dépend de ton besoin. Requeue attrape et rejoue les webhooks que tu reçois, plus les jobs cron et worker en échec. Hook0 envoie les webhooks à tes propres utilisateurs et pilote cette livraison. Si tu dois livrer des webhooks à tes clients de façon fiable, Hook0 est le bon choix. Si tu as besoin d'une inbox pour les webhooks entrants en échec aujourd'hui, c'est le terrain de Requeue." },
      { q: 'Hook0 fait-il du dead-letter de webhooks entrants ?', a: "Pas en tant que produit aujourd'hui. Hook0 est du webhooks-as-a-service sortant. Le CLI Hook0 peut recevoir des webhooks sur localhost via un tunnel intégré pour le développement local, mais c'est un outil de dev, pas une inbox dead-letter entrante de production." },
      { q: 'Puis-je rejouer des webhooks en échec avec Hook0 ?', a: "Oui, côté émission. Chaque tentative de livraison est journalisée, tu inspectes le payload et la réponse, tu sais pourquoi une livraison a échoué, et tu rejoues les événements depuis le dashboard ou l'API. Les relances sont configurables en deux phases et gratuites." },
      { q: 'Requeue est-il open source ?', a: "Requeue propose un core open source auto-hébergeable, avec une API hébergée en liste d'attente. Le serveur de Hook0 est open source sous SSPL-1.0, qui est source disponible et non approuvé par l'OSI, et ses 11 SDK clients sont en MIT. Les deux te laissent auto-héberger, la différence tient au sens de trafic que chacun gère." },
      { q: 'Lequel choisir, Hook0 ou Requeue ?', a: "Choisis selon le sens. Envoyer des webhooks à tes propres clients à l'échelle, avec signatures, journal par tentative, relances gratuites et portail abonnés, c'est Hook0. Attraper et rejouer les webhooks que tu reçois de tiers, plus les jobs cron et worker en échec, dans une petite équipe, c'est Requeue. Ils occupent des faces adjacentes du même problème de fiabilité." },
      { q: 'Hook0 est-il affilié à Requeue ?', a: "Non. Requeue est une marque de son détenteur respectif, et Hook0 est indépendant, ni affilié ni soutenu par Requeue. Cette page est une comparaison factuelle pour les équipes qui évaluent les deux outils." },
    ],
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'migrate-from-webhook-site', label: 'Alternative à Webhook.site' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
      { enSlug: 'open-source-webhooks', label: 'Webhooks open source' },
      { enSlug: 'hook0-alternatives', label: 'Alternatives à Hook0' },
    ],
  },
};
