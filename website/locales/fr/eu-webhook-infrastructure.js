// Per-page strings pour eu-webhook-infrastructure (FR, slug : infrastructure-webhook-europeenne).
// /humanizer pro + contraintes légales appliquées (cf. website/CLAUDE.local.md) :
//   - Data plane = Clever Cloud SAS (France, EEE). CDN Cloudflare, Inc. (USA)
//     DIVULGUÉ, encadré par CCT 2021 + TIA et, le cas échéant, le DPF UE-USA.
//   - JAMAIS « 100 % souverain », « aucune donnée ne quitte l'UE », « hors CLOUD Act ».
//   - RGPD/NIS2/DORA = claims de processus (« conçu pour », « soutient tes
//     exigences »), jamais « certifié ».
//   - Licence = « code source ouvert (SSPL-1.0) », jamais « open source » nu.
// Faits concurrents sourcés des snapshots business.md du 2026-07-08. Prix
// on-premise géré vérifié dans src/includes/_pricing.ejs (1 000 € setup +
// 500 €/mois HT, ou 0 € setup + 6 000 €/an HT).
module.exports = {
  "pageTitle": "Infrastructure webhook européenne : hébergée en France | Hook0",
  "pageDescription": "Le data plane webhook de Hook0 tourne chez Clever Cloud, en France, dès le tier gratuit. Éditeur de droit français, sous-traitants publics, self-host possible.",
  "pageModified": "2026-07-16",
  "track": "fr-infrastructure-webhook-eu",
  "hero": {
    "eyebrow": "Infrastructure webhook européenne",
    "titleLine1": "L'Europe par défaut,",
    "titleLine2": "pas en option payante",
    "subtitle": "Le data plane webhook de Hook0 tourne chez Clever Cloud, en France, dès le tier gratuit. L'éditeur est une société de droit français, sans maison-mère américaine. Et si un jour tu veux partir, le même code s'auto-héberge — code source ouvert (SSPL-1.0).",
    "ctaPrimary": "Démarrer gratuitement",
    "ctaSecondary": "Voir les tarifs",
    "ctaSecondaryHref": "/fr/tarifs",
    "microcopy": "100 events/jour gratuits. Sans carte bancaire. Data plane en France sur chaque offre."
  },
  "socialProof": true,
  "pillars": {
    "eyebrow": "Résidence des données",
    "h2": "Où vivent vraiment tes données webhook",
    "cards": [
      {
        "title": "Data plane en France, dès le tier gratuit",
        "bodyHtml": "Payloads, base de données et sauvegardes tournent sur l'infrastructure de Clever Cloud SAS, en France, dans l'Espace économique européen. Ce n'est pas une option réservée aux grands comptes : le tier gratuit et toutes les offres payantes utilisent le même data plane européen."
      },
      {
        "title": "Un éditeur de droit français",
        "bodyHtml": "Hook0 est développé et opéré par une société de droit français, sans maison-mère américaine. Ton contrat, ton DPA et tes questions de protection des données se traitent sous juridiction européenne."
      },
      {
        "title": "Un edge transparent : Cloudflare, divulgué",
        "bodyHtml": "Notre couche CDN et anti-DDoS est fournie par Cloudflare, Inc. (États-Unis). On le divulgue plutôt que de l'enterrer : ces transferts sont encadrés par les clauses contractuelles types 2021, une analyse d'impact de transfert documentée et, le cas échéant, le Data Privacy Framework UE-États-Unis. La <a href=\"/fr/sous-traitants-rgpd\" class=\"text-green-400 hover:text-green-300 transition-colors\">liste des sous-traitants</a> est publique."
      },
      {
        "title": "Un DPA qui se lit vraiment",
        "bodyHtml": "Le <a href=\"/fr/accord-traitement-donnees\" class=\"text-green-400 hover:text-green-300 transition-colors\">Data Processing Addendum</a> de Hook0 liste chaque sous-traitant et son mécanisme de transfert. Pas besoin d'un call commercial pour savoir où passent tes données."
      }
    ]
  },
  "residency": {
    "eyebrow": "Comparatif",
    "h2": "Résidence EU des données, fournisseur par fournisseur",
    "subtitle": "Ce qu'il faut faire, et payer, pour garder tes données webhook dans l'UE chez chaque fournisseur managé. Tarifs publics, vérifiés en juillet 2026.",
    "headers": {
      "provider": "Fournisseur",
      "residency": "Résidence EU sur le cloud managé",
      "price": "Prix d'entrée"
    },
    "rows": [
      {
        "highlight": true,
        "provider": "Hook0",
        "residencyHtml": "Par défaut. Data plane chez Clever Cloud (France) sur chaque offre, tier gratuit compris.",
        "priceHtml": "0 € (tier gratuit) ; offres payantes à partir de 59 €/mois"
      },
      {
        "highlight": false,
        "provider": "Hookdeck",
        "residencyHtml": "Régions US, EU et Asie sur la plateforme managée ; les régions EU exactes ne sont pas publiées.",
        "priceHtml": "Tier gratuit disponible ; l'offre Growth avec SLA est à 499 $/mois"
      },
      {
        "highlight": false,
        "provider": "Svix",
        "residencyHtml": "Pas de cloud managé hébergé en Europe affiché ; la résidence des données est évoquée sans région EU explicite.",
        "priceHtml": "L'offre managée Pro démarre à 490 $/mois ; self-host (MIT) pour maîtriser toi-même la résidence"
      },
      {
        "highlight": false,
        "provider": "Convoy",
        "residencyHtml": "Pas de résidence géographique managée ; choisir sa région impose de s'auto-héberger.",
        "priceHtml": "La version Community auto-hébergée est gratuite (licence Elastic v2) ; l'offre managée Premium est à 999 $/mois"
      }
    ],
    "footnote": "Sources : pages de tarifs et documentations publiques de chaque fournisseur, vérifiées le 16 juillet 2026. Un chiffre a bougé ? Dis-le-nous, on corrige."
  },
  "reversibility": {
    "eyebrow": "Réversibilité",
    "h2": "La réversibilité, l'autre moitié de la souveraineté",
    "intro": "L'hébergement dans l'UE compte moins si partir est impossible. Certains services de webhooks hébergés dans l'UE sont fermés et cloud-only, et la seule sortie possible est un export suivi d'une reconstruction. Les déploiements cloud, auto-hébergé et on-premise de Hook0 partagent une seule base de code, donc partir revient à faire tourner le même logiciel ailleurs.",
    "cards": [
      {
        "title": "Auto-héberge-le toi-même",
        "bodyHtml": "Le code complet de Hook0 est en code source ouvert (SSPL-1.0). Docker Compose ou Kubernetes, PostgreSQL en dessous. Tes payloads webhook restent dans ton propre réseau. Voir <a href=\"/fr/webhooks-auto-heberges\" class=\"text-green-400 hover:text-green-300 transition-colors\">webhooks auto-hébergés</a>."
      },
      {
        "title": "On-premise géré",
        "bodyHtml": "On déploie une instance Hook0 dédiée dans ton environnement et on la maintient à jour : 1 000 € de setup + 500 €/mois HT, ou 0 € de setup + 6 000 €/an HT. Ton infrastructure, notre maintenance."
      }
    ]
  },
  "compliance": {
    "eyebrow": "Conformité",
    "h2": "Pensé pour les équipes soumises au RGPD, à NIS2 ou à DORA",
    "intro": "Hook0 ne te vend pas un tampon de conformité. Il te donne les éléments concrets que tes auditeurs demandent.",
    "cards": [
      {
        "title": "RGPD",
        "bodyHtml": "Le data plane reste en France (EEE), les sous-traitants sont documentés avec leurs mécanismes de transfert, et un DPA est disponible avant de signer quoi que ce soit. Hook0 est conçu pour la conformité au RGPD — des preuves que tu peux pointer du doigt, pas un badge."
      },
      {
        "title": "NIS2",
        "bodyHtml": "NIS2 est une directive qui s'applique à ton organisation, pas une certification produit. Hook0 soutient tes exigences : résidence EU des données par défaut, liste publique des sous-traitants, logs de livraison pour chaque tentative et une <a href=\"/fr/securite\" class=\"text-green-400 hover:text-green-300 transition-colors\">page sécurité</a> documentée."
      },
      {
        "title": "DORA",
        "bodyHtml": "Les entités financières soumises à DORA scrutent le risque tiers TIC et les stratégies de sortie. Hook0 alimente cette analyse : data plane en Europe, sous-traitants documentés, et une vraie porte de sortie — self-host ou on-premise sur le même code."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Questions sur l'infrastructure webhook européenne",
    "items": [
      {
        "q": "Où sont hébergées les données webhook de Hook0 ?",
        "a": "Le data plane webhook de Hook0 — payloads, base de données, sauvegardes — tourne sur l'infrastructure de Clever Cloud SAS, en France, dans l'Espace économique européen. La couche CDN et anti-DDoS devant le site et l'API est fournie par Cloudflare, Inc. (États-Unis), divulguée dans notre liste publique de sous-traitants et encadrée par les clauses contractuelles types 2021, une analyse d'impact de transfert documentée et, le cas échéant, le Data Privacy Framework UE-États-Unis."
      },
      {
        "q": "Hook0 est-il conforme au RGPD ?",
        "a": "Hook0 est conçu pour la conformité au RGPD : data plane en Europe, liste publique des sous-traitants avec leurs mécanismes de transfert, et un Data Processing Addendum consultable avant toute signature. Il n'existe pas de badge RGPD officiel pour les fournisseurs de webhooks, alors quel que soit celui que tu évalues, demande le DPA et la liste des sous-traitants. Les nôtres sont publics."
      },
      {
        "q": "Hook0 soutient-il les exigences NIS2 ou DORA ?",
        "a": "NIS2 et DORA s'appliquent à ton organisation ; aucun fournisseur de webhooks ne peut être « certifié » sur ces textes. Ce que Hook0 apporte, c'est la matière dont ton équipe conformité a besoin : résidence EU des données par défaut, sous-traitants documentés, logs de livraison pour chaque tentative, et une stratégie de sortie (self-host ou on-premise géré, sur le même code que le cloud)."
      },
      {
        "q": "Est-ce que je peux quitter le cloud Hook0 plus tard ?",
        "a": "Oui. Le cloud, le self-host et l'on-premise partagent un seul code, en code source ouvert (SSPL-1.0). Tu peux t'auto-héberger avec Docker Compose ou Kubernetes, ou nous demander d'opérer une instance dédiée dans ton environnement pour 1 000 € de setup + 500 €/mois HT. Dans les deux cas tu gardes la même API, donc ton code d'intégration ne change pas."
      },
      {
        "q": "Quels fournisseurs de webhooks proposent une résidence EU sur leur cloud managé ?",
        "a": "En juillet 2026 : Hook0 héberge son data plane en France sur chaque offre. Hookdeck affiche des régions US, EU et Asie sur sa plateforme managée, sans publier les régions EU exactes. Svix n'affiche pas de cloud managé hébergé en Europe. Convoy ne propose pas de résidence géographique managée — tu choisis ta région en t'auto-hébergeant. Les offres évoluent, vérifie aussi leur documentation à jour."
      }
    ]
  },
  "related": {
    "h2": "Pour aller plus loin",
    "links": [
      { "label": "Webhooks auto-hébergés", "href": "/fr/webhooks-auto-heberges" },
      { "label": "Tarifs", "href": "/fr/tarifs" },
      { "label": "Comparatif coût webhook", "href": "/fr/comparatif-cout-webhook" },
      { "label": "Sous-traitants RGPD", "href": "/fr/sous-traitants-rgpd" },
      { "label": "Sécurité", "href": "/fr/securite" },
      { "label": "Data Processing Addendum", "href": "/fr/accord-traitement-donnees" }
    ]
  }
};
