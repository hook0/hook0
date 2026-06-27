// Per-page strings for privacy-policy (DE, Datenschutzerklärung, Art. 13
// DSGVO). Übersetzung aus dem korrigierten EN-Basisinhalt (lokales
// en/privacy-policy.js) ; DE existierte vorher nicht in der bilingualen
// EN+FR-Legacy-Vorlage und wird mit dieser Konvertierung neu eingeführt.
//
// Register : Siezen formell strikt (« Sie » / « Ihr »), wie für jeden
// vertragsrechtlichen Text. Kein Duzen. /humanizer pro angewendet. Kein
// Em-Dash, kein Doppel-Bindestrich als Pivot, kein Mittel-Punkt.
//
// Harte Regeln (CLAUDE.local.md) :
//   - DSGVO niemals absolut (« 100 % DSGVO-konform »), nur als Prozess-Claim
//     (« auf DSGVO-Konformität ausgelegt »).
//   - Verboten : « 100 % souverän », « keine Daten verlassen die EU »,
//     « kein US-Konzern im Stack », « CLOUD Act free ».
//   - SSPL = « quelloffen (SSPL-1.0) » (auf dieser Seite nicht erwähnt).
//
// Hook0 Hardfacts verbatim über alle Locales :
//   - Verantwortlicher : FGRibreau SARL, Stammkapital 2 000 EUR,
//     RCS La Roche-sur-Yon 850 824 350, USt-ID FR27850824350, Geschäftsadresse
//     3 rue de l'Aubépine, 85110 Chantonnay, Frankreich.
//   - Verantwortlicher für die Veröffentlichung : David Sferruzza.
//   - Datenschutz-Kontakt : legal@hook0.com (mit dem DPA abgestimmt).
//   - Unterauftragsverarbeiter (gleicher Satz wie in gdpr-subprocessors.js) :
//       Clever Cloud (FR), Scaleway (FR), Cloudflare (USA),
//       Stripe (USA), Brevo (FR), Postmark (USA), BetterUptime (CZ),
//       Sentry (USA), Crisp (FR), Gmail/Google Workspace (USA),
//       Google LLC / Google Ads (USA, serverseitige Konversionsmessung).
//   - Transfermechanismen : SCC 2021 + TIA ; EU-US DPF für zertifizierte
//     Unterauftragsverarbeiter (Cloudflare, Stripe, Google LLC).
//   - Aufbewahrung je Tarif : Developer 7 Tage, Startup 14 Tage,
//     Pro 30 Tage, Enterprise individuell (information-retention-policy.md).
//   - Aufbewahrung Kontodaten : Vertragsdauer + 30 Tage nach Löschung.
//   - Aufbewahrung Rechnungsbelege : 10 Jahre (französisches Steuerrecht,
//     art. L102 B Livre des procédures fiscales).
//   - Meldung von Datenschutzverletzungen : 72 Stunden (Art. 33/34 DSGVO).
//   - TTL Cookie-Einwilligung : 13 Monate maximal (CNIL-Leitlinie).
module.exports = {
  pageTitle: 'Hook0 - Datenschutzerklärung',
  pageDescription: 'Datenschutzerklärung von Hook0, auf DSGVO-Konformität ausgelegt (Art. 13 DSGVO). Rechtsgrundlagen, Aufbewahrungsfristen, Ihre Rechte, Unterauftragsverarbeiter und Drittstaatentransfers.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Rechtliches',
    title: 'Datenschutzerklärung',
    subtitle: 'Wie Hook0 Ihre personenbezogenen Daten erhebt, verwendet und schützt, im Einklang mit Art. 13 DSGVO.',
    lastUpdatedLabel: 'Letzte Aktualisierung:',
    lastUpdatedDate: '27. Juni 2026',
  },
  controller: {
    title: '1. Verantwortlicher',
    p1: 'Der für die Verarbeitung Ihrer personenbezogenen Daten im Rahmen des Hook0-Dienstes Verantwortliche ist:',
    identityHtml: '<strong class="text-white">FGRibreau SARL</strong>, eine Gesellschaft mit beschränkter Haftung nach französischem Recht (Société à Responsabilité Limitée) mit einem Stammkapital von 2 000 EUR, eingetragen im Handels- und Gesellschaftsregister von La Roche-sur-Yon unter der Nummer 850 824 350, USt-ID FR27850824350, mit Geschäftsadresse 3 rue de l\'Aubépine, 85110 Chantonnay, Frankreich.<br>Verantwortlicher für die Veröffentlichung: David Sferruzza.<br>Datenschutz-Kontakt: <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
    note: 'Hook0 ist eine ausschließlich für Geschäftskunden (B2B) bestimmte SaaS-Plattform. Wir erheben nicht absichtlich Daten von Personen, die in privater Eigenschaft handeln.',
  },
  purposes: {
    title: '2. Zwecke und Rechtsgrundlagen',
    intro: 'Die folgende Tabelle beschreibt jede Verarbeitungstätigkeit, die betroffenen Daten und die anwendbare Rechtsgrundlage gemäß Art. 6 DSGVO.',
    headers: ['Zweck', 'Datenkategorien', 'Rechtsgrundlage (Art. 6 DSGVO)'],
    rows: [
      {
        purposeHtml: '<strong class="text-white">Erbringung des Dienstes</strong><br><span class="text-gray-400 text-sm">Kontoerstellung, Authentifizierung, API-Zugang, Zustellung von Webhooks</span>',
        data: 'E-Mail-Adresse, Name, API-Schlüssel, Webhook-Payloads, IP-Adresse, Nutzungsprotokolle',
        basisHtml: 'Art. 6 Abs. 1 lit. b, Erfüllung des Vertrags',
      },
      {
        purposeHtml: '<strong class="text-white">Abrechnung und Zahlung</strong><br><span class="text-gray-400 text-sm">Verwaltung des Abonnements, Ausstellung von Rechnungen, steuerliche Pflichten</span>',
        data: 'Name, E-Mail, Rechnungsadresse, Zahlungsmittel-Daten (verarbeitet durch Stripe), Abonnementverlauf',
        basisHtml: 'Art. 6 Abs. 1 lit. b, Erfüllung des Vertrags<br>Art. 6 Abs. 1 lit. c, rechtliche Verpflichtung (französisches Steuerrecht, 10-jährige Aufbewahrung)',
      },
      {
        purposeHtml: '<strong class="text-white">Website-Analytik</strong><br><span class="text-gray-400 text-sm">Reichweitenmessung über Matomo (selbst gehostet)</span>',
        data: 'Anonymisierte IP-Adresse, besuchte Seiten, Verweisquelle, Gerätetyp, Sitzungsdauer',
        basisHtml: 'Art. 6 Abs. 1 lit. a, Einwilligung (Cookie-Banner)',
      },
      {
        purposeHtml: '<strong class="text-white">Conversion-Tracking (serverseitig)</strong><br><span class="text-gray-400 text-sm">Konversionsmessung für Google Ads, ausschließlich serverseitig über die Klick-Kennung (gclid). Es werden keine E-Mail-Adresse, keine IP-Adresse und kein User-Agent an Google übermittelt. Widerspruchsrecht an <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>.</span>',
        data: 'Klick-Kennung (gclid), pseudonyme Kennung, die Google beim Anzeigenklick erzeugt',
        basisHtml: 'Art. 6 Abs. 1 lit. f, berechtigte Interessen (Messung des Werbeerfolgs)<br><span class="text-gray-400 text-sm">Widerspruchsrecht nach Art. 21 Abs. 2 DSGVO</span>',
      },
      {
        purposeHtml: '<strong class="text-white">Kundensupport, Live-Chat</strong><br><span class="text-gray-400 text-sm">Crisp-Widget (wird erst nach Einwilligung geladen)</span>',
        data: 'Name, E-Mail, Chat-Nachrichten, Browser-Metadaten',
        basisHtml: 'Art. 6 Abs. 1 lit. a, Einwilligung',
      },
      {
        purposeHtml: '<strong class="text-white">Kundensupport, E-Mail</strong><br><span class="text-gray-400 text-sm">Bearbeitung von Anfragen an legal@hook0.com oder support@hook0.com</span>',
        data: 'Name, E-Mail, Inhalt der Korrespondenz',
        basisHtml: 'Art. 6 Abs. 1 lit. f, berechtigte Interessen (Beantwortung von Kundenanfragen)',
      },
      {
        purposeHtml: '<strong class="text-white">Sicherheit und Überwachung</strong><br><span class="text-gray-400 text-sm">Fehler-Tracking, Verfügbarkeitsüberwachung, DDoS-Schutz, Incident-Response</span>',
        data: 'IP-Adresse, Fehler-Stacktraces, Anfrage-Metadaten, Ergebnisse von Verfügbarkeitssonden',
        basisHtml: 'Art. 6 Abs. 1 lit. f, berechtigte Interessen (Sicherstellung von Integrität und Sicherheit des Dienstes)',
      },
      {
        purposeHtml: '<strong class="text-white">Kommerzielle Kommunikation</strong><br><span class="text-gray-400 text-sm">Produkt-Updates, Release Notes, Newsletter</span>',
        data: 'E-Mail-Adresse, Vorname',
        basisHtml: 'Art. 6 Abs. 1 lit. a, Einwilligung',
      },
    ],
  },
  dataCategories: {
    title: '3. Kategorien personenbezogener Daten',
    items: [
      '<strong class="text-white">Identitätsdaten:</strong> Vorname, Nachname, berufliche E-Mail-Adresse',
      '<strong class="text-white">Kontodaten:</strong> Benutzername, verschlüsseltes Passwort, API-Schlüssel',
      '<strong class="text-white">Zahlungsdaten:</strong> Rechnungsadresse, letzte 4 Ziffern der Karte und Ablaufdatum (vollständige Kartendaten werden von Stripe gespeichert, Hook0 hat keinen Zugriff auf vollständige Kartennummern)',
      '<strong class="text-white">Technische Daten:</strong> IP-Adresse, User-Agent des Browsers, Verbindungs-Zeitstempel, Fehlerprotokolle',
      '<strong class="text-white">Nutzungsdaten:</strong> gesendete und empfangene Webhook-Events, API-Aufrufvolumen, Nutzungsmetriken der Funktionen',
      '<strong class="text-white">Kommunikation:</strong> Inhalt der Support-Korrespondenz, Chat-Verläufe',
    ],
    note: 'Hook0 verarbeitet keine besonderen Kategorien personenbezogener Daten (Art. 9 DSGVO) und führt keine automatisierte Entscheidungsfindung oder Profilbildung mit rechtlichen oder ähnlich erheblichen Auswirkungen durch.',
  },
  subprocessors: {
    title: '4. Empfänger und Unterauftragsverarbeiter',
    introHtml: 'Wir geben Daten an unsere Unterauftragsverarbeiter nur in dem für die Erbringung des Dienstes erforderlichen Umfang weiter. Die vollständige und aktuelle Liste finden Sie unter <a href="./dsgvo-unterauftragsverarbeiter" class="text-green-400 hover:text-green-300 transition-colors">/dsgvo-unterauftragsverarbeiter</a>. Eine Zusammenfassung folgt unten.',
    groups: [
      {
        title: 'Infrastruktur',
        headers: ['Unterauftragsverarbeiter', 'Land', 'Zweck'],
        rows: [
          ['Clever Cloud SAS', 'Frankreich (EU)', 'Datenbank, API und Webanwendung Hosting'],
          ['Cloudflare, Inc.', 'USA', 'DNS und DDoS-Schutz'],
        ],
      },
      {
        title: 'Betrieb des Dienstes',
        headers: ['Unterauftragsverarbeiter', 'Land', 'Zweck'],
        rows: [
          ['Clever Cloud SAS', 'Frankreich (EU)', 'Workers, die die Webhook-Abonnement-Endpunkte aufrufen'],
          ['Scaleway SAS', 'Frankreich (EU)', 'Dedizierte private Workers (ausgewählte Tarife)'],
          ['Stripe, Inc.', 'USA', 'Abonnement- und Zahlungsverwaltung'],
          ['Brevo (Sendinblue)', 'Frankreich (EU)', 'Automatisierte transaktionale E-Mails'],
          ['Postmark (ActiveCampaign)', 'USA', 'Automatisierte transaktionale E-Mails'],
          ['BetterUptime', 'Tschechische Republik (EU)', 'Verfügbarkeitsüberwachung und Statusseite'],
          ['Sentry, Inc.', 'USA', 'Anwendungs-Fehler-Tracking'],
          ['Crisp', 'Frankreich (EU)', 'Kundensupport-Chat (an Einwilligung gebunden)'],
          ['Google LLC (Gmail)', 'USA', 'Support-Postfach'],
        ],
      },
      {
        title: 'Marketing-Messung (berechtigtes Interesse, serverseitig)',
        headers: ['Unterauftragsverarbeiter', 'Land', 'Zweck'],
        rows: [
          ['Google LLC (Google Ads)', 'USA', 'Serverseitige Konversionsmessung (nur gclid). Siehe Abschnitt 9b.'],
        ],
      },
      {
        title: 'Analytik (an Einwilligung gebunden)',
        headers: ['Dienst', 'Land', 'Zweck'],
        rows: [
          ['Matomo (selbst gehostet auf matomo.hook0.com)', 'Frankreich (EU)', 'Website-Analytik'],
        ],
      },
    ],
    note: 'Mit jedem Unterauftragsverarbeiter besteht ein Auftragsverarbeitungsvertrag (DPA). Zu Transfers außerhalb der EU siehe Abschnitt 5.',
  },
  transfers: {
    title: '5. Transfers außerhalb der Europäischen Union',
    p1Html: 'Mehrere Unterauftragsverarbeiter sind in den Vereinigten Staaten ansässig, namentlich Cloudflare, Stripe, Postmark, Sentry, Gmail (Google) und Google Ads. Diese Übermittlungen erfolgen auf der Grundlage der von der Europäischen Kommission erlassenen <strong class="text-white">Standardvertragsklauseln (SCC)</strong> (Beschluss 2021/914) sowie eines dokumentierten Transfer Impact Assessment und, soweit anwendbar, des <strong class="text-white">EU-US Data Privacy Framework</strong> (Cloudflare, Stripe und Google LLC sind DPF-zertifiziert). Zusammen gewährleisten diese Mechanismen ein angemessenes Schutzniveau für personenbezogene Daten.',
    cloudActHtml: '<strong>Hinweis CLOUD Act:</strong> In den USA ansässige Anbieter können dem CLOUD Act (Clarifying Lawful Overseas Use of Data Act) unterliegen, der US-Behörden gestatten kann, Zugriff auf Daten zu verlangen, die von diesen Anbietern verwahrt werden, auch wenn diese außerhalb der USA gespeichert sind. Hook0 verfolgt einen Ansatz der Datenminimierung und beschränkt die an US-Unterauftragsverarbeiter weitergegebenen personenbezogenen Daten auf das strikt Erforderliche.',
  },
  retention: {
    title: '6. Aufbewahrungsfristen',
    headers: ['Datenkategorie', 'Aufbewahrungsfrist', 'Begründung'],
    rows: [
      ['Kontodaten', 'Vertragsdauer + 30 Tage nach Kontolöschung', 'Vertragliche Notwendigkeit; 30-tägige Karenzfrist, um den Datenexport zu ermöglichen'],
      ['Buchhaltungs- und Rechnungsbelege', '10 Jahre ab dem Datum der Transaktion', 'Rechtliche Verpflichtung, französisches Steuergesetzbuch (Code général des impôts), art. L102 B Livre des procédures fiscales'],
      ['Webhook-Event-Protokolle', 'Developer 7 Tage, Startup 14 Tage, Pro 30 Tage, Enterprise individuell', 'Erbringung des Dienstes; je Tarif konfigurierbar'],
      ['Website-Analytik (Matomo)', '25 Monate', 'CNIL-Empfehlung für Analytik-Daten'],
      ['Support-Korrespondenz', '3 Jahre nach dem letzten Austausch', 'Berechtigte Interessen; allgemeine Verjährungsfrist für vertragliche Ansprüche'],
      ['Einwilligungs-Nachweise', '5 Jahre ab Erteilung der Einwilligung', 'Nachweisfähigkeit der Konformität (Art. 7 Abs. 1 DSGVO)'],
      ['Server-Protokolle', 'mindestens 30 Tage, danach automatische Rotation und Löschung', 'Betrieb des Dienstes, Sicherheit und Incident-Response'],
    ],
  },
  rights: {
    title: '7. Ihre Rechte',
    intro: 'Nach der DSGVO stehen Ihnen in Bezug auf Ihre personenbezogenen Daten folgende Rechte zu:',
    items: [
      '<strong class="text-white">Auskunftsrecht</strong> (Art. 15), eine Kopie der über Sie gespeicherten personenbezogenen Daten erhalten',
      '<strong class="text-white">Recht auf Berichtigung</strong> (Art. 16), unrichtige oder unvollständige Daten korrigieren',
      '<strong class="text-white">Recht auf Löschung</strong> (Art. 17), Löschung Ihrer Daten verlangen, vorbehaltlich gesetzlicher Aufbewahrungspflichten',
      '<strong class="text-white">Recht auf Einschränkung der Verarbeitung</strong> (Art. 18), die Einschränkung der Verarbeitung unter bestimmten Voraussetzungen verlangen',
      '<strong class="text-white">Recht auf Datenübertragbarkeit</strong> (Art. 20), Ihre Daten in einem strukturierten und maschinenlesbaren Format erhalten, wenn die Verarbeitung auf Einwilligung oder Vertrag beruht',
      '<strong class="text-white">Widerspruchsrecht</strong> (Art. 21), Widerspruch gegen die auf berechtigten Interessen beruhende Verarbeitung oder gegen Direktwerbung einlegen',
      '<strong class="text-white">Recht auf Widerruf der Einwilligung</strong> (Art. 7 Abs. 3), Ihre Einwilligung jederzeit widerrufen, ohne dass die Rechtmäßigkeit der vorausgegangenen Verarbeitung berührt wird',
    ],
    contactHtml: 'Zur Ausübung dieser Rechte richten Sie Ihre Anfrage bitte an <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>. Wir antworten innerhalb von 30 Tagen. Wir können vor Bearbeitung Ihrer Anfrage einen Identitätsnachweis verlangen.',
  },
  cnil: {
    title: '8. Beschwerderecht bei der CNIL',
    p1: 'Sind Sie der Auffassung, dass die Verarbeitung Ihrer personenbezogenen Daten gegen die DSGVO verstößt, steht Ihnen das Recht zu, eine Beschwerde bei der zuständigen Aufsichtsbehörde einzulegen:',
    addressHtml: '<strong class="text-white">Commission Nationale de l\'Informatique et des Libertés (CNIL)</strong><br>3 Place de Fontenoy, TSA 80715<br>75334 Paris Cedex 07, Frankreich<br>Website: <a href="https://www.cnil.fr" class="text-green-400 hover:text-green-300 transition-colors" target="_blank" rel="noopener">www.cnil.fr</a>',
    note: 'Sie können sich darüber hinaus an jede Aufsichtsbehörde im Mitgliedstaat Ihres gewöhnlichen Aufenthaltsorts oder Arbeitsplatzes innerhalb der Europäischen Union wenden.',
  },
  cookies: {
    title: '9. Cookies und Tracker',
    intro: 'Hook0 verfügt auf seiner Website über einen Mechanismus zur Einwilligungsverwaltung. Die folgenden Dienste werden erst nach Ihrer ausdrücklichen Einwilligung geladen:',
    items: [
      '<strong class="text-white">Matomo Analytics</strong> (selbst gehostet), Reichweitenmessung, standardmäßig anonymisiert',
      '<strong class="text-white">Crisp</strong>, Live-Chat-Widget',
      '<strong class="text-white">Cookie hook0_gclid</strong> (Domain <code class="text-green-400">.hook0.com</code>, Laufzeit 30 Tage), überträgt die Google-Ads-Klick-Kennung zwischen www.hook0.com und app.hook0.com, damit eine spätere Anmeldung weiterhin attribuiert werden kann. Wird ausschließlich nach Einwilligung gesetzt und nur, wenn Sie über einen Anzeigenklick gekommen sind. Wird beim Widerruf der Einwilligung gelöscht. Einzelheiten in Abschnitt 9b.',
    ],
    consentScopeHtml: 'Ihre auf www.hook0.com erteilte Einwilligung erstreckt sich auf alle hook0.com-Subdomains (einschließlich app.hook0.com). Die Einwilligungspräferenzen werden im <code class="text-green-400">localStorage</code> des Browsers mit einer Gültigkeit von <strong class="text-white">13 Monaten</strong> gespeichert, im Einklang mit den CNIL-Leitlinien. Sie können Ihre Präferenzen jederzeit ändern.',
    changeButton: 'Cookie-Einstellungen ändern',
  },
  serverSideTracking: {
    title: '9b. Serverseitige Konversionsmessung (Google Ads)',
    intro: 'Wenn Sie unseren Dienst über einen Klick auf eine Google-Ads-Anzeige erreichen, fügt Google Ads automatisch eine Klick-Kennung («gclid») an die Ziel-URL an. Diese gclid wird im Rahmen Ihrer Kontoerstellung an unser Backend übermittelt und serverseitig an Google Ads übertragen, um die Wirksamkeit unserer Werbekampagnen zu messen.',
    items: [
      '<strong class="text-white">Zweck</strong>, Messung der Akquisitionskosten unserer bezahlten Kampagnen, um das Marketing-Budget zuzuweisen.',
      '<strong class="text-white">Rechtsgrundlage</strong>, Art. 6 Abs. 1 lit. f DSGVO, berechtigte Interessen. Dokumentierte Interessenabwägung auf Anfrage verfügbar.',
      '<strong class="text-white">An Google übermittelte Daten</strong>, namentlich gclid, Konversionstyp sowie Datum und Uhrzeit der Konversion. <strong>Es werden keine E-Mail-Adresse, keine IP-Adresse und kein User-Agent</strong> in diesem Rahmen an Google übermittelt.',
      '<strong class="text-white">Gemeinsam Verantwortlicher</strong>, Google LLC, im Rahmen der Customer Data Processing Terms (Art. 26 DSGVO). Der Transfer in die USA ist durch die Standardvertragsklauseln (Beschluss 2021/914) und, soweit anwendbar, durch das EU-US Data Privacy Framework abgesichert (Google LLC ist DPF-zertifiziert).',
      '<strong class="text-white">Aufbewahrung</strong>, die gclid wird während der HTTP-Anmeldeanfrage im Arbeitsspeicher verarbeitet und nach der Übertragung an Google Ads nicht in unseren Datenbanken gespeichert.',
      '<strong class="text-white">Widerspruchsrecht</strong> nach Art. 21 Abs. 2 DSGVO. Sie können dieser Verarbeitung jederzeit widersprechen, indem Sie an <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a> schreiben. Wir vermerken den Widerspruch auf Ihrem Konto, damit Ihre gclid nicht an Google Ads übertragen wird. Ihre Anmeldung wird dadurch nicht beeinträchtigt.',
    ],
    footnoteHtml: 'Hinweis: Diese serverseitige Messung beruht <strong>nicht</strong> auf Cookies, gtag.js oder einem clientseitigen Tracker. Art. 82 des französischen Datenschutzgesetzes (Umsetzung von Art. 5 Abs. 3 der ePrivacy-Richtlinie) findet auf diese Verarbeitung keine Anwendung.',
  },
  security: {
    title: '10. Sicherheit',
    p1: 'Hook0 setzt geeignete technische und organisatorische Maßnahmen ein, um personenbezogene Daten vor zufälligem Verlust, unbefugtem Zugriff, Offenlegung, Veränderung oder Zerstörung zu schützen. Dazu zählen Verschlüsselung bei der Übertragung (TLS 1.2+), Verschlüsselung im Ruhezustand, Zugriffskontrollen und regelmäßige Sicherheitsüberprüfungen.',
    p2Html: 'Einzelheiten zu unseren Sicherheitspraktiken finden Sie auf unserer <a href="./sicherheit" class="text-green-400 hover:text-green-300 transition-colors">Sicherheits-Seite</a>.',
    p3Html: 'Bei einer Verletzung des Schutzes personenbezogener Daten, die voraussichtlich ein Risiko für Ihre Rechte und Freiheiten zur Folge hat, benachrichtigen wir die CNIL innerhalb von 72 Stunden (Art. 33 DSGVO) und die betroffenen Personen unverzüglich, soweit dies erforderlich ist (Art. 34 DSGVO). Falls Ihnen ein möglicher Datenabfluss auffällt, melden Sie ihn bitte unverzüglich an <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>.',
  },
  changes: {
    title: '11. Änderungen dieser Erklärung',
    p1: 'Wir können diese Datenschutzerklärung von Zeit zu Zeit aktualisieren. Geschieht dies, aktualisieren wir das Datum «Letzte Aktualisierung» am Anfang dieser Seite. Bei wesentlichen Änderungen informieren wir Sie per E-Mail an die mit Ihrem Konto verknüpfte Adresse oder durch einen deutlich sichtbaren Hinweis auf der Website, mindestens 30 Tage vor dem Wirksamwerden der Änderung.',
  },
};
