// Per-page strings for terms (DE, Allgemeine Nutzungsbedingungen).
//
// Register : Siezen formell verpflichtend (« Sie » / « Ihr »), wie fuer jeden
// vertragsrechtlichen Text. Kein Duzen (Duzen bleibt Marketing-Seiten
// vorbehalten). /humanizer pro angewendet. Kein Em-Dash, kein Doppel-Bindestrich
// als Pivot, kein Mittel-Punkt.
//
// SSPL = « quelloffen (SSPL-1.0) », niemals « Open Source » allein
// (von OSI abgelehnt, UWG-§5-Risiko in DACH).
// DSGVO niemals absolut (« 100% DSGVO-konform »), nur als Prozess-Claim
// (« auf DSGVO-Konformitaet ausgelegt »).
// Verboten : « 100% souveraen », « keine Daten verlassen die EU »,
// « kein US-Konzern im Stack », « CLOUD Act free ».
//
// Hook0 Hardfacts verbatim ueber alle Locales: FGRibreau SARL, Stammkapital
// 2 000 EUR, Handelsregister La Roche-sur-Yon 850 824 350, USt-IdNr.
// FR27850824350, Sitz 3 rue de l'Aubepine 85110 Chantonnay, verantwortlicher
// Herausgeber David Sferruzza, Hosting Clever Cloud SAS (Frankreich) + CDN
// Cloudflare Inc. (USA) offengelegt.
module.exports = {
  pageTitle: 'Hook0 - Allgemeine Nutzungsbedingungen',
  pageDescription: 'Allgemeine Nutzungsbedingungen fuer Hook0 Webhooks-as-a-Service. Lesen Sie sorgfaeltig die Regeln, die den Zugriff auf die Hook0-Plattform und deren Nutzung regeln.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Rechtliches',
    title: 'Allgemeine Nutzungsbedingungen',
    subtitle: 'Lesen Sie diese Bedingungen sorgfaeltig, bevor Sie die Hook0-Dienste nutzen.',
    lastUpdatedLabel: 'Letzte Aktualisierung:',
    lastUpdatedDate: '27. Juni 2026',
  },
  intro: {
    p1Html: 'Die vorliegenden Allgemeinen Nutzungsbedingungen (die « Bedingungen ») regeln Ihren Zugriff auf die Hook0-Plattform und die damit verbundenen Dienste (zusammen der « Dienst »), die von der FGRibreau SARL betrieben werden, einer franzoesischen Gesellschaft mit beschraenkter Haftung (societe a responsabilite limitee) mit einem Stammkapital von 2 000 EUR, eingetragen im Handels- und Gesellschaftsregister La Roche-sur-Yon unter der Nummer 850 824 350, mit Sitz in 3 rue de l\'Aubepine, 85110 Chantonnay, Frankreich, USt-IdNr. FR27850824350 (« Hook0 », « wir » oder « unser »). Verantwortlicher Herausgeber ist David Sferruzza.',
    p2Html: 'Der Dienst richtet sich ausschliesslich an Unternehmen und gewerbliche Einrichtungen (B2B). Mit der Registrierung, dem Zugriff oder der Nutzung des Dienstes bestaetigen Sie, dass Sie in gewerblicher Eigenschaft fuer eine juristische Person handeln und befugt sind, diese juristische Person an die vorliegenden Bedingungen zu binden.',
    p3Html: 'MIT DER REGISTRIERUNG, DEM ZUGRIFF AUF ODER DER NUTZUNG DES DIENSTES ERKLAEREN SIE SICH MIT DIESEN BEDINGUNGEN EINVERSTANDEN. WENN SIE DAMIT NICHT EINVERSTANDEN SIND, DUERFEN SIE WEDER AUF DEN DIENST ZUGREIFEN NOCH IHN NUTZEN.',
    p4Html: 'Die kommerziellen und Abrechnungsbedingungen (Preise, Rechnungsstellung, Zahlungskonditionen) sind in den <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Allgemeinen Geschaeftsbedingungen</a> festgelegt, die ein separates, durch Verweis in diese Vereinbarung einbezogenes Dokument bilden.',
  },
  sections: [
    {
      id: 'definitions',
      title: '1. Begriffsbestimmungen',
      lead: 'Im Rahmen dieser Bedingungen haben die folgenden Begriffe die nachstehend angegebene Bedeutung:',
      items: [
        '<strong class="text-white">« Konto »</strong> bezeichnet das von Ihnen angelegte Konto fuer den Zugriff auf und die Nutzung des Dienstes.',
        '<strong class="text-white">« Inhalte »</strong> bezeichnet saemtliche Daten, Informationen oder Materialien, die Sie ueber den Dienst uebertragen oder darin speichern, einschliesslich Webhook-Payloads, Konfigurationen und API-Zugangsdaten.',
        '<strong class="text-white">« Dokumentation »</strong> bezeichnet die technische Dokumentation und die Benutzerhandbuecher, die Hook0 auf <a href="https://documentation.hook0.com" class="text-green-400 hover:text-green-300 transition-colors">documentation.hook0.com</a> bereitstellt.',
        '<strong class="text-white">« Dienst »</strong> bezeichnet die Hook0-Webhook-Management-Plattform, einschliesslich saemtlicher zugehoeriger APIs, Schnittstellen und ergaenzender Leistungen, die Hook0 bereitstellt.',
        '<strong class="text-white">« Unterauftragsverarbeiter »</strong> bezeichnet jeden von Hook0 beauftragten Drittauftragsverarbeiter, der Ihre Inhalte im Rahmen des Dienstes in Ihrem Auftrag verarbeitet. Die aktuelle Liste der Unterauftragsverarbeiter ist auf <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a> veroeffentlicht.',
        '<strong class="text-white">« Abonnementplan »</strong> bezeichnet die von Ihnen gewaehlte Servicestufe (Developer, Startup, Pro oder Enterprise), wie auf der Preisseite von hook0.com beschrieben.',
        '<strong class="text-white">« Nutzer »</strong> bezeichnet jede natuerliche Person, die ueber Ihr Konto in Ihrem Auftrag auf den Dienst zugreift oder ihn nutzt.',
        '<strong class="text-white">« Sie » / « Ihr »</strong> bezeichnet die juristische Person, die sich fuer den Dienst registriert hat oder ihn nutzt, sowie jeden Nutzer, der in deren Auftrag handelt.',
      ],
    },
    {
      id: 'acceptance',
      title: '2. Annahme und Geltungsbereich',
      paragraphs: [
        '<strong class="text-white">2.1. Ausschliesslich gewerbliche Nutzung.</strong> Der Dienst ist ausschliesslich fuer gewerbliche und geschaeftliche Nutzung bestimmt. Diese Bedingungen gelten nicht fuer Verbraucher (natuerliche Personen, die ausserhalb einer gewerblichen oder beruflichen Taetigkeit handeln). Da der Dienst ausschliesslich im B2B-Kontext angeboten wird, findet kein verbraucherrechtliches Widerrufsrecht Anwendung. Mit Annahme dieser Bedingungen erklaeren und gewaehrleisten Sie, in gewerblicher Eigenschaft zu handeln.',
        '<strong class="text-white">2.2. Vertretungsbefugnis.</strong> Wenn Sie diese Bedingungen im Namen eines Unternehmens oder einer anderen juristischen Person annehmen, erklaeren und gewaehrleisten Sie, ueber die rechtliche Befugnis zu verfuegen, diese juristische Person zu binden. In diesem Fall bezeichnet « Sie » diese juristische Person.',
        '<strong class="text-white">2.3. Vollstaendige Vereinbarung.</strong> Diese Bedingungen bilden zusammen mit der <a href="/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Datenschutzerklaerung</a>, dem <a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Auftragsverarbeitungsvertrag (AVV)</a> und den <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Allgemeinen Geschaeftsbedingungen</a> die gesamte Vereinbarung zwischen den Parteien ueber den Dienst und ersetzen alle vorherigen oder gleichzeitigen Absprachen, Zusicherungen oder Vereinbarungen.',
      ],
    },
    {
      id: 'description',
      title: '3. Beschreibung des Dienstes',
      paragraphs: [
        '<strong class="text-white">3.1. Plattformueberblick.</strong> Hook0 ist eine Webhook-Management-Plattform, mit der Unternehmen Webhook-Ereignisse senden, empfangen, verwalten und ueberwachen koennen. Der Dienst umfasst die Webhook-Zustellinfrastruktur, die Wiederholungslogik, die Ereignisprotokollierung und zugehoerige Entwicklerwerkzeuge.',
        '<strong class="text-white">3.2. Abonnementplaene.</strong> Der Dienst wird in folgenden Abonnementplaenen angeboten: Developer (kostenlos), Startup, Pro und Enterprise. Funktionen, Nutzungsgrenzen und Preise je Plan sind auf der Preisseite von hook0.com beschrieben. Hook0 behaelt sich vor, die mit einem Abonnementplan verbundenen Funktionen zu aendern, einschliesslich der Einstellung von Funktionen des kostenlosen Developer-Plans, mit der in Abschnitt 13 vorgesehenen Vorankuendigung.',
        '<strong class="text-white">3.3. Aktualisierungen.</strong> Hook0 kann Funktionen des Dienstes jederzeit aktualisieren, aendern oder einstellen. Bei wesentlichen Aenderungen kuendigt Hook0 diese mit angemessener Frist vorab an. Die fortgesetzte Nutzung des Dienstes nach solchen Aenderungen gilt als Annahme des aktualisierten Dienstes.',
        '<strong class="text-white">3.4. Drittanbieter-Dienste und -Infrastruktur.</strong> Der Dienst stuetzt sich auf Drittanbieter-Infrastrukturen, insbesondere das Hosting durch Clever Cloud SAS (Frankreich) sowie die Inhaltsauslieferung und die Edge-Sicherheit durch Cloudflare Inc. (Vereinigte Staaten). Die aktuelle Liste der Unterauftragsverarbeiter und deren Standorte ist auf <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a> veroeffentlicht. Der Dienst kann sich zudem in weitere Drittanbieter-Dienste integrieren oder auf solche verweisen, ueber die Hook0 keine Kontrolle hat. Ihre Nutzung dieser Dienste unterliegt ausschliesslich den jeweils anwendbaren Bedingungen der Drittanbieter.',
      ],
    },
    {
      id: 'account',
      title: '4. Kontoregistrierung',
      paragraphs: [
        '<strong class="text-white">4.1. Korrekte Angaben.</strong> Sie verpflichten sich, bei der Registrierung eines Kontos korrekte, aktuelle und vollstaendige Angaben zu machen und diese aktuell zu halten. Hook0 kann Ihr Konto sperren oder kuendigen, wenn festgestellt wird, dass die Angaben falsch oder irrefuehrend sind.',
        '<strong class="text-white">4.2. Kontosicherheit.</strong> Sie sind fuer die Vertraulichkeit Ihrer Konto-Zugangsdaten und fuer alle Aktivitaeten unter Ihrem Konto verantwortlich. Sie verpflichten sich, Hook0 unverzueglich unter <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> zu benachrichtigen, wenn Sie von einem unbefugten Zugriff auf Ihr Konto oder dessen unbefugter Nutzung Kenntnis erlangen.',
        '<strong class="text-white">4.3. Ein Konto je juristische Person.</strong> Jede juristische Person darf nur ein Konto unterhalten, sofern Hook0 nicht ausdruecklich schriftlich etwas anderes vereinbart hat. Die Anlage mehrerer Konten zur Umgehung von Nutzungsgrenzen oder Abrechnungspflichten ist untersagt.',
        '<strong class="text-white">4.4. Nutzer.</strong> Sie sind dafuer verantwortlich, dass alle Nutzer, die ueber Ihr Konto auf den Dienst zugreifen, diese Bedingungen einhalten. Sie bleiben fuer deren Handlungen und Unterlassungen vollumfaenglich verantwortlich.',
      ],
    },
    {
      id: 'pricing',
      title: '5. Abonnementplaene und Preise',
      paragraphs: [
        '<strong class="text-white">5.1. Preise.</strong> Die aktuellen Preise fuer jeden Abonnementplan sind auf hook0.com veroeffentlicht. Alle Preise verstehen sich netto, zuzueglich der gesetzlichen Umsatzsteuer und sonstiger anwendbarer Abgaben.',
        '<strong class="text-white">5.2. Kommerzielle Bedingungen.</strong> Abrechnungszyklen, Zahlungsarten, Rechnungsstellung und sonstige kommerzielle Bedingungen unterliegen den <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Allgemeinen Geschaeftsbedingungen</a>.',
        '<strong class="text-white">5.3. Preisaenderungen.</strong> Hook0 behaelt sich vor, die Preise eines Abonnementplans zu aendern. Jede Preiserhoehung wird Ihnen mindestens 30 Tage im Voraus per E-Mail an die Ihrem Konto zugeordnete Adresse mitgeteilt. Sind Sie mit dem neuen Preis nicht einverstanden, koennen Sie Ihr Abonnement vor dessen Wirksamwerden kuendigen. Die fortgesetzte Nutzung des Dienstes nach Wirksamwerden einer Preisaenderung gilt als Annahme des neuen Preises.',
        '<strong class="text-white">5.4. Kuendigung und Wechsel.</strong> Sie koennen Ihren Abonnementplan jederzeit kuendigen, hochstufen oder herabstufen, indem Sie <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> kontaktieren. Die Auswirkungen der Kuendigung auf die Abrechnung sind in den Allgemeinen Geschaeftsbedingungen beschrieben.',
        '<strong class="text-white">5.5. Zahlungsverzug.</strong> Gemaess Artikel L441-10 des franzoesischen Code de commerce fallen bei jeder zum Faelligkeitstermin nicht bezahlten Rechnung von Rechts wegen Verzugszinsen in Hoehe des Dreifachen des von der Europaeischen Zentralbank veroeffentlichten gesetzlichen Zinssatzes sowie eine pauschale Beitreibungsgebuehr von 40 EUR je Rechnung an, ohne dass es einer Mahnung beduerfte. Hoehere Beitreibungskosten koennen gegen Nachweis in Rechnung gestellt werden.',
      ],
    },
    {
      id: 'ip',
      title: '6. Geistiges Eigentum',
      paragraphs: [
        '<strong class="text-white">6.1. Geistiges Eigentum von Hook0.</strong> Hook0 und seine Lizenzgeber halten saemtliche Rechte des geistigen Eigentums am Dienst, einschliesslich Software, Quellcode, Schnittstellen, Dokumentation, Marken, Logos und kommerzieller Ausstattung. Keine Bestimmung dieser Bedingungen raeumt Ihnen Rechte am Dienst ein, die ueber das beschraenkte Recht zur Nutzung gemaess diesen Bedingungen hinausgehen.',
        '<strong class="text-white">6.2. Nutzungslizenz fuer den Dienst.</strong> Vorbehaltlich Ihrer Einhaltung dieser Bedingungen und der Zahlung der anwendbaren Entgelte raeumt Hook0 Ihnen ein beschraenktes, nicht ausschliessliches, nicht uebertragbares und nicht unterlizenzierbares Recht ein, waehrend der Laufzeit Ihres Abonnements auf den Dienst zuzugreifen und ihn fuer Ihre internen geschaeftlichen Zwecke zu nutzen.',
        '<strong class="text-white">6.3. Ihre Inhalte.</strong> Sie behalten saemtliche Eigentumsrechte an Ihren Inhalten. Mit der Uebermittlung von Inhalten ueber den Dienst raeumen Sie Hook0 eine beschraenkte, weltweite Lizenz ein, Ihre Inhalte ausschliesslich in dem fuer die Erbringung des Dienstes erforderlichen Umfang zu verarbeiten und zu speichern. Hook0 nutzt Ihre Inhalte zu keinem anderen Zweck.',
        '<strong class="text-white">6.4. Feedback.</strong> Wenn Sie Vorschlaege, Kommentare oder sonstiges Feedback zum Dienst geben (das « Feedback »), raeumen Sie Hook0 eine weltweite, unbefristete, unwiderrufliche und unentgeltliche Lizenz ein, dieses Feedback im Dienst oder in einem anderen Hook0-Produkt oder -Dienst zu nutzen und einzubinden, ohne Verpflichtung zur Verguetung oder Nennung.',
        '<strong class="text-white">6.5. Quelloffene Komponenten.</strong> Der Hook0-Server wird unter der Server Side Public License v1 (SSPL-1.0), einer quelloffenen Lizenz, veroeffentlicht. Bestimmte weitere Komponenten des Dienstes unterliegen separaten Drittlizenzen, quelloffen oder nicht. Keine Bestimmung dieser Bedingungen schraenkt die Ihnen aufgrund dieser Lizenzen zustehenden Rechte ein, die bei Widersprueche Vorrang haben.',
      ],
    },
    {
      id: 'obligations',
      title: '7. Pflichten des Nutzers',
      paragraphs: [
        '<strong class="text-white">7.1. Zulaessige Nutzung.</strong> Sie verpflichten sich, den Dienst ausschliesslich fuer rechtmaessige geschaeftliche Zwecke und im Einklang mit diesen Bedingungen, dem anwendbaren Recht und allen von Hook0 veroeffentlichten Nutzungsrichtlinien zu nutzen.',
        '<strong class="text-white">7.2. Unzulaessige Nutzung.</strong> Ohne Einschraenkung des Vorstehenden verpflichten Sie sich, Folgendes zu unterlassen:',
      ],
      prohibitedList: [
        '(a) den Dienst zum Versand unaufgeforderter kommerzieller Kommunikation (Spam) oder zur Erleichterung von Phishing, Betrug oder sonstigen rechtswidrigen Handlungen einzusetzen;',
        '(b) den Dienst in einer Weise zu nutzen, die geltendes Recht oder geltende Vorschriften verletzt, einschliesslich der Datenschutz- und Privatsphaere-Regeln;',
        '(c) zu versuchen, den Quellcode eines Teils des Dienstes durch Reverse Engineering, Dekompilierung, Disassemblierung oder auf andere Weise abzuleiten, ausser soweit dies durch zwingendes anwendbares Recht oder durch eine anwendbare quelloffene Lizenz ausdruecklich gestattet ist;',
        '(d) Sicherheitsfunktionen, Nutzungsgrenzen oder Zugriffskontrollen des Dienstes zu umgehen, ausser Kraft zu setzen oder zu stoeren;',
        '(e) ueber den Dienst Schadsoftware, Viren oder sonstigen schaedlichen oder stoerenden Code zu uebertragen;',
        '(f) den Dienst ohne vorherige schriftliche Zustimmung von Hook0 weiterzuverkaufen, unterzulizenzieren oder Dritten zugaenglich zu machen;',
        '(g) automatisierte Mittel einzusetzen, um auf den Dienst zuzugreifen oder Inhalte daraus zu extrahieren, ausser ueber die offizielle API und im Einklang mit der Dokumentation;',
        '(h) jede Handlung vorzunehmen, die der Infrastruktur des Dienstes eine unangemessene oder unverhaeltnismaessige Last auferlegt.',
      ],
      paragraphsAfter: [
        '<strong class="text-white">7.3. Verantwortung fuer Inhalte.</strong> Sie sind allein fuer saemtliche Inhalte verantwortlich, die Sie ueber den Dienst uebertragen oder darin speichern. Sie erklaeren und gewaehrleisten, ueber alle Rechte zu verfuegen, die erforderlich sind, um diese Inhalte im Rahmen des Dienstes zu nutzen, und dass Ihre Inhalte keine Rechte Dritter verletzen.',
        '<strong class="text-white">7.4. Sperrung.</strong> Hook0 behaelt sich vor, Ihren Zugriff auf den Dienst mit sofortiger Wirkung und ohne Vorankuendigung zu sperren, wenn vernuenftigerweise davon auszugehen ist, dass Ihre Nutzung diese Bedingungen verletzt oder ein Risiko fuer den Dienst oder Dritte darstellt. Hook0 benachrichtigt Sie ueber eine solche Sperrung baldmoeglichst.',
      ],
    },
    {
      id: 'privacy',
      title: '8. Datenschutz',
      paragraphs: [
        '<strong class="text-white">8.1. Datenschutzerklaerung.</strong> Die Erhebung und Nutzung personenbezogener Daten durch Hook0 im Rahmen des Dienstes richtet sich nach der <a href="/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Datenschutzerklaerung</a>, die durch Verweis in diese Bedingungen einbezogen wird. Lesen Sie sie sorgfaeltig.',
        '<strong class="text-white">8.2. Auftragsverarbeitungsvertrag und Unterauftragsverarbeiter.</strong> Soweit Hook0 personenbezogene Daten in Ihrem Auftrag als Auftragsverarbeiter im Sinne der Verordnung (EU) 2016/679 (DSGVO) verarbeitet, gilt der <a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Auftragsverarbeitungsvertrag (AVV)</a>, der durch Verweis in diese Bedingungen einbezogen wird. Die aktuelle Liste der Unterauftragsverarbeiter, darunter das Hosting (Clever Cloud SAS, Frankreich) sowie die Inhaltsauslieferung und Edge-Sicherheit (Cloudflare Inc., Vereinigte Staaten), ist auf <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a> veroeffentlicht. Uebermittlungen personenbezogener Daten ausserhalb des Europaeischen Wirtschaftsraums werden durch die im Auftragsverarbeitungsvertrag beschriebenen geeigneten Garantien abgesichert. Sie sind dafuer verantwortlich, dass Ihre Nutzung des Dienstes mit dem anwendbaren Datenschutzrecht im Einklang steht, einschliesslich des Einholens erforderlicher Einwilligungen von Betroffenen.',
        '<strong class="text-white">8.3. Ihre Pflichten.</strong> Hinsichtlich personenbezogener Daten in Ihren Inhalten sind Sie Verantwortlicher im Sinne der DSGVO. Sie sind dafuer verantwortlich, ueber eine geeignete Rechtsgrundlage fuer die Verarbeitung dieser Daten ueber den Dienst zu verfuegen und den Betroffenen alle erforderlichen Informationen zu erteilen.',
      ],
    },
    {
      id: 'confidentiality',
      title: '9. Vertraulichkeit',
      paragraphs: [
        '<strong class="text-white">9.1. Definition.</strong> « Vertrauliche Informationen » bezeichnet jede nicht oeffentliche Information, die von einer Partei (die « offenlegende Partei ») gegenueber der anderen Partei (die « empfangende Partei ») offengelegt wird und als vertraulich gekennzeichnet ist oder die nach Art der Information und den Umstaenden der Offenlegung vernuenftigerweise als vertraulich zu verstehen ist. Fuer Hook0 umfassen die Vertraulichen Informationen den Quellcode des Dienstes (mit Ausnahme von Komponenten, die unter einer quelloffenen Lizenz veroeffentlicht sind), seine Architektur, seine Preisstruktur und seine Geschaeftsstrategien. Fuer Sie umfassen sie Ihre Inhalte und Ihre Geschaeftsdaten.',
        '<strong class="text-white">9.2. Pflichten.</strong> Jede Partei verpflichtet sich, (a) die Vertraulichen Informationen der anderen Partei streng vertraulich zu behandeln; (b) die Vertraulichen Informationen ausschliesslich zur Erfuellung ihrer Pflichten oder zur Ausuebung ihrer Rechte nach diesen Bedingungen zu verwenden; und (c) die Vertraulichen Informationen ohne vorherige schriftliche Zustimmung der offenlegenden Partei nicht an Dritte weiterzugeben, ausser an Arbeitnehmer, Dienstleister oder Berater, die einen begruendeten Kenntnisbedarf haben und mindestens ebenso strengen Vertraulichkeitspflichten unterliegen wie nach diesen Bedingungen.',
        '<strong class="text-white">9.3. Ausnahmen.</strong> Die Vertraulichkeitspflichten aus Abschnitt 9.2 gelten nicht fuer Informationen, die: (a) ohne Verschulden der empfangenden Partei oeffentlich zugaenglich sind oder werden; (b) der empfangenden Partei vor der Offenlegung bereits bekannt waren; (c) ohne Einschraenkung von einem Dritten erhalten wurden; oder (d) aufgrund Gesetzes oder gerichtlicher Anordnung offengelegt werden muessen, vorausgesetzt, die empfangende Partei unterrichtet die offenlegende Partei (soweit gesetzlich zulaessig) unverzueglich schriftlich und wirkt bei Bemuehungen um eine Schutzanordnung mit.',
      ],
    },
    {
      id: 'warranties',
      title: '10. Gewaehrleistungsausschluss',
      paragraphs: [
        '<strong class="text-white">10.1. Bereitstellung « wie besehen ».</strong> Der Dienst wird « wie besehen » und « je nach Verfuegbarkeit » bereitgestellt. Im groesstmoeglichen nach anwendbarem Recht zulaessigen Umfang schliesst Hook0 alle ausdruecklichen oder konkludenten Gewaehrleistungen aus, insbesondere fuer handelsuebliche Qualitaet, Eignung fuer einen bestimmten Zweck und Nichtverletzung von Schutzrechten.',
        '<strong class="text-white">10.2. Keine standardmaessigen Service-Level-Garantien.</strong> Hook0 gewaehrleistet nicht, dass der Dienst ununterbrochen, fehlerfrei oder frei von Schwachstellen ist. Standardmaessig werden keine Garantien hinsichtlich Verfuegbarkeit, Antwort- oder Reaktionszeiten, Latenz oder Support gegeben. Etwaige Service-Level-Verpflichtungen sind ausschliesslich in einer separaten, von Hook0 unterzeichneten schriftlichen Enterprise-Vereinbarung festgelegt. Geplante und ungeplante Wartungsarbeiten koennen zu voruebergehender Nichtverfuegbarkeit fuehren. Hook0 unternimmt wirtschaftlich angemessene Anstrengungen, geplante Wartungen vorab anzukuendigen, soweit moeglich.',
        '<strong class="text-white">10.3. Keine Garantie fuer Ergebnisse.</strong> Hook0 gewaehrleistet nicht, dass der Dienst Ihre spezifischen Anforderungen erfuellt oder dass die ueber den Dienst erzielten Ergebnisse zutreffend, vollstaendig oder zuverlaessig sind.',
        '<strong class="text-white">10.4. Reichweite.</strong> Keine Bestimmung dieser Bedingungen schliesst Gewaehrleistungen aus oder beschraenkt sie, die nach zwingenden Vorschriften des anwendbaren Rechts, insbesondere des franzoesischen Rechts, nicht ausgeschlossen oder beschraenkt werden duerfen.',
      ],
    },
    {
      id: 'liability',
      title: '11. Haftungsbeschraenkung',
      paragraphs: [
        '<strong class="text-white">11.1. Haftungsobergrenze.</strong> Im groesstmoeglichen nach anwendbarem Recht zulaessigen Umfang uebersteigt die gesamte Haftung von Hook0 Ihnen gegenueber aus oder im Zusammenhang mit diesen Bedingungen oder dem Dienst, gleichgueltig ob aus Vertrag, unerlaubter Handlung (einschliesslich Fahrlaessigkeit) oder anderweitig, nicht die Summe der Entgelte, die Sie in den zwoelf (12) Monaten unmittelbar vor dem schadensausloesenden Ereignis an Hook0 gezahlt haben. Diese Beschraenkung gilt auch, wenn Hook0 auf die Moeglichkeit solcher Schaeden hingewiesen wurde, und stellt einen wesentlichen Bestandteil der wirtschaftlichen Geschaeftsgrundlage dar.',
        '<strong class="text-white">11.2. Ausschluss mittelbarer Schaeden.</strong> Im groesstmoeglichen nach anwendbarem Recht zulaessigen Umfang haftet Hook0 nicht fuer mittelbare, beilaeufig entstandene, besondere, Folge- oder Strafschaeden, insbesondere entgangenen Gewinn, entgangene Einnahmen, Datenverlust, Verlust von Goodwill oder Betriebsunterbrechung, die aus oder im Zusammenhang mit dem Dienst oder diesen Bedingungen entstehen, ungeachtet der herangezogenen Haftungstheorie. Gemaess Artikel 1231-3 des franzoesischen Code civil ist die Haftung von Hook0 fuer Schaeden, die nicht unmittelbare und direkte Folge einer Pflichtverletzung von Hook0 sind, im groesstmoeglichen nach Gesetz zulaessigen Umfang ausgeschlossen.',
        '<strong class="text-white">11.3. Zwingendes Recht.</strong> Keine Bestimmung dieser Bedingungen beschraenkt oder schliesst die Haftung aus, die nach zwingendem anwendbaren Recht nicht beschraenkt oder ausgeschlossen werden darf, insbesondere die Haftung fuer Tod oder Koerperverletzung infolge Fahrlaessigkeit sowie die Haftung fuer Arglist (dol) und grobe Fahrlaessigkeit (faute lourde).',
      ],
    },
    {
      id: 'term',
      title: '12. Laufzeit und Kuendigung',
      paragraphs: [
        '<strong class="text-white">12.1. Laufzeit.</strong> Diese Bedingungen werden ab dem Zeitpunkt wirksam, zu dem Sie sich erstmals fuer den Dienst registrieren oder darauf zugreifen, und gelten auf unbestimmte Zeit bis zu ihrer Beendigung gemaess diesem Abschnitt 12.',
        '<strong class="text-white">12.2. Kuendigung durch Sie.</strong> Sie koennen Ihr Abonnement jederzeit kuendigen und Ihr Konto schliessen, indem Sie <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> kontaktieren. Die Kuendigung wird zum Ende des laufenden Abrechnungszeitraums wirksam, sofern nichts anderes vereinbart wird. Gezahlte Entgelte sind nicht erstattungsfaehig, ausser wie in den Allgemeinen Geschaeftsbedingungen vorgesehen.',
        '<strong class="text-white">12.3. Kuendigung durch Hook0 aus wichtigem Grund.</strong> Hook0 kann Ihren Zugriff auf den Dienst mit einer schriftlichen Frist von 15 Tagen kuendigen, wenn Sie wesentlich gegen diese Bedingungen verstossen und den Verstoss innerhalb der Frist nicht beheben. Hook0 kann bei schweren Verstoessen ohne Frist und ohne Vorankuendigung kuendigen, etwa bei rechtswidriger Nutzung des Dienstes, bei Verhalten, das ein unmittelbares Sicherheitsrisiko fuer den Dienst oder Dritte darstellt, oder bei Nichtzahlung von Entgelten nach Mahnung.',
        '<strong class="text-white">12.4. Kuendigung durch Hook0 ohne Grund.</strong> Hook0 kann diese Bedingungen aus beliebigem Grund mit einer schriftlichen Frist von 30 Tagen kuendigen. In diesem Fall erhalten Sie eine anteilige Rueckerstattung der vorausgezahlten Entgelte fuer den Zeitraum nach dem Wirksamwerden der Kuendigung.',
        '<strong class="text-white">12.5. Folgen der Kuendigung.</strong> Bei Beendigung dieser Bedingungen aus beliebigem Grund gilt: (a) saemtliche Ihnen nach diesen Bedingungen eingeraeumten Rechte und Lizenzen enden unverzueglich; (b) Sie haben die Nutzung des Dienstes einzustellen; (c) Hook0 bewahrt Ihre Inhalte 30 Tage nach Wirksamwerden der Kuendigung auf; in dieser Zeit koennen Sie unter <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> einen Export Ihrer Daten anfordern; (d) nach Ablauf dieser 30 Tage kann Hook0 Ihre Inhalte ohne weitere Mitteilung dauerhaft loeschen.',
        '<strong class="text-white">12.6. Fortgeltung.</strong> Die Abschnitte 1, 6.1, 6.4, 9, 10, 11, 12.5, 12.6 und 14 gelten ueber die Beendigung dieser Bedingungen hinaus fort.',
      ],
    },
    {
      id: 'modifications',
      title: '13. Aenderungen dieser Bedingungen',
      paragraphs: [
        '<strong class="text-white">13.1. Mitteilung von Aenderungen.</strong> Hook0 kann diese Bedingungen jederzeit aendern. Bei wesentlichen Aenderungen kuendigt Hook0 diese mindestens 30 Tage im Voraus per E-Mail an die Ihrem Konto zugeordnete Adresse an und veroeffentlicht die aktualisierten Bedingungen auf der Website unter Angabe des neuen Wirksamkeitsdatums.',
        '<strong class="text-white">13.2. Annahme.</strong> Die fortgesetzte Nutzung des Dienstes nach Wirksamwerden der geaenderten Bedingungen gilt als Annahme der Aenderungen. Sind Sie mit den geaenderten Bedingungen nicht einverstanden, haben Sie die Nutzung des Dienstes einzustellen und Ihr Abonnement vor dem Wirksamkeitsdatum zu kuendigen.',
        '<strong class="text-white">13.3. Geringfuegige Aenderungen.</strong> Hook0 kann diese Bedingungen ohne Vorankuendigung aendern, wenn es sich um rein administrative Aenderungen handelt (etwa Korrektur typografischer Fehler oder aktualisierte Kontaktdaten) oder die Aenderung gesetzlich vorgeschrieben ist. Solche Aenderungen werden durch ein aktualisiertes « Letzte Aktualisierung »-Datum am Anfang der Bedingungen gekennzeichnet.',
      ],
    },
    {
      id: 'general',
      title: '14. Allgemeine Bestimmungen',
      paragraphs: [
        '<strong class="text-white">14.1. Anwendbares Recht.</strong> Diese Bedingungen und alle daraus oder im Zusammenhang damit entstehenden Streitigkeiten oder Anspruechen (einschliesslich ausservertraglicher) unterliegen franzoesischem Recht. Das UN-Uebereinkommen ueber Vertraege ueber den internationalen Warenkauf (CISG) findet keine Anwendung.',
        '<strong class="text-white">14.2. Gerichtsstand.</strong> Gemaess Artikel 48 des franzoesischen Code de procedure civile, der zwischen Kaufleuten anwendbar ist, vereinbaren die Parteien fuer alle Streitigkeiten aus oder im Zusammenhang mit diesen Bedingungen die ausschliessliche Zustaendigkeit der Gerichte von La Roche-sur-Yon, Frankreich, in deren Bezirk Hook0 seinen Sitz hat, vorbehaltlich zwingender Gerichtsstandsregeln.',
        '<strong class="text-white">14.3. Hoehere Gewalt.</strong> Keine Partei haftet fuer ein Versaeumnis oder einen Verzug bei der Leistung, der auf Ursachen ausserhalb ihres zumutbaren Einflussbereichs zurueckzufuehren ist, insbesondere hoehere Gewalt, Krieg, Terrorismus, Aufruhr, Brand, Ueberschwemmung, Naturkatastrophen, behoerdliche Massnahmen, Streiks, Aussperrungen oder Ausfaelle von Telekommunikationsnetzen oder -infrastrukturen Dritter. Die betroffene Partei unterrichtet die andere Partei unverzueglich und unternimmt wirtschaftlich angemessene Anstrengungen, die Leistung baldmoeglichst wiederaufzunehmen.',
        '<strong class="text-white">14.4. Abtretung.</strong> Sie duerfen Ihre Rechte oder Pflichten aus diesen Bedingungen ohne vorherige schriftliche Zustimmung von Hook0 weder abtreten noch uebertragen. Hook0 kann diese Bedingungen ganz oder teilweise abtreten, insbesondere im Rahmen einer Verschmelzung, einer Uebernahme oder eines Verkaufs aller oder im Wesentlichen aller Vermoegenswerte, mit schriftlicher Mitteilung an Sie.',
        '<strong class="text-white">14.5. Salvatorische Klausel.</strong> Sollte eine Bestimmung dieser Bedingungen von einem zustaendigen Gericht fuer unwirksam, rechtswidrig oder nicht durchsetzbar befunden werden, gilt sie in dem strikt erforderlichen Mass als eingeschraenkt oder entfallen; die uebrigen Bestimmungen bleiben in vollem Umfang in Kraft.',
        '<strong class="text-white">14.6. Verzicht.</strong> Verzichtet eine Partei darauf, ein Recht oder eine Bestimmung dieser Bedingungen durchzusetzen, gilt dies nicht als Verzicht auf dieses Recht oder diese Bestimmung fuer die Zukunft.',
        '<strong class="text-white">14.7. Verhaeltnis der Parteien.</strong> Keine Bestimmung dieser Bedingungen begruendet eine Partnerschaft, ein Joint Venture, eine Stellvertretung oder ein Arbeitsverhaeltnis zwischen den Parteien und ist nicht so auszulegen. Jede Partei handelt als unabhaengiger Vertragspartner.',
        '<strong class="text-white">14.8. Mitteilungen.</strong> Rechtliche Mitteilungen an Hook0 sind schriftlich per E-Mail an <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a> oder per Post an FGRibreau SARL, 3 rue de l\'Aubepine, 85110 Chantonnay, Frankreich, zu uebermitteln. Hook0 kann Mitteilungen an Sie per E-Mail an die Ihrem Konto zugeordnete Adresse uebermitteln. Elektronische Mitteilungen gelten am Tag der Uebermittlung als zugegangen, sofern keine Zustellfehlermeldung eingeht.',
      ],
    },
    {
      id: 'contact',
      title: '15. Kontakt',
      lead: 'Fuer Fragen, Anmerkungen oder Anliegen zu diesen Bedingungen oder zum Dienst erreichen Sie uns wie folgt:',
      contactItems: [
        '<strong class="text-white">Rechtliche Anliegen:</strong> <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
        '<strong class="text-white">Support:</strong> <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>',
        '<strong class="text-white">Postanschrift:</strong> FGRibreau SARL, 3 rue de l\'Aubepine, 85110 Chantonnay, Frankreich',
      ],
    },
  ],
};
