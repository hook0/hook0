// Per-page strings for build-vs-buy-webhooks (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// Hook0 = « code source ouvert (SSPL-1.0) ».
module.exports = {
  pageTitle: 'Build vs Buy webhooks, prod en 30 min | Hook0',
  pageDescription: 'Construire un système webhook de zéro coûte 3 sprints ou plus. Relances, signatures, monitoring, dead letter queues. Ou tu prends Hook0 et tu livres en 30 minutes.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Build vs Buy',
    titleBefore: 'Arrête de construire des webhooks',
    titleAccent: 'de zéro',
    subtitle: 'Ton backlog est plein de features que tes users veulent vraiment. Relances, signatures, monitoring, dead letter queues, c\'est 3 sprints ou plus de plomberie. Hook0 est un service webhook en code source ouvert (SSPL-1.0) qui gère tout ça. 100 events/jour gratuits, sans carte bancaire. Livre en 30 minutes.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Essayer le Playground',
    stats: [
      { value: '3+', label: 'Sprints pour construire en interne', color: 'green' },
      { value: '30 min', label: 'Pour intégrer Hook0', color: 'indigo' },
      { value: '0 €', label: 'Pour démarrer (tier gratuit)', color: 'green' },
    ],
  },
  hiddenCosts: {
    eyebrow: 'Le vrai coût',
    h2: 'Ce que tu dois vraiment construire',
    sub: 'Envoyer un POST HTTP est facile. Construire un système webhook de niveau production ne l\'est pas.',
    cards: [
      { title: 'Logique de relances', body: 'Planning à deux phases, jitter, nombre d\'essais max, configuration par abonnement. Tu vas mettre des bugs ici. Tout le monde le fait.' },
      { title: 'Dead letter queues', body: 'Que se passe-t-il quand les relances sont épuisées ? Tu as besoin de stockage DLQ, d\'alerting et d\'un outillage de replay manuel.' },
      { title: 'Signatures HMAC', body: 'Signature cryptographique, rotation des clés, validation de timestamp, prévention des replay attacks. Foire un seul de ces points et les données de tes clients fuient.' },
      { title: 'Monitoring de livraison', body: 'Dashboards, logs de livraison, taux de succès et d\'échec, tracking de latence. Ton premier client va demander « est-ce que mon webhook est passé ? » dès le premier jour.' },
      { title: 'Gestion des abonnés', body: 'Enregistrement des endpoints, filtrage par type d\'event, validation d\'URL, support multi-abonnement. Rien qu\'à ça, un mois de travail si tu le fais bien.' },
      { title: 'Maintenance continue', body: 'Migrations DB, scaling, rotations on-call, security patches. Six mois après le lancement, quelqu\'un se fait encore réveiller à 3h du mat pour ça.' },
    ],
  },
  comparison: {
    eyebrow: 'Comparaison',
    h2: 'Construire en interne vs utiliser Hook0',
    headers: { aspect: 'Aspect', diy: 'Construire en interne', hook0: 'Hook0' },
    rows: [
      { aspect: 'Time-to-production', diyHtml: '3 sprints ou plus (6-12 semaines)', hook0Html: '30 minutes', diyDim: false },
      { aspect: 'Coût ingénierie', diyHtml: '2-3 FTE pendant des mois', hook0Html: 'Un dev, une après-midi', diyDim: false },
      { aspect: 'Maintenance continue', diyHtml: 'Continue (bugs, scaling, patches)', hook0Html: 'Gérée par Hook0', diyDim: false },
      { aspect: 'Logique de relances', diyHtml: 'À construire de zéro', hook0Html: 'Intégrée avec relances 2-phases configurables (rapide + lent), customisable par abonnement', diyDim: false },
      { aspect: 'Sécurité (HMAC)', diyHtml: 'À implémenter et maintenir', hook0Html: 'Automatique sur chaque event', diyDim: false },
      { aspect: 'Monitoring et logs', diyHtml: 'Dashboards à construire', hook0Html: 'Inclus dès le départ', diyDim: false },
      { aspect: 'Gestion des abonnements', diyHtml: 'Toute une UI à construire', hook0Html: 'Portail embarquable inclus', diyDim: false },
      { aspect: 'Vendor lock-in', diyHtml: 'Aucun (mais coincé avec ton code)', hook0Html: 'Aucun (code source ouvert, auto-hébergeable)', diyDim: true },
    ],
  },
  integration: {
    eyebrow: 'Intégration',
    h2: 'Livre tes webhooks en 30 minutes',
    sub: 'Un appel API pour publier un event sortant. Hook0 est du webhook-as-a-service pour les architectures event-driven. Il gère le reste.',
    codeBlock: 'curl -X POST https://app.hook0.com/api/v1/event \\\n  -H "Authorization: Bearer YOUR_API_KEY" \\\n  -H "Content-Type: application/json" \\\n  -d \'{\n    "event_type": "invoice.paid",\n    "payload": {\n      "invoice_id": "inv_123",\n      "amount": 9900,\n      "currency": "eur"\n    }\n  }\'',
    codeFootnote: 'Relances, signatures HMAC, logs de livraison, notification aux abonnés, géré.',
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions fréquentes',
    items: [
      { q: 'Combien de temps pour construire des webhooks de zéro ?', a: 'Compte au minimum 3 sprints d\'ingénierie. Logique de relances, dead letter queues, signatures HMAC, monitoring de livraison, gestion des abonnés, health checking des endpoints. Et c\'est avant que ton premier client trouve un bug.' },
      { q: 'Quel est le coût caché de construire le tien ?', a: 'Maintenance de la queue de relances, gestion des cas limites (timeouts, redirections, erreurs de certificat), dashboards de monitoring, limitation de débit, stockage des logs, intégration des abonnés. Rien de tout ça ne s\'arrête après la v1. Ça s\'accumule.' },
      { q: 'En combien de temps puis-je intégrer Hook0 ?', a: 'Moins de 30 minutes. Un seul appel API pour déclencher un event. SDKs pour Python, Node.js et d\'autres si tu préfères.' },
      { q: 'Puis-je migrer depuis un système maison ?', a: 'Oui. API REST et SDKs, donc tu peux faire tourner les deux systèmes en parallèle pendant la migration. Pas besoin de bascule big bang.' },
    ],
  },
  deepDiveHtml: 'Tu veux plus de détails ? <a href="https://documentation.hook0.com/tutorials/getting-started" class="text-indigo-400 hover:text-indigo-300 underline">Lis le guide de démarrage dans la doc</a>.',
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
    ],
  },
};
