// Per-page strings for data-processing-addendum (FR, DPA art. 28 RGPD).
//
// Registre : vouvoiement formel strict, comme tout document juridique. Pas de
// tutoiement. /humanizer pro appliqué. Pas d'em-dash, pas de double tiret
// pivot, pas de point médian. Sane defaults : pièces juridiques + factuelles,
// ton sobre.
//
// Faits durs Hook0 verbatim entre locales (CLAUDE.md / CLAUDE.local.md) :
//   - Sous-traitant (Processor) : FGRibreau SARL, capital 2 000 EUR,
//     RCS La Roche-sur-Yon 850 824 350, TVA FR27850824350, siège social
//     3 rue de l'Aubépine, 85110 Chantonnay, France.
//   - Responsable de traitement (Controller) : le Client.
//   - Sous-traitants ultérieurs divulgués à l'Annexe 1 (en cohérence avec la
//     page dédiée sous-traitants RGPD) :
//       * Clever Cloud SAS (France) : plan de données principal
//       * Cloudflare, Inc. (USA, 101 Townsend St, San Francisco, CA 94107) :
//         CDN et protection anti-DDoS, encadrés par SCC 2021 + TIA / EU-US DPF
//   - Notification de violation : 72 heures (RGPD art. 33/34).
//   - Sauvegardes : quotidiennes, conservation 30 jours.
//   - Hachage des mots de passe : Argon2.
//   - MFA : appliquée pour les accès à l'infrastructure (Clever Cloud, GitLab,
//     Stripe) ; pas encore pour les comptes clients, prévue pour une version
//     ultérieure.
//   - Pas de claim « 100 % souverain » ni « aucun partage de données »
//     (L121-1 C. conso). RGPD = claim de processus, jamais absolu.
module.exports = {
  pageTitle: 'Hook0 - Accord de traitement des données (DPA)',
  pageDescription: 'Accord de traitement des données Hook0 (DPA) couvrant la conformité RGPD, les opérations de traitement, les mesures de sécurité et la gestion des sous-traitants ultérieurs.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Mentions légales',
    title: 'Accord de traitement des données',
    subtitle: 'Notre engagement à protéger vos données au titre du RGPD et de la réglementation sur la protection des données.',
    lastUpdatedLabel: 'Dernière mise à jour :',
    lastUpdatedDate: '27 juin 2026',
  },
  preamble: {
    title: 'Conditions de traitement des données',
    partiesHtml: 'Le présent accord de traitement des données (le « <strong>DPA</strong> ») est conclu entre le Client, agissant en qualité de responsable de traitement, et la société <strong>FGRibreau SARL</strong>, société à responsabilité limitée de droit français au capital social de 2 000 EUR, immatriculée au registre du commerce et des sociétés de La Roche-sur-Yon sous le numéro 850 824 350, dont le siège social est situé 3 rue de l\'Aubépine, 85110 Chantonnay, France, agissant en qualité de sous-traitant (désignée « <strong>Hook0</strong> » ou le « <strong>Sous-traitant</strong> »).',
    p1: 'Le présent DPA a pour objet de refléter l\'accord des parties relatif au traitement des données à caractère personnel, conformément aux exigences de la réglementation sur la protection des données.',
    p2: 'S\'agissant du traitement des données à caractère personnel du Client par Hook0 au titre des conditions d\'utilisation, les parties reconnaissent que le Client est le responsable de traitement et Hook0 le sous-traitant, et conviennent toutes deux de respecter l\'ensemble des obligations correspondantes prévues par la réglementation sur la protection des données.',
    p3: 'Le Client donne instruction à Hook0 de traiter ces données à caractère personnel pour son compte, dans la mesure où cela est nécessaire à l\'exécution des conditions d\'utilisation et tel que défini à l\'Annexe 1 « Description du traitement des données à caractère personnel ». L\'Annexe 1 est renseignée par le Client et doit être mise à jour à chaque évolution décidée par le Client.',
  },
  section1: {
    title: '1. Conformité à la réglementation sur la protection des données',
    p1: 'Chaque partie respecte les obligations qui lui incombent au titre de la réglementation sur la protection des données.',
    p2: 'Tous les termes commençant par une majuscule dans le DPA ont le sens qui leur est donné par le RGPD, par la réglementation sur la protection des données et par les conditions d\'utilisation.',
  },
  section2: {
    title: '2. Opérations de traitement réalisées au titre du DPA',
    p1: 'À titre de rappel, pour chaque traitement réalisé au titre du présent DPA, le Client doit :',
    items: [
      'documenter les instructions relatives aux données à caractère personnel,',
      'fournir les informations relatives au traitement permettant de renseigner l\'Annexe 1 en contactant Hook0 à l\'adresse de support : <a href="mailto:support@hook0.com">support@hook0.com</a>.',
    ],
    p2: 'Le Client garantit à Hook0 qu\'il est en droit de transférer les données à caractère personnel à Hook0 ou à ses sous-traitants ultérieurs, dans le respect de la réglementation sur la protection des données, y compris, le cas échéant, l\'accomplissement des formalités préalables requises et le respect des droits des personnes concernées, tels que l\'information ou le recueil du consentement lorsque la réglementation l\'exige.',
    p3: 'Le Client reconnaît qu\'il demeure seul responsable de la détermination des finalités et des moyens du traitement des données à caractère personnel par Hook0. Le responsable de traitement reste seul responsable de l\'exactitude et de l\'adéquation desdites instructions. Toute modification des instructions données ou des mesures de sécurité requises par le Client, y compris pour se conformer à la législation applicable en matière de protection des données, doit être convenue entre les parties ou faire l\'objet d\'un avenant au présent DPA. Les coûts engagés par Hook0 pour se conformer à ces modifications sont à la charge du Client.',
    p4: 'Le Client s\'engage à ce que les personnes concernées aient été informées, ou le soient, préalablement au transfert de leurs données à caractère personnel à Hook0 dans le cadre des services.',
    p5: 'Le produit n\'a pas vocation à traiter des catégories particulières de données à caractère personnel. Le Client s\'engage par conséquent à empêcher tout traitement de catégories particulières de données à caractère personnel via le produit et les services. Toutefois, à la demande du Client, le traitement de catégories particulières de données à caractère personnel pourra être réalisé par Hook0. Dans ce cas, le traitement fera l\'objet d\'un avenant spécifique au DPA conclu entre le Client et Hook0.',
    p6: 'Lorsque le Client sollicite expressément l\'assistance de Hook0 pour l\'accomplissement de ses obligations au titre de la réglementation sur la protection des données, Hook0 lui adresse l\'estimation des coûts liés à cette assistance. Après acceptation expresse de cette estimation, Hook0 apporte son assistance conformément aux instructions du Client et aux stipulations du présent DPA.',
  },
  section3: {
    title: '3. Périmètre et instructions',
    p1: 'Hook0 s\'engage à :',
    items: [
      'ne traiter les données à caractère personnel du Client communiquées par celui-ci, ainsi que celles collectées ou produites pendant la durée des conditions d\'utilisation, qu\'aux seules fins de l\'exécution de ses obligations au titre des conditions d\'utilisation et conformément aux instructions documentées du Client, sauf obligation contraire imposée par la réglementation sur la protection des données ;',
      'veiller à ce que toute personne agissant sous son autorité et ayant accès aux données à caractère personnel du Client communiquées par celui-ci, ainsi qu\'à celles collectées ou produites pendant la durée des conditions d\'utilisation, ne traite ces données qu\'aux seules fins de l\'exécution des obligations de Hook0 au titre des conditions d\'utilisation et sur instruction du Client, sauf obligation contraire imposée par la réglementation sur la protection des données ;',
      'ne pas utiliser les données à caractère personnel du Client à des fins détournées, frauduleuses ou personnelles, y compris à des fins commerciales ;',
      'informer immédiatement le Client si, selon elle, une instruction du Client constitue une violation de la réglementation sur la protection des données.',
    ],
  },
  section4: {
    title: '4. Communication des données à caractère personnel du Client à des tiers',
    p1: 'Les données à caractère personnel du Client traitées au titre du DPA ne peuvent faire l\'objet d\'aucune cession, location, concession, communication ou divulgation à un tiers, y compris aux sous-traitants ultérieurs de Hook0, sauf si les conditions d\'utilisation ou une disposition légale ou réglementaire impérative le prévoient.',
    p2: 'Dans une telle hypothèse, Hook0 informe le Client de cette exigence légale avant le traitement, sauf si la disposition légale ou réglementaire impérative interdit cette information pour des motifs importants d\'intérêt public.',
  },
  section5: {
    title: '5. Sous-traitance ultérieure',
    p1Html: 'Dans les conditions visées aux paragraphes 2 et 4 de l\'article 28 du RGPD pour le recours à un autre sous-traitant (le « <strong>Sous-traitant ultérieur</strong> »), le Client accepte que Hook0 puisse confier à des sous-traitants ultérieurs le traitement de ses données à caractère personnel.',
    p2Html: 'Nonobstant l\'autorisation générale donnée par le Client, Hook0 informe le Client de toute modification envisagée concernant l\'ajout ou le remplacement d\'un sous-traitant ultérieur, dans un délai raisonnable avant la mise en œuvre de cette modification, en lui laissant un délai raisonnable pour formuler une objection avant la prise d\'effet du changement. La liste des sous-traitants ultérieurs intervenant sous l\'autorité de Hook0 est mise à la disposition du Client à l\'adresse <a href="./gdpr-subprocessors">Hook0 / Sous-traitants RGPD</a>.',
    p3: 'Lorsque Hook0 fait appel à un sous-traitant ultérieur pour le traitement des données à caractère personnel du Client, Hook0 impose à ce sous-traitant ultérieur les mêmes obligations de protection des données que celles prévues par le DPA.',
    p4: 'Ce contrat doit en particulier prévoir l\'obligation pour le sous-traitant ultérieur de présenter des garanties suffisantes quant à la mise en œuvre de mesures techniques et organisationnelles appropriées, de manière à ce que le traitement réponde aux exigences de la réglementation sur la protection des données et du DPA.',
  },
  section6: {
    title: '6. Transfert des données à caractère personnel du Client en dehors de l\'Espace économique européen (EEE)',
    p1Html: 'Hook0 garantit que le plan de données des webhooks (charges utiles des webhooks du Client, base de données et sauvegardes applicatives) est localisé en France ou au sein de l\'Espace économique européen (EEE). La couche périphérique accessoire (CDN et protection anti-DDoS assurés par Cloudflare, Inc.) implique des transferts vers les États-Unis, encadrés par les Clauses Contractuelles Types 2021 adoptées par la Commission européenne et par une analyse d\'impact des transferts (TIA) documentée, ainsi que, le cas échéant, par le EU-US Data Privacy Framework. La liste complète des sous-traitants ultérieurs et les mécanismes de transfert applicables sont tenus à jour à l\'adresse <a href="./gdpr-subprocessors">Hook0 / Sous-traitants RGPD</a>.',
    p2Html: 'À la demande du Client et sur instruction de celui-ci, Hook0 pourra stocker ou transférer des données à caractère personnel vers d\'autres entités de Hook0 ou vers des sous-traitants ultérieurs situés dans des pays hors de l\'EEE (les « Pays tiers »). Dans ce cas, et lorsque les Pays tiers n\'ont pas fait l\'objet d\'une décision d\'adéquation de la Commission européenne, Hook0 s\'engage à ce que le transfert soit réalisé conformément à la réglementation sur la protection des données et soit assorti de garanties appropriées permettant de garantir un niveau de protection équivalent à celui de la réglementation sur la protection des données, telles que la signature des Clauses Contractuelles Types adoptées par la Commission européenne et accessibles à l\'adresse <a href="https://commission.europa.eu/law/law-topic/data-protection/international-dimension-data-protection/standard-contractual-clauses-scc_en">commission.europa.eu</a>.',
    p3: 'Par les présentes, le Client mandate Hook0 pour signer en son nom et pour son compte les Clauses Contractuelles Types avec les entités de Hook0 et les sous-traitants ultérieurs situés dans des Pays tiers.',
    p4: 'À la demande du Client, Hook0 accepte d\'assister le Client dans la réalisation d\'une analyse d\'impact des transferts visant à identifier les écarts entre la réglementation sur la protection des données et la législation du Pays tiers, et à mettre en place les mesures complémentaires nécessaires pour garantir un niveau de protection équivalent à celui de la réglementation sur la protection des données.',
  },
  section7: {
    title: '7. Mesures de sécurité et confidentialité du traitement',
    p1: 'Hook0 prend, dans la mesure où cela est pertinent au regard de la fourniture des services ou du respect de ses autres obligations au titre du DPA, les mesures appropriées pour garantir un niveau de sécurité des données à caractère personnel du Client adapté au risque, et prend en compte les principes de protection des données dès la conception et par défaut dans l\'exécution du DPA.',
    p2: 'Hook0 s\'engage à :',
    items: [
      'mettre en œuvre toutes les mesures techniques et organisationnelles appropriées pour protéger les données à caractère personnel contre la destruction accidentelle ou illicite, la perte, l\'altération, la divulgation non autorisée ou l\'accès non autorisé aux données à caractère personnel transmises, conservées ou autrement traitées, et notamment l\'ensemble des mesures mentionnées à l\'Annexe 2 ;',
      'respecter toutes les instructions communiquées par le Client en matière de sécurité et de confidentialité raisonnablement applicables ;',
      'ne rendre les données à caractère personnel du Client accessibles et consultables qu\'aux seules personnes dûment habilitées ;',
      'garantir la confidentialité des données à caractère personnel du Client traitées au titre du DPA, et veiller à ce que toutes les personnes autorisées à traiter les données à caractère personnel du Client sous l\'autorité de Hook0 (y compris les salariés et les sous-traitants ultérieurs) s\'engagent à respecter la confidentialité de ces données ou soient soumises à une obligation légale appropriée de confidentialité.',
    ],
  },
  section8: {
    title: '8. Notification des violations de données à caractère personnel',
    p1Html: 'Hook0 notifie au Client toute violation de données à caractère personnel sans retard injustifié, et en tout état de cause dans un délai maximal de <strong>72 heures</strong> après en avoir pris connaissance, conformément à l\'article 33 du RGPD, et par écrit après avoir pris connaissance d\'une violation de données à caractère personnel. Lorsque l\'information est disponible chez Hook0, cette notification :',
    items: [
      'décrit la nature de la violation de données à caractère personnel, y compris, dans la mesure du possible, les catégories et le nombre approximatif des personnes concernées ainsi que les catégories et le nombre approximatif des données à caractère personnel concernées ;',
      'communique le nom et les coordonnées du contact « vie privée » (<a href="mailto:legal@hook0.com">legal@hook0.com</a>) ou de tout autre point de contact auprès duquel des informations complémentaires peuvent être obtenues ;',
      'décrit les conséquences probables de la violation de données à caractère personnel ;',
      'décrit les mesures prises ou que Hook0 propose de prendre pour remédier à la violation de données à caractère personnel, y compris, le cas échéant, les mesures destinées à en atténuer les éventuelles conséquences négatives.',
    ],
    p2: 'Lorsqu\'il n\'est pas possible de fournir l\'ensemble des informations en même temps, ces dernières peuvent être communiquées de manière échelonnée, sans retard injustifié supplémentaire.',
    p3: 'À la demande du Client, Hook0 s\'engage également à fournir au Client une assistance et une coopération raisonnables pour notifier la violation de données à caractère personnel à l\'autorité de contrôle compétente et pour communiquer cette violation aux personnes concernées au titre de l\'article 34 du RGPD, dans le respect de la réglementation sur la protection des données.',
  },
  section9: {
    title: '9. Droits des personnes concernées',
    p1: 'Au regard de la nature des opérations de traitement des données à caractère personnel, Hook0 s\'engage à :',
    items: [
      'notifier rapidement au Client toute demande ou plainte reçue relative à la protection des données à caractère personnel du Client ;',
      'à la demande du Client, lui apporter une assistance et une coopération raisonnables, afin de lui permettre de répondre (i) aux demandes des personnes concernées au titre de l\'exercice de leurs droits (droit d\'accès, droits de rectification, d\'effacement, de limitation, de portabilité et d\'opposition), ou (ii) aux demandes des autorités de protection des données compétentes ou aux demandes du délégué à la protection des données du Client ; en particulier, mettre en œuvre les mesures techniques et organisationnelles appropriées pour permettre au Client de répondre rapidement et par écrit à toute demande d\'information ;',
      'fournir aux personnes concernées les informations adéquates sur les opérations de traitement de données à caractère personnel réalisées les concernant au titre des conditions d\'utilisation, à la demande et aux frais du Client.',
    ],
  },
  section10: {
    title: '10. Analyse d\'impact relative à la protection des données',
    p1: 'À la demande du Client, Hook0 s\'engage à lui apporter une assistance et une coopération raisonnables pour réaliser une analyse d\'impact des opérations de traitement de données à caractère personnel effectuées au titre du présent DPA sur la protection des données à caractère personnel, et pour consulter les autorités de protection des données compétentes lorsque cela est nécessaire, aux frais du Client (selon un tarif au temps passé).',
  },
  section11: {
    title: '11. Conservation, restitution ou destruction des données à caractère personnel',
    p1: 'Le Client reste seul responsable de la mise en œuvre et de la gestion des durées de conservation des données à caractère personnel, et s\'engage à utiliser le produit en conséquence.',
    p2: 'Sans préjudice des lois et règlements applicables, Hook0 s\'engage, au terme des conditions d\'utilisation, à :',
    items: [
      'restituer ou détruire, à la demande du Client, l\'ensemble des données à caractère personnel du Client, de manière automatisée ou manuelle, selon les procédures et prescriptions préalablement convenues entre les parties ;',
      'supprimer toutes les copies existantes des données à caractère personnel, sauf et dans la mesure où Hook0 est tenue de conserver des copies des données à caractère personnel conformément à la législation applicable (notamment les pièces de facturation et de comptabilité, conservées 10 ans en application de la législation fiscale française) ;',
      'certifier par écrit la destruction des données à caractère personnel.',
    ],
  },
  section12: {
    title: '12. Documentation et audit',
    p1: 'Sur préavis écrit de trente (30) jours ouvrés adressé par le Client, Hook0 communique au Client les informations strictement nécessaires pour démontrer le respect des obligations prévues par les présentes conditions d\'utilisation.',
    p2: 'À la demande du Client et une fois par an, Hook0 s\'engage à permettre la réalisation d\'audits raisonnables, y compris des inspections, menés par le Client ou par un tiers mandaté par celui-ci, et à y contribuer, afin d\'évaluer la conformité de Hook0 à la réglementation sur la protection des données et aux stipulations du DPA.',
    p3: 'Hook0 s\'engage également à permettre la réalisation d\'audits menés par les autorités de protection des données compétentes et à y contribuer.',
    p4: 'Le Client ne dispose d\'aucun droit pour consulter ou accéder aux systèmes, données, dossiers ou autres informations relatifs ou afférents aux autres clients de Hook0.',
    p5: 'Tout audit conduit par le Client ou pour son compte est réalisé à ses frais. Le Client remet à Hook0 une copie du rapport d\'audit.',
    p6: 'Si le Client fait l\'objet d\'une enquête ou d\'une demande d\'information d\'une autorité de protection des données compétente concernant l\'une des opérations de traitement réalisées par Hook0 pour son compte, il s\'engage à en informer Hook0 dans les meilleurs délais et à satisfaire à cette enquête ou demande dans la mesure de ses moyens, à ses frais, et conformément aux procédures adoptées par l\'autorité de protection des données.',
    p7: 'Le Client s\'engage à respecter toutes les stipulations de confidentialité, les politiques ou les règles applicables sur site que Hook0 pourra lui communiquer dans le cadre de l\'audit.',
  },
  appendix1: {
    title: 'Annexe 1 - Activités de traitement des données à caractère personnel réalisées par Hook0 pour le compte du Client',
    rows: [
      {
        label: 'Responsable de traitement',
        valueHtml: 'Le Client (tel qu\'identifié dans les conditions d\'utilisation).',
      },
      {
        label: 'Sous-traitant',
        valueHtml: 'FGRibreau SARL, société à responsabilité limitée de droit français au capital social de 2 000 EUR, immatriculée au RCS de La Roche-sur-Yon sous le numéro 850 824 350, TVA FR27850824350, dont le siège social est situé 3 rue de l\'Aubépine, 85110 Chantonnay, France.',
      },
      {
        label: 'Nature des opérations de traitement',
        valueHtml: '<ul><li>Réception, stockage et transmission d\'événements webhook pour le compte du Client</li><li>Gestion des relances en cas d\'échec de livraison des webhooks</li><li>Journalisation et supervision des tentatives de livraison des webhooks</li><li>Authentification des utilisateurs et gestion des accès à la plateforme Hook0</li><li>Gestion de la facturation et des abonnements (via Stripe)</li></ul>',
      },
      {
        label: 'Finalité(s) du traitement',
        valueHtml: 'Fourniture de la plateforme webhook-as-a-service Hook0 telle que décrite dans les conditions d\'utilisation.',
      },
      {
        label: 'Nom et coordonnées du délégué à la protection des données du Client (le cas échéant)',
        valueHtml: '<em>[à compléter par le Client]</em>',
      },
      {
        label: 'Catégorie(s) de données à caractère personnel',
        valueHtml: 'Adresses e-mail, noms, adresses IP, contenu des charges utiles webhook (déterminé par le Client), jetons d\'authentification, informations de facturation (traitées par Stripe).<br><br><strong>Données sensibles :</strong> aucune par défaut. Il appartient au Client de veiller à ce que les charges utiles webhook ne contiennent pas de catégories particulières de données, sauf accord écrit contraire.<br><br>À la demande du Client, le traitement de catégories particulières de données à caractère personnel peut être réalisé par Hook0. Dans ce cas, le traitement fera l\'objet d\'un avenant spécifique au DPA conclu entre le Client et Hook0.',
      },
      {
        label: 'Catégorie(s) de personnes concernées',
        valueHtml: 'Utilisateurs finaux du Client dont les données sont transmises via les webhooks ; utilisateurs habilités du Client accédant à la plateforme Hook0.',
      },
      {
        label: 'Lieu(x) des opérations de traitement',
        valueHtml: 'Plan de données des webhooks : France / EEE.<br>CDN et protection anti-DDoS : États-Unis (Cloudflare, Inc.), encadrés par SCC 2021 + TIA et, le cas échéant, par le EU-US Data Privacy Framework.<br><br>Si le Client demande que les données à caractère personnel soient localisées hors EEE, ce traitement fera l\'objet d\'un accord distinct entre le Client et Hook0.<br><br>Voir : <a href="./gdpr-subprocessors">Hook0 / Sous-traitants RGPD</a>',
      },
      {
        label: 'Identité du ou des sous-traitants ultérieurs',
        valueHtml: 'Voir : <a href="./gdpr-subprocessors">Hook0 / Sous-traitants RGPD</a>',
      },
      {
        label: 'Fréquence du traitement',
        valueHtml: 'Traitement continu et automatisé.',
      },
      {
        label: 'Durée des opérations de traitement',
        valueHtml: 'Pendant la durée des conditions d\'utilisation, plus 30 jours après la suppression du compte (données de compte). La durée de conservation des événements webhook dépend du plan souscrit par le Client : 7 jours en Developer, 14 jours en Startup, 30 jours en Pro, durée personnalisée en Enterprise. Les pièces de facturation sont conservées 10 ans en application de la législation fiscale française.',
      },
    ],
  },
  appendix2: {
    title: 'Annexe 2 - Mesures techniques et organisationnelles appropriées mises en œuvre',
    intro: 'Les mesures techniques et organisationnelles suivantes sont mises en œuvre par Hook0 pour protéger les données à caractère personnel contre la destruction accidentelle ou illicite, la perte, l\'altération, la divulgation non autorisée ou l\'accès non autorisé aux données à caractère personnel transmises, conservées ou autrement traitées :',
    groups: [
      {
        title: 'Sécurité de l\'infrastructure (gérée par Clever Cloud SAS)',
        items: [
          'Application hébergée sur l\'infrastructure de Clever Cloud SAS en France (UE) ;',
          'Chiffrement de la base de données au repos (géré par Clever Cloud) ;',
          'Chiffrement TLS 1.2+ pour toutes les données en transit (rustls, avec support de la cryptographie post-quantique) ;',
          'Sauvegardes automatiques quotidiennes avec une rétention de 30 jours, stockées sur un système distribué multi-régions (compatible S3) sur Clever Cloud fr-par ; l\'intégrité des sauvegardes est vérifiée et la restauration est testée mensuellement ;',
          'CDN et protection anti-DDoS assurés par Cloudflare, Inc. (États-Unis), encadrés par les SCC 2021 + TIA et, le cas échéant, par le EU-US Data Privacy Framework ;',
          'Le contrôle d\'accès physique aux installations des centres de données est confié à Clever Cloud SAS, conformément à son programme de sécurité documenté.',
        ],
      },
      {
        title: 'Sécurité applicative',
        items: [
          'Hachage des mots de passe avec Argon2 (fonction memory-hard, résistante aux attaques par GPU et ASIC ; sel aléatoire unique par mot de passe ; jamais conservé en clair ni avec un chiffrement réversible) ;',
          'Jetons d\'autorisation basés sur les capabilities (Biscuit) ;',
          'Contrôle d\'accès basé sur les rôles (RBAC) pour l\'accès à la plateforme ;',
          'Expiration automatique des sessions.',
        ],
        noteHtml: '<strong>Note sur l\'authentification multi-facteur (MFA) :</strong> la MFA est appliquée pour tous les accès à l\'infrastructure (Clever Cloud, GitLab, Stripe). La MFA pour les comptes utilisateurs individuels de Hook0 n\'est pas encore implémentée au niveau applicatif et est prévue pour une version ultérieure. En attendant que la MFA côté client soit disponible, les exigences fortes de mot de passe (hachage Argon2, complexité minimale) et l\'expiration des sessions assurent une protection de base.',
      },
      {
        title: 'Sécurité du développement',
        items: [
          'Toute modification du code requiert une revue par un pair via une merge request ;',
          'Pipeline CI/CD automatisé incluant :<ul><li>Tests de sécurité statiques (SAST, modèle GitLab) ;</li><li>Tests de sécurité dynamiques (DAST, modèle GitLab) ;</li><li>Analyse des conteneurs et du système de fichiers (Trivy) ;</li><li>Analyse des vulnérabilités des dépendances (osv-scanner) ;</li><li>Détection de secrets (modèle GitLab).</li></ul>',
          'Linting strict du code (Clippy avec les warnings traités comme des erreurs) et formatage homogène (cargo fmt --check), appliqués en CI.',
        ],
      },
      {
        title: 'Supervision et réponse aux incidents',
        items: [
          'Suivi des erreurs via Sentry ;',
          'Traçage distribué via OpenTelemetry (export OTLP) ;',
          'Supervision de disponibilité via BetterUptime, avec page de statut publique ;',
          'Notification des violations de données à caractère personnel sous 72 heures conformément à l\'article 33 du RGPD (voir section 8) ;',
          'Politique de divulgation responsable avec signalement sécurisé par PGP.',
        ],
      },
      {
        title: 'Mesures organisationnelles',
        items: [
          'Politique de classification de l\'information (Public, Interne, Confidentiel, Sensible) ;',
          'Engagements de confidentialité (NDA) requis pour l\'ensemble du personnel ;',
          'Pratiques de sensibilisation à la sécurité ;',
          'Principe du besoin d\'en connaître appliqué aux accès ;',
          'MFA appliquée pour les accès à l\'infrastructure (Clever Cloud, GitLab, Stripe) ;',
          'Tests d\'intrusion menés annuellement ou après chaque évolution majeure de l\'architecture.',
        ],
      },
      {
        title: 'Durées de conservation',
        items: [
          'Événements webhook, selon le plan du Client, 7 jours en Developer, 14 jours en Startup, 30 jours en Pro, durée personnalisée en Enterprise ;',
          'Données de compte (identifiant, e-mail, mot de passe haché, clés API), conservées pendant la durée du contrat de service plus 30 jours après la suppression du compte ;',
          'Pièces de facturation et de comptabilité, conservées 10 ans (législation fiscale française, art. L102 B du Livre des procédures fiscales) ;',
          'Journaux serveur, 30 jours minimum, puis rotation et suppression automatiques ;',
          'Communications de support, 3 ans à compter du dernier échange (délai de prescription contractuelle de droit commun).',
        ],
      },
    ],
  },
};
