// Per-page strings for mediakit (EN base).
// VERBATIM extraction from the legacy inline template — do not humanize.
// Asset URLs (logo PNGs, founder JPGs, mediakit/* paths) stay HARDCODED in
// the template — they are static assets, not locale-dependent.
// Hex codes are also hardcoded inline in the template; only color name
// labels and role descriptions live here.
module.exports = {
  pageTitle: 'Hook0 - Media Kit - Brand Assets & Press Resources',
  pageDescription: 'Everything you need to feature Hook0 in your publications. Download our logos, learn about our brand, and discover the team behind Hook0.',
  hero: {
    badge: {
      label: 'Press Resources',
      text: 'Brand Assets & Guidelines',
    },
    titleBefore: 'Media Kit',
    titleAccent: 'Brand Resources',
    description: 'Everything you need to feature Hook0 in your publications. Download our logos, learn about our brand, and discover the team behind Hook0.',
  },
  logo: {
    eyebrow: 'Brand Identity',
    h2: 'Our Logo',
    sub: 'The Hook0 logo represents reliability and connectivity. Please use it responsibly and maintain proper spacing.',
    primary: {
      title: 'Primary Logo',
      desc: 'Use this version on light backgrounds',
    },
    variants: [
      { title: 'Color', desc: 'For light backgrounds', button: 'Download PNG' },
      { title: 'White', desc: 'For dark backgrounds', button: 'Download PNG' },
      { title: 'Grayscale', desc: 'For monochrome contexts', button: 'Download PNG' },
    ],
    bannerVariantsH3: 'Banner Variants',
    banner: [
      { title: 'Banner (Light)', desc: 'Logo with wordmark for headers', button: 'Download PNG' },
      { title: 'Banner (Transparent)', desc: 'Transparent version for overlays', button: 'Download PNG' },
    ],
    allDownloadButtonLabel: 'Download PNG',
  },
  colors: {
    eyebrow: 'Visual Identity',
    h2: 'Brand Colors',
    sub: 'Our color palette reflects trust, innovation, and reliability.',
    swatches: [
      { name: 'Green 500', role: 'Primary brand color' },
      { name: 'Indigo 500', role: 'Accent color' },
      { name: 'Surface Primary', role: 'Dark background' },
      { name: 'Gray 50', role: 'Light mode backgrounds' },
    ],
  },
  founders: {
    eyebrow: 'The Team',
    h2: 'Meet the Founders',
    sub: 'Hook0 was founded by two experienced developers from Nantes, France, with a passion for building reliable infrastructure.',
    items: [
      {
        name: 'Francois-Guillaume Ribreau',
        role: 'Co-Founder & CEO',
        bio: 'Serial entrepreneur with 15+ years of experience building developer tools and SaaS products.',
        downloadLabel: 'Download Photo',
      },
      {
        name: 'David Sferruzza',
        role: 'Co-Founder & CTO',
        bio: 'Software architect and functional programming enthusiast with deep expertise in distributed systems.',
        downloadLabel: 'Download Photo',
      },
    ],
  },
  hooky: {
    eyebrow: 'Brand Character',
    h2: 'Meet Hooky',
    sub: 'Our mascot embodies the Hook0 promise: Reliability, Transparency, and Robustness.',
    downloadJpg: 'Download JPG',
    downloadLargePng: 'Download Large PNG',
    characterEssenceH3: 'Character Essence',
    characterEssenceP: 'Hooky is not a marketing gimmick. He is the visual manifestation of the Hook0 promise. While other mascots might be abstract shapes representing "synergy," Hooky is a machine with a purpose—ensuring message deliverability with precision and reliability.',
    pillarsH4: 'The 3 Pillars of Personality',
    pillars: [
      { title: 'The Technical Expert', body: 'He speaks JSON, understands HTTP status codes natively, and prefers exact numbers over approximations.' },
      { title: 'The European Guardian', body: 'Protective and vigilant. GDPR Native. He treats personal data like radioactive material—handled with extreme care.' },
      { title: 'The Independent', body: 'Built to last. Solid, made of high-grade metal, not disposable plastic. Scratches on his armor are badges of honor.' },
    ],
    voiceToneH4: 'Voice & Tone',
    voiceToneP: 'Hooky speaks to developers. His tone is informative, encouraging, concise, and dryly humorous. He avoids corporate jargon entirely.',
    vocabKeep: [
      { label: 'Payload', kind: 'keep' },
      { label: 'Endpoint', kind: 'keep' },
      { label: 'Latency', kind: 'keep' },
      { label: 'Dispatch', kind: 'keep' },
      { label: 'Synergy', kind: 'avoid' },
      { label: 'Leverage', kind: 'avoid' },
    ],
    paletteH3: "Hooky's Color Palette",
    palette: [
      { name: 'Electric Blue', hex: '#00A3FF' },
      { name: 'Titanium White', hex: '#E0E0E0' },
      { name: 'Brushed Steel', hex: '#8C92AC' },
      { name: 'Carbon Black', hex: '#2D3748' },
      { name: 'Success Green', hex: '#48BB78' },
      { name: 'Retry Red', hex: '#F56565' },
    ],
  },
  about: {
    eyebrow: 'Press Information',
    h2: 'About Hook0',
    overviewH3: 'Company Overview',
    overviewP: 'Hook0 is an open-source Webhooks-as-a-Service platform that helps developers send, receive, and manage webhooks at scale. Founded in Nantes, France, Hook0 is 100% bootstrapped with no venture capital funding.',
    keyFactsH3: 'Key Facts',
    facts: [
      { labelHtml: 'Founded:', valueHtml: 'Nantes, France' },
      { labelHtml: 'Funding:', valueHtml: '100% Bootstrapped, no VC' },
      { labelHtml: 'Product:', valueHtml: 'Open-Source Webhooks-as-a-Service' },
      { labelHtml: 'Mission:', valueHtml: 'Build reliable webhook infrastructure that lasts' },
      { labelHtml: 'GDPR:', valueHtml: 'Fully compliant, EU-based' },
    ],
    boilerplateH3: 'Boilerplate',
    boilerplateQuote: '"Hook0 is an open-source webhooks platform that enables developers to build reliable, scalable event-driven integrations. Based in France and 100% bootstrapped, Hook0 is committed to building software that stands the test of time."',
  },
  usage: {
    eyebrow: 'Guidelines',
    h2: 'Brand Usage',
    dos: {
      title: 'Do',
      items: [
        'Use the logo with adequate clear space',
        'Use the white logo on dark backgrounds',
        'Maintain the original aspect ratio',
        'Use high-resolution versions for print',
        'Reference Hook0 correctly in text',
      ],
    },
    donts: {
      title: "Don't",
      items: [
        'Alter the logo colors',
        'Stretch or distort the logo',
        'Add effects like shadows or gradients',
        'Use the logo as part of a sentence',
        'Place on busy or low-contrast backgrounds',
      ],
    },
  },
  contact: {
    titleBefore: 'Need More?',
    titleAccent: 'Get in Touch',
    description: 'For press inquiries, interview requests, or additional assets, our team is here to help.',
    ctaPress: 'Contact Press Team',
    ctaGeneral: 'General Inquiries',
  },
};
