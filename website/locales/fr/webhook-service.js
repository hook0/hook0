// Per-page strings for webhook-service (FR).
// /humanizer pro + legal-reviewer applied. Strict parity with locales/en/webhook-service.js.
// Legal: data plane Clever Cloud SAS (France, EEE) ; CDN Cloudflare, Inc. (USA) divulgué
// (CCT 2021 + analyse d'impact + DPF le cas échéant) ; jamais « 100 % souverain » ; RGPD =
// « conçu pour », jamais « certifié » ; licence « open source (SSPL-1.0) », jamais « open source » nu.
// Prix concurrents relevés le 2026-07-16 (Hookdeck Growth 499 $/mois, Svix Pro 490 $/mois,
// Convoy Premium 999 $/mois). Les réponses faq.items[].a doivent rester identiques au texte visible.
module.exports = {
  "tldr": {
    "h2": "En bref",
    "body": "Hook0 est un webhook service managé : il signe chaque payload en HMAC-SHA256, relance en deux temps quand un destinataire est hors service, journalise chaque tentative pour qu'une livraison contestée devienne une simple recherche, et donne à tes clients un portail prêt à l'emploi pour gérer leurs propres endpoints. Le data plane, c'est-à-dire les payloads, la base et les sauvegardes, tourne chez Clever Cloud en France sur chaque offre, tier gratuit compris, avec Cloudflare, Inc. (USA) comme CDN divulgué. C'est gratuit jusqu'à 100 events par jour, avec des offres payantes à partir de 59 €/mois, face à une offre Growth à 499 $/mois chez Hookdeck, 490 $/mois chez Svix et 999 $/mois chez Convoy au dernier relevé de juillet 2026. Tout le code est open source (SSPL-1.0), sans rétention pour un palier entreprise, donc le jour où un service managé n'est plus la bonne réponse tu héberges le même logiciel toi-même et ton code d'intégration ne change pas."
  },
  "pageTitle": "Webhook service managé, avec une porte de sortie | Hook0",
  "pageDescription": "Un webhook service managé avec signatures HMAC, relances en deux phases et journaux par tentative. Data plane européen sur chaque offre, et le même code open source (SSPL-1.0) s'auto-héberge quand tu veux partir.",
  "pageModified": "2026-09-24",
  "track": "fr-service-webhook",
  "hero": {
    "eyebrow": "Webhook service",
    "titleLine1": "Un webhook service managé",
    "titleLine2": "que tu peux emporter",
    "subtitle": "Hook0 livre tes events, les signe, les relance quand le destinataire est hors service, et journalise chaque tentative. Il tourne sur un data plane européen dès le tier gratuit. Et le jour où un webhook service managé n'est plus la bonne réponse, le même code s'auto-héberge, open source sous SSPL-1.0.",
    "ctaPrimary": "Commencer gratuitement",
    "ctaSecondary": "Voir les tarifs",
    "ctaSecondaryHref": "/fr/tarifs",
    "microcopy": "100 events/jour gratuits. Sans carte bancaire. Data plane européen sur chaque offre."
  },
  "socialProof": false,
  "checklist": {
    "eyebrow": "Évaluation",
    "h2": "Ce qu'un webhook service te doit",
    "subtitle": "Les parties que les équipes découvrent après avoir déjà expédié leur propre boucle de livraison. Voici où se situe Hook0 sur chacune.",
    "cards": [
      {
        "title": "Des payloads signés, à chaque tentative",
        "bodyHtml": "Chaque livraison porte une signature HMAC-SHA256, donc le destinataire distingue tes events de tout ce qui a trouvé son endpoint. Les étapes de vérification sont documentées plutôt que laissées en exercice."
      },
      {
        "title": "Des relances qui survivent à une mauvaise après-midi",
        "bodyHtml": "Relances configurables en deux phases. Un abonné hors service pendant une heure ne te coûte pas les events survenus pendant cette heure, et tu n'écris pas le plan de backoff toi-même."
      },
      {
        "title": "La preuve de ce qui s'est vraiment passé",
        "bodyHtml": "Des journaux de livraison par tentative. Quand un client dit n'avoir jamais reçu l'event, la réponse est une recherche plutôt qu'une discussion."
      },
      {
        "title": "Le rejeu, une fois le bug corrigé",
        "bodyHtml": "Les livraisons se rejouent depuis les journaux. La panne du destinataire ou son bug de parsing cesse d'être un incident de perte de données pour devenir un rejeu."
      },
      {
        "title": "Quelque chose que tes clients utilisent seuls",
        "bodyHtml": "Un portail abonnés prêt à l'emploi où tes utilisateurs enregistrent leurs propres endpoints et lisent leur propre historique de livraison, au lieu d'ouvrir un ticket de support à chaque 500."
      },
      {
        "title": "Un data plane que tu peux montrer",
        "bodyHtml": "Payloads, base de données et sauvegardes tournent sur l'infrastructure de Clever Cloud SAS en France, dans l'EEE, sur chaque offre, tier gratuit compris. Le CDN devant est Cloudflare, Inc. (USA), divulgué dans la <a href=\"/fr/sous-traitants-rgpd\" class=\"text-green-400 hover:text-green-300 transition-colors\">liste publique de sous-traitants</a> avec son mécanisme de transfert. Voir l'<a href=\"/fr/infrastructure-webhook-europeenne\" class=\"text-green-400 hover:text-green-300 transition-colors\">infrastructure webhook européenne</a>."
      },
      {
        "title": "Une sortie qui n'est pas une réécriture",
        "bodyHtml": "Tout le produit est open source sous SSPL-1.0, sans rétention open-core, donc le cloud fait tourner le code que tu peux faire tourner. Partir revient à héberger le même logiciel plutôt qu'à tout reconstruire contre une nouvelle API."
      }
    ]
  },
  "cost": {
    "eyebrow": "Coût",
    "h2": "Ce que coûte un webhook service managé",
    "subtitle": "Là où démarre le tarif public de chaque fournisseur dès que tu veux un service managé et supporté plutôt qu'un essai.",
    "headers": {
      "provider": "Fournisseur",
      "offer": "Comment tu l'exploites",
      "price": "Où démarre le tarif"
    },
    "rows": [
      {
        "highlight": true,
        "provider": "Hook0",
        "offerHtml": "Cloud managé, auto-hébergé ou managé sur site, tout sur un seul code.",
        "priceHtml": "0 € (tier gratuit, 100 events/jour) ; offres payantes à partir de 59 €/mois"
      },
      {
        "highlight": false,
        "provider": "Hookdeck",
        "offerHtml": "Plateforme managée, régions US, UE et Asie.",
        "priceHtml": "Tier gratuit disponible ; l'offre Growth avec SLA est à 499 $/mois"
      },
      {
        "highlight": false,
        "provider": "Svix",
        "offerHtml": "Cloud managé, ou auto-hébergement sous MIT.",
        "priceHtml": "Le Pro managé démarre à 490 $/mois"
      },
      {
        "highlight": false,
        "provider": "Convoy",
        "offerHtml": "Community auto-hébergé, ou l'offre managée Premium.",
        "priceHtml": "Community gratuit (Elastic License v2) ; le Premium managé est à 999 $/mois"
      }
    ],
    "footnote": "Sources : les pages de tarifs publiques de chaque fournisseur, dernier relevé le 2026-07-16. Les fournisseurs bougent leurs prix, alors vérifie les leurs aussi. Un chiffre périmé ? Dis-le-nous et on corrige.",
    "footLabel": "Lire le comparatif complet des coûts webhook →",
    "footHref": "/fr/comparatif-cout-webhook"
  },
  "ownership": {
    "eyebrow": "Propriété",
    "h2": "Managé aujourd'hui, à toi demain",
    "intro": "Un webhook service est une dépendance sur ton chemin de livraison. Autant savoir, avant de signer, ce qui se passe le jour où tu veux le sortir.",
    "cards": [
      {
        "title": "Auto-héberge exactement la même chose",
        "bodyHtml": "Docker Compose ou Kubernetes, PostgreSQL en dessous, open source sous SSPL-1.0. Tes payloads restent dans ton propre réseau. Voir les <a href=\"/fr/webhooks-auto-heberges\" class=\"text-green-400 hover:text-green-300 transition-colors\">webhooks auto-hébergés</a>."
      },
      {
        "title": "Ou garde-le managé, dans ton environnement",
        "bodyHtml": "Nous déployons une instance dédiée dans ton infrastructure et la maintenons : 1 000 € de setup + 500 €/mois (HT), ou 0 € de setup + 6 000 €/an (HT)."
      },
      {
        "title": "L'API ne change pas",
        "bodyHtml": "Cloud, auto-hébergé et sur site font tourner la même <a href=\"/fr/api-webhook\" class=\"text-green-400 hover:text-green-300 transition-colors\">webhook API</a>. Passer de l'un à l'autre change ton déploiement pendant que ton code d'intégration reste tel quel."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Questions sur le webhook service",
    "items": [
      {
        "q": "Qu'est-ce qu'un webhook service ?",
        "a": "Un webhook service prend un event depuis ton backend et le livre aux endpoints HTTP que tes clients ont enregistrés. Il porte les quatre parties pénibles à porter soi-même : signer chaque payload pour que le destinataire lui fasse confiance, relancer quand le destinataire est hors service, garder un journal de chaque tentative, et donner à tes clients un endroit pour gérer leurs propres endpoints. Tu fais un appel d'API ; le service gère le reste de la livraison."
      },
      {
        "q": "Quelle différence entre un webhook service et une webhook platform ?",
        "a": "En pratique les deux mots décrivent le même produit, et les éditeurs les emploient indifféremment. « Service » désigne plutôt le côté managé (quelqu'un d'autre opère la livraison pour toi), tandis que « platform » désigne plutôt la surface sur laquelle tu construis : API, portail, types d'events, journaux. Hook0 est les deux, et c'est pourquoi les mêmes fonctions apparaissent sur cette page et sur la page webhook platform."
      },
      {
        "q": "Combien coûte un webhook service ?",
        "a": "Hook0 est gratuit jusqu'à 100 events/jour, avec des offres payantes à partir de 59 €/mois. Parmi les autres fournisseurs managés, au dernier relevé de juillet 2026, l'offre Growth avec SLA de Hookdeck est à 499 $/mois, le Pro managé de Svix démarre à 490 $/mois, et le Premium managé de Convoy est à 999 $/mois. L'auto-hébergement déplace le coût d'un abonnement vers ta propre infrastructure et ton temps de maintenance."
      },
      {
        "q": "Où Hook0 héberge-t-il les données webhook ?",
        "a": "Le data plane webhook, c'est-à-dire les payloads, la base de données et les sauvegardes, tourne sur l'infrastructure de Clever Cloud SAS en France, dans l'Espace économique européen, sur chaque offre, tier gratuit compris. Le CDN et la couche anti-DDoS devant le site et l'API sont fournis par Cloudflare, Inc. (USA), divulgué dans notre liste publique de sous-traitants et encadré par les clauses contractuelles types 2021, une analyse d'impact de transfert documentée et, le cas échéant, le Data Privacy Framework UE-États-Unis."
      },
      {
        "q": "Puis-je quitter le service managé plus tard ?",
        "a": "Oui, et c'est la raison pour laquelle le code est open source sous SSPL-1.0, sans rétention open-core. Tu peux auto-héberger le même logiciel avec Docker Compose ou Kubernetes, ou nous faire opérer une instance dédiée dans ton propre environnement. L'API reste la même dans les trois cas, donc ton code d'intégration ne change pas."
      },
      {
        "q": "Ai-je encore besoin d'un webhook service si j'envoie déjà des requêtes HTTP ?",
        "a": "Envoyer la première requête, c'est la partie facile. Les sprints passent dans tout ce qui vient après : un plan de relance qui ne martèle pas un destinataire en difficulté, des signatures que tes clients peuvent vérifier, des journaux par tentative pour la conversation avec le support, le rejeu après une panne, et un portail pour que les abonnés gèrent leurs propres endpoints. Si tu as déjà tout construit et que tu aimes le maintenir, tu n'as pas besoin d'un webhook service."
      }
    ]
  },
  "related": {
    "h2": "Pour aller plus loin",
    "links": [
      { "label": "Webhook Platform", "href": "/fr/plateforme-webhook" },
      { "label": "Webhook API", "href": "/fr/api-webhook" },
      { "label": "Serveur MCP Webhook", "href": "/fr/serveur-mcp-webhook" },
      { "label": "Build vs Buy Webhooks", "href": "/fr/build-vs-buy-webhooks" },
      { "label": "Comparatif des coûts webhook", "href": "/fr/comparatif-cout-webhook" },
      { "label": "Infrastructure webhook européenne", "href": "/fr/infrastructure-webhook-europeenne" },
      { "label": "Webhooks auto-hébergés", "href": "/fr/webhooks-auto-heberges" },
      { "label": "Tarifs", "href": "/fr/tarifs" }
    ]
  }
};
