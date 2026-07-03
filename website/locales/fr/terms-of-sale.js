// Per-page strings for terms-of-sale (FR, Conditions Generales de Vente / CGV B2B).
//
// Registre : vouvoiement formel obligatoire (« vous » / « votre »), comme tout
// document contractuel commercial. Pas de tutoiement. /humanizer pro applique.
// Pas d'em-dash, pas de double tiret pivot, pas de point median.
//
// SSPL = « code source ouvert (SSPL-1.0) », jamais « open source » seul
// (rejet OSI, risque L121-1 C. conso).
//
// Faits durs Hook0 conserves verbatim entre locales : FGRibreau SARL,
// capital 2 000 EUR, RCS La Roche-sur-Yon 850 824 350, TVA FR27850824350,
// siege 3 rue de l'Aubepine 85110 Chantonnay, directeur de publication
// David Sferruzza, hebergement Clever Cloud SAS (France) + CDN Cloudflare
// Inc. (USA) divulgues, juridiction tribunaux de La Roche-sur-Yon (art. 48 CPC),
// retard de paiement L441-10 (3x taux BCE) + indemnite forfaitaire 40 EUR (D441-5).
module.exports = {
  pageTitle: 'Hook0 - Conditions Générales de Vente',
  pageDescription: 'Conditions générales de vente de Hook0 Webhooks-as-a-Service. Tarifs, modalités de paiement, facturation et résiliation pour les offres Cloud et On-Premise.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Mentions juridiques',
    title: 'Conditions Générales de Vente',
    subtitle: 'Conditions commerciales applicables à la souscription des offres et services Hook0.',
    lastUpdatedLabel: 'Dernière mise à jour :',
    lastUpdatedDate: '27 juin 2026',
  },
  intro: {
    p1Html: 'Les présentes Conditions générales de vente régissent toute commande et tout abonnement souscrit auprès de FGRibreau SARL, société à responsabilité limitée de droit français au capital de 2 000 EUR, immatriculée au Registre du commerce et des sociétés de La Roche-sur-Yon sous le numéro 850 824 350, dont le siège social est situé au 3 rue de l\'Aubépine, 85110 Chantonnay, France, numéro de TVA intracommunautaire FR27850824350 (ci-après « Hook0 » ou « nous »), pour l\'accès à la plateforme Hook0 et aux services associés. Le directeur de la publication est David Sferruzza.',
    p2Html: 'Les présentes Conditions générales de vente s\'appliquent exclusivement aux transactions entre professionnels (B2B). Elles sont intégrées par renvoi aux <a href="/terms" class="text-green-400 hover:text-green-300 transition-colors">Conditions Générales d\'Utilisation</a> et les complètent. En cas de contradiction entre les présentes Conditions générales de vente et les Conditions Générales d\'Utilisation, les présentes Conditions générales de vente prévalent sur les questions commerciales et de facturation.',
    p3Html: 'En passant commande ou en activant un abonnement payant, le client accepte expressément les présentes Conditions générales de vente dans leur intégralité.',
  },
  sections: [
    {
      id: 'scope',
      title: '1. Champ d\'application',
      paragraphs: [
        '<strong class="text-white">1.1.</strong> Les présentes Conditions générales de vente s\'appliquent à toute souscription aux offres Hook0 Cloud (Developer, Startup, Pro, Enterprise) et aux offres On-Premise (Self-hosted, Pro, Enterprise), quel que soit le canal de passation de la commande.',
        '<strong class="text-white">1.2.</strong> Elles s\'appliquent exclusivement aux clients professionnels (entreprises, associations, entités publiques). Elles ne s\'appliquent pas aux consommateurs au sens du Code de la consommation.',
        '<strong class="text-white">1.3.</strong> Toute condition générale d\'achat du client est expressément exclue et reste sans effet, même si elle est communiquée à Hook0 postérieurement à l\'acceptation des présentes, sauf acceptation écrite expresse de Hook0.',
      ],
    },
    {
      id: 'pricing',
      title: '2. Tarifs',
      paragraphs: [
        '<strong class="text-white">2.1.</strong> Tous les prix sont indiqués hors taxes (HT). La TVA applicable ou les taxes équivalentes sont ajoutées automatiquement lors de la facturation, conformément à la législation applicable et selon le pays d\'établissement du client.',
        '<strong class="text-white">2.2.</strong> Les tarifs en vigueur pour l\'ensemble des offres sont publiés sur <a href="/pricing" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/pricing</a> et sont intégrés par renvoi aux présentes. Les tarifs publiés peuvent être modifiés dans les conditions prévues à la section 9.',
        '<strong class="text-white">2.3. Offres Cloud</strong>, à titre indicatif à la date de la dernière mise à jour :',
      ],
      cloudPlans: [
        '<strong class="text-white">Developer</strong> : gratuit, 100 événements/jour, rétention 7 jours.',
        '<strong class="text-white">Startup</strong> : 59 EUR/mois HT, 30 000 événements/jour ; dépassement facturé 0,003 EUR par événement.',
        '<strong class="text-white">Pro</strong> : 190 EUR/mois ou 1 824 EUR/an HT, 100 000 événements/jour ; dépassement facturé 0,0001 EUR par événement.',
        '<strong class="text-white">Enterprise</strong> : devis sur mesure.',
      ],
      paragraphsBeforeOnPremise: [
        '<strong class="text-white">2.4. Offres On-Premise</strong>, à titre indicatif à la date de la dernière mise à jour :',
      ],
      onPremisePlans: [
        '<strong class="text-white">Self-hosted</strong> : gratuit, code source ouvert sous licence Server Side Public License v1 (SSPL-1.0).',
        '<strong class="text-white">Pro</strong> : 1 000 EUR de frais de mise en service + 500 EUR/mois ou 6 000 EUR/an HT.',
        '<strong class="text-white">Enterprise</strong> : devis sur mesure.',
      ],
      paragraphsAfter: [
        '<strong class="text-white">2.5.</strong> Les prix indiqués aux sections 2.3 et 2.4 le sont à titre purement informatif. Les prix opposables sont ceux affichés sur <a href="/pricing" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/pricing</a> au moment de la commande ou ceux fixés dans un devis spécifique pour les clients Enterprise.',
      ],
    },
    {
      id: 'ordering',
      title: '3. Commande et abonnement',
      paragraphs: [
        '<strong class="text-white">3.1.</strong> Les abonnements aux offres en libre-service (Developer, Startup, Pro) sont souscrits directement via l\'application Hook0 sur <a href="https://app.hook0.com" class="text-green-400 hover:text-green-300 transition-colors">app.hook0.com</a>. Le client choisit l\'offre souhaitée et renseigne un moyen de paiement valide.',
        '<strong class="text-white">3.2.</strong> Les abonnements Enterprise et On-Premise Pro font l\'objet d\'un devis sur mesure. Le client peut solliciter un devis auprès de <a href="mailto:sales@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">sales@hook0.com</a>. Le contrat est formé lors de l\'acceptation écrite du devis par les deux parties.',
        '<strong class="text-white">3.3.</strong> Pour les offres en libre-service, le contrat est formé lorsque le client confirme la commande dans l\'application et que Hook0 active l\'offre sélectionnée.',
      ],
    },
    {
      id: 'invoicing',
      title: '4. Facturation',
      paragraphs: [
        '<strong class="text-white">4.1.</strong> La facturation des offres en libre-service est gérée automatiquement via Stripe, le prestataire de paiement de Hook0. Les factures sont émises au début de chaque période de facturation (mensuelle ou annuelle, selon l\'offre choisie) et sont disponibles dans le portail de facturation Stripe du client.',
        '<strong class="text-white">4.2.</strong> Pour les offres Enterprise et On-Premise Pro, les factures sont émises directement par Hook0 et adressées à l\'adresse de facturation communiquée par le client au moment de la commande.',
        '<strong class="text-white">4.3.</strong> Les abonnements annuels sont facturés en totalité en début d\'année d\'abonnement.',
        '<strong class="text-white">4.4.</strong> Les frais de dépassement, le cas échéant, sont calculés à la fin de chaque période de facturation et facturés sur la facture de la période suivante ou séparément, au choix de Hook0.',
      ],
    },
    {
      id: 'payment',
      title: '5. Modalités de paiement',
      paragraphs: [
        '<strong class="text-white">5.1. Offres en libre-service (Developer, Startup, Pro).</strong> Le paiement est exigible dès l\'émission de la facture par prélèvement automatique sur la carte enregistrée dans Stripe. En souscrivant, le client autorise Hook0 à prélever périodiquement le moyen de paiement enregistré selon le cycle de facturation choisi.',
        '<strong class="text-white">5.2. Offres Enterprise et On-Premise Pro.</strong> Sauf stipulation contraire dans le devis ou le bon de commande applicable, les factures sont payables à trente (30) jours date de facture.',
        '<strong class="text-white">5.3.</strong> Tous les paiements sont effectués en euros (EUR). Les frais bancaires de transfert et les coûts de conversion éventuels sont à la charge du client.',
      ],
    },
    {
      id: 'late-payment',
      title: '6. Retard de paiement',
      paragraphs: [
        '<strong class="text-white">6.1.</strong> Conformément à l\'article L441-10 du Code de commerce, toute somme non réglée à son échéance entraîne de plein droit, sans rappel préalable, des pénalités de retard calculées à un taux égal à trois (3) fois le taux d\'intérêt légal publié par la Banque centrale européenne en vigueur, appliqué au montant impayé depuis la date d\'échéance jusqu\'à la date de paiement effectif.',
        '<strong class="text-white">6.2.</strong> En outre, conformément à l\'article D441-5 du Code de commerce, une indemnité forfaitaire pour frais de recouvrement de quarante euros (40 EUR) est due par le client pour chaque facture impayée, en sus des pénalités de retard. Lorsque les frais de recouvrement réellement engagés par Hook0 dépassent ce montant, Hook0 se réserve le droit de réclamer une indemnisation complémentaire sur justificatif.',
        '<strong class="text-white">6.3.</strong> Sans préjudice de ce qui précède, Hook0 se réserve le droit de suspendre ou de restreindre l\'accès du client au service, sans préavis et sans engager sa responsabilité, en cas de non-paiement de toute somme due après un délai de grâce de sept (7) jours calendaires suivant l\'échéance. La suspension du service ne libère pas le client de ses obligations de paiement.',
      ],
    },
    {
      id: 'overage',
      title: '7. Frais de dépassement',
      paragraphs: [
        '<strong class="text-white">7.1.</strong> Chaque offre payante inclut un quota quotidien d\'événements précisé à la section 2.3. Lorsque l\'usage dépasse le quota inclus, des frais de dépassement s\'appliquent au tarif unitaire indiqué pour l\'offre du client. Pour les offres payantes, l\'ingestion d\'événements n\'est pas interrompue lorsque le quota quotidien est dépassé, garantissant la continuité du service.',
        '<strong class="text-white">7.2.</strong> Pour l\'offre Developer (gratuite), aucun dépassement n\'est facturé. Les événements dépassant le quota quotidien sont bloqués jusqu\'à la réinitialisation du quota à minuit UTC le jour suivant.',
        '<strong class="text-white">7.3.</strong> Les frais de dépassement sont calculés automatiquement et facturés selon les modalités de la section 4.4. Le client accepte que toute utilisation du service au-delà du quota inclus constitue une commande implicite de capacité supplémentaire au tarif unitaire applicable.',
        '<strong class="text-white">7.4.</strong> Le client peut suivre en temps réel sa consommation d\'événements via le tableau de bord de son organisation dans l\'application Hook0. Hook0 envoie des notifications par courrier électronique lorsque la consommation quotidienne approche du seuil de quota.',
      ],
    },
    {
      id: 'plan-changes',
      title: '8. Changement d\'offre',
      paragraphs: [
        '<strong class="text-white">8.1. Montée en gamme.</strong> Le passage à une offre de niveau supérieur prend effet immédiatement après confirmation. Le client est facturé du différentiel au prorata du reste à courir de la période de facturation en cours.',
        '<strong class="text-white">8.2. Descente en gamme.</strong> Le passage à une offre de niveau inférieur prend effet à la fin de la période de facturation en cours. Le client conserve l\'accès aux fonctionnalités et quotas de l\'offre courante jusqu\'à cette date. Aucun remboursement n\'est réalisé pour la période restante.',
        '<strong class="text-white">8.3.</strong> Les changements d\'offre peuvent être réalisés depuis l\'application Hook0 ou en contactant <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>.',
      ],
    },
    {
      id: 'price-changes',
      title: '9. Modifications tarifaires',
      paragraphs: [
        '<strong class="text-white">9.1.</strong> Hook0 se réserve le droit de modifier ses tarifs à tout moment, moyennant un préavis écrit de trente (30) jours adressé à l\'adresse électronique enregistrée du client ou publié sur le site Hook0.',
        '<strong class="text-white">9.2.</strong> Les nouveaux tarifs s\'appliquent à la première période de facturation commençant après la fin du délai de préavis. Les modifications tarifaires sont sans effet rétroactif sur les périodes déjà facturées.',
        '<strong class="text-white">9.3.</strong> Si le client n\'accepte pas le nouveau tarif, il peut résilier son abonnement avant la prise d\'effet des nouveaux tarifs conformément à la section 10. L\'utilisation continue du service après la date d\'effet de la modification tarifaire vaut acceptation des nouveaux tarifs.',
      ],
    },
    {
      id: 'cancellation',
      title: '10. Résiliation',
      paragraphs: [
        '<strong class="text-white">10.1.</strong> Le client peut résilier son abonnement à tout moment en adressant une demande à <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>.',
        '<strong class="text-white">10.2.</strong> La résiliation prend effet à la fin de la période de facturation en cours. Le client conserve l\'accès à l\'offre souscrite jusqu\'à cette date.',
        '<strong class="text-white">10.3. Absence de remboursement.</strong> Les périodes déjà facturées et réglées ne donnent lieu à aucun remboursement, quelle que soit la raison de la résiliation et que le service ait été utilisé ou non pendant cette période. Cette règle s\'applique aussi bien aux abonnements mensuels qu\'aux abonnements annuels.',
        '<strong class="text-white">10.4. Conservation des données après résiliation.</strong> Les données du client sont conservées pendant trente (30) jours après la fin de l\'abonnement, période pendant laquelle le client peut demander l\'export de ses données en contactant <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>. À l\'issue de ce délai de trente jours, l\'ensemble des données du client est supprimé de manière définitive.',
        '<strong class="text-white">10.5.</strong> Hook0 peut résilier un abonnement avec effet immédiat et sans remboursement en cas de manquement grave du client aux Conditions Générales d\'Utilisation, notamment en cas de non-paiement, d\'usage frauduleux ou de violation des règles d\'usage acceptable.',
      ],
    },
    {
      id: 'free-plan',
      title: '11. Offre gratuite',
      paragraphs: [
        '<strong class="text-white">11.1.</strong> L\'offre Developer est mise à disposition gratuitement. Elle ne constitue pas un engagement commercial de Hook0 et peut être modifiée, limitée ou interrompue à la discrétion de Hook0. Aucun engagement de niveau de service (SLA) n\'est attaché à l\'offre Developer ; un SLA sur mesure peut être négocié uniquement avec les clients Enterprise.',
        '<strong class="text-white">11.2.</strong> Hook0 donne un préavis d\'au moins quatre-vingt-dix (90) jours avant d\'interrompre l\'offre Developer gratuite ou de réduire substantiellement ses quotas inclus. Le préavis est adressé à l\'adresse électronique enregistrée ou publié sur le site Hook0.',
      ],
    },
    {
      id: 'taxes',
      title: '12. Taxes',
      paragraphs: [
        '<strong class="text-white">12.1.</strong> Tous les prix sont indiqués hors taxes (HT). La TVA ou les taxes indirectes équivalentes sont ajoutées au montant de la facture et collectées par Hook0 ou par Stripe, conformément aux règles fiscales applicables dans le pays d\'établissement du client.',
        '<strong class="text-white">12.2.</strong> Les clients professionnels établis dans un État membre de l\'Union européenne autre que la France peuvent être exonérés de la TVA française s\'ils fournissent un numéro de TVA intracommunautaire valide. Il appartient au client de communiquer des informations fiscales exactes et à jour dans les paramètres de son compte.',
        '<strong class="text-white">12.3.</strong> Les clients établis hors de l\'Union européenne sont responsables des droits de douane, retenues à la source ou autres prélèvements applicables dans leur pays. Hook0 ne collecte pas de taxes pour le compte d\'autorités fiscales étrangères, sauf disposition légale contraire.',
      ],
    },
    {
      id: 'subprocessors',
      title: '13. Infrastructure et sous-traitants ultérieurs',
      paragraphs: [
        '<strong class="text-white">13.1.</strong> Hook0 Cloud s\'appuie sur des prestataires d\'infrastructure tiers, notamment l\'hébergement assuré par Clever Cloud SAS (France) et la diffusion de contenu et la sécurité périphérique assurées par Cloudflare Inc. (États-Unis). La liste à jour des sous-traitants ultérieurs et leurs localisations est publiée sur <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a> et intégrée par renvoi aux présentes.',
        '<strong class="text-white">13.2.</strong> Les transferts de données à caractère personnel vers des sous-traitants ultérieurs établis hors de l\'Espace économique européen sont encadrés par l\'<a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Avenant relatif au traitement des données</a>, qui précise le mécanisme de transfert applicable (Clauses contractuelles types ou, le cas échéant, EU-US Data Privacy Framework).',
      ],
    },
    {
      id: 'law',
      title: '14. Droit applicable et juridiction',
      paragraphs: [
        '<strong class="text-white">14.1.</strong> Les présentes Conditions générales de vente sont régies exclusivement par le droit français. L\'application de la Convention des Nations unies sur les contrats de vente internationale de marchandises (CVIM) est expressément exclue.',
        '<strong class="text-white">14.2.</strong> Conformément à l\'article 48 du Code de procédure civile, applicable entre commerçants, les parties conviennent de soumettre tout litige découlant des présentes ou en lien avec celles-ci à la compétence exclusive des tribunaux de La Roche-sur-Yon, France, dans le ressort desquels Hook0 a son siège, nonobstant pluralité de défendeurs ou appel en garantie, et sous réserve des règles de compétence impératives applicables à la matière du litige. Les parties s\'efforcent préalablement de parvenir à une résolution amiable ; à défaut d\'accord amiable dans un délai de trente (30) jours suivant la notification écrite du litige, le litige peut être porté devant les juridictions ci-dessus.',
      ],
    },
    {
      id: 'contact',
      title: '15. Contact',
      lead: 'Pour toute question relative aux présentes Conditions générales de vente, à la facturation ou aux sujets commerciaux, veuillez nous contacter :',
      contactItems: [
        '<strong class="text-white">Questions juridiques :</strong> <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
        '<strong class="text-white">Facturation et abonnements :</strong> <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>',
        '<strong class="text-white">Ventes Enterprise :</strong> <a href="mailto:sales@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">sales@hook0.com</a>',
        '<strong class="text-white">Siège social :</strong> FGRibreau SARL, 3 rue de l\'Aubépine, 85110 Chantonnay, France',
      ],
    },
  ],
};
