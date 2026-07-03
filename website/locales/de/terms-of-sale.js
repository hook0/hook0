// Per-page strings for terms-of-sale (DE, Allgemeine Geschaeftsbedingungen / AGB B2B).
//
// Register : Siezen formell verpflichtend (« Sie » / « Ihr »), wie fuer jeden
// vertragsrechtlichen Text. Kein Duzen. /humanizer pro angewendet. Kein Em-Dash,
// kein Doppel-Bindestrich als Pivot, kein Mittel-Punkt.
//
// SSPL = « quelloffen (SSPL-1.0) », niemals « Open Source » allein
// (von OSI abgelehnt, UWG-§5-Risiko in DACH).
// Verboten : « 100% souveraen », « keine Daten verlassen die EU »,
// « kein US-Konzern im Stack », « CLOUD Act free ».
//
// Hook0 Hardfacts verbatim ueber alle Locales: FGRibreau SARL, Stammkapital
// 2 000 EUR, Handelsregister La Roche-sur-Yon 850 824 350, USt-IdNr.
// FR27850824350, Sitz 3 rue de l'Aubepine 85110 Chantonnay, verantwortlicher
// Herausgeber David Sferruzza, Hosting Clever Cloud SAS (Frankreich) + CDN
// Cloudflare Inc. (USA) offengelegt, Gerichtsstand Gerichte von La Roche-sur-Yon
// (Art. 48 CPC), Zahlungsverzug L441-10 (3-facher gesetzlicher EZB-Zinssatz)
// zuzueglich Pauschale 40 EUR (D441-5).
module.exports = {
  pageTitle: 'Hook0 - Allgemeine Geschäftsbedingungen',
  pageDescription: 'AGB für Hook0 Webhooks-as-a-Service: Preise, Zahlungskonditionen, Rechnungsstellung und Kündigung für Cloud und On-Premise.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Rechtliches',
    title: 'Allgemeine Geschäftsbedingungen',
    subtitle: 'Kommerzielle Bedingungen für den Bezug der Hook0-Pläne und -Dienste.',
    lastUpdatedLabel: 'Letzte Aktualisierung:',
    lastUpdatedDate: '27. Juni 2026',
  },
  intro: {
    p1Html: 'Die vorliegenden Allgemeinen Geschäftsbedingungen regeln sämtliche Bestellungen und Abonnements, die bei der FGRibreau SARL platziert werden, einer französischen Gesellschaft mit beschränkter Haftung (société à responsabilité limitée) mit einem Stammkapital von 2 000 EUR, eingetragen im Handels- und Gesellschaftsregister La Roche-sur-Yon unter der Nummer 850 824 350, mit Sitz in 3 rue de l\'Aubépine, 85110 Chantonnay, Frankreich, USt-IdNr. FR27850824350 (nachfolgend « Hook0 » oder « wir »), für den Zugang zur Hook0-Plattform und den damit verbundenen Diensten. Verantwortlicher Herausgeber ist David Sferruzza.',
    p2Html: 'Die vorliegenden Allgemeinen Geschäftsbedingungen gelten ausschließlich für Geschäfte zwischen Unternehmen (B2B). Sie sind durch Verweis in die <a href="/terms" class="text-green-400 hover:text-green-300 transition-colors">Allgemeinen Nutzungsbedingungen</a> einbezogen und ergänzen diese. Im Falle eines Widerspruchs zwischen den vorliegenden Allgemeinen Geschäftsbedingungen und den Allgemeinen Nutzungsbedingungen gehen die vorliegenden Allgemeinen Geschäftsbedingungen in kommerziellen und Abrechnungsfragen vor.',
    p3Html: 'Mit der Aufgabe einer Bestellung oder der Aktivierung eines kostenpflichtigen Abonnements erklärt der Kunde ausdrücklich seine vollumfängliche Annahme der vorliegenden Allgemeinen Geschäftsbedingungen.',
  },
  sections: [
    {
      id: 'scope',
      title: '1. Geltungsbereich',
      paragraphs: [
        '<strong class="text-white">1.1.</strong> Die vorliegenden Allgemeinen Geschäftsbedingungen gelten für jedes Abonnement der Hook0 Cloud-Pläne (Developer, Startup, Pro, Enterprise) und der On-Premise-Pläne (Self-hosted, Pro, Enterprise), unabhängig vom Bestellkanal.',
        '<strong class="text-white">1.2.</strong> Sie gelten ausschließlich für gewerbliche Kunden (Unternehmen, Vereine, öffentliche Einrichtungen). Sie gelten nicht für Verbraucher im Sinne des französischen Verbraucherrechts.',
        '<strong class="text-white">1.3.</strong> Allgemeine Einkaufsbedingungen des Kunden sind ausdrücklich ausgeschlossen und entfalten keine Wirkung, auch dann nicht, wenn sie Hook0 nach Annahme der vorliegenden Bedingungen mitgeteilt werden, es sei denn, Hook0 hat ihnen ausdrücklich schriftlich zugestimmt.',
      ],
    },
    {
      id: 'pricing',
      title: '2. Preise',
      paragraphs: [
        '<strong class="text-white">2.1.</strong> Alle Preise verstehen sich netto (zuzüglich der gesetzlichen Umsatzsteuer). Die anwendbare Umsatzsteuer oder gleichwertige Abgaben werden bei der Rechnungsstellung automatisch ergänzt, gemäß den im Etablierungsland des Kunden geltenden Vorschriften.',
        '<strong class="text-white">2.2.</strong> Die aktuellen Preise sämtlicher Pläne sind auf <a href="/pricing" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/pricing</a> veröffentlicht und werden durch Verweis Bestandteil der vorliegenden Bedingungen. Die veröffentlichten Preise können unter den in Abschnitt 9 vorgesehenen Bedingungen geändert werden.',
        '<strong class="text-white">2.3. Cloud-Pläne</strong>, zum Stand der letzten Aktualisierung zur Information:',
      ],
      cloudPlans: [
        '<strong class="text-white">Developer</strong>: kostenlos, 100 Ereignisse/Tag, 7 Tage Aufbewahrung.',
        '<strong class="text-white">Startup</strong>: 59 EUR/Monat netto, 30 000 Ereignisse/Tag; Überschreitung 0,003 EUR je Ereignis.',
        '<strong class="text-white">Pro</strong>: 190 EUR/Monat oder 1 824 EUR/Jahr netto, 100 000 Ereignisse/Tag; Überschreitung 0,0001 EUR je Ereignis.',
        '<strong class="text-white">Enterprise</strong>: Angebot auf Anfrage.',
      ],
      paragraphsBeforeOnPremise: [
        '<strong class="text-white">2.4. On-Premise-Pläne</strong>, zum Stand der letzten Aktualisierung zur Information:',
      ],
      onPremisePlans: [
        '<strong class="text-white">Self-hosted</strong>: kostenlos, quelloffen unter der Server Side Public License v1 (SSPL-1.0).',
        '<strong class="text-white">Pro</strong>: 1 000 EUR Einrichtungsgebühr zuzüglich 500 EUR/Monat oder 6 000 EUR/Jahr netto.',
        '<strong class="text-white">Enterprise</strong>: Angebot auf Anfrage.',
      ],
      paragraphsAfter: [
        '<strong class="text-white">2.5.</strong> Die in den Abschnitten 2.3 und 2.4 genannten Preise dienen rein informatorischen Zwecken. Verbindlich sind die zum Zeitpunkt der Bestellung auf <a href="/pricing" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/pricing</a> ausgewiesenen Preise beziehungsweise die in einem individuellen Angebot für Enterprise-Kunden festgelegten Preise.',
      ],
    },
    {
      id: 'ordering',
      title: '3. Bestellung und Abonnement',
      paragraphs: [
        '<strong class="text-white">3.1.</strong> Abonnements der Self-Service-Pläne (Developer, Startup, Pro) werden direkt über die Hook0-Anwendung auf <a href="https://app.hook0.com" class="text-green-400 hover:text-green-300 transition-colors">app.hook0.com</a> abgeschlossen. Der Kunde wählt den gewünschten Plan und hinterlegt gültige Zahlungsdaten.',
        '<strong class="text-white">3.2.</strong> Für Enterprise- und On-Premise-Pro-Abonnements ist ein individuelles Angebot erforderlich. Der Kunde kann ein Angebot unter <a href="mailto:sales@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">sales@hook0.com</a> anfordern. Der Vertrag kommt mit der schriftlichen Annahme des Angebots durch beide Parteien zustande.',
        '<strong class="text-white">3.3.</strong> Bei Self-Service-Plänen kommt der Vertrag in dem Moment zustande, in dem der Kunde die Bestellung in der Anwendung bestätigt und Hook0 den gewählten Plan aktiviert.',
      ],
    },
    {
      id: 'invoicing',
      title: '4. Rechnungsstellung',
      paragraphs: [
        '<strong class="text-white">4.1.</strong> Die Rechnungsstellung für Self-Service-Pläne wird automatisch über Stripe abgewickelt, den Zahlungsdienstleister von Hook0. Rechnungen werden zu Beginn jedes Abrechnungszeitraums (monatlich oder jährlich, je nach gewähltem Plan) erstellt und stehen im Stripe-Abrechnungsportal des Kunden bereit.',
        '<strong class="text-white">4.2.</strong> Für Enterprise- und On-Premise-Pro-Pläne werden die Rechnungen direkt von Hook0 ausgestellt und an die vom Kunden bei der Bestellung angegebene Rechnungsanschrift gesandt.',
        '<strong class="text-white">4.3.</strong> Jährliche Abonnements werden zu Beginn des Abonnementjahres in voller Höhe in Rechnung gestellt.',
        '<strong class="text-white">4.4.</strong> Überschreitungsentgelte werden, soweit anwendbar, am Ende jedes Abrechnungszeitraums berechnet und entweder mit der Rechnung des Folgezeitraums oder gesondert in Rechnung gestellt, nach Wahl von Hook0.',
      ],
    },
    {
      id: 'payment',
      title: '5. Zahlungsbedingungen',
      paragraphs: [
        '<strong class="text-white">5.1. Self-Service-Pläne (Developer, Startup, Pro).</strong> Die Zahlung ist mit Rechnungsstellung sofort fällig und erfolgt per Lastschrift von der bei Stripe hinterlegten Zahlungskarte. Mit dem Abschluss des Abonnements ermächtigt der Kunde Hook0, die hinterlegte Zahlungsweise gemäß dem gewählten Abrechnungszyklus wiederkehrend zu belasten.',
        '<strong class="text-white">5.2. Enterprise- und On-Premise-Pro-Pläne.</strong> Sofern im jeweiligen Angebot oder Bestellschein nichts anderes vereinbart ist, sind Rechnungen innerhalb von dreißig (30) Tagen ab Rechnungsdatum fällig.',
        '<strong class="text-white">5.3.</strong> Alle Zahlungen erfolgen in Euro (EUR). Etwaige Banküberweisungsgebühren und Währungsumrechnungskosten trägt der Kunde.',
      ],
    },
    {
      id: 'late-payment',
      title: '6. Zahlungsverzug',
      paragraphs: [
        '<strong class="text-white">6.1.</strong> Gemäß Artikel L441-10 des französischen Code de commerce fallen bei jeder zum Fälligkeitstermin nicht bezahlten Forderung von Rechts wegen und ohne vorherige Mahnung Verzugszinsen in Höhe des Dreifachen (3-fach) des von der Europäischen Zentralbank veröffentlichten gesetzlichen Zinssatzes an, berechnet auf den ausstehenden Betrag ab dem Fälligkeitstermin bis zum Zeitpunkt der tatsächlichen Zahlung.',
        '<strong class="text-white">6.2.</strong> Darüber hinaus ist gemäß Artikel D441-5 des französischen Code de commerce eine pauschale Beitreibungsgebühr in Höhe von vierzig Euro (40 EUR) je unbezahlter Rechnung zusätzlich zu den Verzugszinsen vom Kunden geschuldet. Soweit die Hook0 tatsächlich entstandenen Beitreibungskosten diesen Betrag übersteigen, behält sich Hook0 vor, eine darüber hinausgehende Entschädigung gegen Nachweis zu verlangen.',
        '<strong class="text-white">6.3.</strong> Unbeschadet der vorstehenden Bestimmungen behält sich Hook0 vor, den Zugang des Kunden zum Dienst ohne vorherige Ankündigung und ohne Haftung zu sperren oder einzuschränken, wenn eine fällige Forderung nach einer Karenzfrist von sieben (7) Kalendertagen ab Fälligkeit nicht beglichen ist. Die Sperrung des Dienstes entbindet den Kunden nicht von seinen Zahlungspflichten.',
      ],
    },
    {
      id: 'overage',
      title: '7. Überschreitungsentgelte',
      paragraphs: [
        '<strong class="text-white">7.1.</strong> Jeder kostenpflichtige Plan enthält ein tägliches Ereigniskontingent gemäß Abschnitt 2.3. Wird die enthaltene Kontingentmenge überschritten, fallen Überschreitungsentgelte zum für den Kundenplan angegebenen Stückpreis je Ereignis an. Bei kostenpflichtigen Plänen wird die Ereignisannahme bei Überschreitung des Tageskontingents nicht unterbrochen, sodass der Dienst ohne Unterbrechung weiterläuft.',
        '<strong class="text-white">7.2.</strong> Für den Developer-Plan (kostenlos) wird keine Überschreitung in Rechnung gestellt. Ereignisse, die das Tageskontingent übersteigen, werden blockiert, bis das Kontingent am Folgetag um 0:00 Uhr UTC zurücksetzt wird.',
        '<strong class="text-white">7.3.</strong> Überschreitungsentgelte werden automatisch berechnet und gemäß Abschnitt 4.4 in Rechnung gestellt. Der Kunde akzeptiert, dass jede Nutzung des Dienstes über das enthaltene Kontingent hinaus als stillschweigende Bestellung zusätzlicher Kapazitäten zum geltenden Stückpreis je Ereignis gilt.',
        '<strong class="text-white">7.4.</strong> Der Kunde kann seinen Ereignisverbrauch in Echtzeit über das Organisations-Dashboard in der Hook0-Anwendung verfolgen. Hook0 versendet E-Mail-Benachrichtigungen, wenn sich der Tagesverbrauch dem Kontingent annähert.',
      ],
    },
    {
      id: 'plan-changes',
      title: '8. Planwechsel',
      paragraphs: [
        '<strong class="text-white">8.1. Upgrade.</strong> Ein Wechsel in einen höherwertigen Plan tritt sofort nach Bestätigung in Kraft. Dem Kunden wird die anteilige Differenz für den verbleibenden Teil des laufenden Abrechnungszeitraums in Rechnung gestellt.',
        '<strong class="text-white">8.2. Downgrade.</strong> Ein Wechsel in einen niedrigeren Plan tritt zum Ende des laufenden Abrechnungszeitraums in Kraft. Bis zu diesem Zeitpunkt behält der Kunde Zugriff auf die Funktionen und Kontingente des aktuellen Plans. Für den verbleibenden Zeitraum erfolgt keine Rückerstattung.',
        '<strong class="text-white">8.3.</strong> Planwechsel können aus der Hook0-Anwendung heraus oder per Kontaktaufnahme unter <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> ausgelöst werden.',
      ],
    },
    {
      id: 'price-changes',
      title: '9. Preisänderungen',
      paragraphs: [
        '<strong class="text-white">9.1.</strong> Hook0 behält sich vor, seine Preise jederzeit unter Einhaltung einer schriftlichen Ankündigungsfrist von dreißig (30) Tagen zu ändern; die Ankündigung erfolgt an die hinterlegte E-Mail-Adresse des Kunden oder durch Veröffentlichung auf der Hook0-Website.',
        '<strong class="text-white">9.2.</strong> Die neuen Preise gelten für den ersten Abrechnungszeitraum, der nach Ablauf der Ankündigungsfrist beginnt. Preisänderungen wirken nicht rückwirkend auf bereits abgerechnete Zeiträume.',
        '<strong class="text-white">9.3.</strong> Stimmt der Kunde dem neuen Preis nicht zu, kann er sein Abonnement gemäß Abschnitt 10 vor Wirksamwerden des neuen Preises kündigen. Die fortgesetzte Nutzung des Dienstes nach Wirksamwerden der Preisänderung gilt als Annahme der neuen Preise.',
      ],
    },
    {
      id: 'cancellation',
      title: '10. Kündigung',
      paragraphs: [
        '<strong class="text-white">10.1.</strong> Der Kunde kann sein Abonnement jederzeit durch eine Anfrage an <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> kündigen.',
        '<strong class="text-white">10.2.</strong> Die Kündigung wird zum Ende des laufenden Abrechnungszeitraums wirksam. Der Kunde behält bis zu diesem Zeitpunkt Zugriff auf den gewählten Plan.',
        '<strong class="text-white">10.3. Keine Rückerstattung.</strong> Bereits abgerechnete und bezahlte Zeiträume werden unabhängig vom Kündigungsgrund und unabhängig davon, ob der Dienst in diesem Zeitraum tatsächlich genutzt wurde, nicht zurückerstattet. Diese Regel gilt sowohl für monatliche als auch für jährliche Abonnements.',
        '<strong class="text-white">10.4. Datenaufbewahrung nach Kündigung.</strong> Kundendaten werden nach Ende des Abonnements dreißig (30) Tage lang aufbewahrt; in diesem Zeitraum kann der Kunde durch Kontaktaufnahme unter <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> einen Datenexport anfordern. Nach Ablauf dieser dreißigtägigen Frist werden sämtliche Kundendaten endgültig gelöscht.',
        '<strong class="text-white">10.5.</strong> Hook0 kann ein Abonnement bei schwerwiegendem Verstoß des Kunden gegen die Allgemeinen Nutzungsbedingungen, insbesondere bei Nichtzahlung, missbräuchlicher Nutzung oder Verstoß gegen die Acceptable-Use-Regeln, mit sofortiger Wirkung und ohne Rückerstattung kündigen.',
      ],
    },
    {
      id: 'free-plan',
      title: '11. Kostenloser Plan',
      paragraphs: [
        '<strong class="text-white">11.1.</strong> Der Developer-Plan wird kostenfrei bereitgestellt. Er stellt keine kommerzielle Zusage von Hook0 dar und kann nach freiem Ermessen von Hook0 geändert, eingeschränkt oder eingestellt werden. Mit dem Developer-Plan ist keine Service-Level-Vereinbarung (SLA) verbunden; ein individuelles SLA kann ausschließlich mit Enterprise-Kunden verhandelt werden.',
        '<strong class="text-white">11.2.</strong> Hook0 kündigt die Einstellung des kostenlosen Developer-Plans oder eine wesentliche Reduzierung seiner enthaltenen Kontingente mindestens neunzig (90) Tage im Voraus an. Die Ankündigung erfolgt an die hinterlegte E-Mail-Adresse und/oder durch Veröffentlichung auf der Hook0-Website.',
      ],
    },
    {
      id: 'taxes',
      title: '12. Steuern',
      paragraphs: [
        '<strong class="text-white">12.1.</strong> Alle Preise verstehen sich netto (zuzüglich der gesetzlichen Umsatzsteuer). Die anwendbare Umsatzsteuer oder gleichwertige indirekte Abgaben werden dem Rechnungsbetrag hinzugerechnet und von Hook0 oder über Stripe erhoben, gemäß den im Etablierungsland des Kunden geltenden Steuervorschriften.',
        '<strong class="text-white">12.2.</strong> Gewerbliche Kunden mit Sitz in einem anderen EU-Mitgliedstaat als Frankreich können von der französischen Umsatzsteuer befreit sein, sofern sie eine gültige innergemeinschaftliche USt-IdNr. mitteilen. Es obliegt dem Kunden, in seinen Kontoeinstellungen aktuelle und korrekte Steuerangaben zu hinterlegen.',
        '<strong class="text-white">12.3.</strong> Kunden mit Sitz außerhalb der Europäischen Union sind für Einfuhrabgaben, Quellensteuern oder sonstige in ihrem Land anwendbare Abgaben verantwortlich. Hook0 zieht keine Steuern für ausländische Steuerbehörden ein, außer dies ist gesetzlich vorgeschrieben.',
      ],
    },
    {
      id: 'subprocessors',
      title: '13. Infrastruktur und Unterauftragsverarbeiter',
      paragraphs: [
        '<strong class="text-white">13.1.</strong> Hook0 Cloud stützt sich auf Drittanbieter-Infrastrukturen, insbesondere das Hosting durch Clever Cloud SAS (Frankreich) sowie die Inhaltsauslieferung und die Edge-Sicherheit durch Cloudflare Inc. (Vereinigte Staaten). Die aktuelle Liste der Unterauftragsverarbeiter und deren Standorte ist auf <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a> veröffentlicht und durch Verweis Bestandteil dieser Bedingungen.',
        '<strong class="text-white">13.2.</strong> Übermittlungen personenbezogener Daten an Unterauftragsverarbeiter außerhalb des Europäischen Wirtschaftsraums werden durch den <a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Auftragsverarbeitungsvertrag (AVV)</a> geregelt, der den jeweils geltenden Übermittlungsmechanismus (Standardvertragsklauseln oder, soweit anwendbar, EU-US Data Privacy Framework) angibt.',
      ],
    },
    {
      id: 'law',
      title: '14. Anwendbares Recht und Gerichtsstand',
      paragraphs: [
        '<strong class="text-white">14.1.</strong> Die vorliegenden Allgemeinen Geschäftsbedingungen unterliegen ausschließlich französischem Recht. Die Anwendung des UN-Übereinkommens über Verträge über den internationalen Warenkauf (CISG) ist ausdrücklich ausgeschlossen.',
        '<strong class="text-white">14.2.</strong> Gemäß Artikel 48 des französischen Code de procédure civile, der zwischen Kaufleuten gilt, vereinbaren die Parteien, dass sämtliche Streitigkeiten aus oder im Zusammenhang mit den vorliegenden Allgemeinen Geschäftsbedingungen der ausschließlichen Zuständigkeit der Gerichte von La Roche-sur-Yon, Frankreich, unterliegen, in deren Sprengel Hook0 seinen Sitz hat, unbeschadet mehrerer Beklagter oder einer Streitverkündung und vorbehaltlich zwingender Zuständigkeitsregeln für den jeweiligen Streitgegenstand. Die Parteien bemühen sich zunächst um eine gütliche Beilegung; kommt innerhalb von dreißig (30) Tagen nach schriftlicher Mitteilung der Streitigkeit keine gütliche Einigung zustande, kann die Streitigkeit von jeder Partei den vorgenannten Gerichten vorgelegt werden.',
      ],
    },
    {
      id: 'contact',
      title: '15. Kontakt',
      lead: 'Bei Fragen zu den vorliegenden Allgemeinen Geschäftsbedingungen, zur Rechnungsstellung oder zu kommerziellen Themen wenden Sie sich bitte an:',
      contactItems: [
        '<strong class="text-white">Rechtliches:</strong> <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
        '<strong class="text-white">Rechnungen und Abonnements:</strong> <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>',
        '<strong class="text-white">Enterprise-Vertrieb:</strong> <a href="mailto:sales@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">sales@hook0.com</a>',
        '<strong class="text-white">Geschäftssitz:</strong> FGRibreau SARL, 3 rue de l\'Aubépine, 85110 Chantonnay, France',
      ],
    },
  ],
};
