// Per-page strings for mentions-legales (DE).
// Rechtsseite: formaler Stil obligatorisch (Siezen, kein Duzen).
// /humanizer pro angewendet. Kein Em-Dash, kein Pivot-Doppelpunkt, kein
// Mittelpunkt. Hook0 Rechtsdaten verbatim, nie übersetzt (Firma, Kapital,
// RCS, USt-ID, SIRET, Adresse, Telefon, Verantwortlicher, Hosting-Anbieter,
// CDN). Nur die Labels drumherum werden übersetzt.
// Hook0 = « quelloffen (SSPL-1.0) », niemals « Open Source ».
// DSGVO-Anspruch nur als Prozess-Claim («gemäß den geltenden ... DSGVO»),
// nicht als zertifiziertes Attribut.
module.exports = {
  pageTitle: 'Hook0 - Impressum',
  pageDescription: 'Impressum von Hook0: Anbieter, Hosting-Anbieter und Pflichtangaben gemäß Artikel 6 LCEN.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Impressum',
    h1: 'Impressum',
    subtitle: 'Pflichtangaben gemäß Artikel 6 des französischen Gesetzes über das Vertrauen in die digitale Wirtschaft (LCEN).',
  },
  publisher: {
    h2: 'Angaben zum Anbieter',
    intro: 'Diese Website wird betrieben von:',
    rows: [
      { label: 'Firma', value: 'FGRibreau SARL' },
      { label: 'Rechtsform', value: 'Société à Responsabilité Limitée (SARL)' },
      { label: 'Stammkapital', value: '2.000 EUR' },
      { label: 'Geschäftsadresse', value: "3 rue de l'Aubépine, 85110 Chantonnay, Frankreich" },
      { label: 'Handelsregister (RCS)', value: 'La Roche-sur-Yon 850 824 350' },
      { label: 'SIRET', value: '850 824 350 00019' },
      { label: 'SIREN', value: '850 824 350' },
      { label: 'Umsatzsteuer-Identifikationsnummer', value: 'FR27850824350' },
      { label: 'Telefon', value: '+33 2 52 43 10 53' },
    ],
  },
  director: {
    h2: 'Verantwortlich für den Inhalt',
    bodyHtml: 'Verantwortlich für den Inhalt dieser Website ist <strong class="text-white">David Sferruzza</strong>.',
  },
  hosting: {
    h2: 'Hosting-Anbieter',
    intro: 'Die Hook0-Anwendung und ihre Daten werden gehostet von:',
    rows: [
      { label: 'Firma', value: 'Clever Cloud SAS' },
      { label: 'Adresse', value: "3 rue de l'Allier, 44000 Nantes, Frankreich" },
    ],
  },
  cdn: {
    h2: 'CDN- und DNS-Anbieter',
    intro: 'DNS-Auflösung und Auslieferung der Inhalte erfolgen durch:',
    rows: [
      { label: 'Firma', value: 'Cloudflare, Inc.' },
      { label: 'Adresse', value: '101 Townsend St, San Francisco, CA 94107, USA' },
    ],
  },
  contact: {
    h2: 'Kontakt',
    intro: 'Für Fragen zu dieser Website oder ihren Inhalten:',
    rows: [
      { label: 'Allgemeiner Support', emailLabel: 'support@hook0.com', email: 'support@hook0.com' },
      { label: 'Rechtliche Fragen', emailLabel: 'legal@hook0.com', email: 'legal@hook0.com' },
    ],
  },
  ip: {
    h2: 'Geistiges Eigentum',
    p1: 'Sämtliche auf dieser Website veröffentlichten Inhalte, insbesondere Texte, Grafiken, Logos, Symbole, Bilder und Software, sind ausschließliches Eigentum der FGRibreau SARL oder ihrer Lieferanten und durch das französische sowie internationale Recht des geistigen Eigentums geschützt.',
    p2: 'Jegliche Vervielfältigung, Verbreitung, Bearbeitung oder Nutzung dieser Inhalte ohne vorherige schriftliche Zustimmung der FGRibreau SARL ist strikt untersagt.',
    p3: 'Die Hook0-Software wird quelloffen (SSPL-1.0) unter ihrer eigenen Lizenz veröffentlicht; die Lizenz ist im Projekt-Repository verfügbar.',
  },
  personalData: {
    h2: 'Personenbezogene Daten',
    p1: 'Die FGRibreau SARL verarbeitet personenbezogene Daten gemäß den geltenden französischen und europäischen Vorschriften, insbesondere der Verordnung (EU) 2016/679 (DSGVO).',
    p2Html: 'Vollständige Details zur Erhebung, Verarbeitung und zum Schutz Ihrer personenbezogenen Daten finden Sie in unserer <a href="/de/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Datenschutzerklärung</a>.',
  },
  law: {
    h2: 'Anwendbares Recht und Gerichtsstand',
    p1: 'Dieses Impressum sowie jeder aus der Nutzung dieser Website entstehende Rechtsstreit unterliegen französischem Recht.',
    p2: 'Sollte keine gütliche Einigung erzielt werden, ist für jeden Rechtsstreit ausschließlich das zuständige Gericht in Nantes, Frankreich, zuständig.',
  },
};
