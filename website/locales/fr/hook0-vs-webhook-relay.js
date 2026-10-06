// Per-page strings for hook0-vs-webhook-relay (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// Faits capturés le 2026-10-06 (accueil et tarifs webhookrelay.com, registre
// Companies House, code du CLI et du relais Hook0).
// Nuance qui structure la page : sens opposés. Webhook Relay reçoit les webhooks
// de tiers et les route (inbound), Hook0 envoie les tiens à tes utilisateurs
// (outbound). Recoupement limité au dev local (`hook0 listen`).
// Honnêteté : Webhook Relay AFFICHE une option auto-hébergée (via l'équipe
// commerciale), ne jamais écrire qu'il est uniquement hébergé. Tout attribut
// non sourcé est écrit « Non précisé ».
module.exports = {
  pageTitle: 'Hook0 vs Webhook Relay : envoyer ou recevoir des webhooks | Hook0',
  pageDescription: "Webhook Relay reçoit les webhooks de tiers et les route vers localhost ou des serveurs privés. Hook0 envoie des webhooks à tes utilisateurs, hébergé en UE.",
  pageModified: '2026-10-06',
  breadcrumb: 'Hook0 vs Webhook Relay',
  tldr: {
    h2: 'En bref',
    body: "Webhook Relay et Hook0 traitent les deux bouts opposés du tuyau. Webhook Relay est une passerelle entrante qui reçoit les webhooks de services comme GitHub ou Stripe, les filtre, les transforme et les transfère vers localhost, des serveurs derrière un pare-feu, Kubernetes ou des files cloud. Hook0 fait du webhooks-as-a-service sortant. Il envoie les événements de ton produit aux endpoints de tes utilisateurs, avec journal par tentative, relances gratuites, signature HMAC et replay. Les deux se rejoignent sur le développement local, où le CLI Hook0 transfère aussi les webhooks vers localhost. Si tu reçois des webhooks, prends Webhook Relay. Si tu en envoies, prends Hook0.",
  },
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs Webhook Relay',
    titleAccent: 'Envoyer ou recevoir',
    subtitle: "Webhook Relay fait entrer les webhooks de tiers dans des réseaux sans IP publique. Hook0 livre tes propres événements aux endpoints de tes clients. Cette page montre le sens couvert par chaque outil, là où ils se recoupent, et quand Webhook Relay est le bon choix.",
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
  },
  differentiators: {
    eyebrow: 'Ce que fait chacun',
    h2: 'Différences clés',
    cards: [
      { title: 'Passerelle entrante ou livraison sortante', body: "Webhook Relay se place devant tes systèmes et récupère les webhooks que GitHub, Stripe ou Shopify t'envoient, puis les route là où tu en as besoin. Hook0 se place derrière ton produit et publie ses événements vers chaque abonné, donc la livraison qu'il gère part <em>vers</em> tes utilisateurs." },
      { title: 'Les deux atteignent localhost', body: "Webhook Relay transfère vers localhost et vers des serveurs sans IP publique grâce à son agent et à ses tunnels. Le CLI Hook0 couvre le cas du développement avec <code>hook0 listen 3000</code>, un tunnel WebSocket vers ton port local qui ne demande aucun compte et passe par un relais que tu peux héberger toi-même." },
      { title: 'Éditeur européen, cloud UE par défaut', body: "Hook0 est édité par une société française sans maison mère, filiale ni établissement aux États-Unis, et son cloud managé tourne en UE par défaut. Webhook Relay est édité par AppScension Ltd, une société britannique, et indique tourner sur Google Cloud. Ses pages publiques ne précisent pas de région de données." },
      { title: 'Open source, utilisable dès aujourd\'hui', body: "Le serveur et le relais Hook0 sont open source sous SSPL-1.0 (source disponible, non approuvé OSI), et ses SDK clients sont sous MIT. Tu peux auto-héberger toute la pile sans passer par personne. Webhook Relay propose une option auto-hébergée via son équipe commerciale." },
    ],
  },
  comparison: {
    eyebrow: 'Côte à côte',
    h2: 'Ce que gère chacun',
    headers: { feature: 'Critère', hook0: 'Hook0', relay: 'Webhook Relay' },
    rows: [
      { feature: 'Rôle principal', hook0Html: 'Webhooks-as-a-service sortant', relayHtml: 'Passerelle de webhooks entrants et tunnels' },
      { feature: 'Sens des webhooks', hook0Html: 'Sortant (tu envoies à tes utilisateurs)', relayHtml: 'Entrant (tu reçois de tiers)' },
      { feature: 'Destinations', hook0Html: 'Les endpoints HTTP de tes abonnés', relayHtml: 'URL publiques, serveurs privés, localhost, Kubernetes, services AWS, GCP et Azure' },
      { feature: 'Développement local', hook0Html: 'Tunnel <code>hook0 listen</code>, sans compte', relayHtml: 'Tunnels à partir de l\'offre Basic' },
      { feature: 'Transformations', hook0Html: 'Non proposées (payloads envoyés tels que publiés)', relayHtml: 'Fonctions JavaScript et Lua, filtres' },
      { feature: 'Signatures', hook0Html: 'HMAC-SHA256 sur chaque livraison', relayHtml: 'Non précisé' },
      { feature: 'Portail abonnés', hook0Html: 'Intégrable', relayHtml: 'Sans objet (pas un émetteur sortant)' },
      { feature: 'Offre gratuite', hook0Html: '100 événements par jour', relayHtml: '150 webhooks par mois, sans tunnel' },
      { feature: 'Éditeur', hook0Html: 'Société française, sans entité américaine', relayHtml: 'AppScension Ltd (Royaume-Uni)' },
      { feature: 'Hébergement', hook0Html: 'Cloud UE managé (Clever Cloud FR) + auto-hébergement', relayHtml: 'Google Cloud (région non précisée) + option auto-hébergée via les ventes' },
      { feature: 'Code source', hook0Html: 'Serveur et relais SSPL-1.0, SDK MIT', relayHtml: 'Non précisé' },
    ],
  },
  fit: {
    h2: 'Quand Webhook Relay convient mieux',
    body: "Prends Webhook Relay quand ton problème est côté réception. Les cas typiques sont faire arriver des webhooks GitHub, Stripe ou Shopify sur un serveur de CI ou un service Kubernetes sans IP publique, remodeler les payloads avec une fonction avant qu'ils arrivent, ou répartir un webhook entrant vers plusieurs cibles internes et files cloud. Hook0 ne transforme pas le trafic entrant, et son tunnel est un outil de développement.",
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Hook0 est-il une alternative à Webhook Relay ?', a: "Seulement là où ils se recoupent. Les deux peuvent transférer des webhooks vers localhost pendant le développement. Au-delà, ils vont dans des sens opposés. Webhook Relay reçoit et route les webhooks que tu reçois de tiers, Hook0 livre les webhooks que ton produit envoie à ses utilisateurs." },
      { q: 'Hook0 peut-il transférer des webhooks vers localhost ?', a: "Oui, pour le développement. Lance hook0 listen 3000 et le CLI affiche une URL publique sur play.hook0.com. Chaque requête envoyée dessus est transférée vers localhost:3000 via un tunnel WebSocket. Aucun compte n'est requis, et tu peux faire tourner ton propre relais avec --relay-url." },
      { q: 'Hook0 transforme-t-il ou route-t-il les webhooks entrants ?', a: "Non. Hook0 fait du webhooks-as-a-service sortant. Il signe, livre, relance et journalise les événements que tu publies. Filtrer, transformer et router les webhooks que tu reçois de tiers, c'est le cœur de Webhook Relay." },
      { q: 'Où sont hébergées mes données ?', a: "Hook0 Cloud tourne en UE par défaut, et Hook0 est édité par une société française sans entité américaine. Webhook Relay indique tourner sur Google Cloud, et ses pages publiques ne nomment pas de région. Les deux peuvent être auto-hébergés, Hook0 à partir de son code open source et Webhook Relay via son équipe commerciale." },
      { q: 'Lequel choisir, Hook0 ou Webhook Relay ?', a: "Choisis selon le sens. Pour envoyer des webhooks à tes propres clients avec signatures, journal par tentative, relances gratuites et portail abonnés, prends Hook0. Pour recevoir des webhooks de services tiers et les router vers des réseaux privés, de la CI ou des files cloud, prends Webhook Relay." },
      { q: 'Hook0 est-il affilié à Webhook Relay ?', a: "Non. Webhook Relay est une marque de son propriétaire. Hook0 est indépendant, sans affiliation ni soutien de Webhook Relay. Cette page est une comparaison factuelle pour les équipes qui évaluent les deux." },
    ],
  },
  comparisonSources: {
    competitors: ['Webhook Relay'],
    scope: "Cette page compare le rôle, les destinations, l'hébergement et les offres publiques de Hook0 et de Webhook Relay. Les prix et quotas sont ceux publiés par chaque éditeur. Les informations sur la société viennent du registre britannique Companies House.",
    sources: [
      { label: "Page d'accueil de Webhook Relay", url: 'https://webhookrelay.com/', consulted: '2026-10-06' },
      { label: 'Tarifs Webhook Relay', url: 'https://webhookrelay.com/pricing/', consulted: '2026-10-06' },
      { label: 'AppScension Ltd sur Companies House', url: 'https://find-and-update.company-information.service.gov.uk/company/10705698', consulted: '2026-10-06' },
    ],
    updated: '2026-10-06',
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-ngrok', label: 'Hook0 vs ngrok' },
      { enSlug: 'hook0-vs-requeue', label: 'Hook0 vs Requeue' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'webhook-playground', label: 'Testeur de webhooks' },
      { enSlug: 'eu-webhook-infrastructure', label: 'Infrastructure webhook européenne' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
      { enSlug: 'hook0-alternatives', label: 'Alternatives à Hook0' },
    ],
  },
};
