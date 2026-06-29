// Per-page strings for gdpr-subprocessors (FR, Sous-traitants RGPD / art. 28 RGPD).
//
// Registre : vouvoiement formel strict, comme tout document juridique. Pas de
// tutoiement. /humanizer pro applique. Pas d'em-dash, pas de double tiret pivot,
// pas de point median. Sane defaults : pieces juridiques + factuelles, ton sobre.
//
// Faits durs Hook0 verbatim entre locales (CLAUDE.md / CLAUDE.local.md) :
//   - FGRibreau SARL, traitant pour le Contenu Client
//   - Entites sous-traitantes (noms et adresses VERBATIM) :
//       * Clever Cloud SAS (France)
//       * Scaleway SAS (France)
//       * Cloudflare, Inc. (USA, 101 Townsend St, San Francisco, CA 94107)
//       * Stripe Inc. (USA)
//       * Brevo (France)
//       * Postmark (USA)
//       * BetterUptime (Republique tcheque)
//       * Sentry (USA)
//       * Crisp (France)
//       * Gmail / Google Workspace (USA)
//   - Mecanismes de transfert : SCC 2021 (Clauses Contractuelles Types) + TIA
//     (Transfer Impact Assessment) pour les transferts US ; EU-US DPF (Data
//     Privacy Framework) lorsque le sous-traitant y est certifie.
//   - Pas de claim « 100% souverain » / « aucun partage de donnees » (L121-1
//     C. conso). RGPD = claim de processus, jamais absolu.
module.exports = {
  pageTitle: 'Hook0 - Sous-traitants RGPD',
  pageDescription: 'Conformité RGPD de Hook0 et liste des sous-traitants utilisés pour fournir nos services webhook. Transparence sur les traitements en Europe et sur les transferts vers les États-Unis.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Conformité',
    title: 'RGPD et sous-traitants',
    subtitle: 'Notre engagement en matière de protection des données et les partenaires avec lesquels nous travaillons.',
    lastUpdatedLabel: 'Dernière mise à jour :',
    lastUpdatedDate: '27 juin 2026',
  },
  intro: {
    p1Html: 'Le Règlement général sur la protection des données (RGPD / DSGVO) est la législation la plus stricte au monde en matière de vie privée et de sécurité. Il impose des obligations aux organisations partout dans le monde, dès lors qu\'elles ciblent ou collectent des données relatives à des personnes situées dans l\'Union européenne. Le règlement a été adopté par le Parlement européen en avril 2016 et est entré en vigueur le 25 mai 2018.',
    p2Html: 'Hook0 fait appel à certains sous-traitants ultérieurs pour fournir les Services applicatifs à ses clients, dans les conditions décrites par le contrat-cadre de services ou les conditions d\'utilisation publiées à l\'adresse <a href="./conditions-utilisation" class="text-green-400 hover:text-green-300 transition-colors">conditions d\'utilisation</a> ou à toute autre adresse à laquelle ces conditions seraient publiées ultérieurement (le « Contrat »). Les termes définis dans le présent document ont la signification qui leur est donnée dans le Contrat.',
  },
  whatIsPersonalData: {
    title: 'Qu\'est-ce qu\'une donnée à caractère personnel ?',
    bodyHtml: 'Le RGPD porte une attention particulière à la protection des données à caractère personnel des personnes physiques. Une donnée à caractère personnel (art. 4 RGPD) désigne toute information permettant d\'identifier une personne, directement ou indirectement. Il peut s\'agir, par exemple, d\'un nom, d\'une adresse e-mail, d\'un numéro de carte bancaire ou de documents contenant des informations personnelles.',
  },
  howWeProcess: {
    title: 'Comment nous traitons les données à caractère personnel',
    bodyHtml: 'Lorsque vous consultez nos sites ou utilisez nos services, nous sommes amenés à traiter vos données à caractère personnel sous une forme ou une autre. Vous trouverez toutes les informations utiles sur les données traitées, les bases légales applicables et vos droits dans notre <a href="./politique-confidentialite" class="text-green-400 hover:text-green-300 transition-colors">politique de confidentialité</a>.',
  },
  roles: {
    title: 'Sous-traitants et leurs rôles',
    p1Html: 'Un sous-traitant ultérieur est un tiers traitant de données engagé par Hook0, y compris une entité du groupe Hook0, qui a ou pourrait avoir accès au Contenu Client (lequel peut contenir des données à caractère personnel) ou le traiter. Hook0 fait appel à différents types de sous-traitants ultérieurs pour les fonctions détaillées dans les tableaux ci-dessous.',
    p2Html: 'Conformément aux articles 28(2) et 28(4) du RGPD, vous accordez à Hook0 une autorisation écrite générale pour recourir aux sous-traitants listés ci-dessous. Nous vous informerons de tout changement envisagé de cette liste, y compris l\'ajout ou le remplacement d\'un sous-traitant, en vous laissant un délai raisonnable pour formuler une objection avant la prise d\'effet du changement.',
  },
  infrastructure: {
    title: 'Infrastructure',
    intro: 'Nous utilisons les sous-traitants suivants pour notre environnement d\'infrastructure cloud et le stockage du Contenu Client :',
    table: {
      headers: ['Sous-traitant', 'Pays de traitement', 'Finalité', 'Mécanisme de transfert'],
      rows: [
        {
          name: 'Clever Cloud SAS',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Base de données clients, API et application web Hook0',
          transfer: 'Traitement UE (aucun transfert hors EEE)',
        },
        {
          name: 'Cloudflare, Inc. (101 Townsend St, San Francisco, CA 94107)',
          country: 'USA',
          countryIsEU: false,
          purpose: 'DNS et protection anti-DDoS',
          transfer: 'SCC 2021 + TIA ; EU-US DPF (Cloudflare est certifié DPF)',
        },
      ],
    },
  },
  customerContent: {
    title: 'Traitement du Contenu Client',
    intro: 'Hook0 fait appel à plusieurs sous-traitants pour superviser, maintenir et accompagner les Services applicatifs. Ces sous-traitants peuvent, sans que cela soit systématique, avoir accès au Contenu Client :',
    table: {
      headers: ['Sous-traitant', 'Pays', 'Finalité', 'Mécanisme de transfert'],
      rows: [
        {
          name: 'Clever Cloud SAS',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Workers qui appellent les endpoints d\'abonnement aux webhooks',
          transfer: 'Traitement UE (aucun transfert hors EEE)',
        },
        {
          name: 'Scaleway SAS',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Workers dédiés privés qui appellent les endpoints d\'abonnement aux webhooks (uniquement pour les clients concernés)',
          transfer: 'Traitement UE (aucun transfert hors EEE)',
        },
        {
          name: 'Stripe Inc.',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Gestion des abonnements clients Hook0',
          transfer: 'SCC 2021 + TIA ; EU-US DPF (Stripe est certifié DPF)',
        },
        {
          name: 'Brevo',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Envoi d\'e-mails transactionnels',
          transfer: 'Traitement UE (aucun transfert hors EEE)',
        },
        {
          name: 'Postmark',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Envoi d\'e-mails transactionnels',
          transfer: 'SCC 2021 + TIA',
        },
        {
          name: 'BetterUptime',
          country: 'République tchèque, Europe',
          countryIsEU: true,
          purpose: 'Supervision de disponibilité, page de statut et gestion des astreintes',
          transfer: 'Traitement UE (aucun transfert hors EEE)',
        },
        {
          name: 'Sentry',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Suivi des erreurs',
          transfer: 'SCC 2021 + TIA',
        },
        {
          name: 'Crisp',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Gestion de la relation client',
          transfer: 'Traitement UE (aucun transfert hors EEE)',
        },
        {
          name: 'Gmail (Google Workspace)',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Boîte mail de support',
          transfer: 'SCC 2021 + TIA ; EU-US DPF (Google LLC est certifié DPF)',
        },
      ],
    },
    footnoteHtml: '* La liste des sous-traitants ci-dessus s\'applique à tout nouveau client Hook0 à compter de la date indiquée en haut de cette page, ainsi qu\'aux clients existants qui n\'ont pas reçu de notification mentionnant une autre date d\'effet.',
  },
  control: {
    title: 'Vous gardez la main',
    bodyHtml: 'Hook0 est un SaaS français conçu pour la conformité au RGPD. Nous nous appuyons sur une infrastructure et des partenaires sélectionnés pour la confidentialité, l\'intégrité et la disponibilité de vos données. Si vous préférez ne pas vous reposer uniquement sur nos mesures ou sur celles de nos sous-traitants, vous pouvez continuer à bénéficier de nos services de support sans avoir à divulguer vos données de production.',
  },
  dataOwnership: {
    title: 'Propriété et gestion des données',
    p1Html: 'Le plan de données des charges utiles de vos webhooks (workers Clever Cloud et, en option, workers dédiés Scaleway) est exploité dans l\'UE et le contenu de vos webhooks n\'est pas transféré hors EEE pour leur livraison. Les sauvegardes sont stockées dans des centres de données français. Les services accessoires comme la facturation (Stripe), le suivi des erreurs (Sentry), la solution de secours d\'envoi d\'e-mails (Postmark), la boîte mail de support (Gmail) et la couche CDN / protection anti-DDoS (Cloudflare) impliquent en revanche des transferts vers les États-Unis, encadrés par les Clauses Contractuelles Types 2021 (SCC 2021) et une analyse d\'impact des transferts (TIA) documentée, ainsi que, le cas échéant, par le EU-US Data Privacy Framework. L\'ensemble du personnel et des consultants Hook0 susceptibles d\'accéder à votre déploiement est établi dans l\'UE.',
    p2Html: 'Concernant votre propre base d\'utilisateurs, il vous appartient de mettre en place les procédures nécessaires pour respecter le RGPD et de déclarer les transferts de données que vous gérez de manière autonome. Dans ce cas, Hook0 agit en qualité de sous-traitant ultérieur et notre <a href="./data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">DPA (Data Processing Agreement)</a> précise notre périmètre.',
  },
};
