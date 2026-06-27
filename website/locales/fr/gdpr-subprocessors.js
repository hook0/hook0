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
  pageDescription: 'Conformite RGPD de Hook0 et liste des sous-traitants utilises pour fournir nos services webhook. Transparence sur les traitements en Europe et sur les transferts vers les Etats-Unis.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Conformite',
    title: 'RGPD et sous-traitants',
    subtitle: 'Notre engagement en matiere de protection des donnees et les partenaires avec lesquels nous travaillons.',
    lastUpdatedLabel: 'Derniere mise a jour :',
    lastUpdatedDate: '27 juin 2026',
  },
  intro: {
    p1Html: 'Le Reglement general sur la protection des donnees (RGPD / DSGVO) est la legislation la plus stricte au monde en matiere de vie privee et de securite. Il impose des obligations aux organisations partout dans le monde, des lors qu\'elles ciblent ou collectent des donnees relatives a des personnes situees dans l\'Union europeenne. Le reglement a ete adopte par le Parlement europeen en avril 2016 et est entre en vigueur le 25 mai 2018.',
    p2Html: 'Hook0 fait appel a certains sous-traitants ulterieurs pour fournir les Services applicatifs a ses clients, dans les conditions decrites par le contrat-cadre de services ou les conditions d\'utilisation publiees a l\'adresse <a href="./terms" class="text-green-400 hover:text-green-300 transition-colors">conditions d\'utilisation</a> ou a toute autre adresse a laquelle ces conditions seraient publiees ulterieurement (le « Contrat »). Les termes definis dans le present document ont la signification qui leur est donnee dans le Contrat.',
  },
  whatIsPersonalData: {
    title: 'Qu\'est-ce qu\'une donnee a caractere personnel ?',
    bodyHtml: 'Le RGPD porte une attention particuliere a la protection des donnees a caractere personnel des personnes physiques. Une donnee a caractere personnel (art. 4 RGPD) designe toute information permettant d\'identifier une personne, directement ou indirectement. Il peut s\'agir, par exemple, d\'un nom, d\'une adresse e-mail, d\'un numero de carte bancaire ou de documents contenant des informations personnelles.',
  },
  howWeProcess: {
    title: 'Comment nous traitons les donnees a caractere personnel',
    bodyHtml: 'Lorsque vous consultez nos sites ou utilisez nos services, nous sommes amenes a traiter vos donnees a caractere personnel sous une forme ou une autre. Vous trouverez toutes les informations utiles sur les donnees traitees, les bases legales applicables et vos droits dans notre <a href="./privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">politique de confidentialite</a>.',
  },
  roles: {
    title: 'Sous-traitants et leurs roles',
    p1Html: 'Un sous-traitant ulterieur est un tiers traitant de donnees engage par Hook0, y compris une entite du groupe Hook0, qui a ou pourrait avoir acces au Contenu Client (lequel peut contenir des donnees a caractere personnel) ou le traiter. Hook0 fait appel a differents types de sous-traitants ulterieurs pour les fonctions detaillees dans les tableaux ci-dessous.',
    p2Html: 'Conformement aux articles 28(2) et 28(4) du RGPD, vous accordez a Hook0 une autorisation ecrite generale pour recourir aux sous-traitants listes ci-dessous. Nous vous informerons de tout changement envisage de cette liste, y compris l\'ajout ou le remplacement d\'un sous-traitant, en vous laissant un delai raisonnable pour formuler une objection avant la prise d\'effet du changement.',
  },
  infrastructure: {
    title: 'Infrastructure',
    intro: 'Nous utilisons les sous-traitants suivants pour notre environnement d\'infrastructure cloud et le stockage du Contenu Client :',
    table: {
      headers: ['Sous-traitant', 'Pays de traitement', 'Finalite', 'Mecanisme de transfert'],
      rows: [
        {
          name: 'Clever Cloud SAS',
          country: 'France, Europe',
          countryIsEU: true,
          purpose: 'Base de donnees clients, API et application web Hook0',
          transfer: 'Traitement UE (aucun transfert hors EEE)',
        },
        {
          name: 'Cloudflare, Inc. (101 Townsend St, San Francisco, CA 94107)',
          country: 'USA',
          countryIsEU: false,
          purpose: 'DNS et protection anti-DDoS',
          transfer: 'SCC 2021 + TIA ; EU-US DPF (Cloudflare est certifie DPF)',
        },
      ],
    },
  },
  customerContent: {
    title: 'Traitement du Contenu Client',
    intro: 'Hook0 fait appel a plusieurs sous-traitants pour superviser, maintenir et accompagner les Services applicatifs. Ces sous-traitants peuvent, sans que cela soit systematique, avoir acces au Contenu Client :',
    table: {
      headers: ['Sous-traitant', 'Pays', 'Finalite', 'Mecanisme de transfert'],
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
          purpose: 'Workers dedies prives qui appellent les endpoints d\'abonnement aux webhooks (uniquement pour les clients concernes)',
          transfer: 'Traitement UE (aucun transfert hors EEE)',
        },
        {
          name: 'Stripe Inc.',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Gestion des abonnements clients Hook0',
          transfer: 'SCC 2021 + TIA ; EU-US DPF (Stripe est certifie DPF)',
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
          country: 'Republique tcheque, Europe',
          countryIsEU: true,
          purpose: 'Supervision de disponibilite, page de statut et gestion des astreintes',
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
          purpose: 'Boite mail de support',
          transfer: 'SCC 2021 + TIA ; EU-US DPF (Google LLC est certifie DPF)',
        },
      ],
    },
    footnoteHtml: '* La liste des sous-traitants ci-dessus s\'applique a tout nouveau client Hook0 a compter de la date indiquee en haut de cette page, ainsi qu\'aux clients existants qui n\'ont pas recu de notification mentionnant une autre date d\'effet.',
  },
  control: {
    title: 'Vous gardez la main',
    bodyHtml: 'Hook0 est un SaaS francais concu pour la conformite au RGPD. Nous nous appuyons sur une infrastructure et des partenaires selectionnes pour la confidentialite, l\'integrite et la disponibilite de vos donnees. Si vous preferez ne pas vous reposer uniquement sur nos mesures ou sur celles de nos sous-traitants, vous pouvez continuer a beneficier de nos services de support sans avoir a divulguer vos donnees de production.',
  },
  dataOwnership: {
    title: 'Propriete et gestion des donnees',
    p1Html: 'Le plan de donnees des charges utiles de vos webhooks (workers Clever Cloud et, en option, workers dedies Scaleway) est exploite dans l\'UE et le contenu de vos webhooks n\'est pas transfere hors EEE pour leur livraison. Les sauvegardes sont stockees dans des centres de donnees francais. Les services accessoires comme la facturation (Stripe), le suivi des erreurs (Sentry), la solution de secours d\'envoi d\'e-mails (Postmark), la boite mail de support (Gmail) et la couche CDN / protection anti-DDoS (Cloudflare) impliquent en revanche des transferts vers les Etats-Unis, encadres par les Clauses Contractuelles Types 2021 (SCC 2021) et une analyse d\'impact des transferts (TIA) documentee, ainsi que, le cas echeant, par le EU-US Data Privacy Framework. L\'ensemble du personnel et des consultants Hook0 susceptibles d\'acceder a votre deploiement est etabli dans l\'UE.',
    p2Html: 'Concernant votre propre base d\'utilisateurs, il vous appartient de mettre en place les procedures necessaires pour respecter le RGPD et de declarer les transferts de donnees que vous gerez de maniere autonome. Dans ce cas, Hook0 agit en qualite de sous-traitant ulterieur et notre <a href="./data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">DPA (Data Processing Agreement)</a> precise notre perimetre.',
  },
};
