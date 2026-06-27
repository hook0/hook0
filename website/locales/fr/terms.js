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
  pageTitle: 'Hook0 - Conditions Generales d\'Utilisation',
  pageDescription: 'Conditions generales d\'utilisation de Hook0 Webhooks-as-a-Service. Lisez attentivement les regles qui encadrent l\'acces a la plateforme Hook0 et a ses services.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Mentions juridiques',
    title: 'Conditions Generales d\'Utilisation',
    subtitle: 'Lisez attentivement ces conditions avant d\'utiliser les services Hook0.',
    lastUpdatedLabel: 'Derniere mise a jour :',
    lastUpdatedDate: '27 juin 2026',
  },
  intro: {
    p1Html: 'Les presentes Conditions generales d\'utilisation (les « Conditions ») regissent votre acces a la plateforme Hook0 et aux services associes (collectivement, le « Service ») exploites par FGRibreau SARL, societe a responsabilite limitee de droit francais au capital de 2 000 EUR, immatriculee au Registre du commerce et des societes de La Roche-sur-Yon sous le numero 850 824 350, dont le siege social est situe au 3 rue de l\'Aubepine, 85110 Chantonnay, France, numero de TVA intracommunautaire FR27850824350 (« Hook0 », « nous » ou « notre »). Le directeur de la publication est David Sferruzza.',
    p2Html: 'Le Service est destine exclusivement aux entreprises et entites professionnelles (B2B). En vous inscrivant au Service, en y accedant ou en l\'utilisant, vous confirmez agir a titre professionnel pour le compte d\'une personne morale et disposer du pouvoir d\'engager cette personne morale au titre des presentes Conditions.',
    p3Html: 'EN VOUS INSCRIVANT AU SERVICE, EN Y ACCEDANT OU EN L\'UTILISANT, VOUS ACCEPTEZ D\'ETRE LIE PAR LES PRESENTES CONDITIONS. SI VOUS NE LES ACCEPTEZ PAS, VOUS NE DEVEZ NI ACCEDER AU SERVICE NI L\'UTILISER.',
    p4Html: 'Les conditions commerciales et de facturation (tarifs, facturation, modalites de paiement) figurent dans les <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Conditions Generales de Vente</a>, document distinct integre par renvoi au present contrat.',
  },
  sections: [
    {
      id: 'definitions',
      title: '1. Definitions',
      lead: 'Dans les presentes Conditions, les termes suivants ont la signification qui leur est donnee ci-apres :',
      items: [
        '<strong class="text-white">« Compte »</strong> designe le compte cree par vous pour acceder au Service et l\'utiliser.',
        '<strong class="text-white">« Contenu »</strong> designe toute donnee, information ou tout materiel que vous transmettez via le Service ou que vous y stockez, y compris les charges utiles de webhooks, les configurations et les identifiants d\'API.',
        '<strong class="text-white">« Documentation »</strong> designe la documentation technique et les guides utilisateurs mis a disposition par Hook0 sur <a href="https://documentation.hook0.com" class="text-green-400 hover:text-green-300 transition-colors">documentation.hook0.com</a>.',
        '<strong class="text-white">« Service »</strong> designe la plateforme de gestion de webhooks Hook0, y compris l\'ensemble des API, interfaces et services accessoires fournis par Hook0.',
        '<strong class="text-white">« Sous-traitant »</strong> designe tout sous-traitant ulterieur engage par Hook0 pour traiter votre Contenu dans le cadre du Service. La liste des sous-traitants ulterieurs est publiee sur <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a>.',
        '<strong class="text-white">« Plan d\'abonnement »</strong> designe le niveau de service que vous avez choisi (Developer, Startup, Pro ou Enterprise), tel que decrit sur la page tarifs de hook0.com.',
        '<strong class="text-white">« Utilisateur »</strong> designe toute personne physique qui accede au Service ou l\'utilise via votre Compte pour votre compte.',
        '<strong class="text-white">« vous » / « votre »</strong> designe la personne morale qui s\'est inscrite au Service ou qui l\'utilise, ainsi que tout Utilisateur agissant pour son compte.',
      ],
    },
    {
      id: 'acceptance',
      title: '2. Acceptation et perimetre',
      paragraphs: [
        '<strong class="text-white">2.1. Usage professionnel exclusif.</strong> Le Service est concu exclusivement pour un usage professionnel et commercial. Les presentes Conditions ne s\'appliquent pas aux consommateurs (personnes physiques agissant en dehors de toute activite commerciale ou professionnelle). Le Service etant propose dans un cadre strictement B2B, aucun droit de retractation au sens du droit de la consommation ne s\'applique. En acceptant les presentes Conditions, vous declarez et garantissez agir a titre professionnel.',
        '<strong class="text-white">2.2. Pouvoir d\'engager.</strong> Si vous acceptez les presentes Conditions pour le compte d\'une societe ou de toute autre personne morale, vous declarez et garantissez disposer du pouvoir juridique pour engager celle-ci. Dans ce cas, le terme « vous » designe cette entite.',
        '<strong class="text-white">2.3. Integralite de l\'accord.</strong> Les presentes Conditions, ensemble avec la <a href="/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Politique de confidentialite</a>, l\'<a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Avenant relatif au traitement des donnees (DPA)</a> et les <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Conditions Generales de Vente</a>, forment l\'integralite de l\'accord entre les parties relatif au Service et remplacent toute entente, declaration ou accord anterieur ou contemporain.',
      ],
    },
    {
      id: 'description',
      title: '3. Description du Service',
      paragraphs: [
        '<strong class="text-white">3.1. Presentation de la plateforme.</strong> Hook0 est une plateforme de gestion de webhooks qui permet aux entreprises d\'envoyer, de recevoir, de gerer et de superviser des evenements webhook. Le Service comprend l\'infrastructure de livraison de webhooks, la logique de relance, la journalisation des evenements et les outils developpeur associes.',
        '<strong class="text-white">3.2. Plans d\'abonnement.</strong> Le Service est disponible selon les Plans d\'abonnement suivants : Developer (gratuit), Startup, Pro et Enterprise. Les fonctionnalites, plafonds d\'usage et tarifs applicables a chaque plan sont decrits sur la page tarifs de hook0.com. Hook0 se reserve le droit de modifier les fonctionnalites associees a tout Plan d\'abonnement, y compris l\'arret de fonctionnalites disponibles sur le plan gratuit Developer, moyennant le preavis prevu a la section 13.',
        '<strong class="text-white">3.3. Mises a jour.</strong> Hook0 peut a tout moment mettre a jour, modifier ou interrompre des fonctionnalites du Service. Lorsqu\'un changement est important, Hook0 donne un preavis raisonnable. L\'utilisation continue du Service apres de telles modifications vaut acceptation du Service mis a jour.',
        '<strong class="text-white">3.4. Services et infrastructures tiers.</strong> Le Service s\'appuie sur des prestataires d\'infrastructure tiers, notamment l\'hebergement assure par Clever Cloud SAS (France) et la diffusion de contenu et la securite peripherique assurees par Cloudflare Inc. (Etats-Unis). La liste a jour des sous-traitants ulterieurs et leurs localisations est publiee sur <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a>. Le Service peut aussi s\'integrer a d\'autres services tiers ou comporter des liens vers ceux-ci, sur lesquels Hook0 n\'a pas de controle. Leur utilisation est regie par les conditions tierces applicables.',
      ],
    },
    {
      id: 'account',
      title: '4. Creation de Compte',
      paragraphs: [
        '<strong class="text-white">4.1. Informations exactes.</strong> Vous vous engagez a fournir des informations exactes, a jour et completes lors de la creation de votre Compte et a les maintenir a jour. Hook0 peut suspendre ou resilier votre Compte s\'il constate que les informations fournies sont fausses ou trompeuses.',
        '<strong class="text-white">4.2. Securite du Compte.</strong> Vous etes responsable de la confidentialite de vos identifiants de Compte et de l\'ensemble des activites realisees sous votre Compte. Vous vous engagez a notifier immediatement Hook0 a <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> si vous prenez connaissance d\'un acces ou d\'un usage non autorise de votre Compte.',
        '<strong class="text-white">4.3. Un seul compte par entite.</strong> Chaque personne morale ne peut detenir qu\'un seul Compte, sauf accord ecrit express de Hook0. La creation de comptes multiples destines a contourner des plafonds d\'usage ou des obligations de facturation est interdite.',
        '<strong class="text-white">4.4. Utilisateurs.</strong> Vous veillez a ce que l\'ensemble des Utilisateurs accedant au Service via votre Compte respectent les presentes Conditions. Vous demeurez pleinement responsable de leurs actes et omissions.',
      ],
    },
    {
      id: 'pricing',
      title: '5. Plans d\'abonnement et tarification',
      paragraphs: [
        '<strong class="text-white">5.1. Tarifs.</strong> Les tarifs en vigueur pour chaque Plan d\'abonnement sont publies sur hook0.com. Tous les prix sont indiques hors taxes (HT), hors TVA et toute autre taxe applicable.',
        '<strong class="text-white">5.2. Conditions commerciales.</strong> Les cycles de facturation, les modes de paiement, la facturation et les autres conditions commerciales sont regis par les <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Conditions Generales de Vente</a>.',
        '<strong class="text-white">5.3. Modifications tarifaires.</strong> Hook0 se reserve le droit de modifier les tarifs de tout Plan d\'abonnement. Toute hausse tarifaire est notifiee au moins 30 jours a l\'avance par courrier electronique a l\'adresse associee a votre Compte. Si vous n\'acceptez pas le nouveau tarif, vous pouvez resilier votre abonnement avant sa date d\'effet. L\'utilisation continue du Service apres la date d\'effet d\'une modification tarifaire vaut acceptation du nouveau tarif.',
        '<strong class="text-white">5.4. Resiliation et changement de plan.</strong> Vous pouvez resilier, monter ou descendre de Plan d\'abonnement a tout moment en contactant <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>. Les effets de la resiliation sur la facturation sont decrits dans les Conditions Generales de Vente.',
        '<strong class="text-white">5.5. Retard de paiement.</strong> Conformement a l\'article L441-10 du Code de commerce, toute facture impayee a son echeance entraine de plein droit des penalites de retard egales a trois fois le taux d\'interet legal publie par la Banque centrale europeenne, ainsi qu\'une indemnite forfaitaire pour frais de recouvrement de 40 EUR par facture, sans qu\'un rappel soit necessaire. Les frais de recouvrement superieurs a cette indemnite forfaitaire peuvent etre factures sur justificatif.',
      ],
    },
    {
      id: 'ip',
      title: '6. Propriete intellectuelle',
      paragraphs: [
        '<strong class="text-white">6.1. Propriete intellectuelle de Hook0.</strong> Hook0 et ses concedants detiennent l\'ensemble des droits de propriete intellectuelle sur le Service, y compris le logiciel, le code source, les interfaces, la Documentation, les marques, logos et habillages commerciaux. Aucune disposition des presentes Conditions ne vous confere de droit sur le Service autre que le droit limite de l\'utiliser conformement aux presentes Conditions.',
        '<strong class="text-white">6.2. Licence d\'utilisation du Service.</strong> Sous reserve du respect des presentes Conditions et du paiement des redevances applicables, Hook0 vous concede un droit limite, non exclusif, non transferable et non sous-licenciable d\'acceder au Service et de l\'utiliser pour vos besoins professionnels internes pendant la duree de votre abonnement.',
        '<strong class="text-white">6.3. Votre Contenu.</strong> Vous conservez l\'integralite des droits de propriete sur votre Contenu. En soumettant du Contenu via le Service, vous concedez a Hook0 une licence limitee, mondiale, pour traiter et stocker votre Contenu uniquement dans la mesure necessaire a la fourniture du Service. Hook0 n\'utilisera pas votre Contenu a d\'autres fins.',
        '<strong class="text-white">6.4. Retours et suggestions.</strong> Si vous communiquez des suggestions, commentaires ou autres retours sur le Service (les « Retours »), vous concedez a Hook0 une licence mondiale, perpetuelle, irrevocable et libre de redevance pour utiliser et integrer ces Retours au Service ou a tout autre produit ou service de Hook0, sans obligation de compensation ni d\'attribution.',
        '<strong class="text-white">6.5. Composants au code source ouvert.</strong> Le serveur Hook0 est publie sous licence Server Side Public License v1 (SSPL-1.0), une licence au code source ouvert. Certains autres composants du Service sont regis par des licences tierces distinctes, au code source ouvert ou non. Aucune disposition des presentes Conditions ne restreint les droits dont vous beneficiez au titre de ces licences, qui prevalent en cas de conflit.',
      ],
    },
    {
      id: 'obligations',
      title: '7. Obligations de l\'utilisateur',
      paragraphs: [
        '<strong class="text-white">7.1. Usage autorise.</strong> Vous vous engagez a utiliser le Service uniquement a des fins professionnelles licites et conformement aux presentes Conditions, au droit applicable et aux regles d\'usage publiees par Hook0.',
        '<strong class="text-white">7.2. Usage interdit.</strong> Sans limiter ce qui precede, vous vous engagez a ne pas :',
      ],
      prohibitedList: [
        '(a) utiliser le Service pour envoyer des communications commerciales non sollicitees (spam) ou pour faciliter le phishing, la fraude ou toute autre activite illegale ;',
        '(b) utiliser le Service d\'une maniere qui viole une loi ou un reglement applicable, notamment en matiere de protection des donnees et de la vie privee ;',
        '(c) tenter de retro-concevoir, decompiler, desassembler ou autrement deriver le code source d\'une partie quelconque du Service, sauf dans la mesure expressement permise par une regle d\'ordre public ou par une licence au code source ouvert applicable ;',
        '(d) contourner, desactiver ou perturber un dispositif de securite, un plafond d\'usage ou un controle d\'acces du Service ;',
        '(e) utiliser le Service pour transmettre des logiciels malveillants, virus ou tout autre code nuisible ou perturbateur ;',
        '(f) revendre, sous-licencier ou rendre le Service accessible a des tiers sans le consentement ecrit prealable de Hook0 ;',
        '(g) utiliser des moyens automatises pour acceder au Service ou en extraire le contenu, en dehors de l\'API officielle et conformement a la Documentation ;',
        '(h) realiser toute action imposant une charge deraisonnable ou disproportionnee a l\'infrastructure du Service.',
      ],
      paragraphsAfter: [
        '<strong class="text-white">7.3. Responsabilite quant au Contenu.</strong> Vous etes seul responsable de l\'ensemble du Contenu que vous transmettez via le Service ou que vous y stockez. Vous declarez et garantissez disposer de tous les droits necessaires a l\'utilisation de ce Contenu dans le cadre du Service et que votre Contenu ne porte atteinte a aucun droit de tiers.',
        '<strong class="text-white">7.4. Suspension.</strong> Hook0 se reserve le droit de suspendre votre acces au Service immediatement et sans preavis s\'il estime raisonnablement que votre usage viole les presentes Conditions ou represente un risque pour le Service ou des tiers. Hook0 vous notifie toute suspension de cette nature dans les meilleurs delais.',
      ],
    },
    {
      id: 'privacy',
      title: '8. Vie privee et protection des donnees',
      paragraphs: [
        '<strong class="text-white">8.1. Politique de confidentialite.</strong> La collecte et l\'utilisation par Hook0 de donnees a caractere personnel dans le cadre du Service sont regies par la <a href="/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Politique de confidentialite</a>, integree par renvoi aux presentes Conditions. Lisez-la attentivement.',
        '<strong class="text-white">8.2. Avenant relatif au traitement des donnees et sous-traitants ulterieurs.</strong> Lorsque Hook0 traite des donnees a caractere personnel pour votre compte en qualite de sous-traitant au sens du Reglement (UE) 2016/679 (RGPD), l\'<a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Avenant relatif au traitement des donnees (DPA)</a> s\'applique et est integre par renvoi aux presentes Conditions. La liste a jour des sous-traitants ulterieurs, y compris l\'hebergement (Clever Cloud SAS, France) et la diffusion de contenu et la securite peripherique (Cloudflare Inc., Etats-Unis), est publiee sur <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a>. Les transferts de donnees a caractere personnel hors de l\'Espace economique europeen sont encadres par les garanties appropriees decrites dans l\'Avenant relatif au traitement des donnees. Vous etes responsable de la conformite de votre utilisation du Service au droit de la protection des donnees applicable, notamment de l\'obtention des consentements necessaires des personnes concernees.',
        '<strong class="text-white">8.3. Vos obligations.</strong> Vous etes responsable de traitement pour toute donnee a caractere personnel contenue dans votre Contenu. Vous etes responsable de disposer d\'une base legale appropriee pour traiter ces donnees via le Service et de delivrer aux personnes concernees toute information requise.',
      ],
    },
    {
      id: 'confidentiality',
      title: '9. Confidentialite',
      paragraphs: [
        '<strong class="text-white">9.1. Definition.</strong> Les « Informations Confidentielles » designent toute information non publique divulguee par une partie (la « Partie Divulgatrice ») a l\'autre partie (la « Partie Receptrice ») et identifiee comme confidentielle ou dont le caractere confidentiel peut raisonnablement etre compris compte tenu de sa nature et des circonstances de sa communication. Pour Hook0, les Informations Confidentielles comprennent le code source du Service (a l\'exclusion des composants publies sous une licence au code source ouvert), son architecture, ses grilles tarifaires et ses strategies commerciales. Pour vous, elles comprennent votre Contenu et vos donnees professionnelles.',
        '<strong class="text-white">9.2. Obligations.</strong> Chaque partie s\'engage a : (a) conserver les Informations Confidentielles de l\'autre partie dans la plus stricte confidentialite ; (b) utiliser les Informations Confidentielles uniquement pour executer ses obligations ou exercer ses droits au titre des presentes Conditions ; et (c) ne pas divulguer les Informations Confidentielles a des tiers sans le consentement ecrit prealable de la Partie Divulgatrice, sauf a des salaries, prestataires ou conseils ayant un besoin d\'en connaitre et tenus a des obligations de confidentialite au moins aussi protectrices que celles des presentes.',
        '<strong class="text-white">9.3. Exceptions.</strong> Les obligations de confidentialite de la section 9.2 ne s\'appliquent pas aux informations qui : (a) sont ou deviennent accessibles au public sans faute de la Partie Receptrice ; (b) etaient deja connues de la Partie Receptrice avant leur communication ; (c) sont recues d\'un tiers sans restriction ; ou (d) doivent etre divulguees en vertu de la loi ou d\'une decision de justice, sous reserve que la Partie Receptrice en informe sans delai et par ecrit la Partie Divulgatrice (lorsque la loi le permet) et coopere a toute demarche visant a obtenir une ordonnance de protection.',
      ],
    },
    {
      id: 'warranties',
      title: '10. Exclusion de garanties',
      paragraphs: [
        '<strong class="text-white">10.1. Service « en l\'etat ».</strong> Le Service est fourni « en l\'etat » et « selon disponibilite ». Dans toute la mesure permise par la loi applicable, Hook0 exclut toute garantie expresse ou implicite, y compris notamment les garanties de qualite marchande, d\'adequation a un usage particulier et d\'absence de contrefacon.',
        '<strong class="text-white">10.2. Pas d\'engagement de niveau de service par defaut.</strong> Hook0 ne garantit pas que le Service sera ininterrompu, exempt d\'erreurs ou de vulnerabilites. Aucun engagement de disponibilite, de temps de reponse, de latence ou de support n\'est consenti par defaut. Les engagements de niveau de service, le cas echeant, figurent exclusivement dans un accord Enterprise ecrit distinct signe par Hook0. Les operations de maintenance, planifiees ou non, peuvent entrainer une indisponibilite temporaire. Hook0 fournit ses meilleurs efforts commerciaux pour annoncer a l\'avance les maintenances planifiees, dans la mesure du possible.',
        '<strong class="text-white">10.3. Absence de garantie de resultats.</strong> Hook0 ne garantit pas que le Service repondra a vos exigences specifiques ni que les resultats obtenus via le Service seront exacts, complets ou fiables.',
        '<strong class="text-white">10.4. Portee.</strong> Aucune disposition des presentes Conditions n\'exclut ni ne limite une garantie qui ne peut etre exclue ou limitee au titre d\'une regle imperative applicable, notamment du droit francais.',
      ],
    },
    {
      id: 'liability',
      title: '11. Limitation de responsabilite',
      paragraphs: [
        '<strong class="text-white">11.1. Plafond de responsabilite.</strong> Dans toute la mesure permise par la loi applicable, la responsabilite totale et cumulee de Hook0 envers vous au titre ou en lien avec les presentes Conditions ou le Service, qu\'elle soit contractuelle, delictuelle (y compris pour negligence) ou autre, n\'excedera pas le montant total des redevances payees par vous a Hook0 au cours des douze (12) mois precedant immediatement le fait generateur de la reclamation. Cette limitation s\'applique meme si Hook0 a ete avisee de la possibilite de tels dommages et constitue un element fondamental de l\'equilibre economique des presentes.',
        '<strong class="text-white">11.2. Exclusion des dommages indirects.</strong> Dans toute la mesure permise par la loi applicable, Hook0 n\'est pas responsable des dommages indirects, accessoires, speciaux, consecutifs ou punitifs, notamment perte de profits, de chiffre d\'affaires, de donnees ou de clientele, ou interruption d\'activite, decoulant ou lies au Service ou aux presentes Conditions, quel que soit le fondement juridique invoque. Conformement a l\'article 1231-3 du Code civil, la responsabilite de Hook0 pour des dommages qui ne sont pas la suite immediate et directe d\'un manquement de Hook0 est exclue dans toute la mesure permise par la loi.',
        '<strong class="text-white">11.3. Regles d\'ordre public.</strong> Aucune disposition des presentes Conditions ne limite ni n\'exclut la responsabilite qui ne peut l\'etre au titre d\'une regle imperative applicable, notamment la responsabilite pour atteinte a la vie ou a l\'integrite physique resultant d\'une negligence, ainsi que la responsabilite pour dol ou faute lourde.',
      ],
    },
    {
      id: 'term',
      title: '12. Duree et resiliation',
      paragraphs: [
        '<strong class="text-white">12.1. Duree.</strong> Les presentes Conditions prennent effet a la date a laquelle vous vous inscrivez pour la premiere fois au Service ou y accedez, et se poursuivent pour une duree indeterminee jusqu\'a leur resiliation conformement a la presente section 12.',
        '<strong class="text-white">12.2. Resiliation a votre initiative.</strong> Vous pouvez resilier votre abonnement et fermer votre Compte a tout moment en contactant <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>. La resiliation prend effet a la fin de la periode de facturation en cours, sauf accord contraire. Les redevances payees ne sont pas remboursables, sauf disposition contraire des Conditions Generales de Vente.',
        '<strong class="text-white">12.3. Resiliation a l\'initiative de Hook0 pour cause.</strong> Hook0 peut resilier votre acces au Service moyennant un preavis ecrit de 15 jours en cas de manquement substantiel aux presentes Conditions auquel vous n\'avez pas remedie pendant le preavis. Hook0 peut resilier immediatement et sans preavis en cas de manquement grave, par exemple lorsque le Service est utilise a des fins illicites, lorsque votre comportement represente un risque de securite immediat pour le Service ou des tiers, ou en cas de non-paiement de redevances apres rappel.',
        '<strong class="text-white">12.4. Resiliation a l\'initiative de Hook0 sans motif.</strong> Hook0 peut resilier les presentes Conditions pour tout motif moyennant un preavis ecrit de 30 jours. Dans ce cas, vous beneficierez d\'un remboursement au prorata des redevances prepayees couvrant la periode posterieure a la date d\'effet de la resiliation.',
        '<strong class="text-white">12.5. Effets de la resiliation.</strong> A la resiliation des presentes Conditions pour quelque motif que ce soit : (a) l\'ensemble des droits et licences qui vous sont concedes au titre des presentes prennent fin immediatement ; (b) vous devez cesser d\'utiliser le Service ; (c) Hook0 conserve votre Contenu pendant 30 jours apres la date d\'effet de la resiliation, periode durant laquelle vous pouvez demander un export de vos donnees en contactant <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> ; (d) a l\'issue de ce delai de 30 jours, Hook0 peut supprimer definitivement votre Contenu sans autre notification.',
        '<strong class="text-white">12.6. Survie.</strong> Les sections 1, 6.1, 6.4, 9, 10, 11, 12.5, 12.6 et 14 survivent a la resiliation des presentes Conditions.',
      ],
    },
    {
      id: 'modifications',
      title: '13. Modifications des presentes Conditions',
      paragraphs: [
        '<strong class="text-white">13.1. Notification des modifications.</strong> Hook0 peut modifier les presentes Conditions a tout moment. Lorsque les modifications sont substantielles, Hook0 donne un preavis d\'au moins 30 jours par courrier electronique a l\'adresse associee a votre Compte et par publication des Conditions mises a jour sur le Site, en indiquant la nouvelle date d\'effet.',
        '<strong class="text-white">13.2. Acceptation.</strong> L\'utilisation continue du Service apres la date d\'effet des Conditions modifiees vaut acceptation des modifications. Si vous n\'acceptez pas les Conditions modifiees, vous devez cesser d\'utiliser le Service et resilier votre abonnement avant cette date d\'effet.',
        '<strong class="text-white">13.3. Modifications mineures.</strong> Hook0 peut modifier les presentes Conditions sans preavis pour des changements purement administratifs (corrections typographiques, mise a jour des coordonnees) ou imposes par la loi. Ces changements sont indiques par la mise a jour de la date « Derniere mise a jour » en tete des presentes.',
      ],
    },
    {
      id: 'general',
      title: '14. Dispositions generales',
      paragraphs: [
        '<strong class="text-white">14.1. Droit applicable.</strong> Les presentes Conditions et tout litige ou differend qui en decoulerait ou s\'y rapporterait (y compris les litiges extracontractuels) sont regis par le droit francais. La Convention des Nations unies sur les contrats de vente internationale de marchandises (CVIM) ne s\'applique pas.',
        '<strong class="text-white">14.2. Juridiction.</strong> Conformement a l\'article 48 du Code de procedure civile, applicable entre commercants, les parties conviennent que tout litige relatif aux presentes Conditions releve de la competence exclusive des tribunaux de La Roche-sur-Yon, France, dont depend le siege de Hook0, sous reserve des regles imperatives de competence applicables.',
        '<strong class="text-white">14.3. Force majeure.</strong> Aucune partie ne sera responsable d\'un manquement ou d\'un retard d\'execution resultant de causes echappant raisonnablement a son controle, notamment cas fortuit, guerre, terrorisme, emeute, incendie, inondation, catastrophe naturelle, decision d\'une autorite publique, greves, lock-out, ou defaillance des reseaux ou infrastructures de telecommunication d\'un tiers. La partie affectee en informe sans delai l\'autre partie et fournit ses meilleurs efforts commerciaux pour reprendre l\'execution dans les meilleurs delais.',
        '<strong class="text-white">14.4. Cession.</strong> Vous ne pouvez ceder ou transferer aucun de vos droits ou obligations au titre des presentes sans le consentement ecrit prealable de Hook0. Hook0 peut ceder les presentes Conditions en tout ou partie, notamment dans le cadre d\'une fusion, d\'une acquisition ou de la cession de tout ou substantielle partie de ses actifs, moyennant notification ecrite a votre attention.',
        '<strong class="text-white">14.5. Divisibilite.</strong> Si une disposition des presentes Conditions est jugee invalide, illicite ou inopposable par une juridiction competente, elle sera limitee ou ecartee dans la mesure strictement necessaire, le reste des dispositions demeurant pleinement en vigueur.',
        '<strong class="text-white">14.6. Renonciation.</strong> Le defaut, pour l\'une ou l\'autre partie, d\'exiger l\'execution d\'un droit ou d\'une disposition des presentes Conditions ne saurait valoir renonciation a ce droit ou a cette disposition pour l\'avenir.',
        '<strong class="text-white">14.7. Relation entre les parties.</strong> Aucune disposition des presentes Conditions ne cree ni ne saurait etre interpretee comme creant un partenariat, une coentreprise, un mandat ou une relation de travail entre les parties. Chaque partie agit en qualite de prestataire independant.',
        '<strong class="text-white">14.8. Notifications.</strong> Les notifications juridiques a Hook0 doivent etre adressees par ecrit, par courrier electronique a <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a> ou par courrier postal a FGRibreau SARL, 3 rue de l\'Aubepine, 85110 Chantonnay, France. Hook0 peut vous notifier par courrier electronique a l\'adresse associee a votre Compte. Les notifications electroniques sont reputees recues le jour de leur transmission, sauf accuse de non-remise.',
      ],
    },
    {
      id: 'contact',
      title: '15. Contact',
      lead: 'Pour toute question, observation ou preoccupation relative aux presentes Conditions ou au Service, contactez-nous :',
      contactItems: [
        '<strong class="text-white">Questions juridiques :</strong> <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
        '<strong class="text-white">Support :</strong> <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>',
        '<strong class="text-white">Adresse postale :</strong> FGRibreau SARL, 3 rue de l\'Aubepine, 85110 Chantonnay, France',
      ],
    },
  ],
};
