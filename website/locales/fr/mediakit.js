// Per-page strings for mediakit (FR).
// /humanizer pro appliqué. Tutoiement. Pas d'em-dash, pas de pivot colon.
// Hook0 = « code source ouvert (SSPL-1.0) », jamais « open source ».
// Noms des fondateurs verbatim. Codes hex verbatim.
module.exports = {
  pageTitle: 'Hook0 - Media kit, ressources de marque et presse',
  pageDescription: 'Tout ce qu\'il te faut pour parler de Hook0 dans tes publications. Télécharge nos logos, découvre notre identité de marque et l\'équipe derrière Hook0.',
  pageModified: '2026-06-27',
  hero: {
    badge: {
      label: 'Ressources presse',
      text: 'Identité de marque et guidelines',
    },
    titleBefore: 'Media kit',
    titleAccent: 'Ressources de marque',
    description: 'Tout ce qu\'il te faut pour parler de Hook0 dans tes publications. Télécharge nos logos, découvre notre identité de marque et l\'équipe derrière Hook0.',
  },
  logo: {
    eyebrow: 'Identité de marque',
    h2: 'Notre logo',
    sub: 'Le logo Hook0 incarne la fiabilité et la connectivité. Utilise-le avec soin et respecte ses espaces de protection.',
    primary: {
      title: 'Logo principal',
      desc: 'À utiliser sur fond clair',
    },
    variants: [
      { title: 'Couleur', desc: 'Pour fonds clairs', button: 'Télécharger le PNG' },
      { title: 'Blanc', desc: 'Pour fonds sombres', button: 'Télécharger le PNG' },
      { title: 'Niveaux de gris', desc: 'Pour contextes monochromes', button: 'Télécharger le PNG' },
    ],
    bannerVariantsH3: 'Variantes bannière',
    banner: [
      { title: 'Bannière (clair)', desc: 'Logo avec wordmark pour les en-têtes', button: 'Télécharger le PNG' },
      { title: 'Bannière (transparent)', desc: 'Version transparente pour les overlays', button: 'Télécharger le PNG' },
    ],
    allDownloadButtonLabel: 'Télécharger le PNG',
  },
  colors: {
    eyebrow: 'Identité visuelle',
    h2: 'Couleurs de marque',
    sub: 'Notre palette traduit la confiance, l\'innovation et la fiabilité.',
    swatches: [
      { name: 'Green 500', role: 'Couleur principale de marque' },
      { name: 'Indigo 500', role: 'Couleur d\'accentuation' },
      { name: 'Surface Primary', role: 'Fond sombre' },
      { name: 'Gray 50', role: 'Fonds en mode clair' },
    ],
  },
  founders: {
    eyebrow: 'L\'équipe',
    h2: 'Les fondateurs',
    sub: 'Hook0 a été fondé par deux développeurs expérimentés de Nantes, France, passionnés par la construction d\'infrastructures fiables.',
    items: [
      {
        name: 'Francois-Guillaume Ribreau',
        role: 'Co-fondateur & CEO',
        bio: 'Serial entrepreneur avec plus de 15 ans d\'expérience dans la création d\'outils pour développeurs et de produits SaaS.',
        downloadLabel: 'Télécharger la photo',
      },
      {
        name: 'David Sferruzza',
        role: 'Co-fondateur & CTO',
        bio: 'Architecte logiciel et passionné de programmation fonctionnelle, avec une expertise pointue des systèmes distribués.',
        downloadLabel: 'Télécharger la photo',
      },
    ],
  },
  hooky: {
    eyebrow: 'Personnage de marque',
    h2: 'Voici Hooky',
    sub: 'Notre mascotte incarne la promesse Hook0, fiabilité, transparence et robustesse.',
    downloadJpg: 'Télécharger le JPG',
    downloadLargePng: 'Télécharger le PNG grand format',
    characterEssenceH3: 'Essence du personnage',
    characterEssenceP: 'Hooky n\'est pas un gadget marketing. C\'est la manifestation visuelle de la promesse Hook0. Là où d\'autres mascottes sont des formes abstraites censées incarner la « synergie », Hooky est une machine avec une mission, garantir la livraison des messages avec précision et fiabilité.',
    pillarsH4: 'Les 3 piliers de personnalité',
    pillars: [
      { title: 'L\'expert technique', body: 'Il parle JSON, comprend les codes HTTP nativement et préfère les chiffres exacts aux approximations.' },
      { title: 'Le gardien européen', body: 'Protecteur et vigilant. Né avec le RGPD dans les gènes. Il traite les données personnelles comme du matériau radioactif, à manipuler avec une extrême prudence.' },
      { title: 'L\'indépendant', body: 'Construit pour durer. Solide, en métal de haute qualité, pas en plastique jetable. Les rayures sur son armure sont ses médailles.' },
    ],
    voiceToneH4: 'Voix et ton',
    voiceToneP: 'Hooky s\'adresse aux développeurs. Son ton est informatif, encourageant, concis et drôle au second degré. Il fuit le jargon corporate.',
    vocabKeep: [
      { label: 'Payload', kind: 'keep' },
      { label: 'Endpoint', kind: 'keep' },
      { label: 'Latence', kind: 'keep' },
      { label: 'Livraison', kind: 'keep' },
      { label: 'Synergy', kind: 'avoid' },
      { label: 'Leverage', kind: 'avoid' },
    ],
    paletteH3: 'La palette de couleurs de Hooky',
    palette: [
      { name: 'Bleu électrique', hex: '#00A3FF' },
      { name: 'Blanc titane', hex: '#E0E0E0' },
      { name: 'Acier brossé', hex: '#8C92AC' },
      { name: 'Noir carbone', hex: '#2D3748' },
      { name: 'Vert succès', hex: '#48BB78' },
      { name: 'Rouge relance', hex: '#F56565' },
    ],
  },
  about: {
    eyebrow: 'Informations presse',
    h2: 'À propos de Hook0',
    overviewH3: 'Présentation de la société',
    overviewP: 'Hook0 est une plateforme Webhooks-as-a-Service au code source ouvert (SSPL-1.0) qui aide les développeurs à envoyer, recevoir et gérer les webhooks à grande échelle. Fondée à Nantes, France, Hook0 est 100% bootstrappée, sans financement par capital-risque.',
    keyFactsH3: 'Faits clés',
    facts: [
      { labelHtml: 'Fondation :', valueHtml: 'Nantes, France' },
      { labelHtml: 'Financement :', valueHtml: '100% bootstrappée, sans VC' },
      { labelHtml: 'Produit :', valueHtml: 'Webhooks-as-a-Service au code source ouvert (SSPL-1.0)' },
      { labelHtml: 'Mission :', valueHtml: 'Construire une infrastructure webhook fiable qui dure' },
      { labelHtml: 'RGPD :', valueHtml: 'conçu pour la conformité RGPD, plan de données en UE (Clever Cloud FR), CDN Cloudflare US divulgué dans le <a href="/fr/accord-traitement-donnees">DPA</a>' },
    ],
    boilerplateH3: 'Texte de présentation',
    boilerplateQuote: '« Hook0 est une plateforme webhook au code source ouvert (SSPL-1.0) qui permet aux développeurs de bâtir des intégrations event-driven fiables et passant à l\'échelle. Basée en France et 100% bootstrappée, Hook0 s\'engage à construire un logiciel qui dure. »',
  },
  usage: {
    eyebrow: 'Guidelines',
    h2: 'Usage de la marque',
    dos: {
      title: 'À faire',
      items: [
        'Utiliser le logo avec un espace de protection suffisant',
        'Utiliser le logo blanc sur fonds sombres',
        'Conserver le ratio d\'aspect original',
        'Utiliser les versions haute résolution pour l\'impression',
        'Citer Hook0 correctement dans le texte',
      ],
    },
    donts: {
      title: 'À éviter',
      items: [
        'Modifier les couleurs du logo',
        'Étirer ou déformer le logo',
        'Ajouter des effets comme des ombres ou des dégradés',
        'Intégrer le logo dans une phrase',
        'Le placer sur des fonds chargés ou peu contrastés',
      ],
    },
  },
  contact: {
    titleBefore: 'Besoin de plus ?',
    titleAccent: 'Contacte-nous',
    description: 'Pour les demandes presse, les interviews ou des ressources supplémentaires, notre équipe est là pour aider.',
    ctaPress: 'Contacter l\'équipe presse',
    ctaGeneral: 'Demandes générales',
  },
};
