// Per-page strings for webhook-playground (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// SSPL = « code source ouvert (SSPL-1.0) » dans la pageDescription.
module.exports = {
  pageTitle: 'Testeur de webhooks gratuit en ligne, instantané | Hook0',
  pageDescription: 'Testeur de webhooks gratuit, sans inscription. Envoie des events de test, inspecte les payloads, vérifie les signatures HMAC, debug la livraison. Compatible Stripe, GitHub, Shopify. Code source ouvert (SSPL-1.0).',
  hero: {
    badge: 'Gratuit, sans inscription',
    titleBefore: 'Teste tes webhooks',
    titleAccent: 'en quelques secondes',
    subtitle: 'Envoie des events, inspecte les payloads, vérifie les signatures HMAC et debug la livraison, le tout dans ton navigateur. Le chemin le plus rapide pour tester ton intégration webhook.',
    ctaPrimary: 'Ouvrir le Playground',
    ctaSecondary: 'Voir les tarifs, offre gratuite incluse',
  },
  features: {
    eyebrow: 'Ce que tu peux faire',
    h2: 'Tout ce dont tu as besoin pour tester tes webhooks',
    cards: [
      { title: 'Envoyer des events de test', body: 'Tire des events webhook avec des payloads JSON custom vers n\'importe quel endpoint. Vois la réponse en temps réel.' },
      { title: 'Vérifier les signatures HMAC', body: 'Vérifie que ton receveur webhook valide bien les signatures HMAC-SHA256 et rejette les payloads modifiés.' },
      { title: 'Inspecter les payloads', body: 'Affiche les headers HTTP, le corps de requête, les codes de statut et la latence pour chaque tentative de livraison.' },
      { title: 'Tester les relances', body: 'Simule des pannes d\'endpoint et regarde la logique de relances en deux phases de Hook0 à l\'œuvre (rapides puis lentes).' },
      { title: 'Exemples de code', body: 'Code prêt à copier-coller pour Python, Node.js, Go et Rust, intégration en quelques minutes.' },
      { title: 'Sans inscription, sans install', body: 'Tout dans ton navigateur. Pas de compte à créer, pas de CLI, pas de Docker pour démarrer les tests.' },
    ],
  },
  toProduction: {
    h2: 'Du test à la production en 5 minutes',
    subtitle: 'Quand tu es prêt à envoyer tes webhooks en prod, Hook0 gère les relances, les signatures HMAC, le monitoring de livraison et le routage multi-tenant. Démarre avec le tier gratuit, sans carte bancaire.',
    ctaPrimary: 'Essayer le Playground',
    ctaSecondary: 'Voir les tarifs',
  },
  faq: {
    items: [
      { q: 'Hook0 Playground est-il gratuit ?', a: 'Oui. Hook0 Playground est totalement gratuit, sans inscription. Tu peux envoyer des events webhook, inspecter les payloads, vérifier les signatures HMAC et debug la livraison, tout dans ton navigateur.' },
      { q: 'Que peux-tu tester avec le webhook playground ?', a: 'Tu peux envoyer des events webhook de test avec des payloads custom, inspecter les headers HTTP et les codes de réponse, vérifier les signatures HMAC-SHA256, tester le comportement des relances et debug la connectivité endpoint. Le playground prend en charge tous les patterns webhook standards (Stripe, GitHub, Shopify, etc.).' },
      { q: 'Faut-il créer un compte ?', a: 'Non. Le playground marche instantanément, sans inscription. Si tu veux sauvegarder tes configs ou envoyer plus de 100 events/jour, tu peux créer un compte Hook0 gratuit.' },
      { q: 'Existe-t-il un service webhook gratuit utilisable en production ?', a: 'Oui. Hook0 Cloud passe du gratuit à la production sans heurts, le tier Developer inclut 100 events/jour, les signatures HMAC, le monitoring de livraison et 7 jours de rétention des données. Sans carte bancaire. L\'auto-hébergement est aussi possible pour des besoins infra spécifiques.' },
    ],
  },
};
