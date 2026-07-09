// Per-page strings for webhook-cost-comparison (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de middle-dot.
// Hook0 = « code source ouvert (SSPL-1.0) », JAMAIS « open source » nu (SSPL hors OSI, risque L121-1).
// Chiffres Hook0 vérifiés dans locales/en/pricing.js + src/includes/_pricing.ejs.
// Chiffres concurrents relevés sur leurs pages tarifs publiques le 2026-07-08.
module.exports = {
  pageTitle: 'Coût d\'un service webhook : Hook0 vs Svix vs Hookdeck vs Convoy',
  pageDescription: 'Ce que coûte un service webhook de 100k à 10M events/mois : Hook0 dès 59 €, Svix dès 490 $, Hookdeck Outpost 10 $/M, Convoy 0 ou 999 $. Prix publics, juillet 2026.',
  pageModified: '2026-07-08',
  breadcrumb: 'Comparatif de coût webhook',
  track: 'fr-comparatif-cout-webhook',
  hero: {
    eyebrow: 'Comparatif de coût',
    titleBefore: 'Comparatif du coût des webhooks :',
    titleAccent: 'ce que tu paies vraiment en 2026',
    subtitle: 'Quatre services webhook, quatre modèles de facturation. On a posé les prix publics de Hook0, Svix, Hookdeck Outpost et Convoy sur les mêmes volumes, de 100 000 à 10 millions d\'events par mois, avec les astérisques que les pages tarifs ont tendance à oublier.',
    ctaPrimary: 'Démarrer gratuitement',
    ctaSecondary: 'Voir les tarifs Hook0',
    microcopy: '100 events/jour gratuits. Sans carte bancaire. Code source ouvert (SSPL-1.0).',
  },
  socialProof: true,
  costTable: {
    eyebrow: 'Cloud vs cloud',
    h2: 'Prix mensuel par volume d\'events',
    subtitle: 'Plans cloud managés uniquement, prix publics relevés le 8 juillet 2026. L\'auto-hébergement est traité plus bas.',
    headers: {
      provider: 'Service',
      volumes: ['100k events/mois', '1M events/mois', '3M events/mois', '10M events/mois'],
      eu: 'Résidence des données en UE (prix d\'entrée)',
    },
    rows: [
      {
        provider: 'Hook0 Cloud¹',
        highlight: true,
        cells: ['59 € (Startup)', '190 € (Pro)', '190 € (Pro)', '≈ 890 € (Pro + dépassement)'],
        eu: 'Incluse sur tous les plans, tier gratuit compris. Plan de données hébergé en France (Clever Cloud).⁶',
      },
      {
        provider: 'Svix Free²',
        highlight: false,
        cells: ['≈ 5 $', '≈ 95 $', '≈ 295 $', '≈ 995 $'],
        eu: 'Aucune région cloud UE documentée sur la page tarifs publique.',
      },
      {
        provider: 'Svix Professional³',
        highlight: false,
        cells: ['≈ 495 $', '≈ 585 $', '≈ 785 $', '≈ 1 485 $'],
        eu: 'Dès 490 $/mois : DPA EEE inclus, pas de région hébergée en UE documentée.',
      },
      {
        provider: 'Hookdeck Outpost (managé)⁴',
        highlight: false,
        cells: ['1 $', '10 $', '30 $', '100 $'],
        eu: 'Région UE disponible au même tarif de 10 $/M (régions exactes non publiées).',
      },
      {
        provider: 'Convoy⁵',
        highlight: false,
        cells: ['n.c.', 'n.c.', 'n.c.', 'n.c.'],
        eu: 'Pas d\'option UE managée, tu choisis ta région en auto-hébergeant.',
      },
    ],
    footnotes: [
      '¹ Les quotas Hook0 sont journaliers ; les chiffres supposent des events répartis uniformément sur un mois de 30 jours. Startup (59 €/mois) couvre jusqu\'à 30 000 events/jour, puis 0,003 €/event. Pro (190 €/mois, ou 1 824 €/an avec la remise annuelle de 20%) couvre jusqu\'à 100 000 events/jour, puis 0,0001 €/event. Les souscriptions et les relances sont gratuites, et le dépassement ne bloque jamais la livraison.',
      '² Svix Free inclut 50 000 messages/mois, puis 0,0001 $/message, plafonné à 200 msg/s avec un seul connecteur et le branding Svix. SOC 2 Type II, retrait du branding, IP statiques et rétention 90 jours démarrent sur Professional.',
      '³ Svix Professional démarre à 490 $/mois avec 50 000 messages inclus, puis 0,0001 $/message. Des remises de volume s\'appliquent aux gros volumes, le chiffre à 10M est donc une borne haute.',
      '⁴ Hookdeck Outpost managé est une infrastructure de livraison sortante facturée à l\'usage, 10 $ par million d\'events, sans minimum mensuel. Hookdeck vend aussi une Event Gateway entrante, produit séparé avec ses propres plans (0 à 499 $/mois).',
      '⁵ Convoy ne publie pas de tarif cloud managé (son trial cloud est limité à 1 projet et 100 events/jour). En auto-hébergé : Community est gratuit, le palier Premium est licencié à 999 $/mois forfaitaires, infrastructure non comprise.',
      '⁶ Le plan de données Hook0 (API et base de données) tourne chez Clever Cloud en France, opéré par une société française. Cloudflare (USA) sert de CDN et figure dans notre DPA.',
    ],
    pricesChecked: 'Tous les prix ont été relevés sur les pages tarifs publiques le 8 juillet 2026. Un chiffre périmé ? Dis-le nous, on corrige.',
  },
  methodology: {
    eyebrow: 'Méthode',
    h2: 'Comment lire ce tableau',
    items: [
      'Plans cloud managés uniquement. L\'auto-hébergement (licences plus infrastructure) est comparé dans la section suivante.',
      'Devises telles que publiées : euros pour Hook0, dollars pour les autres. Aucune conversion appliquée.',
      'Hook0 facture par event, pas par livraison : un event qui part vers plusieurs souscriptions, relances comprises, compte une seule fois. Svix facture par message, Hookdeck Outpost par event.',
      'Le prix affiché n\'est pas le TCO : rétention, débit, résidence des données en UE, support et conformité varient fortement d\'un plan à l\'autre. Les notes portent les réserves.',
    ],
  },
  selfHost: {
    eyebrow: 'Auto-hébergement',
    h2: 'TCO auto-hébergé : la licence est gratuite, pas les ops',
    intro: 'Les quatre produits tournent sur ta propre infrastructure. La ligne licence est ce qui les distingue ; tout le reste coûte pareil.',
    cards: [
      {
        title: 'Hook0 (SSPL-1.0)',
        body: 'Code source ouvert (SSPL-1.0) : le serveur complet est publié, rien n\'est retenu pour un palier enterprise. Licence à 0 €. Tourne sur Docker Compose ou Kubernetes avec PostgreSQL.',
      },
      {
        title: 'Svix (MIT, open-core)',
        body: 'Le cœur du serveur est en MIT, mais plusieurs fonctions enterprise restent fermées et Svix réserve l\'auto-hébergement à ses clients enterprise. Licence à 0 $ pour le cœur.',
      },
      {
        title: 'Hookdeck Outpost (Apache 2.0)',
        body: 'Outpost est en Apache 2.0 sans fork privé : le code auto-hébergé est le même que le managé. Licence à 0 $. Le service managé ajoute le scaling serverless, SOC 2, SSO, RBAC et le support.',
      },
      {
        title: 'Convoy (Elastic License v2)',
        body: 'Source disponible mais non approuvée par l\'OSI : la licence interdit de proposer Convoy en service managé. Community est gratuit ; le palier Premium (transformations JS, RBAC, portail en marque blanche) est licencié à 999 $/mois.',
      },
    ],
    opsCard: {
      title: 'La facture que personne ne met sur sa page tarifs',
      body: 'Quelle que soit la licence, l\'auto-hébergement coûte à peu près pareil : du compute pour l\'API et les workers, un PostgreSQL de production (plus des files ou Redis selon le produit), le monitoring, les sauvegardes, les patchs de sécurité, les montées de version et une astreinte. Ces lignes se moquent du logo sur le dépôt. Compare les licences et les fonctions, puis chiffre les ops honnêtement : elles dépassent vite un abonnement à 59 € ou 490 $.',
      close: 'C\'est aussi pour ça qu\'on publie les deux : Hook0 Cloud quand tu veux les ops incluses, l\'auto-hébergement quand tu as l\'équipe plateforme pour.',
    },
  },
  faq: {
    eyebrow: 'FAQ',
    h2: 'Questions sur le coût des webhooks',
    items: [
      {
        q: 'Combien coûte Hook0 ?',
        a: 'Hook0 Cloud a un tier Developer gratuit : 100 events/jour, sans carte bancaire. Startup coûte 59 €/mois pour jusqu\'à 30 000 events/jour, puis 0,003 €/event. Pro coûte 190 €/mois pour jusqu\'à 100 000 events/jour, puis 0,0001 €/event, ou 1 824 €/an avec la remise annuelle de 20%. Enterprise est sur devis. Auto-héberger le code source ouvert (SSPL-1.0) est gratuit.',
      },
      {
        q: 'Combien coûte un service webhook à 1 million d\'events par mois ?',
        a: 'À 1M events/mois sur les prix publics de juillet 2026 : Hook0 Pro coûte 190 €, Svix Professional environ 585 $, Hookdeck Outpost managé environ 10 $ et Convoy ne publie pas de prix cloud (l\'auto-hébergement est gratuit, sa licence Premium coûte 999 $/mois). Le prix à l\'event n\'est qu\'une partie de la facture : rétention, débit, résidence des données en UE, support et conformité varient fortement entre ces plans.',
      },
      {
        q: 'Pourquoi Hookdeck Outpost est-il tellement moins cher à l\'event ?',
        a: '10 $ par million d\'events est un prix de livraison réellement bas et on ne va pas prétendre le contraire. Outpost managé est une infrastructure de livraison sortante facturée à l\'usage. Hook0 vend des plans forfaitaires qui incluent le dashboard, le filtrage par souscription sur des attributs métier, 7 à 30 jours de rétention selon le plan, un plan de données en UE et une option on-premise. Selon ce dont tu as besoin, l\'un ou l\'autre peut être le total le moins cher.',
      },
      {
        q: 'Auto-héberger un service webhook est-il vraiment gratuit ?',
        a: 'La licence l\'est en général : Hook0 (SSPL-1.0), le cœur de Svix (MIT) et Hookdeck Outpost (Apache 2.0) ne coûtent rien à faire tourner toi-même, tandis que Convoy est gratuit en Community et à 999 $/mois pour les fonctions Premium. L\'infrastructure et l\'exploitation ne sont jamais gratuites : compute, PostgreSQL, monitoring, montées de version et temps d\'astreinte forment le vrai TCO auto-hébergé, et il est identique quel que soit le produit déployé.',
      },
      {
        q: 'Quel service webhook inclut la résidence des données en UE ?',
        a: 'Hook0 l\'inclut sur tous les plans, tier gratuit compris : le plan de données tourne chez Clever Cloud en France, et Cloudflare (société américaine) sert de CDN, comme divulgué dans notre DPA. Hookdeck Outpost managé propose une région UE au même tarif de 10 $/M. Svix documente un DPA EEE dès Professional (490 $/mois) mais aucune région hébergée en UE sur sa page tarifs publique. Convoy n\'a pas d\'option UE managée ; tu choisis ta région en auto-hébergeant.',
      },
    ],
  },
  related: {
    h2: 'Sur le même sujet',
    links: [
      { enSlug: 'pricing', label: 'Tarifs Hook0' },
      { enSlug: 'hook0-vs-svix', label: 'Hook0 vs Svix' },
      { enSlug: 'hook0-vs-hookdeck', label: 'Hook0 vs Hookdeck' },
      { enSlug: 'hook0-vs-convoy', label: 'Hook0 vs Convoy' },
      { enSlug: 'build-vs-buy-webhooks', label: 'Construire vs acheter ses webhooks' },
      { enSlug: 'self-hosted-webhooks', label: 'Webhooks auto-hébergés' },
    ],
  },
};
