// Per-page strings for terms (FR, Conditions Generales d'Utilisation / CGU).
//
// Registre : vouvoiement formel obligatoire (« vous » / « votre »), comme tout
// document contractuel. Pas de tutoiement (le tutoiement reste reserve aux
// pages marketing). /humanizer pro applique. Pas d'em-dash, pas de double tiret
// pivot, pas de point median.
//
// SSPL = « code source ouvert (SSPL-1.0) », jamais « open source » seul
// (rejet OSI, risque L121-1 C. conso).
//
// Faits durs Hook0 conserves verbatim entre locales : FGRibreau SARL,
// capital 2 000 EUR, RCS La Roche-sur-Yon 850 824 350, TVA FR27850824350,
// siege 3 rue de l'Aubepine 85110 Chantonnay, directeur de publication
// David Sferruzza, hebergement Clever Cloud SAS (France) + CDN Cloudflare
// Inc. (USA) divulgues.
module.exports = {
  pageTitle: 'Hook0 - Conditions Générales d\'Utilisation',
  pageDescription: 'Conditions générales d\'utilisation de Hook0 Webhooks-as-a-Service. Lisez attentivement les règles qui encadrent l\'accès à la plateforme Hook0 et à ses services.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Mentions juridiques',
    title: 'Conditions Générales d\'Utilisation',
    subtitle: 'Lisez attentivement ces conditions avant d\'utiliser les services Hook0.',
    lastUpdatedLabel: 'Dernière mise à jour :',
    lastUpdatedDate: '27 juin 2026',
  },
  intro: {
    p1Html: 'Les présentes Conditions générales d\'utilisation (les « Conditions ») régissent votre accès à la plateforme Hook0 et aux services associés (collectivement, le « Service ») exploités par FGRibreau SARL, société à responsabilité limitée de droit français au capital de 2 000 EUR, immatriculée au Registre du commerce et des sociétés de La Roche-sur-Yon sous le numéro 850 824 350, dont le siège social est situé au 3 rue de l\'Aubépine, 85110 Chantonnay, France, numéro de TVA intracommunautaire FR27850824350 (« Hook0 », « nous » ou « notre »). Le directeur de la publication est David Sferruzza.',
    p2Html: 'Le Service est destiné exclusivement aux entreprises et entités professionnelles (B2B). En vous inscrivant au Service, en y accédant ou en l\'utilisant, vous confirmez agir à titre professionnel pour le compte d\'une personne morale et disposer du pouvoir d\'engager cette personne morale au titre des présentes Conditions.',
    p3Html: 'EN VOUS INSCRIVANT AU SERVICE, EN Y ACCÉDANT OU EN L\'UTILISANT, VOUS ACCEPTEZ D\'ÊTRE LIÉ PAR LES PRÉSENTES CONDITIONS. SI VOUS NE LES ACCEPTEZ PAS, VOUS NE DEVEZ NI ACCÉDER AU SERVICE NI L\'UTILISER.',
    p4Html: 'Les conditions commerciales et de facturation (tarifs, facturation, modalités de paiement) figurent dans les <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Conditions Générales de Vente</a>, document distinct intégré par renvoi au présent contrat.',
  },
  sections: [
    {
      id: 'definitions',
      title: '1. Définitions',
      lead: 'Dans les présentes Conditions, les termes suivants ont la signification qui leur est donnée ci-après :',
      items: [
        '<strong class="text-white">« Compte »</strong> désigne le compte créé par vous pour accéder au Service et l\'utiliser.',
        '<strong class="text-white">« Contenu »</strong> désigne toute donnée, information ou tout matériel que vous transmettez via le Service ou que vous y stockez, y compris les charges utiles de webhooks, les configurations et les identifiants d\'API.',
        '<strong class="text-white">« Documentation »</strong> désigne la documentation technique et les guides utilisateurs mis à disposition par Hook0 sur <a href="https://documentation.hook0.com" class="text-green-400 hover:text-green-300 transition-colors">documentation.hook0.com</a>.',
        '<strong class="text-white">« Service »</strong> désigne la plateforme de gestion de webhooks Hook0, y compris l\'ensemble des API, interfaces et services accessoires fournis par Hook0.',
        '<strong class="text-white">« Sous-traitant »</strong> désigne tout sous-traitant ultérieur engagé par Hook0 pour traiter votre Contenu dans le cadre du Service. La liste des sous-traitants ultérieurs est publiée sur <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a>.',
        '<strong class="text-white">« Plan d\'abonnement »</strong> désigne le niveau de service que vous avez choisi (Developer, Startup, Pro ou Enterprise), tel que décrit sur la page tarifs de hook0.com.',
        '<strong class="text-white">« Utilisateur »</strong> désigne toute personne physique qui accède au Service ou l\'utilise via votre Compte pour votre compte.',
        '<strong class="text-white">« vous » / « votre »</strong> désigne la personne morale qui s\'est inscrite au Service ou qui l\'utilise, ainsi que tout Utilisateur agissant pour son compte.',
      ],
    },
    {
      id: 'acceptance',
      title: '2. Acceptation et périmètre',
      paragraphs: [
        '<strong class="text-white">2.1. Usage professionnel exclusif.</strong> Le Service est conçu exclusivement pour un usage professionnel et commercial. Les présentes Conditions ne s\'appliquent pas aux consommateurs (personnes physiques agissant en dehors de toute activité commerciale ou professionnelle). Le Service étant proposé dans un cadre strictement B2B, aucun droit de rétractation au sens du droit de la consommation ne s\'applique. En acceptant les présentes Conditions, vous déclarez et garantissez agir à titre professionnel.',
        '<strong class="text-white">2.2. Pouvoir d\'engager.</strong> Si vous acceptez les présentes Conditions pour le compte d\'une société ou de toute autre personne morale, vous déclarez et garantissez disposer du pouvoir juridique pour engager celle-ci. Dans ce cas, le terme « vous » désigne cette entité.',
        '<strong class="text-white">2.3. Intégralité de l\'accord.</strong> Les présentes Conditions, ensemble avec la <a href="/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Politique de confidentialité</a>, l\'<a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Avenant relatif au traitement des données (DPA)</a> et les <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Conditions Générales de Vente</a>, forment l\'intégralité de l\'accord entre les parties relatif au Service et remplacent toute entente, déclaration ou accord antérieur ou contemporain.',
      ],
    },
    {
      id: 'description',
      title: '3. Description du Service',
      paragraphs: [
        '<strong class="text-white">3.1. Présentation de la plateforme.</strong> Hook0 est une plateforme de gestion de webhooks qui permet aux entreprises d\'envoyer, de recevoir, de gérer et de superviser des événements webhook. Le Service comprend l\'infrastructure de livraison de webhooks, la logique de relance, la journalisation des événements et les outils développeur associés.',
        '<strong class="text-white">3.2. Plans d\'abonnement.</strong> Le Service est disponible selon les Plans d\'abonnement suivants : Developer (gratuit), Startup, Pro et Enterprise. Les fonctionnalités, plafonds d\'usage et tarifs applicables à chaque plan sont décrits sur la page tarifs de hook0.com. Hook0 se réserve le droit de modifier les fonctionnalités associées à tout Plan d\'abonnement, y compris l\'arrêt de fonctionnalités disponibles sur le plan gratuit Developer, moyennant le préavis prévu à la section 13.',
        '<strong class="text-white">3.3. Mises à jour.</strong> Hook0 peut à tout moment mettre à jour, modifier ou interrompre des fonctionnalités du Service. Lorsqu\'un changement est important, Hook0 donne un préavis raisonnable. L\'utilisation continue du Service après de telles modifications vaut acceptation du Service mis à jour.',
        '<strong class="text-white">3.4. Services et infrastructures tiers.</strong> Le Service s\'appuie sur des prestataires d\'infrastructure tiers, notamment l\'hébergement assuré par Clever Cloud SAS (France) et la diffusion de contenu et la sécurité périphérique assurées par Cloudflare Inc. (États-Unis). La liste à jour des sous-traitants ultérieurs et leurs localisations est publiée sur <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a>. Le Service peut aussi s\'intégrer à d\'autres services tiers ou comporter des liens vers ceux-ci, sur lesquels Hook0 n\'a pas de contrôle. Leur utilisation est régie par les conditions tierces applicables.',
      ],
    },
    {
      id: 'account',
      title: '4. Création de Compte',
      paragraphs: [
        '<strong class="text-white">4.1. Informations exactes.</strong> Vous vous engagez à fournir des informations exactes, à jour et complètes lors de la création de votre Compte et à les maintenir à jour. Hook0 peut suspendre ou résilier votre Compte s\'il constate que les informations fournies sont fausses ou trompeuses.',
        '<strong class="text-white">4.2. Sécurité du Compte.</strong> Vous êtes responsable de la confidentialité de vos identifiants de Compte et de l\'ensemble des activités réalisées sous votre Compte. Vous vous engagez à notifier immédiatement Hook0 à <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> si vous prenez connaissance d\'un accès ou d\'un usage non autorisé de votre Compte.',
        '<strong class="text-white">4.3. Un seul compte par entité.</strong> Chaque personne morale ne peut détenir qu\'un seul Compte, sauf accord écrit exprès de Hook0. La création de comptes multiples destinés à contourner des plafonds d\'usage ou des obligations de facturation est interdite.',
        '<strong class="text-white">4.4. Utilisateurs.</strong> Vous veillez à ce que l\'ensemble des Utilisateurs accédant au Service via votre Compte respectent les présentes Conditions. Vous demeurez pleinement responsable de leurs actes et omissions.',
      ],
    },
    {
      id: 'pricing',
      title: '5. Plans d\'abonnement et tarification',
      paragraphs: [
        '<strong class="text-white">5.1. Tarifs.</strong> Les tarifs en vigueur pour chaque Plan d\'abonnement sont publiés sur hook0.com. Tous les prix sont indiqués hors taxes (HT), hors TVA et toute autre taxe applicable.',
        '<strong class="text-white">5.2. Conditions commerciales.</strong> Les cycles de facturation, les modes de paiement, la facturation et les autres conditions commerciales sont régis par les <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Conditions Générales de Vente</a>.',
        '<strong class="text-white">5.3. Modifications tarifaires.</strong> Hook0 se réserve le droit de modifier les tarifs de tout Plan d\'abonnement. Toute hausse tarifaire est notifiée au moins 30 jours à l\'avance par courrier électronique à l\'adresse associée à votre Compte. Si vous n\'acceptez pas le nouveau tarif, vous pouvez résilier votre abonnement avant sa date d\'effet. L\'utilisation continue du Service après la date d\'effet d\'une modification tarifaire vaut acceptation du nouveau tarif.',
        '<strong class="text-white">5.4. Résiliation et changement de plan.</strong> Vous pouvez résilier, monter ou descendre de Plan d\'abonnement à tout moment en contactant <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>. Les effets de la résiliation sur la facturation sont décrits dans les Conditions Générales de Vente.',
        '<strong class="text-white">5.5. Retard de paiement.</strong> Conformément à l\'article L441-10 du Code de commerce, toute facture impayée à son échéance entraîne de plein droit des pénalités de retard égales à trois fois le taux d\'intérêt légal publié par la Banque centrale européenne, ainsi qu\'une indemnité forfaitaire pour frais de recouvrement de 40 EUR par facture, sans qu\'un rappel soit nécessaire. Les frais de recouvrement supérieurs à cette indemnité forfaitaire peuvent être facturés sur justificatif.',
      ],
    },
    {
      id: 'ip',
      title: '6. Propriété intellectuelle',
      paragraphs: [
        '<strong class="text-white">6.1. Propriété intellectuelle de Hook0.</strong> Hook0 et ses concédants détiennent l\'ensemble des droits de propriété intellectuelle sur le Service, y compris le logiciel, le code source, les interfaces, la Documentation, les marques, logos et habillages commerciaux. Aucune disposition des présentes Conditions ne vous confère de droit sur le Service autre que le droit limité de l\'utiliser conformément aux présentes Conditions.',
        '<strong class="text-white">6.2. Licence d\'utilisation du Service.</strong> Sous réserve du respect des présentes Conditions et du paiement des redevances applicables, Hook0 vous concède un droit limité, non exclusif, non transférable et non sous-licenciable d\'accéder au Service et de l\'utiliser pour vos besoins professionnels internes pendant la durée de votre abonnement.',
        '<strong class="text-white">6.3. Votre Contenu.</strong> Vous conservez l\'intégralité des droits de propriété sur votre Contenu. En soumettant du Contenu via le Service, vous concédez à Hook0 une licence limitée, mondiale, pour traiter et stocker votre Contenu uniquement dans la mesure nécessaire à la fourniture du Service. Hook0 n\'utilisera pas votre Contenu à d\'autres fins.',
        '<strong class="text-white">6.4. Retours et suggestions.</strong> Si vous communiquez des suggestions, commentaires ou autres retours sur le Service (les « Retours »), vous concédez à Hook0 une licence mondiale, perpétuelle, irrévocable et libre de redevance pour utiliser et intégrer ces Retours au Service ou à tout autre produit ou service de Hook0, sans obligation de compensation ni d\'attribution.',
        '<strong class="text-white">6.5. Composants au code source ouvert.</strong> Le serveur Hook0 est publié sous licence Server Side Public License v1 (SSPL-1.0), une licence au code source ouvert. Certains autres composants du Service sont régis par des licences tierces distinctes, au code source ouvert ou non. Aucune disposition des présentes Conditions ne restreint les droits dont vous bénéficiez au titre de ces licences, qui prévalent en cas de conflit.',
      ],
    },
    {
      id: 'obligations',
      title: '7. Obligations de l\'utilisateur',
      paragraphs: [
        '<strong class="text-white">7.1. Usage autorisé.</strong> Vous vous engagez à utiliser le Service uniquement à des fins professionnelles licites et conformément aux présentes Conditions, au droit applicable et aux règles d\'usage publiées par Hook0.',
        '<strong class="text-white">7.2. Usage interdit.</strong> Sans limiter ce qui précède, vous vous engagez à ne pas :',
      ],
      prohibitedList: [
        '(a) utiliser le Service pour envoyer des communications commerciales non sollicitées (spam) ou pour faciliter le phishing, la fraude ou toute autre activité illégale ;',
        '(b) utiliser le Service d\'une manière qui viole une loi ou un règlement applicable, notamment en matière de protection des données et de la vie privée ;',
        '(c) tenter de rétro-concevoir, décompiler, désassembler ou autrement dériver le code source d\'une partie quelconque du Service, sauf dans la mesure expressément permise par une règle d\'ordre public ou par une licence au code source ouvert applicable ;',
        '(d) contourner, désactiver ou perturber un dispositif de sécurité, un plafond d\'usage ou un contrôle d\'accès du Service ;',
        '(e) utiliser le Service pour transmettre des logiciels malveillants, virus ou tout autre code nuisible ou perturbateur ;',
        '(f) revendre, sous-licencier ou rendre le Service accessible à des tiers sans le consentement écrit préalable de Hook0 ;',
        '(g) utiliser des moyens automatisés pour accéder au Service ou en extraire le contenu, en dehors de l\'API officielle et conformément à la Documentation ;',
        '(h) réaliser toute action imposant une charge déraisonnable ou disproportionnée à l\'infrastructure du Service.',
      ],
      paragraphsAfter: [
        '<strong class="text-white">7.3. Responsabilité quant au Contenu.</strong> Vous êtes seul responsable de l\'ensemble du Contenu que vous transmettez via le Service ou que vous y stockez. Vous déclarez et garantissez disposer de tous les droits nécessaires à l\'utilisation de ce Contenu dans le cadre du Service et que votre Contenu ne porte atteinte à aucun droit de tiers.',
        '<strong class="text-white">7.4. Suspension.</strong> Hook0 se réserve le droit de suspendre votre accès au Service immédiatement et sans préavis s\'il estime raisonnablement que votre usage viole les présentes Conditions ou représente un risque pour le Service ou des tiers. Hook0 vous notifie toute suspension de cette nature dans les meilleurs délais.',
      ],
    },
    {
      id: 'privacy',
      title: '8. Vie privée et protection des données',
      paragraphs: [
        '<strong class="text-white">8.1. Politique de confidentialité.</strong> La collecte et l\'utilisation par Hook0 de données à caractère personnel dans le cadre du Service sont régies par la <a href="/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Politique de confidentialité</a>, intégrée par renvoi aux présentes Conditions. Lisez-la attentivement.',
        '<strong class="text-white">8.2. Avenant relatif au traitement des données et sous-traitants ultérieurs.</strong> Lorsque Hook0 traite des données à caractère personnel pour votre compte en qualité de sous-traitant au sens du Règlement (UE) 2016/679 (RGPD), l\'<a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Avenant relatif au traitement des données (DPA)</a> s\'applique et est intégré par renvoi aux présentes Conditions. La liste à jour des sous-traitants ultérieurs, y compris l\'hébergement (Clever Cloud SAS, France) et la diffusion de contenu et la sécurité périphérique (Cloudflare Inc., États-Unis), est publiée sur <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a>. Les transferts de données à caractère personnel hors de l\'Espace économique européen sont encadrés par les garanties appropriées décrites dans l\'Avenant relatif au traitement des données. Vous êtes responsable de la conformité de votre utilisation du Service au droit de la protection des données applicable, notamment de l\'obtention des consentements nécessaires des personnes concernées.',
        '<strong class="text-white">8.3. Vos obligations.</strong> Vous êtes responsable de traitement pour toute donnée à caractère personnel contenue dans votre Contenu. Vous êtes responsable de disposer d\'une base légale appropriée pour traiter ces données via le Service et de délivrer aux personnes concernées toute information requise.',
      ],
    },
    {
      id: 'confidentiality',
      title: '9. Confidentialité',
      paragraphs: [
        '<strong class="text-white">9.1. Définition.</strong> Les « Informations Confidentielles » désignent toute information non publique divulguée par une partie (la « Partie Divulgatrice ») à l\'autre partie (la « Partie Réceptrice ») et identifiée comme confidentielle ou dont le caractère confidentiel peut raisonnablement être compris compte tenu de sa nature et des circonstances de sa communication. Pour Hook0, les Informations Confidentielles comprennent le code source du Service (à l\'exclusion des composants publiés sous une licence au code source ouvert), son architecture, ses grilles tarifaires et ses stratégies commerciales. Pour vous, elles comprennent votre Contenu et vos données professionnelles.',
        '<strong class="text-white">9.2. Obligations.</strong> Chaque partie s\'engage à : (a) conserver les Informations Confidentielles de l\'autre partie dans la plus stricte confidentialité ; (b) utiliser les Informations Confidentielles uniquement pour exécuter ses obligations ou exercer ses droits au titre des présentes Conditions ; et (c) ne pas divulguer les Informations Confidentielles à des tiers sans le consentement écrit préalable de la Partie Divulgatrice, sauf à des salariés, prestataires ou conseils ayant un besoin d\'en connaître et tenus à des obligations de confidentialité au moins aussi protectrices que celles des présentes.',
        '<strong class="text-white">9.3. Exceptions.</strong> Les obligations de confidentialité de la section 9.2 ne s\'appliquent pas aux informations qui : (a) sont ou deviennent accessibles au public sans faute de la Partie Réceptrice ; (b) étaient déjà connues de la Partie Réceptrice avant leur communication ; (c) sont reçues d\'un tiers sans restriction ; ou (d) doivent être divulguées en vertu de la loi ou d\'une décision de justice, sous réserve que la Partie Réceptrice en informe sans délai et par écrit la Partie Divulgatrice (lorsque la loi le permet) et coopère à toute démarche visant à obtenir une ordonnance de protection.',
      ],
    },
    {
      id: 'warranties',
      title: '10. Exclusion de garanties',
      paragraphs: [
        '<strong class="text-white">10.1. Service « en l\'état ».</strong> Le Service est fourni « en l\'état » et « selon disponibilité ». Dans toute la mesure permise par la loi applicable, Hook0 exclut toute garantie expresse ou implicite, y compris notamment les garanties de qualité marchande, d\'adéquation à un usage particulier et d\'absence de contrefaçon.',
        '<strong class="text-white">10.2. Pas d\'engagement de niveau de service par défaut.</strong> Hook0 ne garantit pas que le Service sera ininterrompu, exempt d\'erreurs ou de vulnérabilités. Aucun engagement de disponibilité, de temps de réponse, de latence ou de support n\'est consenti par défaut. Les engagements de niveau de service, le cas échéant, figurent exclusivement dans un accord Enterprise écrit distinct signé par Hook0. Les opérations de maintenance, planifiées ou non, peuvent entraîner une indisponibilité temporaire. Hook0 fournit ses meilleurs efforts commerciaux pour annoncer à l\'avance les maintenances planifiées, dans la mesure du possible.',
        '<strong class="text-white">10.3. Absence de garantie de résultats.</strong> Hook0 ne garantit pas que le Service répondra à vos exigences spécifiques ni que les résultats obtenus via le Service seront exacts, complets ou fiables.',
        '<strong class="text-white">10.4. Portée.</strong> Aucune disposition des présentes Conditions n\'exclut ni ne limite une garantie qui ne peut être exclue ou limitée au titre d\'une règle impérative applicable, notamment du droit français.',
      ],
    },
    {
      id: 'liability',
      title: '11. Limitation de responsabilité',
      paragraphs: [
        '<strong class="text-white">11.1. Plafond de responsabilité.</strong> Dans toute la mesure permise par la loi applicable, la responsabilité totale et cumulée de Hook0 envers vous au titre ou en lien avec les présentes Conditions ou le Service, qu\'elle soit contractuelle, délictuelle (y compris pour négligence) ou autre, n\'excédera pas le montant total des redevances payées par vous à Hook0 au cours des douze (12) mois précédant immédiatement le fait générateur de la réclamation. Cette limitation s\'applique même si Hook0 a été avisée de la possibilité de tels dommages et constitue un élément fondamental de l\'équilibre économique des présentes.',
        '<strong class="text-white">11.2. Exclusion des dommages indirects.</strong> Dans toute la mesure permise par la loi applicable, Hook0 n\'est pas responsable des dommages indirects, accessoires, spéciaux, consécutifs ou punitifs, notamment perte de profits, de chiffre d\'affaires, de données ou de clientèle, ou interruption d\'activité, découlant ou liés au Service ou aux présentes Conditions, quel que soit le fondement juridique invoqué. Conformément à l\'article 1231-3 du Code civil, la responsabilité de Hook0 pour des dommages qui ne sont pas la suite immédiate et directe d\'un manquement de Hook0 est exclue dans toute la mesure permise par la loi.',
        '<strong class="text-white">11.3. Règles d\'ordre public.</strong> Aucune disposition des présentes Conditions ne limite ni n\'exclut la responsabilité qui ne peut l\'être au titre d\'une règle impérative applicable, notamment la responsabilité pour atteinte à la vie ou à l\'intégrité physique résultant d\'une négligence, ainsi que la responsabilité pour dol ou faute lourde.',
      ],
    },
    {
      id: 'term',
      title: '12. Durée et résiliation',
      paragraphs: [
        '<strong class="text-white">12.1. Durée.</strong> Les présentes Conditions prennent effet à la date à laquelle vous vous inscrivez pour la première fois au Service ou y accédez, et se poursuivent pour une durée indéterminée jusqu\'à leur résiliation conformément à la présente section 12.',
        '<strong class="text-white">12.2. Résiliation à votre initiative.</strong> Vous pouvez résilier votre abonnement et fermer votre Compte à tout moment en contactant <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>. La résiliation prend effet à la fin de la période de facturation en cours, sauf accord contraire. Les redevances payées ne sont pas remboursables, sauf disposition contraire des Conditions Générales de Vente.',
        '<strong class="text-white">12.3. Résiliation à l\'initiative de Hook0 pour cause.</strong> Hook0 peut résilier votre accès au Service moyennant un préavis écrit de 15 jours en cas de manquement substantiel aux présentes Conditions auquel vous n\'avez pas remédié pendant le préavis. Hook0 peut résilier immédiatement et sans préavis en cas de manquement grave, par exemple lorsque le Service est utilisé à des fins illicites, lorsque votre comportement représente un risque de sécurité immédiat pour le Service ou des tiers, ou en cas de non-paiement de redevances après rappel.',
        '<strong class="text-white">12.4. Résiliation à l\'initiative de Hook0 sans motif.</strong> Hook0 peut résilier les présentes Conditions pour tout motif moyennant un préavis écrit de 30 jours. Dans ce cas, vous bénéficierez d\'un remboursement au prorata des redevances prépayées couvrant la période postérieure à la date d\'effet de la résiliation.',
        '<strong class="text-white">12.5. Effets de la résiliation.</strong> À la résiliation des présentes Conditions pour quelque motif que ce soit : (a) l\'ensemble des droits et licences qui vous sont concédés au titre des présentes prennent fin immédiatement ; (b) vous devez cesser d\'utiliser le Service ; (c) Hook0 conserve votre Contenu pendant 30 jours après la date d\'effet de la résiliation, période durant laquelle vous pouvez demander un export de vos données en contactant <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> ; (d) à l\'issue de ce délai de 30 jours, Hook0 peut supprimer définitivement votre Contenu sans autre notification.',
        '<strong class="text-white">12.6. Survie.</strong> Les sections 1, 6.1, 6.4, 9, 10, 11, 12.5, 12.6 et 14 survivent à la résiliation des présentes Conditions.',
      ],
    },
    {
      id: 'modifications',
      title: '13. Modifications des présentes Conditions',
      paragraphs: [
        '<strong class="text-white">13.1. Notification des modifications.</strong> Hook0 peut modifier les présentes Conditions à tout moment. Lorsque les modifications sont substantielles, Hook0 donne un préavis d\'au moins 30 jours par courrier électronique à l\'adresse associée à votre Compte et par publication des Conditions mises à jour sur le Site, en indiquant la nouvelle date d\'effet.',
        '<strong class="text-white">13.2. Acceptation.</strong> L\'utilisation continue du Service après la date d\'effet des Conditions modifiées vaut acceptation des modifications. Si vous n\'acceptez pas les Conditions modifiées, vous devez cesser d\'utiliser le Service et résilier votre abonnement avant cette date d\'effet.',
        '<strong class="text-white">13.3. Modifications mineures.</strong> Hook0 peut modifier les présentes Conditions sans préavis pour des changements purement administratifs (corrections typographiques, mise à jour des coordonnées) ou imposés par la loi. Ces changements sont indiqués par la mise à jour de la date « Dernière mise à jour » en tête des présentes.',
      ],
    },
    {
      id: 'general',
      title: '14. Dispositions générales',
      paragraphs: [
        '<strong class="text-white">14.1. Droit applicable.</strong> Les présentes Conditions et tout litige ou différend qui en découlerait ou s\'y rapporterait (y compris les litiges extracontractuels) sont régis par le droit français. La Convention des Nations unies sur les contrats de vente internationale de marchandises (CVIM) ne s\'applique pas.',
        '<strong class="text-white">14.2. Juridiction.</strong> Conformément à l\'article 48 du Code de procédure civile, applicable entre commerçants, les parties conviennent que tout litige relatif aux présentes Conditions relève de la compétence exclusive des tribunaux de La Roche-sur-Yon, France, dont dépend le siège de Hook0, sous réserve des règles impératives de compétence applicables.',
        '<strong class="text-white">14.3. Force majeure.</strong> Aucune partie ne sera responsable d\'un manquement ou d\'un retard d\'exécution résultant de causes échappant raisonnablement à son contrôle, notamment cas fortuit, guerre, terrorisme, émeute, incendie, inondation, catastrophe naturelle, décision d\'une autorité publique, grèves, lock-out, ou défaillance des réseaux ou infrastructures de télécommunication d\'un tiers. La partie affectée en informe sans délai l\'autre partie et fournit ses meilleurs efforts commerciaux pour reprendre l\'exécution dans les meilleurs délais.',
        '<strong class="text-white">14.4. Cession.</strong> Vous ne pouvez céder ou transférer aucun de vos droits ou obligations au titre des présentes sans le consentement écrit préalable de Hook0. Hook0 peut céder les présentes Conditions en tout ou partie, notamment dans le cadre d\'une fusion, d\'une acquisition ou de la cession de tout ou substantielle partie de ses actifs, moyennant notification écrite à votre attention.',
        '<strong class="text-white">14.5. Divisibilité.</strong> Si une disposition des présentes Conditions est jugée invalide, illicite ou inopposable par une juridiction compétente, elle sera limitée ou écartée dans la mesure strictement nécessaire, le reste des dispositions demeurant pleinement en vigueur.',
        '<strong class="text-white">14.6. Renonciation.</strong> Le défaut, pour l\'une ou l\'autre partie, d\'exiger l\'exécution d\'un droit ou d\'une disposition des présentes Conditions ne saurait valoir renonciation à ce droit ou à cette disposition pour l\'avenir.',
        '<strong class="text-white">14.7. Relation entre les parties.</strong> Aucune disposition des présentes Conditions ne crée ni ne saurait être interprétée comme créant un partenariat, une coentreprise, un mandat ou une relation de travail entre les parties. Chaque partie agit en qualité de prestataire indépendant.',
        '<strong class="text-white">14.8. Notifications.</strong> Les notifications juridiques à Hook0 doivent être adressées par écrit, par courrier électronique à <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a> ou par courrier postal à FGRibreau SARL, 3 rue de l\'Aubépine, 85110 Chantonnay, France. Hook0 peut vous notifier par courrier électronique à l\'adresse associée à votre Compte. Les notifications électroniques sont réputées reçues le jour de leur transmission, sauf accusé de non-remise.',
      ],
    },
    {
      id: 'contact',
      title: '15. Contact',
      lead: 'Pour toute question, observation ou préoccupation relative aux présentes Conditions ou au Service, contactez-nous :',
      contactItems: [
        '<strong class="text-white">Questions juridiques :</strong> <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
        '<strong class="text-white">Support :</strong> <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>',
        '<strong class="text-white">Adresse postale :</strong> FGRibreau SARL, 3 rue de l\'Aubépine, 85110 Chantonnay, France',
      ],
    },
  ],
};
