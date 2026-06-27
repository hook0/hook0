// Per-page strings for mentions-legales (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
// Legal page (LCEN Art. 6). Hook0 legal facts kept verbatim across locales:
// FGRibreau SARL, capital 2,000 EUR, RCS La Roche-sur-Yon 850 824 350,
// VAT FR27850824350, director of publication David Sferruzza, hosting
// Clever Cloud SAS (France), CDN Cloudflare Inc. (USA — CLOUD Act disclosed).
module.exports = {
  pageTitle: 'Hook0 - Legal Notice',
  pageDescription: 'Legal notice for Hook0 - Publisher information, hosting provider, and legal information required under French law (LCEN Article 6).',
  hero: {
    eyebrow: 'Legal',
    h1: 'Legal Notice',
    subtitle: 'Legal information required under Article 6 of the French Law on Confidence in the Digital Economy (LCEN).',
  },
  publisher: {
    h2: 'Publisher Information',
    intro: 'This website is published by:',
    rows: [
      { label: 'Company name', value: 'FGRibreau SARL' },
      { label: 'Legal form', value: 'Société à Responsabilité Limitée (SARL)' },
      { label: 'Share capital', value: '2,000 EUR' },
      { label: 'Registered office', value: "3 rue de l'Aubépine, 85110 Chantonnay, France" },
      { label: 'RCS', value: 'La Roche-sur-Yon 850 824 350' },
      { label: 'SIRET', value: '850 824 350 00019' },
      { label: 'SIREN', value: '850 824 350' },
      { label: 'EU VAT number', value: 'FR27850824350' },
      { label: 'Phone', value: '+33 2 52 43 10 53' },
    ],
  },
  director: {
    h2: 'Publication Director',
    bodyHtml: 'The publication director of this website is <strong class="text-white">David Sferruzza</strong>.',
  },
  hosting: {
    h2: 'Hosting Provider',
    intro: 'The Hook0 application and its data are hosted by:',
    rows: [
      { label: 'Company name', value: 'Clever Cloud SAS' },
      { label: 'Address', value: "3 rue de l'Allier, 44000 Nantes, France" },
    ],
  },
  cdn: {
    h2: 'CDN and DNS Provider',
    intro: 'DNS resolution and content delivery are provided by:',
    rows: [
      { label: 'Company name', value: 'Cloudflare, Inc.' },
      { label: 'Address', value: '101 Townsend St, San Francisco, CA 94107, USA' },
    ],
  },
  contact: {
    h2: 'Contact',
    intro: 'For any inquiry regarding this website or its content:',
    rows: [
      { label: 'General support', emailLabel: 'support@hook0.com', email: 'support@hook0.com' },
      { label: 'Legal inquiries', emailLabel: 'legal@hook0.com', email: 'legal@hook0.com' },
    ],
  },
  ip: {
    h2: 'Intellectual Property',
    p1: 'All content published on this website — including but not limited to text, graphics, logos, icons, images, and software — is the exclusive property of FGRibreau SARL or its content suppliers and is protected by applicable French and international intellectual property laws.',
    p2: 'Any reproduction, distribution, modification, or use of these materials without prior written consent from FGRibreau SARL is strictly prohibited.',
    p3: 'The Hook0 open-source software is distributed under its own license, available in the project repository.',
  },
  personalData: {
    h2: 'Personal Data',
    p1: 'FGRibreau SARL processes personal data in accordance with applicable French and European regulations, including Regulation (EU) 2016/679 (GDPR).',
    p2Html: 'For full details on how your personal data is collected, processed, and protected, please consult our <a href="/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Privacy Policy</a>.',
  },
  law: {
    h2: 'Applicable Law and Jurisdiction',
    p1: 'This legal notice and any dispute arising from use of this website are governed by French law.',
    p2: 'In the absence of an amicable resolution, any dispute shall be submitted to the exclusive jurisdiction of the courts of Nantes, France.',
  },
};
