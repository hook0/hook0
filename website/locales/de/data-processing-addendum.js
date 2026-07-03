// Per-page strings for data-processing-addendum (DE, DPA Art. 28 DSGVO).
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
//   - Auftragsverarbeiter (Processor) : FGRibreau SARL, Stammkapital 2 000 EUR,
//     RCS La Roche-sur-Yon 850 824 350, USt-ID FR27850824350, Geschäftsadresse
//     3 rue de l'Aubépine, 85110 Chantonnay, Frankreich.
//   - Verantwortlicher (Controller) : der Kunde.
//   - Unterauftragsverarbeiter in Anhang 1 (konsistent mit der dedizierten
//     DSGVO-Unterauftragsverarbeiter-Seite) :
//       * Clever Cloud SAS (Frankreich) : primäre Datenebene
//       * Cloudflare, Inc. (USA, 101 Townsend St, San Francisco, CA 94107) :
//         CDN und DDoS-Schutz, abgesichert durch SCC 2021 + TIA / EU-US DPF
//   - Meldung von Datenschutzverletzungen : 72 Stunden (Art. 33/34 DSGVO).
//   - Backups : täglich, 30 Tage Aufbewahrung.
//   - Passwort-Hashing : Argon2.
//   - MFA : aktiviert für alle Infrastrukturzugänge (Clever Cloud, GitLab,
//     Stripe) ; noch nicht für individuelle Kundenkonten, geplant für eine
//     spätere Version.
module.exports = {
  pageTitle: 'Hook0 - Auftragsverarbeitungsvertrag (DPA)',
  pageDescription: 'Hook0 DPA: DSGVO-Verarbeitungsvorgänge, Sicherheitsmaßnahmen und Verwaltung der Unterauftragsverarbeiter.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Rechtliches',
    title: 'Auftragsverarbeitungsvertrag',
    subtitle: 'Unsere Verpflichtung zum Schutz Ihrer Daten gemäß DSGVO und den geltenden Datenschutzvorschriften.',
    lastUpdatedLabel: 'Letzte Aktualisierung:',
    lastUpdatedDate: '27. Juni 2026',
  },
  preamble: {
    title: 'Bedingungen der Auftragsverarbeitung',
    partiesHtml: 'Dieser Auftragsverarbeitungsvertrag (das « <strong>DPA</strong> ») wird zwischen dem Kunden, der als Verantwortlicher handelt, und der <strong>FGRibreau SARL</strong>, einer Gesellschaft mit beschränkter Haftung nach französischem Recht (Société à Responsabilité Limitée) mit einem Stammkapital von 2 000 EUR, eingetragen im Handels- und Gesellschaftsregister von La Roche-sur-Yon unter der Nummer 850 824 350, mit Sitz in 3 rue de l\'Aubépine, 85110 Chantonnay, Frankreich, die als Auftragsverarbeiter handelt (bezeichnet als « <strong>Hook0</strong> » oder der « <strong>Auftragsverarbeiter</strong> »), geschlossen.',
    p1: 'Dieses DPA hat zum Zweck, die Vereinbarung der Parteien hinsichtlich der Verarbeitung personenbezogener Daten gemäß den Anforderungen der Datenschutzvorschriften wiederzugeben.',
    p2: 'In Bezug auf die Verarbeitung der personenbezogenen Daten des Kunden durch Hook0 im Rahmen der Nutzungsbedingungen erkennen die Parteien an, dass der Kunde der Verantwortliche und Hook0 der Auftragsverarbeiter ist, und beide kommen überein, sämtliche entsprechenden Pflichten gemäß den Datenschutzvorschriften zu erfüllen.',
    p3: 'Der Kunde erteilt Hook0 die Weisung, diese personenbezogenen Daten in seinem Auftrag zu verarbeiten, soweit dies für die Zwecke der Nutzungsbedingungen erforderlich ist und in Anhang 1 « Beschreibung der Verarbeitung personenbezogener Daten » definiert ist. Anhang 1 wird vom Kunden ausgefüllt und ist bei jeder vom Kunden vorgenommenen Änderung zu aktualisieren.',
  },
  section1: {
    title: '1. Einhaltung der Datenschutzvorschriften',
    p1: 'Jede Partei kommt ihren Pflichten gemäß den Datenschutzvorschriften nach.',
    p2: 'Alle in diesem DPA großgeschriebenen Begriffe haben die Bedeutung, die ihnen in der DSGVO, in den Datenschutzvorschriften und in den Nutzungsbedingungen zugewiesen wird.',
  },
  section2: {
    title: '2. Verarbeitungsvorgänge im Rahmen des DPA',
    p1: 'Zur Erinnerung, für jede im Rahmen dieses DPA durchgeführte Verarbeitung muss der Kunde:',
    items: [
      'die Weisungen zu den personenbezogenen Daten dokumentieren,',
      'die für das Ausfüllen von Anhang 1 erforderlichen Informationen zur Verarbeitung bereitstellen, indem er Hook0 unter der Support-Adresse kontaktiert: <a href="mailto:support@hook0.com">support@hook0.com</a>.',
    ],
    p2: 'Der Kunde sichert Hook0 zu, dass er berechtigt ist, die personenbezogenen Daten an Hook0 und/oder die Unterauftragsverarbeiter zu übermitteln, und zwar unter vollständiger Einhaltung der Datenschutzvorschriften, einschließlich, soweit erforderlich, der Einhaltung etwaiger vorgängiger Formalitäten und der Rechte der betroffenen Personen, etwa der Informationspflicht oder der Einholung einer Einwilligung, sofern die Datenschutzvorschriften dies verlangen.',
    p3: 'Der Kunde erkennt an, dass er allein verantwortlich bleibt für die Festlegung der Zwecke und der Mittel der Verarbeitung der personenbezogenen Daten durch Hook0. Der Verantwortliche bleibt allein verantwortlich für die Richtigkeit und Angemessenheit der vorgenannten Weisungen. Jede Änderung der erteilten Weisungen oder der vom Kunden geforderten Sicherheitsmaßnahmen, auch zur Einhaltung der geltenden Datenschutzgesetze, ist von den Parteien zu vereinbaren oder per Nachtrag zu diesem DPA festzuhalten. Die Kosten, die Hook0 zur Umsetzung solcher Änderungen entstehen, trägt der Kunde.',
    p4: 'Der Kunde verpflichtet sich, sicherzustellen, dass die betroffenen Personen vor der Übermittlung ihrer personenbezogenen Daten an Hook0 im Rahmen der Dienste informiert wurden oder werden.',
    p5: 'Das Produkt ist nicht zur Verarbeitung besonderer Kategorien personenbezogener Daten bestimmt. Der Kunde verpflichtet sich daher, jede Verarbeitung besonderer Kategorien personenbezogener Daten über das Produkt und die Dienste zu unterbinden. Auf Wunsch des Kunden kann Hook0 jedoch besondere Kategorien personenbezogener Daten verarbeiten. In diesem Fall wird die Verarbeitung in einem gesonderten Nachtrag zum DPA geregelt, der zwischen dem Kunden und Hook0 abzuschließen ist.',
    p6: 'Verlangt der Kunde ausdrücklich die Unterstützung von Hook0 bei der Erfüllung seiner Pflichten gemäß den Datenschutzvorschriften, übermittelt Hook0 dem Kunden eine Kostenschätzung für diese Unterstützung. Nach ausdrücklicher Annahme der Kostenschätzung leistet Hook0 die Unterstützung gemäß den Weisungen des Kunden und den Bedingungen des vorliegenden DPA.',
  },
  section3: {
    title: '3. Umfang und Weisungen',
    p1: 'Hook0 verpflichtet sich:',
    items: [
      'die vom Kunden offengelegten personenbezogenen Daten sowie die während der Laufzeit der Nutzungsbedingungen erhobenen oder erzeugten Daten ausschließlich zur Erfüllung der Pflichten aus den Nutzungsbedingungen und in Übereinstimmung mit den dokumentierten Weisungen des Kunden zu verarbeiten, sofern die geltenden Datenschutzvorschriften nichts anderes vorschreiben;',
      'sicherzustellen, dass jede unter ihrer Aufsicht handelnde Person, die Zugang zu den vom Kunden offengelegten personenbezogenen Daten sowie zu den während der Laufzeit der Nutzungsbedingungen erhobenen oder erzeugten Daten hat, diese Daten ausschließlich zur Erfüllung der Pflichten von Hook0 aus den Nutzungsbedingungen und auf Weisung des Kunden verarbeitet, sofern die geltenden Datenschutzvorschriften nichts anderes vorschreiben;',
      'die personenbezogenen Daten des Kunden nicht für zweckentfremdete, betrügerische oder persönliche Zwecke zu verwenden, einschließlich kommerzieller Zwecke;',
      'den Kunden unverzüglich zu informieren, sofern eine Weisung des Kunden nach ihrer Auffassung gegen die geltenden Datenschutzvorschriften verstößt.',
    ],
  },
  section4: {
    title: '4. Übermittlung der personenbezogenen Daten des Kunden an Dritte',
    p1: 'Die im Rahmen des DPA verarbeiteten personenbezogenen Daten des Kunden dürfen weder abgetreten, vermietet, lizenziert, übermittelt noch an Dritte offengelegt werden, einschließlich an Unterauftragsverarbeiter von Hook0, es sei denn, dies wird durch die Nutzungsbedingungen oder durch eine zwingende rechtliche oder regulatorische Bestimmung verlangt.',
    p2: 'In einem solchen Fall informiert Hook0 den Kunden über diese rechtliche Anforderung vor der Verarbeitung, sofern die zwingende rechtliche oder regulatorische Bestimmung diese Information nicht aus wichtigen Gründen des öffentlichen Interesses untersagt.',
  },
  section5: {
    title: '5. Unterauftragsverarbeitung',
    p1Html: 'Unter den in den Absätzen 2 und 4 des Artikels 28 DSGVO genannten Bedingungen für die Inanspruchnahme eines weiteren Auftragsverarbeiters (des « <strong>Unterauftragsverarbeiters</strong> ») stimmt der Kunde zu, dass Hook0 die Verarbeitung der personenbezogenen Daten des Kunden im Wege der Unterauftragsverarbeitung erbringen darf.',
    p2Html: 'Ungeachtet der allgemeinen Genehmigung des Kunden informiert Hook0 den Kunden über jede geplante Änderung in Bezug auf die Hinzufügung oder Ersetzung eines Unterauftragsverarbeiters innerhalb einer angemessenen Frist vor der Umsetzung dieser Änderung und räumt dem Kunden eine angemessene Frist zur Erhebung eines Widerspruchs vor Wirksamwerden der Änderung ein. Die Liste der Unterauftragsverarbeiter, die unter der Aufsicht von Hook0 stehen, ist für den Kunden unter <a href="./dsgvo-unterauftragsverarbeiter">Hook0 / DSGVO-Unterauftragsverarbeiter</a> einsehbar.',
    p3: 'Beauftragt Hook0 einen Unterauftragsverarbeiter mit der Verarbeitung der personenbezogenen Daten des Kunden, erlegt Hook0 dem Unterauftragsverarbeiter dieselben Datenschutzpflichten auf, wie sie im DPA festgelegt sind.',
    p4: 'Diese Vereinbarung muss insbesondere eine Verpflichtung des Unterauftragsverarbeiters vorsehen, hinreichende Garantien für die Umsetzung geeigneter technischer und organisatorischer Maßnahmen zu bieten, sodass die Verarbeitung den Anforderungen der Datenschutzvorschriften und des DPA entspricht.',
  },
  section6: {
    title: '6. Übermittlung der personenbezogenen Daten des Kunden außerhalb des Europäischen Wirtschaftsraums (EWR)',
    p1Html: 'Hook0 sichert zu, dass die Datenebene der Webhooks (Webhook-Payloads des Kunden, Datenbank und Anwendungs-Backups) in Frankreich oder innerhalb des Europäischen Wirtschaftsraums (EWR) verortet ist. Die ergänzende Edge-Schicht (CDN und DDoS-Schutz durch Cloudflare, Inc.) umfasst Transfers in die Vereinigten Staaten, die durch die von der Europäischen Kommission erlassenen Standardvertragsklauseln 2021 und ein dokumentiertes Transfer Impact Assessment sowie gegebenenfalls durch das EU-US Data Privacy Framework abgesichert werden. Die vollständige Liste der Unterauftragsverarbeiter und die anwendbaren Transfermechanismen werden unter <a href="./dsgvo-unterauftragsverarbeiter">Hook0 / DSGVO-Unterauftragsverarbeiter</a> aktuell gehalten.',
    p2Html: 'Auf Wunsch und nach Weisung des Kunden kann Hook0 personenbezogene Daten an andere Hook0-Einheiten und/oder an Unterauftragsverarbeiter in Ländern außerhalb des EWR (« Drittländer ») speichern oder übermitteln. In diesem Fall und sofern für die Drittländer kein Angemessenheitsbeschluss der Europäischen Kommission vorliegt, verpflichtet sich Hook0 dazu, dass die Übermittlung gemäß den Datenschutzvorschriften erfolgt und mit geeigneten Garantien versehen ist, die ein dem Datenschutzrecht gleichwertiges Schutzniveau gewährleisten, etwa durch die Unterzeichnung der von der Europäischen Kommission erlassenen Standardvertragsklauseln, abrufbar unter <a href="https://commission.europa.eu/law/law-topic/data-protection/international-dimension-data-protection/standard-contractual-clauses-scc_en">commission.europa.eu</a>.',
    p3: 'Der Kunde bevollmächtigt Hook0 hiermit, in seinem Namen und für seine Rechnung die Standardvertragsklauseln mit Hook0-Einheiten und Unterauftragsverarbeitern in Drittländern zu unterzeichnen.',
    p4: 'Auf Wunsch des Kunden unterstützt Hook0 den Kunden bei der Durchführung eines Transfer Impact Assessment, um Lücken zwischen den Datenschutzvorschriften und dem Recht des Drittlands zu identifizieren und die erforderlichen ergänzenden Maßnahmen umzusetzen, die ein dem Datenschutzrecht gleichwertiges Schutzniveau gewährleisten.',
  },
  section7: {
    title: '7. Sicherheitsmaßnahmen und Vertraulichkeit der Verarbeitung',
    p1: 'Hook0 ergreift, soweit dies für die Bereitstellung der Dienste oder die Einhaltung der übrigen Pflichten aus dem DPA relevant ist, angemessene Maßnahmen, um ein dem Risiko angemessenes Schutzniveau für die personenbezogenen Daten des Kunden zu gewährleisten, und berücksichtigt bei der Ausführung des DPA die Grundsätze des Datenschutzes durch Technikgestaltung und durch datenschutzfreundliche Voreinstellungen.',
    p2: 'Hook0 verpflichtet sich:',
    items: [
      'sämtliche geeigneten technischen und organisatorischen Maßnahmen zu treffen, um personenbezogene Daten vor der unbeabsichtigten oder unrechtmäßigen Vernichtung, dem Verlust, der Veränderung, der unbefugten Offenlegung oder dem unbefugten Zugriff auf übermittelte, gespeicherte oder anderweitig verarbeitete personenbezogene Daten zu schützen, insbesondere alle in Anhang 2 genannten Maßnahmen;',
      'sämtliche vom Kunden mitgeteilten Weisungen zu Sicherheits- und Vertraulichkeitsmaßnahmen, die in zumutbarer Weise umsetzbar sind, einzuhalten;',
      'die personenbezogenen Daten des Kunden ausschließlich ordnungsgemäß autorisierten Personen zugänglich zu machen;',
      'die Vertraulichkeit der im Rahmen des DPA verarbeiteten personenbezogenen Daten des Kunden zu gewährleisten und sicherzustellen, dass sich alle unter der Aufsicht von Hook0 zur Verarbeitung der personenbezogenen Daten des Kunden befugten Personen (einschließlich Mitarbeiter und Unterauftragsverarbeiter) zur Vertraulichkeit dieser Daten verpflichten oder einer angemessenen gesetzlichen Verschwiegenheitspflicht unterliegen.',
    ],
  },
  section8: {
    title: '8. Meldung von Verletzungen des Schutzes personenbezogener Daten',
    p1Html: 'Hook0 meldet dem Kunden jede Verletzung des Schutzes personenbezogener Daten unverzüglich und in jedem Fall innerhalb von <strong>72 Stunden</strong> nach Kenntniserlangung von der Verletzung, gemäß Artikel 33 DSGVO, und schriftlich nach Kenntniserlangung von einer Verletzung des Schutzes personenbezogener Daten. Sofern die Informationen Hook0 vorliegen, enthält die Meldung:',
    items: [
      'eine Beschreibung der Art der Verletzung des Schutzes personenbezogener Daten, soweit möglich mit Angabe der Kategorien und ungefähren Zahl der betroffenen Personen sowie der Kategorien und ungefähren Zahl der betroffenen personenbezogenen Daten;',
      'die Mitteilung des Namens und der Kontaktdaten des Datenschutz-Ansprechpartners (<a href="mailto:legal@hook0.com">legal@hook0.com</a>) oder einer anderen Kontaktstelle, bei der weitere Informationen eingeholt werden können;',
      'eine Beschreibung der wahrscheinlichen Folgen der Verletzung des Schutzes personenbezogener Daten;',
      'eine Beschreibung der ergriffenen oder vorgeschlagenen Maßnahmen zur Behebung der Verletzung des Schutzes personenbezogener Daten, gegebenenfalls einschließlich Maßnahmen zur Minderung ihrer möglichen nachteiligen Auswirkungen.',
    ],
    p2: 'Soweit die Informationen nicht zur gleichen Zeit bereitgestellt werden können, dürfen sie ohne unangemessene weitere Verzögerung schrittweise zur Verfügung gestellt werden.',
    p3: 'Auf Wunsch des Kunden verpflichtet sich Hook0 zudem, dem Kunden angemessene Unterstützung und Mitwirkung dabei zu leisten, die Verletzung des Schutzes personenbezogener Daten der zuständigen Aufsichtsbehörde zu melden und diese Verletzung den betroffenen Personen gemäß Artikel 34 DSGVO mitzuteilen, in Übereinstimmung mit den geltenden Datenschutzvorschriften.',
  },
  section9: {
    title: '9. Rechte der betroffenen Personen',
    p1: 'Aufgrund der Art der Verarbeitungstätigkeiten verpflichtet sich Hook0:',
    items: [
      'den Kunden unverzüglich über jede erhaltene Anfrage oder Beschwerde im Zusammenhang mit dem Schutz der personenbezogenen Daten des Kunden zu informieren;',
      'dem Kunden auf dessen Wunsch angemessene Unterstützung und Mitwirkung zu leisten, damit der Kunde (i) Anträge der betroffenen Personen zur Ausübung ihrer Rechte (Auskunftsrecht, Rechte auf Berichtigung, Löschung, Einschränkung, Datenübertragbarkeit und Widerspruch) oder (ii) Anfragen der zuständigen Datenschutzbehörden oder des Datenschutzbeauftragten des Kunden beantworten kann; insbesondere geeignete technische und organisatorische Maßnahmen umzusetzen, damit der Kunde jeder Auskunftsanfrage des Kunden zeitnah und schriftlich nachkommen kann;',
      'den betroffenen Personen die angemessenen Informationen über die im Rahmen der Nutzungsbedingungen durchgeführten Verarbeitungsvorgänge zu ihren personenbezogenen Daten ordnungsgemäß bereitzustellen, sofern dies vom Kunden angefordert und auf dessen Kosten erfolgt.',
    ],
  },
  section10: {
    title: '10. Datenschutz-Folgenabschätzung',
    p1: 'Auf Wunsch des Kunden verpflichtet sich Hook0, dem Kunden angemessene Unterstützung und Mitwirkung zu leisten, um eine Folgenabschätzung der im Rahmen des vorliegenden DPA durchgeführten Verarbeitungsvorgänge personenbezogener Daten für den Schutz personenbezogener Daten durchzuführen und gegebenenfalls die zuständigen Datenschutzbehörden zu konsultieren, auf Kosten des Kunden (nach Zeitaufwand).',
  },
  section11: {
    title: '11. Aufbewahrung, Rückgabe oder Vernichtung der personenbezogenen Daten',
    p1: 'Der Kunde bleibt allein verantwortlich für die Umsetzung und die Verwaltung der Aufbewahrungsfristen für personenbezogene Daten und verpflichtet sich, das Produkt entsprechend zu nutzen.',
    p2: 'Unbeschadet der geltenden Gesetze und Vorschriften verpflichtet sich Hook0, zum Ende der Nutzungsbedingungen:',
    items: [
      'auf Wunsch des Kunden alle personenbezogenen Daten des Kunden automatisiert oder manuell zurückzugeben oder zu vernichten, nach zuvor zwischen den Parteien vereinbarten Verfahren und Vorgaben;',
      'alle vorhandenen Kopien der personenbezogenen Daten zu löschen, es sei denn und soweit Hook0 verpflichtet ist, Kopien der personenbezogenen Daten gemäß geltendem Recht aufzubewahren (insbesondere Rechnungs- und Buchhaltungsunterlagen, die nach französischem Steuerrecht 10 Jahre lang aufzubewahren sind);',
      'die Vernichtung der personenbezogenen Daten schriftlich zu bescheinigen.',
    ],
  },
  section12: {
    title: '12. Dokumentation und Audit',
    p1: 'Nach schriftlicher Vorankündigung von dreißig (30) Werktagen durch den Kunden legt Hook0 dem Kunden die zum Nachweis der Einhaltung der in diesen Nutzungsbedingungen festgelegten Pflichten zwingend erforderlichen Informationen offen.',
    p2: 'Auf Wunsch des Kunden und einmal jährlich verpflichtet sich Hook0, angemessene Audits, einschließlich Inspektionen, die vom Kunden oder von einem von ihm beauftragten Dritten durchgeführt werden, zu ermöglichen und an ihnen mitzuwirken, um die Einhaltung der Datenschutzvorschriften und der Bestimmungen des DPA durch Hook0 zu beurteilen.',
    p3: 'Hook0 verpflichtet sich zudem, Audits zuständiger Datenschutzbehörden zu ermöglichen und an ihnen mitzuwirken.',
    p4: 'Der Kunde hat keinerlei Recht, Systeme, Daten, Aufzeichnungen oder sonstige Informationen einzusehen oder darauf zuzugreifen, die andere Kunden von Hook0 betreffen.',
    p5: 'Jedes derartige Audit durch den Kunden oder in seinem Auftrag erfolgt auf eigene Kosten. Der Kunde übermittelt Hook0 eine Kopie des Auditberichts.',
    p6: 'Wird der Kunde Gegenstand einer Untersuchung oder einer Auskunftsanfrage einer zuständigen Datenschutzbehörde, die einen der von Hook0 in seinem Auftrag durchgeführten Verarbeitungsvorgänge betrifft, verpflichtet sich der Kunde, Hook0 schnellstmöglich zu informieren und der Untersuchung oder Anfrage nach besten Möglichkeiten, auf eigene Kosten und in Übereinstimmung mit den von der Datenschutzbehörde festgelegten Verfahren nachzukommen.',
    p7: 'Der Kunde verpflichtet sich, sämtliche Vertraulichkeitsbestimmungen, Richtlinien und/oder Standortregeln einzuhalten, die ihm Hook0 im Zusammenhang mit dem Audit mitteilen kann.',
  },
  appendix1: {
    title: 'Anhang 1 - Verarbeitungstätigkeiten personenbezogener Daten, die Hook0 im Auftrag des Kunden durchführt',
    rows: [
      {
        label: 'Verantwortlicher',
        valueHtml: 'Der Kunde (wie in den Nutzungsbedingungen identifiziert).',
      },
      {
        label: 'Auftragsverarbeiter',
        valueHtml: 'FGRibreau SARL, eine Gesellschaft mit beschränkter Haftung nach französischem Recht mit einem Stammkapital von 2 000 EUR, eingetragen im RCS La Roche-sur-Yon unter der Nummer 850 824 350, USt-ID FR27850824350, Geschäftsadresse 3 rue de l\'Aubépine, 85110 Chantonnay, Frankreich.',
      },
      {
        label: 'Art der Verarbeitungsvorgänge',
        valueHtml: '<ul><li>Empfang, Speicherung und Weiterleitung von Webhook-Ereignissen im Auftrag des Kunden</li><li>Verwaltung der Wiederholungsversuche bei fehlgeschlagenen Webhook-Zustellungen</li><li>Protokollierung und Überwachung der Webhook-Zustellversuche</li><li>Authentifizierung der Nutzer und Zugriffsverwaltung auf der Hook0-Plattform</li><li>Abrechnungs- und Abonnementverwaltung (über Stripe)</li></ul>',
      },
      {
        label: 'Zweck(e) der Verarbeitung',
        valueHtml: 'Bereitstellung der Webhook-as-a-Service-Plattform Hook0, wie in den Nutzungsbedingungen beschrieben.',
      },
      {
        label: 'Name und Kontaktdaten des Datenschutzbeauftragten des Kunden (sofern zutreffend)',
        valueHtml: '<em>[vom Kunden auszufüllen]</em>',
      },
      {
        label: 'Kategorie(n) personenbezogener Daten',
        valueHtml: 'E-Mail-Adressen, Namen, IP-Adressen, Inhalte der Webhook-Payloads (vom Kunden bestimmt), Authentifizierungstoken, Abrechnungsinformationen (verarbeitet durch Stripe).<br><br><strong>Sensible Daten:</strong> standardmäßig keine. Der Kunde ist dafür verantwortlich, sicherzustellen, dass die Webhook-Payloads keine besonderen Kategorien personenbezogener Daten enthalten, sofern nichts Abweichendes schriftlich vereinbart wurde.<br><br>Auf Wunsch des Kunden kann Hook0 besondere Kategorien personenbezogener Daten verarbeiten. In diesem Fall wird die Verarbeitung in einem gesonderten Nachtrag zum DPA zwischen Kunde und Hook0 geregelt.',
      },
      {
        label: 'Kategorie(n) betroffener Personen',
        valueHtml: 'Endnutzer des Kunden, deren Daten über Webhooks übermittelt werden; befugte Nutzer des Kunden mit Zugang zur Hook0-Plattform.',
      },
      {
        label: 'Ort(e) der Verarbeitungsvorgänge',
        valueHtml: 'Datenebene der Webhooks: Frankreich / EWR.<br>CDN und DDoS-Schutz: Vereinigte Staaten (Cloudflare, Inc.), abgesichert durch SCC 2021 + TIA und, sofern zutreffend, durch das EU-US Data Privacy Framework.<br><br>Verlangt der Kunde die Speicherung der personenbezogenen Daten außerhalb des EWR, wird diese Verarbeitung in einer gesonderten Vereinbarung zwischen Kunde und Hook0 geregelt.<br><br>Siehe: <a href="./dsgvo-unterauftragsverarbeiter">Hook0 / DSGVO-Unterauftragsverarbeiter</a>',
      },
      {
        label: 'Identität der Unterauftragsverarbeiter',
        valueHtml: 'Siehe: <a href="./dsgvo-unterauftragsverarbeiter">Hook0 / DSGVO-Unterauftragsverarbeiter</a>',
      },
      {
        label: 'Häufigkeit der Verarbeitung',
        valueHtml: 'Kontinuierliche, automatisierte Verarbeitung.',
      },
      {
        label: 'Dauer der Verarbeitungsvorgänge',
        valueHtml: 'Für die Laufzeit der Nutzungsbedingungen, zuzüglich 30 Tage nach Löschung des Kontos (Kontodaten). Die Aufbewahrungsdauer der Webhook-Ereignisdaten richtet sich nach dem vom Kunden gebuchten Plan, 7 Tage bei Developer, 14 Tage bei Startup, 30 Tage bei Pro, individuelle Dauer bei Enterprise. Rechnungs- und Buchhaltungsunterlagen werden nach französischem Steuerrecht 10 Jahre lang aufbewahrt.',
      },
    ],
  },
  appendix2: {
    title: 'Anhang 2 - Umgesetzte angemessene technische und organisatorische Maßnahmen',
    intro: 'Die folgenden technischen und organisatorischen Maßnahmen werden von Hook0 umgesetzt, um personenbezogene Daten vor der unbeabsichtigten oder unrechtmäßigen Vernichtung, dem Verlust, der Veränderung, der unbefugten Offenlegung oder dem unbefugten Zugriff auf übermittelte, gespeicherte oder anderweitig verarbeitete personenbezogene Daten zu schützen:',
    groups: [
      {
        title: 'Infrastruktursicherheit (verwaltet durch Clever Cloud SAS)',
        items: [
          'Anwendung gehostet auf der Infrastruktur der Clever Cloud SAS in Frankreich (EU);',
          'Verschlüsselung der Datenbank im Ruhezustand (verwaltet durch Clever Cloud);',
          'TLS 1.2+ Verschlüsselung für alle Daten in Übertragung (rustls, mit Unterstützung post-quantenfähiger Kryptografie);',
          'Tägliche automatisierte Backups mit 30-tägiger Aufbewahrung, gespeichert in einem multi-regionalen verteilten System (S3-kompatibel) auf Clever Cloud fr-par; die Integrität der Backups wird verifiziert und die Wiederherstellung wird monatlich getestet;',
          'CDN und DDoS-Schutz bereitgestellt durch Cloudflare, Inc. (Vereinigte Staaten), abgesichert durch SCC 2021 + TIA und, sofern zutreffend, durch das EU-US Data Privacy Framework;',
          'Die physischen Zugangskontrollen zu den Räumlichkeiten der Rechenzentren werden der Clever Cloud SAS gemäß deren dokumentiertem Sicherheitsprogramm übertragen.',
        ],
      },
      {
        title: 'Anwendungssicherheit',
        items: [
          'Passwort-Hashing mit Argon2 (speicherintensive Funktion, resistent gegen GPU- und ASIC-Angriffe; eindeutiges zufälliges Salt pro Passwort; niemals im Klartext oder mit reversibler Verschlüsselung gespeichert);',
          'Autorisierungstoken auf Capability-Basis (Biscuit);',
          'Rollenbasierte Zugriffskontrolle (RBAC) für den Plattformzugang;',
          'Automatische Sitzungs-Expiration.',
        ],
        noteHtml: '<strong>Hinweis zur Mehr-Faktor-Authentifizierung (MFA):</strong> Die MFA ist für alle Infrastrukturzugänge (Clever Cloud, GitLab, Stripe) aktiviert. Die MFA für individuelle Hook0-Nutzerkonten ist auf Anwendungsebene noch nicht umgesetzt und ist für eine spätere Version vorgesehen. Bis die kundenseitige MFA verfügbar ist, sorgen strenge Passwortanforderungen (Argon2-Hashing, Mindestkomplexität) und die Sitzungs-Expiration für einen Grundschutz.',
      },
      {
        title: 'Entwicklungssicherheit',
        items: [
          'Jede Codeänderung erfordert eine Peer-Review über eine Merge Request;',
          'Automatisierte CI/CD-Pipeline mit:<ul><li>statischer Anwendungssicherheitsprüfung (SAST, GitLab-Vorlage);</li><li>dynamischer Anwendungssicherheitsprüfung (DAST, GitLab-Vorlage);</li><li>Container- und Dateisystem-Scanning (Trivy);</li><li>Abhängigkeits-Schwachstellenscan (osv-scanner);</li><li>Geheimnis-Erkennung (GitLab-Vorlage).</li></ul>',
          'Striktes Code-Linting (Clippy mit Warnungen als Fehler behandelt) und einheitliche Formatierung (cargo fmt --check), erzwungen in der CI.',
        ],
      },
      {
        title: 'Überwachung und Reaktion auf Vorfälle',
        items: [
          'Fehler-Tracking über Sentry;',
          'Verteiltes Tracing über OpenTelemetry (OTLP-Export);',
          'Verfügbarkeitsüberwachung über BetterUptime mit öffentlicher Statusseite;',
          'Meldung von Verletzungen des Schutzes personenbezogener Daten innerhalb von 72 Stunden gemäß Artikel 33 DSGVO (siehe Abschnitt 8);',
          'Responsible-Disclosure-Richtlinie mit PGP-gesicherter Meldung.',
        ],
      },
      {
        title: 'Organisatorische Maßnahmen',
        items: [
          'Klassifikationsrichtlinie für Informationen (Public, Internal, Confidential, Sensitive);',
          'Vertraulichkeitsvereinbarungen (NDA) für sämtliches Personal verpflichtend;',
          'Praktiken zur Sicherheitssensibilisierung;',
          'Need-to-know-Prinzip beim Zugriff;',
          'MFA aktiviert für Infrastrukturzugänge (Clever Cloud, GitLab, Stripe);',
          'Penetrationstests jährlich oder nach wesentlichen architektonischen Änderungen durchgeführt.',
        ],
      },
      {
        title: 'Aufbewahrungsfristen',
        items: [
          'Webhook-Ereignisdaten, je nach Plan des Kunden, 7 Tage bei Developer, 14 Tage bei Startup, 30 Tage bei Pro, individuelle Dauer bei Enterprise;',
          'Kontodaten (Benutzername, E-Mail, gehashtes Passwort, API-Schlüssel), Laufzeit des Servicevertrags zuzüglich 30 Tage nach Löschung des Kontos;',
          'Rechnungs- und Buchhaltungsunterlagen, 10 Jahre (französisches Steuerrecht, art. L102 B Livre des procédures fiscales);',
          'Serverprotokolle, mindestens 30 Tage, danach automatische Rotation und Löschung;',
          'Support-Kommunikation, 3 Jahre ab dem letzten Austausch (gesetzliche Verjährungsfrist für vertragliche Ansprüche).',
        ],
      },
    ],
  },
};
