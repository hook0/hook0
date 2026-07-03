// Per-page strings for privacy-policy (FR, Politique de confidentialité,
// art. 13 RGPD). Extraction VERBATIM de la prose française originale du
// template legacy (src/privacy-policy.ejs, lignes ~568-1036), préservant
// l'auteur original ; corrections ISMS uniquement là où la prose française
// dérivait déjà de la vérité ISMS (durées par plan, DPF, identité responsable
// de traitement complète).
//
// Registre : vouvoiement formel strict, comme tout document juridique. Pas
// de tutoiement. /humanizer pro appliqué. Pas d'em-dash, pas de double tiret
// pivot, pas de point médian. Ton sobre, factuel.
//
// Faits durs Hook0 verbatim entre locales (CLAUDE.md / CLAUDE.local.md) :
//   - Responsable de traitement : FGRibreau SARL, capital 2 000 EUR,
//     RCS La Roche-sur-Yon 850 824 350, TVA FR27850824350, siège social
//     3 rue de l'Aubépine, 85110 Chantonnay, France.
//   - Directeur de la publication : David Sferruzza.
//   - Contact protection des données : legal@hook0.com (aligné sur le DPA).
//   - Sous-traitants ultérieurs (mêmes que gdpr-subprocessors.js) :
//       Clever Cloud (FR), Scaleway (FR), Cloudflare (USA),
//       Stripe (USA), Brevo (FR), Postmark (USA), BetterUptime (CZ),
//       Sentry (USA), Crisp (FR), Gmail/Google Workspace (USA),
//       Google LLC / Google Ads (USA, mesure de conversion server-side).
//   - Mécanismes de transfert : SCC 2021 + TIA ; EU-US DPF pour les
//     sous-traitants certifiés (Cloudflare, Stripe, Google LLC).
//   - Rétention par plan : Developer 7 j, Startup 14 j, Pro 30 j,
//     Enterprise sur mesure (information-retention-policy.md).
//   - Rétention compte : durée du contrat + 30 jours après suppression.
//   - Rétention facturation : 10 ans (art. L102 B Livre des procédures
//     fiscales).
//   - Notification de violation : 72 heures (art. 33/34 RGPD).
//   - TTL consentement cookies : 13 mois max (CNIL).
//   - Pas de claim absolu (« 100 % souverain », « aucun partage »,
//     « aucun prestataire US ») : risque L121-1 C. conso. RGPD = claim
//     de processus.
module.exports = {
  pageTitle: 'Hook0 - Politique de confidentialité',
  pageDescription: 'Politique de confidentialité Hook0, conforme à l\'article 13 du RGPD. Bases légales, durées de conservation, vos droits, sous-traitants et transferts hors UE.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Mentions légales',
    title: 'Politique de confidentialité',
    subtitle: 'Comment Hook0 collecte, utilise et protège vos données personnelles, conformément à l\'article 13 du RGPD.',
    lastUpdatedLabel: 'Dernière mise à jour :',
    lastUpdatedDate: '27 juin 2026',
  },
  controller: {
    title: '1. Responsable du traitement',
    p1: 'Le responsable du traitement de vos données personnelles dans le cadre du service Hook0 est :',
    // [LEGAL-CORRECTION L580-585] Ajout du capital, RCS, TVA et directeur de la
    // publication pour alignement avec la page Mentions légales et le DPA.
    // L'extrait legacy se limitait à l'adresse postale et au « SIRET sur
    // demande », ce qui est en dessous du standard art. 13(1)(a) RGPD + art.
    // 6-III LCEN.
    identityHtml: '<strong class="text-white">FGRibreau SARL</strong>, société à responsabilité limitée de droit français au capital de 2 000 EUR, immatriculée au registre du commerce et des sociétés de La Roche-sur-Yon sous le numéro 850 824 350, TVA FR27850824350, dont le siège social est situé 3 rue de l\'Aubépine, 85110 Chantonnay, France.<br>Directeur de la publication : David Sferruzza.<br>Contact protection des données : <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
    note: 'Hook0 est une plateforme SaaS exclusivement destinée aux professionnels (B2B). Nous ne collectons pas intentionnellement de données relatives à des personnes physiques agissant à titre privé.',
  },
  purposes: {
    title: '2. Finalités et bases légales des traitements',
    intro: 'Le tableau ci-dessous décrit chaque traitement, les données concernées et la base légale applicable au sens de l\'article 6 du RGPD.',
    headers: ['Finalité', 'Catégories de données', 'Base légale (art. 6 RGPD)'],
    rows: [
      {
        purposeHtml: '<strong class="text-white">Fourniture du service</strong><br><span class="text-gray-400 text-sm">Création de compte, authentification, accès à l\'API, livraison des webhooks</span>',
        data: 'Adresse e-mail, nom, clés API, charges utiles des webhooks, adresse IP, journaux d\'utilisation',
        basisHtml: 'Art. 6(1)(b), exécution du contrat',
      },
      {
        purposeHtml: '<strong class="text-white">Facturation et paiement</strong><br><span class="text-gray-400 text-sm">Gestion de l\'abonnement, émission de factures, obligations fiscales</span>',
        data: 'Nom, e-mail, adresse de facturation, données d\'instrument de paiement (traitées par Stripe), historique d\'abonnement',
        basisHtml: 'Art. 6(1)(b), exécution du contrat<br>Art. 6(1)(c), obligation légale (droit fiscal français, conservation 10 ans)',
      },
      {
        purposeHtml: '<strong class="text-white">Analytique du site web</strong><br><span class="text-gray-400 text-sm">Mesure de fréquentation via Matomo (auto-hébergé)</span>',
        data: 'Adresse IP anonymisée, pages visitées, référent, type d\'appareil, durée de session',
        basisHtml: 'Art. 6(1)(a), consentement (bandeau cookies)',
      },
      {
        purposeHtml: '<strong class="text-white">Suivi des conversions publicitaires (server-side)</strong><br><span class="text-gray-400 text-sm">Mesure des conversions Google Ads, exclusivement côté serveur via l\'identifiant de clic (gclid). Aucun e-mail, aucune adresse IP, aucun User-Agent n\'est transmis à Google. Droit d\'opposition à <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>.</span>',
        data: 'Identifiant de clic (gclid), identifiant pseudonyme généré par Google lors du clic publicitaire',
        basisHtml: 'Art. 6(1)(f), intérêt légitime (mesure de la performance publicitaire)<br><span class="text-gray-400 text-sm">Droit d\'opposition au titre de l\'art. 21(2) RGPD</span>',
      },
      {
        purposeHtml: '<strong class="text-white">Support client, chat en direct</strong><br><span class="text-gray-400 text-sm">Widget Crisp (chargé uniquement après consentement)</span>',
        data: 'Nom, e-mail, messages de chat, métadonnées du navigateur',
        basisHtml: 'Art. 6(1)(a), consentement',
      },
      {
        purposeHtml: '<strong class="text-white">Support client, e-mail</strong><br><span class="text-gray-400 text-sm">Traitement des demandes adressées à legal@hook0.com ou support@hook0.com</span>',
        data: 'Nom, e-mail, contenu des échanges',
        basisHtml: 'Art. 6(1)(f), intérêt légitime (répondre aux demandes des clients)',
      },
      {
        purposeHtml: '<strong class="text-white">Sécurité et supervision</strong><br><span class="text-gray-400 text-sm">Suivi des erreurs, monitoring de disponibilité, protection DDoS, gestion des incidents</span>',
        data: 'Adresse IP, traces d\'erreurs, métadonnées des requêtes, résultats de sondes de disponibilité',
        basisHtml: 'Art. 6(1)(f), intérêt légitime (garantir l\'intégrité et la sécurité du service)',
      },
      {
        purposeHtml: '<strong class="text-white">Communications commerciales</strong><br><span class="text-gray-400 text-sm">Mises à jour produit, notes de version, newsletters</span>',
        data: 'Adresse e-mail, prénom',
        basisHtml: 'Art. 6(1)(a), consentement',
      },
    ],
  },
  dataCategories: {
    title: '3. Catégories de données traitées',
    items: [
      '<strong class="text-white">Données d\'identité :</strong> prénom, nom de famille, adresse e-mail professionnelle',
      '<strong class="text-white">Données de compte :</strong> identifiant, mot de passe chiffré, clés API',
      '<strong class="text-white">Données de paiement :</strong> adresse de facturation, 4 derniers chiffres de la carte et date d\'expiration (les données complètes de carte sont stockées par Stripe, Hook0 n\'y a pas accès)',
      '<strong class="text-white">Données techniques :</strong> adresse IP, user-agent du navigateur, horodatages de connexion, journaux d\'erreurs',
      '<strong class="text-white">Données d\'usage :</strong> événements webhook envoyés et reçus, volume d\'appels API, métriques d\'utilisation des fonctionnalités',
      '<strong class="text-white">Communications :</strong> contenu des échanges de support, transcriptions de chat',
    ],
    note: 'Hook0 ne traite pas de catégories particulières de données (art. 9 RGPD) et ne procède à aucune prise de décision automatisée ni à aucun profilage produisant des effets juridiques ou similairement significatifs.',
  },
  subprocessors: {
    title: '4. Destinataires et sous-traitants',
    introHtml: 'Nous partageons vos données avec nos sous-traitants dans la stricte mesure nécessaire à la fourniture du Service. La liste complète et à jour est disponible sur <a href="./sous-traitants-rgpd" class="text-green-400 hover:text-green-300 transition-colors">/sous-traitants-rgpd</a>. Un résumé est présenté ci-dessous.',
    groups: [
      {
        title: 'Infrastructure',
        headers: ['Sous-traitant', 'Pays', 'Finalité'],
        rows: [
          ['Clever Cloud SAS', 'France (UE)', 'Hébergement base de données, API, application web'],
          ['Cloudflare, Inc.', 'États-Unis', 'DNS et protection DDoS'],
        ],
      },
      {
        title: 'Exploitation du service',
        headers: ['Sous-traitant', 'Pays', 'Finalité'],
        rows: [
          ['Clever Cloud SAS', 'France (UE)', 'Workers appelant les endpoints webhook'],
          ['Scaleway SAS', 'France (UE)', 'Workers dédiés privés (offres sélectionnées)'],
          ['Stripe, Inc.', 'États-Unis', 'Gestion des abonnements et paiements'],
          ['Brevo (Sendinblue)', 'France (UE)', 'Emails transactionnels automatisés'],
          ['Postmark (ActiveCampaign)', 'États-Unis', 'Emails transactionnels automatisés'],
          ['BetterUptime', 'République tchèque (UE)', 'Monitoring de disponibilité et page de statut'],
          ['Sentry, Inc.', 'États-Unis', 'Suivi des erreurs applicatives'],
          ['Crisp', 'France (UE)', 'Chat de support client (conditionné au consentement)'],
          ['Google LLC (Gmail)', 'États-Unis', 'Boîte email de support'],
        ],
      },
      {
        title: 'Mesure marketing (intérêt légitime, server-side)',
        headers: ['Sous-traitant', 'Pays', 'Finalité'],
        rows: [
          ['Google LLC (Google Ads)', 'États-Unis', 'Mesure de conversion côté serveur (gclid uniquement). Voir section 9b.'],
        ],
      },
      {
        title: 'Analytique (conditionné au consentement)',
        headers: ['Service', 'Pays', 'Finalité'],
        rows: [
          ['Matomo (auto-hébergé sur matomo.hook0.com)', 'France (UE)', 'Analytique du site web'],
        ],
      },
    ],
    note: 'Un contrat de traitement de données (DPA) est en vigueur avec chaque sous-traitant. Pour les transferts hors UE, voir la section 5.',
  },
  transfers: {
    title: '5. Transferts hors de l\'Union européenne',
    // [LEGAL-CORRECTION L828] Ajout du EU-US DPF en complément des CCT pour les
    // sous-traitants certifiés (Cloudflare, Stripe, Google LLC). L'extrait legacy
    // ne mentionnait que les CCT, ce qui est incomplet et fait reposer indûment
    // l'analyse sur la TIA alors que la certification DPF fournit déjà une voie
    // d'adéquation. Aligné sur gdpr-subprocessors.js + DPA section 6.
    p1Html: 'Plusieurs sous-traitants sont établis aux États-Unis : Cloudflare, Stripe, Postmark, Sentry, Gmail (Google) et Google Ads. Ces transferts sont encadrés par les <strong class="text-white">clauses contractuelles types (CCT)</strong> adoptées par la Commission européenne (décision 2021/914) et par une analyse d\'impact des transferts (TIA) documentée, ainsi que, le cas échéant, par le <strong class="text-white">EU-US Data Privacy Framework</strong> (Cloudflare, Stripe et Google LLC y sont certifiés). Ces mécanismes garantissent ensemble un niveau de protection adéquat pour les données personnelles.',
    cloudActHtml: '<strong>Note CLOUD Act :</strong> Les prestataires établis aux États-Unis peuvent être soumis au CLOUD Act (Clarifying Lawful Overseas Use of Data Act), qui peut contraindre ces prestataires à communiquer des données aux autorités américaines, y compris lorsque ces données sont hébergées hors des États-Unis. Hook0 applique un principe de minimisation des données et limite les données personnelles transmises à ses sous-traitants américains au strict nécessaire.',
  },
  retention: {
    title: '6. Durées de conservation',
    headers: ['Catégorie de données', 'Durée de conservation', 'Justification'],
    rows: [
      ['Données de compte', 'Durée du contrat + 30 jours après suppression du compte', 'Nécessité contractuelle ; période de grâce de 30 jours pour permettre l\'export des données'],
      ['Pièces comptables et factures', '10 ans à compter de la date de transaction', 'Obligation légale, Code général des impôts, art. L102 B du Livre des procédures fiscales'],
      // [ISMS-SYNC L860] Remplacement de l'intervalle « 7 à 30 jours selon le
      // plan d'abonnement » par le détail par plan issu de
      // information-retention-policy.md et de l'Annexe 2 du DPA.
      ['Journaux d\'événements webhook', 'Developer 7 jours, Startup 14 jours, Pro 30 jours, Enterprise sur mesure', 'Fourniture du service ; configurable selon l\'offre souscrite'],
      ['Données analytiques (Matomo)', '25 mois', 'Recommandation CNIL pour les données d\'analyse d\'audience'],
      ['Communications de support', '3 ans après le dernier échange', 'Intérêt légitime ; prescription de droit commun des actions contractuelles'],
      ['Preuves de consentement', '5 ans à compter de la date du consentement', 'Capacité à démontrer la conformité (art. 7(1) RGPD)'],
      // [ISMS-SYNC] Ajout des journaux serveur pour alignement avec l'Annexe 2
      // du DPA.
      ['Journaux serveur', '30 jours minimum, puis rotation et suppression automatiques', 'Exploitation du service, sécurité et gestion des incidents'],
    ],
  },
  rights: {
    title: '7. Vos droits',
    intro: 'Conformément au RGPD, vous disposez des droits suivants sur vos données personnelles :',
    items: [
      '<strong class="text-white">Droit d\'accès</strong> (art. 15), obtenir une copie des données personnelles que nous détenons vous concernant',
      '<strong class="text-white">Droit de rectification</strong> (art. 16), corriger des données inexactes ou incomplètes',
      '<strong class="text-white">Droit à l\'effacement</strong> (art. 17), demander la suppression de vos données, sous réserve des obligations légales de conservation',
      '<strong class="text-white">Droit à la limitation du traitement</strong> (art. 18), demander la restriction du traitement dans certaines circonstances',
      '<strong class="text-white">Droit à la portabilité</strong> (art. 20), recevoir vos données dans un format structuré et lisible par machine, lorsque le traitement est fondé sur le consentement ou sur un contrat',
      '<strong class="text-white">Droit d\'opposition</strong> (art. 21), vous opposer au traitement fondé sur l\'intérêt légitime ou à des fins de prospection commerciale',
      '<strong class="text-white">Droit de retrait du consentement</strong> (art. 7(3)), retirer votre consentement à tout moment, sans que cela n\'affecte la licéité des traitements antérieurs',
    ],
    contactHtml: 'Pour exercer l\'un de ces droits, adressez votre demande à <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>. Nous y répondrons dans un délai de 30 jours. Nous pourrons vous demander de justifier de votre identité avant de traiter votre demande.',
  },
  cnil: {
    title: '8. Droit de réclamation auprès de la CNIL',
    p1: 'Si vous estimez que le traitement de vos données personnelles constitue une violation du RGPD, vous avez le droit d\'introduire une réclamation auprès de l\'autorité de contrôle compétente :',
    addressHtml: '<strong class="text-white">Commission Nationale de l\'Informatique et des Libertés (CNIL)</strong><br>3 Place de Fontenoy, TSA 80715<br>75334 Paris Cedex 07<br>Site internet : <a href="https://www.cnil.fr" class="text-green-400 hover:text-green-300 transition-colors" target="_blank" rel="noopener">www.cnil.fr</a>',
    note: 'Vous pouvez également saisir l\'autorité de contrôle de votre État membre de résidence habituelle ou de lieu de travail au sein de l\'Union européenne.',
  },
  cookies: {
    title: '9. Cookies et traceurs',
    intro: 'Hook0 dispose d\'un mécanisme de gestion du consentement sur son site web. Les services suivants ne sont chargés qu\'après recueil de votre consentement explicite :',
    items: [
      '<strong class="text-white">Matomo Analytics</strong> (auto-hébergé), mesure d\'audience, anonymisée par défaut',
      '<strong class="text-white">Crisp</strong>, widget de chat en direct',
      '<strong class="text-white">Cookie hook0_gclid</strong> (Domain <code class="text-green-400">.hook0.com</code>, durée 30 jours), relaie l\'identifiant de clic Google Ads entre www.hook0.com et app.hook0.com pour permettre l\'attribution d\'une inscription différée. Posé uniquement après recueil du consentement et uniquement si vous arrivez depuis un clic publicitaire. Effacé au retrait du consentement. Voir Section 9b pour les détails.',
    ],
    consentScopeHtml: 'Votre consentement donné sur www.hook0.com s\'étend à l\'ensemble des sous-domaines hook0.com (y compris app.hook0.com). Les préférences de consentement sont stockées dans le <code class="text-green-400">localStorage</code> du navigateur pour une durée de <strong class="text-white">13 mois</strong>, conformément aux lignes directrices de la CNIL. Vous pouvez modifier vos préférences à tout moment :',
    changeButton: 'Modifier les paramètres de cookies',
  },
  serverSideTracking: {
    title: '9b. Mesure des conversions publicitaires (Google Ads, server-side)',
    intro: 'Lorsque vous accédez à notre service en cliquant sur une annonce Google Ads, Google Ads ajoute automatiquement un identifiant de clic publicitaire (« gclid ») à l\'URL de destination. Ce gclid est transmis à notre backend lors de la création de votre compte et envoyé côté serveur à Google Ads pour mesurer l\'efficacité de nos campagnes publicitaires.',
    items: [
      '<strong class="text-white">Finalité</strong> : mesurer le coût par acquisition de nos campagnes payantes afin d\'allouer notre budget marketing.',
      '<strong class="text-white">Base légale</strong> : art. 6(1)(f) RGPD, intérêt légitime. Test de proportionnalité documenté disponible sur demande.',
      '<strong class="text-white">Données transmises à Google</strong>, à savoir le gclid, le type de conversion et la date/heure de conversion. <strong>Aucun e-mail, aucune adresse IP, aucun User-Agent</strong> n\'est transmis à Google dans ce cadre.',
      '<strong class="text-white">Co-responsable de traitement</strong> : Google LLC, dans le cadre des Customer Data Processing Terms (art. 26 RGPD). Le transfert vers les États-Unis est encadré par les clauses contractuelles types (décision 2021/914) et, le cas échéant, par le EU-US Data Privacy Framework (Google LLC y est certifié).',
      '<strong class="text-white">Conservation</strong> : le gclid est traité en mémoire vive lors de la requête HTTP d\'inscription et n\'est pas conservé dans nos bases de données après transmission à Google Ads.',
      '<strong class="text-white">Droit d\'opposition</strong> au titre de l\'art. 21(2) RGPD. Vous pouvez vous opposer à ce traitement à tout moment en écrivant à <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>. Nous noterons cette opposition sur votre compte afin que votre gclid ne soit pas transmis à Google Ads. Votre inscription n\'en est pas affectée.',
    ],
    footnoteHtml: 'Note : cette mesure côté serveur ne repose <strong>pas</strong> sur des cookies, gtag.js, ni aucun traceur côté client. L\'article 82 de la loi Informatique et Libertés (transposition de l\'article 5(3) de la Directive e-Privacy) ne s\'applique pas à ce traitement.',
  },
  security: {
    title: '10. Sécurité',
    p1: 'Hook0 met en œuvre des mesures techniques et organisationnelles appropriées pour protéger les données personnelles contre la perte accidentelle, l\'accès non autorisé, la divulgation, l\'altération ou la destruction. Ces mesures comprennent le chiffrement en transit (TLS 1.2+), le chiffrement au repos, des contrôles d\'accès et des revues de sécurité régulières.',
    p2Html: 'Le détail de nos pratiques de sécurité est disponible sur notre <a href="./securite" class="text-green-400 hover:text-green-300 transition-colors">page Sécurité</a>.',
    p3Html: 'En cas de violation de données personnelles susceptible d\'engendrer un risque pour vos droits et libertés, nous notifierons la CNIL dans les 72 heures (art. 33 RGPD) et les personnes concernées dans les meilleurs délais lorsque cela est requis (art. 34 RGPD). Si vous constatez une fuite de données potentielle, signalez-la immédiatement à <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>.',
  },
  changes: {
    title: '11. Modifications de la présente politique',
    p1: 'Nous pouvons mettre à jour la présente politique de confidentialité ponctuellement. Toute mise à jour sera signalée par la modification de la date figurant en haut de cette page. En cas de changement substantiel, nous vous en informerons par e-mail à l\'adresse associée à votre compte, ou par un avis bien visible sur le site web, au moins 30 jours avant l\'entrée en vigueur de la modification.',
  },
};
