// Per-page strings for mentions-legales (FR).
// Page légale : registre formel obligatoire (vouvoiement, pas tutoiement).
// /humanizer pro appliqué. Pas d'em-dash, pas de pivot colon, pas de point médian.
// Faits Hook0 verbatim, jamais traduits : raison sociale, capital, RCS, TVA,
// SIRET, adresse, téléphone, identité directeur de la publication, identité
// hébergeur et CDN. Seuls les libellés autour sont traduits.
// Hook0 = « code source ouvert (SSPL-1.0) », jamais « open source ».
module.exports = {
  pageTitle: 'Hook0 - Mentions légales',
  pageDescription: 'Mentions légales Hook0 : éditeur, hébergeur et informations légales exigées par l\'article 6 de la LCEN.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Mentions légales',
    h1: 'Mentions légales',
    subtitle: 'Informations légales exigées par l\'article 6 de la Loi pour la Confiance dans l\'Économie Numérique (LCEN).',
  },
  publisher: {
    h2: 'Informations sur l\'éditeur',
    intro: 'Ce site est édité par :',
    rows: [
      { label: 'Dénomination sociale', value: 'FGRibreau SARL' },
      { label: 'Forme juridique', value: 'Société à Responsabilité Limitée (SARL)' },
      { label: 'Capital social', value: '2 000 EUR' },
      { label: 'Siège social', value: "3 rue de l'Aubépine, 85110 Chantonnay, France" },
      { label: 'RCS', value: 'La Roche-sur-Yon 850 824 350' },
      { label: 'SIRET', value: '850 824 350 00019' },
      { label: 'SIREN', value: '850 824 350' },
      { label: 'Numéro de TVA intracommunautaire', value: 'FR27850824350' },
      { label: 'Téléphone', value: '+33 2 52 43 10 53' },
    ],
  },
  director: {
    h2: 'Directeur de la publication',
    bodyHtml: 'Le directeur de la publication de ce site est <strong class="text-white">David Sferruzza</strong>.',
  },
  hosting: {
    h2: 'Hébergeur',
    intro: 'L\'application Hook0 et ses données sont hébergées par :',
    rows: [
      { label: 'Dénomination sociale', value: 'Clever Cloud SAS' },
      { label: 'Adresse', value: "3 rue de l'Allier, 44000 Nantes, France" },
    ],
  },
  cdn: {
    h2: 'Prestataire CDN et DNS',
    intro: 'La résolution DNS et la diffusion des contenus sont assurées par :',
    rows: [
      { label: 'Dénomination sociale', value: 'Cloudflare, Inc.' },
      { label: 'Adresse', value: '101 Townsend St, San Francisco, CA 94107, USA' },
    ],
  },
  contact: {
    h2: 'Contact',
    intro: 'Pour toute question relative à ce site ou à son contenu :',
    rows: [
      { label: 'Support général', emailLabel: 'support@hook0.com', email: 'support@hook0.com' },
      { label: 'Questions juridiques', emailLabel: 'legal@hook0.com', email: 'legal@hook0.com' },
    ],
  },
  ip: {
    h2: 'Propriété intellectuelle',
    p1: 'L\'ensemble des contenus publiés sur ce site, notamment les textes, graphismes, logos, icônes, images et logiciels, sont la propriété exclusive de FGRibreau SARL ou de ses fournisseurs de contenu et sont protégés par les lois françaises et internationales relatives à la propriété intellectuelle.',
    p2: 'Toute reproduction, distribution, modification ou utilisation de ces éléments sans autorisation écrite préalable de FGRibreau SARL est strictement interdite.',
    p3: 'Le logiciel Hook0 est diffusé à code source ouvert (SSPL-1.0) sous sa propre licence, disponible dans le dépôt du projet.',
  },
  personalData: {
    h2: 'Données personnelles',
    p1: 'FGRibreau SARL traite les données personnelles conformément aux réglementations françaises et européennes applicables, notamment le Règlement (UE) 2016/679 (RGPD).',
    p2Html: 'Pour le détail complet de la collecte, du traitement et de la protection de vos données personnelles, veuillez consulter notre <a href="/fr/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Politique de confidentialité</a>.',
  },
  law: {
    h2: 'Droit applicable et juridiction compétente',
    p1: 'Les présentes mentions légales et tout litige né de l\'utilisation de ce site sont régis par le droit français.',
    p2: 'À défaut de résolution amiable, tout litige sera soumis à la compétence exclusive des tribunaux de Nantes, France.',
  },
};
