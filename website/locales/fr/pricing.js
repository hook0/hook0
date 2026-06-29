// Per-page strings for pricing (FR).
// /humanizer pro + legal-reviewer applied.
// SSPL = «code source ouvert (SSPL-1.0)», jamais «open source» (L121-1).
module.exports = {
  pageTitle: 'Tarifs Hook0 : offre gratuite, cloud UE | Webhooks',
  pageDescription: 'Developer gratuit à vie. Cloud dès 59 € HT/mois, code source ouvert (SSPL-1.0), auto-hébergeable. Aucun frais caché.',
  "pageModified": "2026-06-27",
  "track": "fr-tarifs",
  "hero": {
    "h1": "Tarifs Hook0",
    "tagline": "Choisis le plan qui colle à ton équipe. Démarre gratuitement, scale en production."
  },
  "differentiators": {
    "h2": "Une grille de prix qui change vraiment",
    "cards": [
      {
        "title": "Bootstrappé, sans VC",
        "body": "Aucune pression pour faire grimper les prix. On grandit avec toi, jamais contre toi."
      },
      {
        "title": "Code source ouvert, zéro lock-in",
        "body": "Audite chaque ligne de code. Auto-héberge pour la conformité. Commence sur le Cloud pour aller en production très vite."
      },
      {
        "title": "Aucun frais caché",
        "body": "Les relances sont gratuites. Les signatures HMAC incluses. Aucun coût par endpoint. Les tarifs au dépassement sont affichés sur chaque plan."
      }
    ]
  },
  "faq": {
    "h2": "FAQ tarifs",
    "items": [
      {
        "q": "Que se passe-t-il si je dépasse ma limite d'events quotidienne ?",
        "a": "Sur l'offre Developer gratuite, les events supplémentaires sont bloqués (HTTP 429). Sur les offres payantes (Startup et Pro), les events supplémentaires <strong>ne sont jamais bloqués</strong>. Ils sont facturés à l'event (0,003 € par event sur Startup, 0,0001 € par event sur Pro). On a choisi de ne pas interrompre la livraison pour éviter de poser problème aux clients qui construisent des produits sur Hook0."
      },
      {
        "q": "Comment je surveille ma consommation ?",
        "a": "Le tableau de bord Organisation dans l'app Hook0 affiche ta consommation d'events pour la journée en cours et les jours précédents. Pour le détail de facturation et l'historique des factures, va dans ton portail Stripe."
      },
      {
        "q": "Hook0 est-il gratuit ?",
        "a": "Oui. Hook0 a une offre Developer gratuite qui inclut 100 webhook events par jour, les signatures HMAC, et le monitoring de livraison. Aucune carte bancaire requise. Hook0 est aussi à code source ouvert et auto-hébergeable quand tu as besoin de souveraineté des données ou de contraintes d'infrastructure spécifiques."
      },
      {
        "q": "Puis-je auto-héberger Hook0 gratuitement ?",
        "a": "Oui. Hook0 est entièrement à code source ouvert sous licence SSPL-1.0. Tu peux l'auto-héberger avec Docker Compose ou Kubernetes. En auto-hébergement, tu gères ta propre infrastructure, ton scaling, tes mises à jour et ton monitoring. La plupart des équipes démarrent sur Hook0 Cloud pour aller en production très vite."
      },
      {
        "q": "Comment se compare le tarif Hook0 à Svix et Hookdeck ?",
        "a": "Hook0 Cloud démarre à 59 € HT/mois face à Svix à 490 $/mois pour des fonctions comparables. Svix verrouille l'auto-hébergement derrière un tarif entreprise. Hookdeck n'a pas d'option auto-hébergée. Hook0 est aussi à code source ouvert sous SSPL, donc tu peux auto-héberger quand tu as besoin de souveraineté des données."
      }
    ]
  }
};
