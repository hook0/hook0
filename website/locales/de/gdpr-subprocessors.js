// Per-page strings for gdpr-subprocessors (DE, DSGVO-Unterauftragsverarbeiter / Art. 28 DSGVO).
//
// Register : Siezen formell strikt (« Sie » / « Ihr »), wie fuer jeden
// vertragsrechtlichen Text. Kein Duzen. /humanizer pro angewendet. Kein
// Em-Dash, kein Doppel-Bindestrich als Pivot, kein Mittel-Punkt.
//
// Harte Regeln (CLAUDE.local.md) :
//   - DSGVO niemals absolut (« 100% DSGVO-konform »), nur als Prozess-Claim
//     (« auf DSGVO-Konformitaet ausgelegt »).
//   - Verboten : « 100% souveraen », « keine Daten verlassen die EU »,
//     « kein US-Konzern im Stack », « CLOUD Act free ».
//   - SSPL = « quelloffen (SSPL-1.0) » (auf dieser Seite nicht erwaehnt).
//
// Hook0 Hardfacts verbatim ueber alle Locales :
//   - FGRibreau SARL (Verantwortlicher Auftragsverarbeiter fuer Kundendaten)
//   - Unterauftragsverarbeiter (Namen und Adressen VERBATIM) :
//       * Clever Cloud SAS (Frankreich)
//       * Scaleway SAS (Frankreich)
//       * Cloudflare, Inc. (USA, 101 Townsend St, San Francisco, CA 94107)
//       * Stripe Inc. (USA)
//       * Brevo (Frankreich)
//       * Postmark (USA)
//       * BetterUptime (Tschechische Republik)
//       * Sentry (USA)
//       * Crisp (Frankreich)
//       * Gmail / Google Workspace (USA)
//   - Transfermechanismen : SCC 2021 (Standardvertragsklauseln) + TIA
//     (Transfer Impact Assessment) fuer US-Transfers ; EU-US DPF (Data
//     Privacy Framework), wenn der Unterauftragsverarbeiter zertifiziert ist.
module.exports = {
  pageTitle: 'Hook0 - DSGVO-Unterauftragsverarbeiter',
  pageDescription: 'DSGVO-Konformität von Hook0 und Liste der Unterauftragsverarbeiter, die unsere Webhook-Dienste ermöglichen. Transparenz zur Verarbeitung in Europa und zu Transfers in die USA.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Compliance',
    title: 'DSGVO und Unterauftragsverarbeiter',
    subtitle: 'Unser Engagement für den Datenschutz und die Partner, mit denen wir arbeiten.',
    lastUpdatedLabel: 'Letzte Aktualisierung:',
    lastUpdatedDate: '27. Juni 2026',
  },
  intro: {
    p1Html: 'Die Datenschutz-Grundverordnung (DSGVO / GDPR) ist das strengste Datenschutz- und Sicherheitsgesetz der Welt. Sie verpflichtet Organisationen weltweit, sobald diese Personen in der Europäischen Union ansprechen oder Daten von ihnen erheben. Die Verordnung wurde im April 2016 vom Europäischen Parlament verabschiedet und ist am 25. Mai 2018 in Kraft getreten.',
    p2Html: 'Hook0 setzt bestimmte Unterauftragsverarbeiter ein, um die Anwendungsdienste für die Kunden bereitzustellen, wie im Rahmenvertrag oder in den Nutzungsbedingungen unter <a href="./nutzungsbedingungen" class="text-green-400 hover:text-green-300 transition-colors">Nutzungsbedingungen</a> oder unter einer anderen, jeweils gültigen Adresse beschrieben (der « Vertrag »). Begriffe mit Großbuchstaben haben hier dieselbe Bedeutung wie im Vertrag.',
  },
  whatIsPersonalData: {
    title: 'Was sind personenbezogene Daten?',
    bodyHtml: 'Die DSGVO legt besonderen Wert auf den Schutz personenbezogener Daten natürlicher Personen. Personenbezogene Daten (Art. 4 DSGVO) sind alle Informationen, die eine direkte oder indirekte Identifizierung einer Person ermöglichen. Dazu zählen beispielsweise ein Name, eine E-Mail-Adresse, eine Kreditkartennummer oder Dokumente mit persönlichen Angaben.',
  },
  howWeProcess: {
    title: 'Wie wir personenbezogene Daten verarbeiten',
    bodyHtml: 'Wenn Sie unsere Websites besuchen oder unsere Dienste nutzen, verarbeiten wir in der Regel Ihre personenbezogenen Daten in irgendeiner Form. Sämtliche relevanten Informationen zu den verarbeiteten Daten, unserer Rechtsgrundlage und Ihren Rechten finden Sie in unserer <a href="./datenschutzerklaerung" class="text-green-400 hover:text-green-300 transition-colors">Datenschutzerklärung</a>.',
  },
  roles: {
    title: 'Unterauftragsverarbeiter und ihre Rolle',
    p1Html: 'Ein Unterauftragsverarbeiter ist ein von Hook0 beauftragter Drittdienstleister, einschließlich Konzerngesellschaften von Hook0, der Zugang zu Kundendaten haben oder diese verarbeiten kann (Kundendaten können personenbezogene Daten enthalten). Hook0 setzt verschiedene Arten von Unterauftragsverarbeitern für die in den folgenden Tabellen erläuterten Aufgaben ein.',
    p2Html: 'Gemäß Artikel 28 Absatz 2 und Absatz 4 DSGVO erteilen Sie Hook0 eine allgemeine schriftliche Genehmigung zur Beauftragung der nachstehend aufgeführten Unterauftragsverarbeiter. Wir werden Sie über jede geplante Änderung dieser Liste, einschließlich der Hinzufügung oder Ersetzung von Unterauftragsverarbeitern, informieren und Ihnen eine angemessene Frist zur Einlegung eines Widerspruchs vor Wirksamwerden der Änderung einräumen.',
  },
  infrastructure: {
    title: 'Infrastruktur',
    intro: 'Wir setzen die folgenden Unterauftragsverarbeiter für unsere Cloud-Infrastruktur und die Speicherung der Kundendaten ein:',
    table: {
      headers: ['Unterauftragsverarbeiter', 'Verarbeitungsland', 'Zweck', 'Transfermechanismus'],
      rows: [
        {
          name: 'Clever Cloud SAS',
          country: 'Frankreich, Europa',
          countryIsEU: true,
          purpose: 'Hook0-Kundendatenbank, API und Webanwendung',
          transfer: 'EU-Verarbeitung (kein Transfer außerhalb des EWR)',
        },
        {
          name: 'Cloudflare, Inc. (101 Townsend St, San Francisco, CA 94107)',
          country: 'USA',
          countryIsEU: false,
          purpose: 'DNS und DDoS-Schutz',
          transfer: 'SCC 2021 + TIA ; EU-US DPF (Cloudflare ist DPF-zertifiziert)',
        },
      ],
    },
  },
  customerContent: {
    title: 'Verarbeitung der Kundendaten',
    intro: 'Hook0 arbeitet mit verschiedenen Unterauftragsverarbeitern zusammen, die die Anwendungsdienste überwachen, warten und unterstützen. Diese Unterauftragsverarbeiter können, müssen aber nicht zwingend, Zugang zu den Kundendaten haben:',
    table: {
      headers: ['Unterauftragsverarbeiter', 'Land', 'Zweck', 'Transfermechanismus'],
      rows: [
        {
          name: 'Clever Cloud SAS',
          country: 'Frankreich, Europa',
          countryIsEU: true,
          purpose: 'Worker, die die Webhook-Abonnement-Endpunkte aufrufen',
          transfer: 'EU-Verarbeitung (kein Transfer außerhalb des EWR)',
        },
        {
          name: 'Scaleway SAS',
          country: 'Frankreich, Europa',
          countryIsEU: true,
          purpose: 'Private dedizierte Worker, die die Webhook-Abonnement-Endpunkte aufrufen (nur für entsprechende Kunden)',
          transfer: 'EU-Verarbeitung (kein Transfer außerhalb des EWR)',
        },
        {
          name: 'Stripe Inc.',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Abonnementverwaltung der Hook0-Kunden',
          transfer: 'SCC 2021 + TIA ; EU-US DPF (Stripe ist DPF-zertifiziert)',
        },
        {
          name: 'Brevo',
          country: 'Frankreich, Europa',
          countryIsEU: true,
          purpose: 'Versand von Transaktions-E-Mails',
          transfer: 'EU-Verarbeitung (kein Transfer außerhalb des EWR)',
        },
        {
          name: 'Postmark',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Versand von Transaktions-E-Mails',
          transfer: 'SCC 2021 + TIA',
        },
        {
          name: 'BetterUptime',
          country: 'Tschechische Republik, Europa',
          countryIsEU: true,
          purpose: 'Verfügbarkeitsüberwachung, Statusseite und Bereitschaftsverwaltung',
          transfer: 'EU-Verarbeitung (kein Transfer außerhalb des EWR)',
        },
        {
          name: 'Sentry',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Fehler-Tracking',
          transfer: 'SCC 2021 + TIA',
        },
        {
          name: 'Crisp',
          country: 'Frankreich, Europa',
          countryIsEU: true,
          purpose: 'Kundenbeziehungspflege',
          transfer: 'EU-Verarbeitung (kein Transfer außerhalb des EWR)',
        },
        {
          name: 'Gmail (Google Workspace)',
          country: 'USA',
          countryIsEU: false,
          purpose: 'Support-Postfach',
          transfer: 'SCC 2021 + TIA ; EU-US DPF (Google LLC ist DPF-zertifiziert)',
        },
      ],
    },
    footnoteHtml: '* Die obige Liste der Unterauftragsverarbeiter gilt für alle neuen Hook0-Kunden ab dem oben auf dieser Seite angegebenen Datum sowie für bestehende Hook0-Kunden, die keine anderweitige Mitteilung mit einem abweichenden Geltungsbeginn erhalten haben.',
  },
  control: {
    title: 'Sie behalten die Kontrolle',
    bodyHtml: 'Hook0 ist ein französisches SaaS, das auf DSGVO-Konformität ausgelegt ist. Wir setzen auf eine Infrastruktur und Partner, die nach Vertraulichkeit, Integrität und Verfügbarkeit Ihrer Daten ausgewählt wurden. Wenn Sie sich nicht ausschließlich auf unsere Maßnahmen oder die unserer Unterauftragsverarbeiter verlassen möchten, können Sie unsere Support-Dienste weiterhin nutzen, ohne Ihre Produktivdaten offenzulegen.',
  },
  dataOwnership: {
    title: 'Datenhoheit und Verwaltung',
    p1Html: 'Die Datenebene Ihrer Webhook-Payloads (Clever-Cloud-Worker und optional dedizierte Scaleway-Worker) wird in der EU betrieben, und Ihre Webhook-Inhalte werden zum Zweck der Webhook-Zustellung nicht außerhalb des EWR übertragen. Backups werden in französischen Rechenzentren gespeichert. Begleitende Dienste wie Abrechnung (Stripe), Fehler-Tracking (Sentry), Ausweich-Versand für Transaktions-E-Mails (Postmark), Support-Postfach (Gmail) und die CDN- und DDoS-Schutzschicht (Cloudflare) umfassen hingegen Transfers in die Vereinigten Staaten. Diese werden durch die Standardvertragsklauseln 2021 (SCC 2021) und ein dokumentiertes Transfer Impact Assessment (TIA) abgesichert sowie gegebenenfalls durch das EU-US Data Privacy Framework. Sämtliche Hook0-Mitarbeiter und Berater, die Zugang zu Ihrer Bereitstellung haben können, sind in der EU ansässig.',
    p2Html: 'Für Ihre eigene Nutzerdatenbank obliegt es Ihnen, die zur Einhaltung der DSGVO erforderlichen Prozesse einzurichten und sämtliche Datenübermittlungen, die Sie eigenständig vornehmen, zu erklären. In diesem Fall handelt Hook0 als Unterauftragsverarbeiter, und unser <a href="./data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">DPA (Data Processing Agreement)</a> beschreibt unseren Umfang.',
  },
};
