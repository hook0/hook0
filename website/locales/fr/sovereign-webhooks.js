// Chaînes FR pour sovereign-webhooks. Passé au humanizer pro (voix Hooky, « tu »,
// vocabulaire dev, sans AI-slop). Mêmes garde-fous que la base EN :
//   - aucune certification revendiquée pour Hook0 (SecNumCloud, HDS, niveau SEAL,
//     ISO 27001, SOC 2) ; aucun badge ; aucune mention négative de certif ;
//   - « open source (SSPL-1.0) », jamais « 100 % open source » ni « source-available » ;
//   - pas de chiffrement at-rest par défaut ;
//   - la section CLOUD Act est pédagogique + checklist générique ; elle n'affirme
//     PAS que Hook0 est hors du champ du CLOUD Act (conclusion juridique réservée à
//     /legal). Elle ne reprend que des faits déjà publiés : éditeur de droit français
//     sans maison-mère US, data plane en France, edge Cloudflare (USA) divulgué sous
//     CCT 2021 + TIA + DPF. Jamais « aucune donnée ne quitte l'UE » ;
//   - aucun concurrent nommé ici ; les liens « Pour aller plus loin » renvoient aux
//     pages comparatives existantes.
// Le texte de faq.items[].a doit correspondre mot pour mot à la carte visible ; le
// JSON-LD FAQPage est généré depuis ce même tableau.
module.exports = {
  "pageTitle": "Webhooks souverains : la souveraineté numérique par conception | Hook0",
  "pageDescription": "La souveraineté, c'est qui peut être légalement contraint de produire tes données, pas seulement où elles sont stockées. Hook0 : éditeur de droit français, data plane en France, open source, réversible.",
  "pageModified": "2026-10-06",
  "track": "fr-webhooks-souverains",
  "tldr": {
    "h2": "En bref",
    "body": "La souveraineté numérique ne se joue pas à l'emplacement d'un serveur, mais à la question de savoir qui peut être légalement contraint de produire tes données. Hook0 y répond par des propriétés vérifiables plutôt que par un badge. L'éditeur est une société de droit français, sans maison-mère américaine. Son data plane webhook tourne en France (Clever Cloud) sur chaque offre, tier gratuit compris. Toute la plateforme est open source : serveur sous SSPL-1.0, SDK sous MIT, et le code auto-hébergé est exactement celui du cloud managé, donc tu peux rapatrier ton infrastructure webhook sans fournisseur à qui faire confiance. La couche CDN et anti-DDoS est Cloudflare, Inc. (États-Unis), divulguée dans une liste publique de sous-traitants. Hook0 est conçu pour les exigences RGPD, NIS2 et DORA plutôt que certifié sur ces textes. Gratuit jusqu'à 100 events par jour, sans carte bancaire."
  },
  "hero": {
    "eyebrow": "Souveraineté numérique",
    "titleLine1": "Des webhooks souverains,",
    "titleLine2": "pas seulement hébergés en UE",
    "subtitle": "Localiser les données, c'est la partie facile. La souveraineté, c'est savoir qui peut être légalement contraint de les livrer. Hook0 est édité par une société de droit français sans maison-mère américaine, héberge son data plane en France et reste open source de bout en bout. Partir, c'est juste faire tourner le même code toi-même.",
    "ctaPrimary": "Démarrer gratuitement",
    "ctaSecondary": "Où vivent tes données",
    "ctaSecondaryHref": "/fr/infrastructure-webhook-europeenne",
    "microcopy": "100 events/jour gratuits. Sans carte bancaire. Data plane UE sur chaque offre."
  },
  "socialProof": true,
  "dimensions": {
    "eyebrow": "Localiser ≠ souverain",
    "h2": "La souveraineté, c'est trois questions, pas une",
    "intro": "Un datacenter à Paris est nécessaire, pas suffisant. Les grilles de souveraineté européennes découpent le mot en trois questions distinctes, et l'emplacement des données ne répond qu'à la première.",
    "cards": [
      {
        "title": "Juridique",
        "bodyHtml": "Quel droit s'applique au fournisseur, et quel droit peut le contraindre ? Un datacenter en UE, exploité par une société soumise à un droit hors UE, reste exposé aux demandes d'accès extraterritoriales. La juridiction de la société compte autant que l'emplacement du disque."
      },
      {
        "title": "Opérationnelle",
        "bodyHtml": "Qui exploite, supporte, met à jour et peut couper le service ? Où sont les équipes d'astreinte, et qui détient les clés de chiffrement ? La souveraineté s'érode dès que le contrôle quotidien sort de la juridiction que tu as choisie."
      },
      {
        "title": "Technologique",
        "bodyHtml": "De quoi est faite la pile (logiciel, base de données, identité, DNS) et est-elle ouverte, auditable, substituable ? Un service fermé et cloud-only ne te laisse qu'un export et une reconstruction comme porte de sortie. L'open source garde la pile substituable."
      }
    ]
  },
  "compulsion": {
    "eyebrow": "La vraie question",
    "h2": "Qui peut être légalement contraint de produire tes données ?",
    "intro": "Le CLOUD Act américain (2018) permet aux autorités US d'exiger d'un fournisseur soumis au droit américain qu'il produise des données qu'il contrôle, où qu'elles soient stockées, y compris dans une région UE. La question à poser à tout fournisseur de webhooks n'est donc pas seulement où sont les serveurs, mais qui pourrait recevoir l'ordre de livrer les données.",
    "checklistTitle": "Sept questions à poser à tout fournisseur de webhooks",
    "checklist": [
      "Qui détient et contrôle la société, et sous quel droit ?",
      "Où tournent réellement les données <strong>et</strong> les opérations ?",
      "Qui détient les clés de chiffrement ?",
      "Quelles dépendances cachées voyagent avec : identité, DNS, CDN, e-mail, registres de conteneurs, IA ?",
      "Quelle réversibilité : peux-tu exporter et faire tourner ailleurs, et est-ce déjà testé ?",
      "Quel niveau est revendiqué, et avec quelles preuves d'audit réellement consultables ?",
      "Quel plan si le fournisseur disparaît ou se fait couper ?"
    ],
    "hook0Title": "La réponse de Hook0, en faits vérifiables",
    "hook0Html": "La société derrière Hook0 est de droit français, sans maison-mère américaine. Son data plane webhook (payloads, base de données et sauvegardes) tourne en France (Clever Cloud), sur chaque offre. La plateforme est open source (SSPL-1.0) : les clés, la pile et la porte de sortie sont à toi, à inspecter et à emporter. La seule dépendance US divulguée est la couche CDN et anti-DDoS Cloudflare, listée dans une <a href=\"/fr/sous-traitants-rgpd\" class=\"text-green-400 hover:text-green-300 transition-colors\">liste publique de sous-traitants</a> et encadrée par les clauses contractuelles types 2021, une analyse d'impact de transfert documentée et, le cas échéant, le Data Privacy Framework UE-États-Unis. Lis le <a href=\"/fr/accord-traitement-donnees\" class=\"text-green-400 hover:text-green-300 transition-colors\">DPA</a> et décide par toi-même, sans appel commercial.",
    // legalTitle/legalHtml : énoncé borné validé par /legal (verbatim, ne pas
    // reformuler, ne pas passer au humanizer). Cadre l'analyse CLOUD Act à l'entité
    // éditrice et aux données qu'elle contrôle uniquement ; nomme Cloudflare (US)
    // comme restant dans le champ ; garde la réserve « aucune garantie absolue ».
    // Modifier ce texte impose une nouvelle validation /legal.
    "legalTitle": "Hook0 et le CLOUD Act américain",
    "legalHtml": "L'éditeur de Hook0, FGRibreau SARL, est une société française, sans filiale, maison-mère ni établissement aux États-Unis. Le CLOUD Act (18 U.S.C. § 2713, modifiant le Stored Communications Act) impose à un fournisseur soumis à la compétence des juridictions américaines de produire les données sous sa « possession, custody, or control ». En l'absence de tout rattachement aux États-Unis, l'éditeur n'est pas, à notre connaissance et en l'état actuel du droit, un fournisseur que les autorités américaines pourraient contraindre, au titre du CLOUD Act, à produire les données qu'il contrôle. Cette analyse vise l'entité éditrice et les données qu'elle contrôle ; elle ne s'étend pas aux tiers soumis au droit américain — en particulier Cloudflare (États-Unis), utilisé en edge, qui reste dans le champ du CLOUD Act pour les données qu'il traite. Le plan de données est hébergé par défaut en France (Clever Cloud). Hook0 étant open source et auto-hébergeable, un déploiement auto-hébergé ne fait intervenir aucun tiers susceptible d'être contraint."
  },
  "levers": {
    "eyebrow": "Les leviers",
    "h2": "Ce que Hook0 te donne concrètement",
    "intro": "Ce sont les leviers reconnus pour la résilience numérique et la réversibilité. Hook0 te donne les propriétés, pas une étiquette.",
    "cards": [
      {
        "title": "Juridiction et hébergement UE",
        "bodyHtml": "Une société de droit français, sans maison-mère US, avec le data plane webhook hébergé en France (Clever Cloud) sur chaque offre, tier gratuit compris, pas en option grand compte. Voir <a href=\"/fr/infrastructure-webhook-europeenne\" class=\"text-green-400 hover:text-green-300 transition-colors\">infrastructure webhook européenne</a> pour la résidence des données en détail."
      },
      {
        "title": "Open source de bout en bout",
        "bodyHtml": "Le serveur est open source sous SSPL-1.0, les SDK sous MIT. Pas d'édition entreprise fermée : le code que tu auto-héberges est celui qui fait tourner le cloud managé. Rien derrière un paywall que tu ne peux pas auditer. Voir <a href=\"/fr/webhooks-open-source\" class=\"text-green-400 hover:text-green-300 transition-colors\">webhooks open source</a>."
      },
      {
        "title": "Réversibilité, testée",
        "bodyHtml": "Fais tourner la même plateforme sur ton infrastructure avec Docker Compose ou Kubernetes, ou demande-nous d'exploiter une instance dédiée on-premise. En auto-hébergé, il ne reste aucun fournisseur tiers à contraindre : tu possèdes le code et les serveurs. Voir <a href=\"/fr/webhooks-auto-heberges\" class=\"text-green-400 hover:text-green-300 transition-colors\">webhooks auto-hébergés</a>."
      },
      {
        "title": "Dépendances divulguées",
        "bodyHtml": "Chaque sous-traitant et son mécanisme de transfert sont publiés, pas enfouis. L'edge Cloudflare (États-Unis) est nommé, et le <a href=\"/fr/accord-traitement-donnees\" class=\"text-green-400 hover:text-green-300 transition-colors\">Data Processing Addendum</a> est disponible avant toute signature. La transparence, c'est la part de souveraineté que tu peux vérifier dès aujourd'hui."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Questions sur les webhooks souverains",
    "items": [
      {
        "q": "Qu'est-ce qui rend une plateforme de webhooks souveraine ?",
        "a": "La souveraineté dépasse l'hébergement UE. Elle tient à qui peut être légalement contraint de produire tes données, qui exploite le service et détient les clés, et si la pile est ouverte et réversible. Les leviers vérifiables sont la juridiction de la société, l'emplacement des données et des opérations, le code open source, les dépendances divulguées et une porte de sortie testée. Hook0 fournit ces leviers plutôt qu'un badge de certification."
      },
      {
        "q": "Qu'est-ce que le CLOUD Act américain et quel rapport avec les données webhook ?",
        "a": "Le CLOUD Act (2018) permet aux autorités américaines d'exiger d'un fournisseur soumis au droit US qu'il produise des données qu'il contrôle, où qu'elles soient stockées, y compris dans une région UE. C'est pourquoi l'emplacement des données ne règle pas à lui seul la question de la souveraineté. La société derrière Hook0 est de droit français, sans maison-mère US, son data plane webhook tourne en France, et sa seule dépendance US divulguée est la couche CDN et anti-DDoS Cloudflare, publiée dans la liste des sous-traitants avec ses garanties de transfert. Lis le DPA et la liste des sous-traitants pour évaluer tes propres exigences."
      },
      {
        "q": "Hook0 est-il open source ?",
        "a": "Oui. Le serveur est open source sous SSPL-1.0 et les SDK sous MIT. Pas d'édition entreprise fermée : le code que tu auto-héberges est celui qui fait tourner le cloud managé, donc toute la pile reste auditable et réversible."
      },
      {
        "q": "Où Hook0 héberge-t-il les données webhook ?",
        "a": "Le data plane webhook de Hook0 (payloads, base de données et sauvegardes) tourne sur l'infrastructure Clever Cloud en France, dans l'Espace économique européen, sur chaque offre y compris le tier gratuit. La couche CDN et anti-DDoS devant le site et l'API est Cloudflare, Inc. (États-Unis), divulguée dans une liste publique de sous-traitants avec ses mécanismes de transfert."
      },
      {
        "q": "Puis-je faire tourner Hook0 sans aucun fournisseur tiers ?",
        "a": "Oui. Auto-héberge le serveur open source (SSPL-1.0) avec Docker Compose ou Kubernetes : tes payloads webhook restent dans ton propre réseau, sans fournisseur tiers à contraindre. Tu gardes la même API, donc ton code d'intégration ne change pas si tu passes plus tard du self-host à l'on-premise ou au cloud."
      },
      {
        "q": "Hook0 est-il soumis au CLOUD Act ?",
        "a": "L'éditeur (société française sans rattachement US) n'est pas, à notre connaissance et en l'état du droit, un fournisseur atteignable par une injonction CLOUD Act pour les données qu'il contrôle. En revanche, notre prestataire edge Cloudflare, société américaine, relève du CLOUD Act pour les données qu'il traite."
      },
      {
        "q": "Une autorité américaine peut-elle obtenir mes données ?",
        "a": "Pas en contraignant l'éditeur au titre du CLOUD Act, tant qu'il n'a aucun rattachement US. Mais aucune garantie absolue n'est possible : les données transitant par notre edge Cloudflare (US) relèvent du droit américain, et d'autres voies (coopération judiciaire internationale) existent indépendamment du CLOUD Act. L'auto-hébergement supprime tout tiers susceptible d'être contraint."
      },
      {
        "q": "La future structure au Luxembourg change-t-elle cette analyse ?",
        "a": "Non, sous une condition : l'entité luxembourgeoise reste une entité de l'Union européenne. Le raisonnement tient tant qu'elle n'a ni établissement, ni filiale, ni maison-mère, ni autre rattachement aux États-Unis."
      }
    ]
  },
  "related": {
    "h2": "Pour aller plus loin",
    "links": [
      { "enSlug": "eu-webhook-infrastructure", "label": "Infrastructure webhook européenne" },
      { "enSlug": "open-source-webhooks", "label": "Webhooks open source" },
      { "enSlug": "self-hosted-webhooks", "label": "Webhooks auto-hébergés" },
      { "enSlug": "gdpr-subprocessors", "label": "Sous-traitants RGPD" },
      { "enSlug": "security", "label": "Sécurité" },
      { "enSlug": "pricing", "label": "Tarifs" },
      { "enSlug": "webhook-cost-comparison", "label": "Comparatif coût webhook" }
    ]
  }
};
