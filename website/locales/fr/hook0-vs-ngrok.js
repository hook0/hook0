// Per-page strings for hook0-vs-ngrok (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// Faits capturés le 2026-10-06 (tarifs et docs ngrok, code du CLI et du relais Hook0).
// Nuance qui structure la page : les deux outils ne se recoupent que sur un usage,
// recevoir des webhooks en local pendant le développement. ngrok est un point
// d'entrée généraliste, `hook0 listen` est un tunnel de dev intégré au CLI Hook0.
// Honnêteté : ne jamais présenter Hook0 comme un ingress généraliste ou un tunnel
// de prod. L'installation de ngrok est plus simple (brew, apt, Windows Store),
// le CLI Hook0 s'installe aujourd'hui via Cargo, on le dit.
module.exports = {
  pageTitle: 'Hook0 vs ngrok pour les webhooks : tunnel local comparé | Hook0',
  pageDescription: "ngrok expose n'importe quel service local. Le CLI Hook0 transfère tes webhooks vers localhost sans compte, via un relais open source auto-hébergeable.",
  pageModified: '2026-10-06',
  breadcrumb: 'Hook0 vs ngrok',
  tldr: {
    h2: 'En bref',
    body: "ngrok et Hook0 se recoupent sur un seul usage, recevoir des webhooks sur ta machine pendant que tu développes. ngrok est un point d'entrée généraliste qui expose n'importe quel service HTTP, TCP ou TLS local, avec domaines personnalisés et règles de trafic sur les offres payantes. Hook0 fait du webhooks-as-a-service sortant, et son CLI embarque hook0 listen, un tunnel WebSocket qui transfère les webhooks vers localhost sans compte. Le relais derrière est open source et tu peux héberger le tien. Prends ngrok si tu dois exposer autre chose que des webhooks. Prends Hook0 si les webhooks que tu testes sont ceux que ton produit enverra à ses utilisateurs.",
  },
  hero: {
    eyebrow: 'Comparaison',
    titleBefore: 'Hook0 vs ngrok',
    titleAccent: 'Recevoir des webhooks en local',
    subtitle: "ngrok met n'importe quel service local sur une URL publique. Le CLI Hook0 fait une chose plus étroite. Il transfère les webhooks vers ton serveur local via un relais que tu peux auto-héberger. Cette page compare les deux sur cet usage, et dit quand ngrok est le meilleur choix.",
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Lire la doc du CLI',
  },
  differentiators: {
    eyebrow: 'Ce que fait chacun',
    h2: 'Différences clés',
    cards: [
      { title: 'Ingress généraliste ou tunnel de webhooks', body: "ngrok expose des services HTTP, TCP et TLS locaux, donc il sert aussi bien pour une démo, une API, du SSH que des webhooks. <code>hook0 listen 3000</code> ne sert qu'aux webhooks. Il ouvre un WebSocket vers le relais, affiche une URL publique et transfère chaque requête entrante vers ton port local." },
      { title: 'Aucun compte pour démarrer', body: "L'agent ngrok demande un compte et un authtoken avant le premier tunnel. <code>hook0 listen</code> génère un token sur ta machine et se connecte directement au relais public, tu obtiens donc une URL sans inscription. Passe <code>--token</code> pour garder la même URL d'un redémarrage à l'autre." },
      { title: 'Un relais que tu héberges toi-même', body: "Le serveur de relais Hook0 est open source sous SSPL-1.0 et livré avec un chart Helm dans le dépôt Hook0. Pointe le CLI vers ta propre instance avec <code>--relay-url</code>, et tes payloads de test restent sur une infra que tu opères. ngrok est un service hébergé." },
      { title: "Pensé pour l'émission", body: "Le produit principal de Hook0 livre des webhooks à tes utilisateurs, avec journal par tentative, relances gratuites, signature HMAC et replay. Le même CLI envoie des événements, gère les souscriptions et rejoue les échecs, tu peux donc tester toute la boucle, de <code>hook0 event send</code> jusqu'à ton handler local." },
    ],
  },
  comparison: {
    eyebrow: 'Côte à côte',
    h2: 'Recevoir des webhooks en local',
    headers: { feature: 'Critère', hook0: 'CLI Hook0', ngrok: 'ngrok' },
    rows: [
      { feature: 'Rôle principal', hook0Html: 'Webhooks-as-a-service sortant, avec un tunnel de dev dans le CLI', ngrokHtml: "Point d'entrée et tunnels généralistes" },
      { feature: 'Protocoles exposés', hook0Html: 'Webhooks HTTP transférés vers une URL locale', ngrokHtml: 'HTTP/S et TCP, TLS sur Pay-as-you-go' },
      { feature: 'Compte requis', hook0Html: 'Non, pour <code>hook0 listen</code>', ngrokHtml: 'Oui, compte et authtoken' },
      { feature: 'URL stable', hook0Html: 'Même token, même URL (<code>--token</code>)', ngrokHtml: 'Un domaine de développement sur chaque offre, domaines personnalisés sur Pay-as-you-go' },
      { feature: 'Offre gratuite', hook0Html: 'Relais public sans frais', ngrokHtml: '1 Go de transfert, 20 000 requêtes HTTP/S, jusqu\'à 3 endpoints' },
      { feature: 'Inspection des requêtes', hook0Html: 'Tableau de bord terminal (TUI) avec en-têtes, corps et statut', ngrokHtml: 'Traffic Inspector, rétention 24 h en Free' },
      { feature: 'Vérification de signature', hook0Html: "Signature HMAC-SHA256 côté émission", ngrokHtml: 'Action de traffic policy verify-webhook pour de nombreux fournisseurs' },
      { feature: 'Garde-fou de transfert', hook0Html: 'Localhost uniquement, sauf <code>--allow-external</code>', ngrokHtml: "Sans objet (tu choisis l'amont)" },
      { feature: 'Auto-héberger le relais', hook0Html: 'Oui, open source (SSPL-1.0) avec chart Helm', ngrokHtml: 'Non, service hébergé' },
      { feature: 'Installation', hook0Html: '<code>cargo install</code> (Rust requis)', ngrokHtml: 'Homebrew, apt, Windows Store ou téléchargement direct' },
    ],
  },
  fit: {
    h2: 'Quand ngrok convient mieux',
    body: "Prends ngrok quand tu dois exposer autre chose que des webhooks, par exemple une app de démo, une API pour un collègue, un service TCP ou un endpoint derrière un domaine personnalisé avec des règles de trafic. Il s'installe aussi en une commande sur la plupart des systèmes, alors que le CLI Hook0 passe aujourd'hui par Cargo.",
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Hook0 est-il une alternative à ngrok ?', a: "Pour un usage, oui, recevoir des webhooks sur localhost pendant le développement. hook0 listen transfère les webhooks vers ton serveur local via un relais WebSocket, sans compte. Pour tout ce qui dépasse les webhooks (services TCP, démos, domaines personnalisés, règles de trafic), ngrok est l'outil le plus large." },
      { q: 'Comment recevoir des webhooks sur localhost avec Hook0 ?', a: "Installe le CLI, puis lance hook0 listen 3000. Il affiche une URL publique sur play.hook0.com. Colle cette URL chez le fournisseur ou dans une souscription Hook0, et chaque requête est transférée vers localhost:3000 et affichée dans le tableau de bord du terminal." },
      { q: 'Faut-il un compte pour utiliser hook0 listen ?', a: "Non. Le CLI génère un token sur ta machine et se connecte au relais public. Un compte Hook0 ne sert que pour envoyer des événements via l'API Hook0, par exemple avec hook0 event send." },
      { q: 'Puis-je auto-héberger le relais Hook0 ?', a: "Oui. Le serveur de relais est dans le dépôt Hook0 sous SSPL-1.0, avec un chart Helm. Lance ton instance, puis pointe le CLI dessus avec --relay-url. Ajoute --insecure s'il utilise un certificat auto-signé." },
      { q: "Puis-je garder la même URL de webhook d'une session à l'autre ?", a: "Oui. L'URL dépend du token, donc hook0 listen 3000 --token mon-token-stable te donne la même URL à chaque fois. Chez ngrok, chaque offre inclut un domaine de développement." },
      { q: 'Hook0 est-il affilié à ngrok ?', a: "Non. ngrok est une marque de son propriétaire. Hook0 est indépendant, sans affiliation ni soutien de ngrok. Cette page est une comparaison factuelle pour les développeurs qui évaluent les deux." },
    ],
  },
  comparisonSources: {
    competitors: ['ngrok'],
    scope: "Cette page compare la façon dont chaque outil reçoit des webhooks sur une machine locale pendant le développement. Les chiffres ngrok viennent de sa page de tarifs et de sa documentation publiques. Les chiffres Hook0 viennent du code source du CLI et du relais Hook0, et de la référence du CLI.",
    sources: [
      { label: 'Tarifs ngrok', url: 'https://ngrok.com/pricing', consulted: '2026-10-06' },
      { label: 'Guide Share Localhost de ngrok', url: 'https://ngrok.com/docs/share-localhost/quickstart', consulted: '2026-10-06' },
      { label: 'Action verify-webhook de ngrok', url: 'https://ngrok.com/docs/traffic-policy/actions/verify-webhook/', consulted: '2026-10-06' },
      { label: 'Référence du CLI Hook0', url: 'https://documentation.hook0.com/reference/cli', consulted: '2026-10-06' },
    ],
    updated: '2026-10-06',
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-webhook-relay', label: 'Hook0 vs Webhook Relay' },
      { enSlug: 'webhook-playground', label: 'Testeur de webhooks' },
      { enSlug: 'migrate-from-webhook-site', label: 'Alternative à Webhook.site' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
      { enSlug: 'open-source-webhooks', label: 'Webhooks open source' },
    ],
  },
};
