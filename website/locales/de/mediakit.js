// Per-page strings for mediakit (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// Hook0 = « quelloffen (SSPL-1.0) », jamais « Open Source ».
// DSGVO = process claim (« auf DSGVO-Konformität ausgelegt »), nicht absolut.
// Founder-Namen verbatim. Hex-Codes verbatim.
module.exports = {
  pageTitle: 'Hook0 - Media Kit, Markenmaterialien und Presseressourcen',
  pageDescription: 'Alles, was du brauchst, um Hook0 in deinen Publikationen vorzustellen. Lade unsere Logos herunter, lerne unsere Marke kennen und entdecke das Team hinter Hook0.',
  pageModified: '2026-06-27',
  hero: {
    badge: {
      label: 'Presseressourcen',
      text: 'Markenmaterialien und Guidelines',
    },
    titleBefore: 'Media Kit',
    titleAccent: 'Markenressourcen',
    description: 'Alles, was du brauchst, um Hook0 in deinen Publikationen vorzustellen. Lade unsere Logos herunter, lerne unsere Marke kennen und entdecke das Team hinter Hook0.',
  },
  logo: {
    eyebrow: 'Markenidentität',
    h2: 'Unser Logo',
    sub: 'Das Hook0-Logo steht für Zuverlässigkeit und Konnektivität. Setze es verantwortungsvoll ein und halte ausreichend Schutzraum ein.',
    primary: {
      title: 'Primärlogo',
      desc: 'Diese Version auf hellen Hintergründen verwenden',
    },
    variants: [
      { title: 'Farbe', desc: 'Für helle Hintergründe', button: 'PNG herunterladen' },
      { title: 'Weiß', desc: 'Für dunkle Hintergründe', button: 'PNG herunterladen' },
      { title: 'Graustufen', desc: 'Für monochrome Kontexte', button: 'PNG herunterladen' },
    ],
    bannerVariantsH3: 'Banner-Varianten',
    banner: [
      { title: 'Banner (hell)', desc: 'Logo mit Wortmarke für Header', button: 'PNG herunterladen' },
      { title: 'Banner (transparent)', desc: 'Transparente Version für Overlays', button: 'PNG herunterladen' },
    ],
    allDownloadButtonLabel: 'PNG herunterladen',
  },
  colors: {
    eyebrow: 'Visuelle Identität',
    h2: 'Markenfarben',
    sub: 'Unsere Farbpalette steht für Vertrauen, Innovation und Zuverlässigkeit.',
    swatches: [
      { name: 'Green 500', role: 'Primäre Markenfarbe' },
      { name: 'Indigo 500', role: 'Akzentfarbe' },
      { name: 'Surface Primary', role: 'Dunkler Hintergrund' },
      { name: 'Gray 50', role: 'Hintergründe im hellen Modus' },
    ],
  },
  founders: {
    eyebrow: 'Das Team',
    h2: 'Lerne die Gründer kennen',
    sub: 'Hook0 wurde von zwei erfahrenen Entwicklern aus Nantes, Frankreich, gegründet, mit einer Leidenschaft für den Bau zuverlässiger Infrastruktur.',
    items: [
      {
        name: 'Francois-Guillaume Ribreau',
        role: 'Mitgründer & CEO',
        bio: 'Serien-Unternehmer mit über 15 Jahren Erfahrung im Bau von Entwicklertools und SaaS-Produkten.',
        downloadLabel: 'Foto herunterladen',
      },
      {
        name: 'David Sferruzza',
        role: 'Mitgründer & CTO',
        bio: 'Softwarearchitekt und Liebhaber funktionaler Programmierung, mit tiefer Expertise in verteilten Systemen.',
        downloadLabel: 'Foto herunterladen',
      },
    ],
  },
  hooky: {
    eyebrow: 'Markencharakter',
    h2: 'Das ist Hooky',
    sub: 'Unser Maskottchen verkörpert das Hook0-Versprechen, Zuverlässigkeit, Transparenz und Robustheit.',
    downloadJpg: 'JPG herunterladen',
    downloadLargePng: 'Großes PNG herunterladen',
    characterEssenceH3: 'Charakteressenz',
    characterEssenceP: 'Hooky ist kein Marketinggag. Er ist die visuelle Manifestation des Hook0-Versprechens. Wo andere Maskottchen abstrakte Formen sind, die « Synergie » darstellen sollen, ist Hooky eine Maschine mit einem Auftrag, die Zustellung von Nachrichten mit Präzision und Zuverlässigkeit zu garantieren.',
    pillarsH4: 'Die 3 Persönlichkeits-Säulen',
    pillars: [
      { title: 'Der technische Experte', body: 'Er spricht JSON, versteht HTTP-Statuscodes nativ und bevorzugt exakte Zahlen statt Annäherungen.' },
      { title: 'Der europäische Wächter', body: 'Schützend und wachsam. DSGVO-bewusst von Geburt. Er behandelt personenbezogene Daten wie radioaktives Material, mit äußerster Sorgfalt zu handhaben.' },
      { title: 'Der Unabhängige', body: 'Gebaut, um zu bleiben. Solide, aus hochwertigem Metall, kein Wegwerfplastik. Kratzer auf seiner Rüstung sind Auszeichnungen.' },
    ],
    voiceToneH4: 'Stimme und Tonalität',
    voiceToneP: 'Hooky spricht zu Entwicklern. Sein Ton ist informativ, ermutigend, prägnant und trocken-humorvoll. Corporate-Jargon meidet er konsequent.',
    vocabKeep: [
      { label: 'Payload', kind: 'keep' },
      { label: 'Endpoint', kind: 'keep' },
      { label: 'Latenz', kind: 'keep' },
      { label: 'Zustellung', kind: 'keep' },
      { label: 'Synergy', kind: 'avoid' },
      { label: 'Leverage', kind: 'avoid' },
    ],
    paletteH3: 'Hookys Farbpalette',
    palette: [
      { name: 'Elektroblau', hex: '#00A3FF' },
      { name: 'Titanweiß', hex: '#E0E0E0' },
      { name: 'Gebürsteter Stahl', hex: '#8C92AC' },
      { name: 'Karbonschwarz', hex: '#2D3748' },
      { name: 'Erfolgsgrün', hex: '#48BB78' },
      { name: 'Wiederholungs-Rot', hex: '#F56565' },
    ],
  },
  about: {
    eyebrow: 'Presseinformationen',
    h2: 'Über Hook0',
    overviewH3: 'Unternehmensüberblick',
    overviewP: 'Hook0 ist eine quelloffene Webhooks-as-a-Service-Plattform (SSPL-1.0), die Entwicklern hilft, Webhooks im großen Maßstab zu senden, zu empfangen und zu verwalten. Gegründet in Nantes, Frankreich, ist Hook0 zu 100% bootstrappt, ohne Wagniskapital-Finanzierung.',
    keyFactsH3: 'Eckdaten',
    facts: [
      { labelHtml: 'Gegründet:', valueHtml: 'Nantes, Frankreich' },
      { labelHtml: 'Finanzierung:', valueHtml: '100% bootstrappt, ohne VC' },
      { labelHtml: 'Produkt:', valueHtml: 'Webhooks-as-a-Service mit quelloffenem Code (SSPL-1.0)' },
      { labelHtml: 'Mission:', valueHtml: 'Zuverlässige Webhook-Infrastruktur bauen, die bleibt' },
      { labelHtml: 'DSGVO:', valueHtml: 'auf DSGVO-Konformität ausgelegt, in der EU ansässig' },
    ],
    boilerplateH3: 'Boilerplate',
    boilerplateQuote: '« Hook0 ist eine quelloffene Webhook-Plattform (SSPL-1.0), die Entwicklern ermöglicht, zuverlässige, skalierbare event-getriebene Integrationen zu bauen. Mit Sitz in Frankreich und zu 100% bootstrappt, engagiert sich Hook0 dafür, Software zu bauen, die der Zeit standhält. »',
  },
  usage: {
    eyebrow: 'Guidelines',
    h2: 'Markennutzung',
    dos: {
      title: 'Tu das',
      items: [
        'Das Logo mit ausreichend Freiraum verwenden',
        'Auf dunklen Hintergründen das weiße Logo verwenden',
        'Das ursprüngliche Seitenverhältnis beibehalten',
        'Hochauflösende Versionen für den Druck verwenden',
        'Hook0 korrekt im Text referenzieren',
      ],
    },
    donts: {
      title: 'Lass das',
      items: [
        'Die Logo-Farben ändern',
        'Das Logo strecken oder verzerren',
        'Effekte wie Schatten oder Verläufe hinzufügen',
        'Das Logo in einen Satz einbauen',
        'Es auf unruhigen oder kontrastarmen Hintergründen platzieren',
      ],
    },
  },
  contact: {
    titleBefore: 'Brauchst du mehr?',
    titleAccent: 'Kontaktiere uns',
    description: 'Für Presseanfragen, Interview-Anfragen oder zusätzliche Assets ist unser Team für dich da.',
    ctaPress: 'Presseteam kontaktieren',
    ctaGeneral: 'Allgemeine Anfragen',
  },
};
