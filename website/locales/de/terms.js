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
  pageDescription: 'Allgemeine Nutzungsbedingungen für Hook0 Webhooks-as-a-Service. Lesen Sie sorgfältig die Regeln, die den Zugriff auf die Hook0-Plattform und deren Nutzung regeln.',
  pageModified: '2026-06-27',
  hero: {
    eyebrow: 'Rechtliches',
    title: 'Allgemeine Nutzungsbedingungen',
    subtitle: 'Lesen Sie diese Bedingungen sorgfältig, bevor Sie die Hook0-Dienste nutzen.',
    lastUpdatedLabel: 'Letzte Aktualisierung:',
    lastUpdatedDate: '27. Juni 2026',
  },
  intro: {
    p1Html: 'Die vorliegenden Allgemeinen Nutzungsbedingungen (die « Bedingungen ») regeln Ihren Zugriff auf die Hook0-Plattform und die damit verbundenen Dienste (zusammen der « Dienst »), die von der FGRibreau SARL betrieben werden, einer französischen Gesellschaft mit beschränkter Haftung (société à responsabilité limitée) mit einem Stammkapital von 2 000 EUR, eingetragen im Handels- und Gesellschaftsregister La Roche-sur-Yon unter der Nummer 850 824 350, mit Sitz in 3 rue de l\'Aubépine, 85110 Chantonnay, Frankreich, USt-IdNr. FR27850824350 (« Hook0 », « wir » oder « unser »). Verantwortlicher Herausgeber ist David Sferruzza.',
    p2Html: 'Der Dienst richtet sich ausschließlich an Unternehmen und gewerbliche Einrichtungen (B2B). Mit der Registrierung, dem Zugriff oder der Nutzung des Dienstes bestätigen Sie, dass Sie in gewerblicher Eigenschaft für eine juristische Person handeln und befugt sind, diese juristische Person an die vorliegenden Bedingungen zu binden.',
    p3Html: 'MIT DER REGISTRIERUNG, DEM ZUGRIFF AUF ODER DER NUTZUNG DES DIENSTES ERKLÄREN SIE SICH MIT DIESEN BEDINGUNGEN EINVERSTANDEN. WENN SIE DAMIT NICHT EINVERSTANDEN SIND, DÜRFEN SIE WEDER AUF DEN DIENST ZUGREIFEN NOCH IHN NUTZEN.',
    p4Html: 'Die kommerziellen und Abrechnungsbedingungen (Preise, Rechnungsstellung, Zahlungskonditionen) sind in den <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Allgemeinen Geschäftsbedingungen</a> festgelegt, die ein separates, durch Verweis in diese Vereinbarung einbezogenes Dokument bilden.',
  },
  sections: [
    {
      id: 'definitions',
      title: '1. Begriffsbestimmungen',
      lead: 'Im Rahmen dieser Bedingungen haben die folgenden Begriffe die nachstehend angegebene Bedeutung:',
      items: [
        '<strong class="text-white">« Konto »</strong> bezeichnet das von Ihnen angelegte Konto für den Zugriff auf und die Nutzung des Dienstes.',
        '<strong class="text-white">« Inhalte »</strong> bezeichnet sämtliche Daten, Informationen oder Materialien, die Sie über den Dienst übertragen oder darin speichern, einschließlich Webhook-Payloads, Konfigurationen und API-Zugangsdaten.',
        '<strong class="text-white">« Dokumentation »</strong> bezeichnet die technische Dokumentation und die Benutzerhandbücher, die Hook0 auf <a href="https://documentation.hook0.com" class="text-green-400 hover:text-green-300 transition-colors">documentation.hook0.com</a> bereitstellt.',
        '<strong class="text-white">« Dienst »</strong> bezeichnet die Hook0-Webhook-Management-Plattform, einschließlich sämtlicher zugehöriger APIs, Schnittstellen und ergänzender Leistungen, die Hook0 bereitstellt.',
        '<strong class="text-white">« Unterauftragsverarbeiter »</strong> bezeichnet jeden von Hook0 beauftragten Drittauftragsverarbeiter, der Ihre Inhalte im Rahmen des Dienstes in Ihrem Auftrag verarbeitet. Die aktuelle Liste der Unterauftragsverarbeiter ist auf <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a> veröffentlicht.',
        '<strong class="text-white">« Abonnementplan »</strong> bezeichnet die von Ihnen gewählte Servicestufe (Developer, Startup, Pro oder Enterprise), wie auf der Preisseite von hook0.com beschrieben.',
        '<strong class="text-white">« Nutzer »</strong> bezeichnet jede natürliche Person, die über Ihr Konto in Ihrem Auftrag auf den Dienst zugreift oder ihn nutzt.',
        '<strong class="text-white">« Sie » / « Ihr »</strong> bezeichnet die juristische Person, die sich für den Dienst registriert hat oder ihn nutzt, sowie jeden Nutzer, der in deren Auftrag handelt.',
      ],
    },
    {
      id: 'acceptance',
      title: '2. Annahme und Geltungsbereich',
      paragraphs: [
        '<strong class="text-white">2.1. Ausschließlich gewerbliche Nutzung.</strong> Der Dienst ist ausschließlich für gewerbliche und geschäftliche Nutzung bestimmt. Diese Bedingungen gelten nicht für Verbraucher (natürliche Personen, die außerhalb einer gewerblichen oder beruflichen Tätigkeit handeln). Da der Dienst ausschließlich im B2B-Kontext angeboten wird, findet kein verbraucherrechtliches Widerrufsrecht Anwendung. Mit Annahme dieser Bedingungen erklären und gewährleisten Sie, in gewerblicher Eigenschaft zu handeln.',
        '<strong class="text-white">2.2. Vertretungsbefugnis.</strong> Wenn Sie diese Bedingungen im Namen eines Unternehmens oder einer anderen juristischen Person annehmen, erklären und gewährleisten Sie, über die rechtliche Befugnis zu verfügen, diese juristische Person zu binden. In diesem Fall bezeichnet « Sie » diese juristische Person.',
        '<strong class="text-white">2.3. Vollständige Vereinbarung.</strong> Diese Bedingungen bilden zusammen mit der <a href="/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Datenschutzerklärung</a>, dem <a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Auftragsverarbeitungsvertrag (AVV)</a> und den <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Allgemeinen Geschäftsbedingungen</a> die gesamte Vereinbarung zwischen den Parteien über den Dienst und ersetzen alle vorherigen oder gleichzeitigen Absprachen, Zusicherungen oder Vereinbarungen.',
      ],
    },
    {
      id: 'description',
      title: '3. Beschreibung des Dienstes',
      paragraphs: [
        '<strong class="text-white">3.1. Plattformüberblick.</strong> Hook0 ist eine Webhook-Management-Plattform, mit der Unternehmen Webhook-Ereignisse senden, empfangen, verwalten und überwachen können. Der Dienst umfasst die Webhook-Zustellinfrastruktur, die Wiederholungslogik, die Ereignisprotokollierung und zugehörige Entwicklerwerkzeuge.',
        '<strong class="text-white">3.2. Abonnementpläne.</strong> Der Dienst wird in folgenden Abonnementplänen angeboten: Developer (kostenlos), Startup, Pro und Enterprise. Funktionen, Nutzungsgrenzen und Preise je Plan sind auf der Preisseite von hook0.com beschrieben. Hook0 behält sich vor, die mit einem Abonnementplan verbundenen Funktionen zu ändern, einschließlich der Einstellung von Funktionen des kostenlosen Developer-Plans, mit der in Abschnitt 13 vorgesehenen Vorankündigung.',
        '<strong class="text-white">3.3. Aktualisierungen.</strong> Hook0 kann Funktionen des Dienstes jederzeit aktualisieren, ändern oder einstellen. Bei wesentlichen Änderungen kündigt Hook0 diese mit angemessener Frist vorab an. Die fortgesetzte Nutzung des Dienstes nach solchen Änderungen gilt als Annahme des aktualisierten Dienstes.',
        '<strong class="text-white">3.4. Drittanbieter-Dienste und -Infrastruktur.</strong> Der Dienst stützt sich auf Drittanbieter-Infrastrukturen, insbesondere das Hosting durch Clever Cloud SAS (Frankreich) sowie die Inhaltsauslieferung und die Edge-Sicherheit durch Cloudflare Inc. (Vereinigte Staaten). Die aktuelle Liste der Unterauftragsverarbeiter und deren Standorte ist auf <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a> veröffentlicht. Der Dienst kann sich zudem in weitere Drittanbieter-Dienste integrieren oder auf solche verweisen, über die Hook0 keine Kontrolle hat. Ihre Nutzung dieser Dienste unterliegt ausschließlich den jeweils anwendbaren Bedingungen der Drittanbieter.',
      ],
    },
    {
      id: 'account',
      title: '4. Kontoregistrierung',
      paragraphs: [
        '<strong class="text-white">4.1. Korrekte Angaben.</strong> Sie verpflichten sich, bei der Registrierung eines Kontos korrekte, aktuelle und vollständige Angaben zu machen und diese aktuell zu halten. Hook0 kann Ihr Konto sperren oder kündigen, wenn festgestellt wird, dass die Angaben falsch oder irreführend sind.',
        '<strong class="text-white">4.2. Kontosicherheit.</strong> Sie sind für die Vertraulichkeit Ihrer Konto-Zugangsdaten und für alle Aktivitäten unter Ihrem Konto verantwortlich. Sie verpflichten sich, Hook0 unverzüglich unter <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> zu benachrichtigen, wenn Sie von einem unbefugten Zugriff auf Ihr Konto oder dessen unbefugter Nutzung Kenntnis erlangen.',
        '<strong class="text-white">4.3. Ein Konto je juristische Person.</strong> Jede juristische Person darf nur ein Konto unterhalten, sofern Hook0 nicht ausdrücklich schriftlich etwas anderes vereinbart hat. Die Anlage mehrerer Konten zur Umgehung von Nutzungsgrenzen oder Abrechnungspflichten ist untersagt.',
        '<strong class="text-white">4.4. Nutzer.</strong> Sie sind dafür verantwortlich, dass alle Nutzer, die über Ihr Konto auf den Dienst zugreifen, diese Bedingungen einhalten. Sie bleiben für deren Handlungen und Unterlassungen vollumfänglich verantwortlich.',
      ],
    },
    {
      id: 'pricing',
      title: '5. Abonnementpläne und Preise',
      paragraphs: [
        '<strong class="text-white">5.1. Preise.</strong> Die aktuellen Preise für jeden Abonnementplan sind auf hook0.com veröffentlicht. Alle Preise verstehen sich netto, zuzüglich der gesetzlichen Umsatzsteuer und sonstiger anwendbarer Abgaben.',
        '<strong class="text-white">5.2. Kommerzielle Bedingungen.</strong> Abrechnungszyklen, Zahlungsarten, Rechnungsstellung und sonstige kommerzielle Bedingungen unterliegen den <a href="/terms-of-sale" class="text-green-400 hover:text-green-300 transition-colors">Allgemeinen Geschäftsbedingungen</a>.',
        '<strong class="text-white">5.3. Preisänderungen.</strong> Hook0 behält sich vor, die Preise eines Abonnementplans zu ändern. Jede Preiserhöhung wird Ihnen mindestens 30 Tage im Voraus per E-Mail an die Ihrem Konto zugeordnete Adresse mitgeteilt. Sind Sie mit dem neuen Preis nicht einverstanden, können Sie Ihr Abonnement vor dessen Wirksamwerden kündigen. Die fortgesetzte Nutzung des Dienstes nach Wirksamwerden einer Preisänderung gilt als Annahme des neuen Preises.',
        '<strong class="text-white">5.4. Kündigung und Wechsel.</strong> Sie können Ihren Abonnementplan jederzeit kündigen, hochstufen oder herabstufen, indem Sie <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> kontaktieren. Die Auswirkungen der Kündigung auf die Abrechnung sind in den Allgemeinen Geschäftsbedingungen beschrieben.',
        '<strong class="text-white">5.5. Zahlungsverzug.</strong> Gemäß Artikel L441-10 des französischen Code de commerce fallen bei jeder zum Fälligkeitstermin nicht bezahlten Rechnung von Rechts wegen Verzugszinsen in Höhe des Dreifachen des von der Europäischen Zentralbank veröffentlichten gesetzlichen Zinssatzes sowie eine pauschale Beitreibungsgebühr von 40 EUR je Rechnung an, ohne dass es einer Mahnung bedürfte. Höhere Beitreibungskosten können gegen Nachweis in Rechnung gestellt werden.',
      ],
    },
    {
      id: 'ip',
      title: '6. Geistiges Eigentum',
      paragraphs: [
        '<strong class="text-white">6.1. Geistiges Eigentum von Hook0.</strong> Hook0 und seine Lizenzgeber halten sämtliche Rechte des geistigen Eigentums am Dienst, einschließlich Software, Quellcode, Schnittstellen, Dokumentation, Marken, Logos und kommerzieller Ausstattung. Keine Bestimmung dieser Bedingungen räumt Ihnen Rechte am Dienst ein, die über das beschränkte Recht zur Nutzung gemäß diesen Bedingungen hinausgehen.',
        '<strong class="text-white">6.2. Nutzungslizenz für den Dienst.</strong> Vorbehaltlich Ihrer Einhaltung dieser Bedingungen und der Zahlung der anwendbaren Entgelte räumt Hook0 Ihnen ein beschränktes, nicht ausschließliches, nicht übertragbares und nicht unterlizenzierbares Recht ein, während der Laufzeit Ihres Abonnements auf den Dienst zuzugreifen und ihn für Ihre internen geschäftlichen Zwecke zu nutzen.',
        '<strong class="text-white">6.3. Ihre Inhalte.</strong> Sie behalten sämtliche Eigentumsrechte an Ihren Inhalten. Mit der Übermittlung von Inhalten über den Dienst räumen Sie Hook0 eine beschränkte, weltweite Lizenz ein, Ihre Inhalte ausschließlich in dem für die Erbringung des Dienstes erforderlichen Umfang zu verarbeiten und zu speichern. Hook0 nutzt Ihre Inhalte zu keinem anderen Zweck.',
        '<strong class="text-white">6.4. Feedback.</strong> Wenn Sie Vorschläge, Kommentare oder sonstiges Feedback zum Dienst geben (das « Feedback »), räumen Sie Hook0 eine weltweite, unbefristete, unwiderrufliche und unentgeltliche Lizenz ein, dieses Feedback im Dienst oder in einem anderen Hook0-Produkt oder -Dienst zu nutzen und einzubinden, ohne Verpflichtung zur Vergütung oder Nennung.',
        '<strong class="text-white">6.5. Quelloffene Komponenten.</strong> Der Hook0-Server wird unter der Server Side Public License v1 (SSPL-1.0), einer quelloffenen Lizenz, veröffentlicht. Bestimmte weitere Komponenten des Dienstes unterliegen separaten Drittlizenzen, quelloffen oder nicht. Keine Bestimmung dieser Bedingungen schränkt die Ihnen aufgrund dieser Lizenzen zustehenden Rechte ein, die bei Widersprüche Vorrang haben.',
      ],
    },
    {
      id: 'obligations',
      title: '7. Pflichten des Nutzers',
      paragraphs: [
        '<strong class="text-white">7.1. Zulässige Nutzung.</strong> Sie verpflichten sich, den Dienst ausschließlich für rechtmäßige geschäftliche Zwecke und im Einklang mit diesen Bedingungen, dem anwendbaren Recht und allen von Hook0 veröffentlichten Nutzungsrichtlinien zu nutzen.',
        '<strong class="text-white">7.2. Unzulässige Nutzung.</strong> Ohne Einschränkung des Vorstehenden verpflichten Sie sich, Folgendes zu unterlassen:',
      ],
      prohibitedList: [
        '(a) den Dienst zum Versand unaufgeforderter kommerzieller Kommunikation (Spam) oder zur Erleichterung von Phishing, Betrug oder sonstigen rechtswidrigen Handlungen einzusetzen;',
        '(b) den Dienst in einer Weise zu nutzen, die geltendes Recht oder geltende Vorschriften verletzt, einschließlich der Datenschutz- und Privatsphäre-Regeln;',
        '(c) zu versuchen, den Quellcode eines Teils des Dienstes durch Reverse Engineering, Dekompilierung, Disassemblierung oder auf andere Weise abzuleiten, außer soweit dies durch zwingendes anwendbares Recht oder durch eine anwendbare quelloffene Lizenz ausdrücklich gestattet ist;',
        '(d) Sicherheitsfunktionen, Nutzungsgrenzen oder Zugriffskontrollen des Dienstes zu umgehen, außer Kraft zu setzen oder zu stören;',
        '(e) über den Dienst Schadsoftware, Viren oder sonstigen schädlichen oder störenden Code zu übertragen;',
        '(f) den Dienst ohne vorherige schriftliche Zustimmung von Hook0 weiterzuverkaufen, unterzulizenzieren oder Dritten zugänglich zu machen;',
        '(g) automatisierte Mittel einzusetzen, um auf den Dienst zuzugreifen oder Inhalte daraus zu extrahieren, außer über die offizielle API und im Einklang mit der Dokumentation;',
        '(h) jede Handlung vorzunehmen, die der Infrastruktur des Dienstes eine unangemessene oder unverhältnismäßige Last auferlegt.',
      ],
      paragraphsAfter: [
        '<strong class="text-white">7.3. Verantwortung für Inhalte.</strong> Sie sind allein für sämtliche Inhalte verantwortlich, die Sie über den Dienst übertragen oder darin speichern. Sie erklären und gewährleisten, über alle Rechte zu verfügen, die erforderlich sind, um diese Inhalte im Rahmen des Dienstes zu nutzen, und dass Ihre Inhalte keine Rechte Dritter verletzen.',
        '<strong class="text-white">7.4. Sperrung.</strong> Hook0 behält sich vor, Ihren Zugriff auf den Dienst mit sofortiger Wirkung und ohne Vorankündigung zu sperren, wenn vernünftigerweise davon auszugehen ist, dass Ihre Nutzung diese Bedingungen verletzt oder ein Risiko für den Dienst oder Dritte darstellt. Hook0 benachrichtigt Sie über eine solche Sperrung baldmöglichst.',
      ],
    },
    {
      id: 'privacy',
      title: '8. Datenschutz',
      paragraphs: [
        '<strong class="text-white">8.1. Datenschutzerklärung.</strong> Die Erhebung und Nutzung personenbezogener Daten durch Hook0 im Rahmen des Dienstes richtet sich nach der <a href="/privacy-policy" class="text-green-400 hover:text-green-300 transition-colors">Datenschutzerklärung</a>, die durch Verweis in diese Bedingungen einbezogen wird. Lesen Sie sie sorgfältig.',
        '<strong class="text-white">8.2. Auftragsverarbeitungsvertrag und Unterauftragsverarbeiter.</strong> Soweit Hook0 personenbezogene Daten in Ihrem Auftrag als Auftragsverarbeiter im Sinne der Verordnung (EU) 2016/679 (DSGVO) verarbeitet, gilt der <a href="/data-processing-addendum" class="text-green-400 hover:text-green-300 transition-colors">Auftragsverarbeitungsvertrag (AVV)</a>, der durch Verweis in diese Bedingungen einbezogen wird. Die aktuelle Liste der Unterauftragsverarbeiter, darunter das Hosting (Clever Cloud SAS, Frankreich) sowie die Inhaltsauslieferung und Edge-Sicherheit (Cloudflare Inc., Vereinigte Staaten), ist auf <a href="/gdpr-subprocessors" class="text-green-400 hover:text-green-300 transition-colors">hook0.com/gdpr-subprocessors</a> veröffentlicht. Übermittlungen personenbezogener Daten außerhalb des Europäischen Wirtschaftsraums werden durch die im Auftragsverarbeitungsvertrag beschriebenen geeigneten Garantien abgesichert. Sie sind dafür verantwortlich, dass Ihre Nutzung des Dienstes mit dem anwendbaren Datenschutzrecht im Einklang steht, einschließlich des Einholens erforderlicher Einwilligungen von Betroffenen.',
        '<strong class="text-white">8.3. Ihre Pflichten.</strong> Hinsichtlich personenbezogener Daten in Ihren Inhalten sind Sie Verantwortlicher im Sinne der DSGVO. Sie sind dafür verantwortlich, über eine geeignete Rechtsgrundlage für die Verarbeitung dieser Daten über den Dienst zu verfügen und den Betroffenen alle erforderlichen Informationen zu erteilen.',
      ],
    },
    {
      id: 'confidentiality',
      title: '9. Vertraulichkeit',
      paragraphs: [
        '<strong class="text-white">9.1. Definition.</strong> « Vertrauliche Informationen » bezeichnet jede nicht öffentliche Information, die von einer Partei (die « offenlegende Partei ») gegenüber der anderen Partei (die « empfangende Partei ») offengelegt wird und als vertraulich gekennzeichnet ist oder die nach Art der Information und den Umständen der Offenlegung vernünftigerweise als vertraulich zu verstehen ist. Für Hook0 umfassen die Vertraulichen Informationen den Quellcode des Dienstes (mit Ausnahme von Komponenten, die unter einer quelloffenen Lizenz veröffentlicht sind), seine Architektur, seine Preisstruktur und seine Geschäftsstrategien. Für Sie umfassen sie Ihre Inhalte und Ihre Geschäftsdaten.',
        '<strong class="text-white">9.2. Pflichten.</strong> Jede Partei verpflichtet sich, (a) die Vertraulichen Informationen der anderen Partei streng vertraulich zu behandeln; (b) die Vertraulichen Informationen ausschließlich zur Erfüllung ihrer Pflichten oder zur Ausübung ihrer Rechte nach diesen Bedingungen zu verwenden; und (c) die Vertraulichen Informationen ohne vorherige schriftliche Zustimmung der offenlegenden Partei nicht an Dritte weiterzugeben, außer an Arbeitnehmer, Dienstleister oder Berater, die einen begründeten Kenntnisbedarf haben und mindestens ebenso strengen Vertraulichkeitspflichten unterliegen wie nach diesen Bedingungen.',
        '<strong class="text-white">9.3. Ausnahmen.</strong> Die Vertraulichkeitspflichten aus Abschnitt 9.2 gelten nicht für Informationen, die: (a) ohne Verschulden der empfangenden Partei öffentlich zugänglich sind oder werden; (b) der empfangenden Partei vor der Offenlegung bereits bekannt waren; (c) ohne Einschränkung von einem Dritten erhalten wurden; oder (d) aufgrund Gesetzes oder gerichtlicher Anordnung offengelegt werden müssen, vorausgesetzt, die empfangende Partei unterrichtet die offenlegende Partei (soweit gesetzlich zulässig) unverzüglich schriftlich und wirkt bei Bemühungen um eine Schutzanordnung mit.',
      ],
    },
    {
      id: 'warranties',
      title: '10. Gewährleistungsausschluss',
      paragraphs: [
        '<strong class="text-white">10.1. Bereitstellung « wie besehen ».</strong> Der Dienst wird « wie besehen » und « je nach Verfügbarkeit » bereitgestellt. Im größtmöglichen nach anwendbarem Recht zulässigen Umfang schließt Hook0 alle ausdrücklichen oder konkludenten Gewährleistungen aus, insbesondere für handelsübliche Qualität, Eignung für einen bestimmten Zweck und Nichtverletzung von Schutzrechten.',
        '<strong class="text-white">10.2. Keine standardmäßigen Service-Level-Garantien.</strong> Hook0 gewährleistet nicht, dass der Dienst ununterbrochen, fehlerfrei oder frei von Schwachstellen ist. Standardmäßig werden keine Garantien hinsichtlich Verfügbarkeit, Antwort- oder Reaktionszeiten, Latenz oder Support gegeben. Etwaige Service-Level-Verpflichtungen sind ausschließlich in einer separaten, von Hook0 unterzeichneten schriftlichen Enterprise-Vereinbarung festgelegt. Geplante und ungeplante Wartungsarbeiten können zu vorübergehender Nichtverfügbarkeit führen. Hook0 unternimmt wirtschaftlich angemessene Anstrengungen, geplante Wartungen vorab anzukündigen, soweit möglich.',
        '<strong class="text-white">10.3. Keine Garantie für Ergebnisse.</strong> Hook0 gewährleistet nicht, dass der Dienst Ihre spezifischen Anforderungen erfüllt oder dass die über den Dienst erzielten Ergebnisse zutreffend, vollständig oder zuverlässig sind.',
        '<strong class="text-white">10.4. Reichweite.</strong> Keine Bestimmung dieser Bedingungen schließt Gewährleistungen aus oder beschränkt sie, die nach zwingenden Vorschriften des anwendbaren Rechts, insbesondere des französischen Rechts, nicht ausgeschlossen oder beschränkt werden dürfen.',
      ],
    },
    {
      id: 'liability',
      title: '11. Haftungsbeschränkung',
      paragraphs: [
        '<strong class="text-white">11.1. Haftungsobergrenze.</strong> Im größtmöglichen nach anwendbarem Recht zulässigen Umfang übersteigt die gesamte Haftung von Hook0 Ihnen gegenüber aus oder im Zusammenhang mit diesen Bedingungen oder dem Dienst, gleichgültig ob aus Vertrag, unerlaubter Handlung (einschließlich Fahrlässigkeit) oder anderweitig, nicht die Summe der Entgelte, die Sie in den zwölf (12) Monaten unmittelbar vor dem schadensauslösenden Ereignis an Hook0 gezahlt haben. Diese Beschränkung gilt auch, wenn Hook0 auf die Möglichkeit solcher Schäden hingewiesen wurde, und stellt einen wesentlichen Bestandteil der wirtschaftlichen Geschäftsgrundlage dar.',
        '<strong class="text-white">11.2. Ausschluss mittelbarer Schäden.</strong> Im größtmöglichen nach anwendbarem Recht zulässigen Umfang haftet Hook0 nicht für mittelbare, beiläufig entstandene, besondere, Folge- oder Strafschäden, insbesondere entgangenen Gewinn, entgangene Einnahmen, Datenverlust, Verlust von Goodwill oder Betriebsunterbrechung, die aus oder im Zusammenhang mit dem Dienst oder diesen Bedingungen entstehen, ungeachtet der herangezogenen Haftungstheorie. Gemäß Artikel 1231-3 des französischen Code civil ist die Haftung von Hook0 für Schäden, die nicht unmittelbare und direkte Folge einer Pflichtverletzung von Hook0 sind, im größtmöglichen nach Gesetz zulässigen Umfang ausgeschlossen.',
        '<strong class="text-white">11.3. Zwingendes Recht.</strong> Keine Bestimmung dieser Bedingungen beschränkt oder schließt die Haftung aus, die nach zwingendem anwendbaren Recht nicht beschränkt oder ausgeschlossen werden darf, insbesondere die Haftung für Tod oder Körperverletzung infolge Fahrlässigkeit sowie die Haftung für Arglist (dol) und grobe Fahrlässigkeit (faute lourde).',
      ],
    },
    {
      id: 'term',
      title: '12. Laufzeit und Kündigung',
      paragraphs: [
        '<strong class="text-white">12.1. Laufzeit.</strong> Diese Bedingungen werden ab dem Zeitpunkt wirksam, zu dem Sie sich erstmals für den Dienst registrieren oder darauf zugreifen, und gelten auf unbestimmte Zeit bis zu ihrer Beendigung gemäß diesem Abschnitt 12.',
        '<strong class="text-white">12.2. Kündigung durch Sie.</strong> Sie können Ihr Abonnement jederzeit kündigen und Ihr Konto schließen, indem Sie <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> kontaktieren. Die Kündigung wird zum Ende des laufenden Abrechnungszeitraums wirksam, sofern nichts anderes vereinbart wird. Gezahlte Entgelte sind nicht erstattungsfähig, außer wie in den Allgemeinen Geschäftsbedingungen vorgesehen.',
        '<strong class="text-white">12.3. Kündigung durch Hook0 aus wichtigem Grund.</strong> Hook0 kann Ihren Zugriff auf den Dienst mit einer schriftlichen Frist von 15 Tagen kündigen, wenn Sie wesentlich gegen diese Bedingungen verstoßen und den Verstoß innerhalb der Frist nicht beheben. Hook0 kann bei schweren Verstößen ohne Frist und ohne Vorankündigung kündigen, etwa bei rechtswidriger Nutzung des Dienstes, bei Verhalten, das ein unmittelbares Sicherheitsrisiko für den Dienst oder Dritte darstellt, oder bei Nichtzahlung von Entgelten nach Mahnung.',
        '<strong class="text-white">12.4. Kündigung durch Hook0 ohne Grund.</strong> Hook0 kann diese Bedingungen aus beliebigem Grund mit einer schriftlichen Frist von 30 Tagen kündigen. In diesem Fall erhalten Sie eine anteilige Rückerstattung der vorausgezahlten Entgelte für den Zeitraum nach dem Wirksamwerden der Kündigung.',
        '<strong class="text-white">12.5. Folgen der Kündigung.</strong> Bei Beendigung dieser Bedingungen aus beliebigem Grund gilt: (a) sämtliche Ihnen nach diesen Bedingungen eingeräumten Rechte und Lizenzen enden unverzüglich; (b) Sie haben die Nutzung des Dienstes einzustellen; (c) Hook0 bewahrt Ihre Inhalte 30 Tage nach Wirksamwerden der Kündigung auf; in dieser Zeit können Sie unter <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a> einen Export Ihrer Daten anfordern; (d) nach Ablauf dieser 30 Tage kann Hook0 Ihre Inhalte ohne weitere Mitteilung dauerhaft löschen.',
        '<strong class="text-white">12.6. Fortgeltung.</strong> Die Abschnitte 1, 6.1, 6.4, 9, 10, 11, 12.5, 12.6 und 14 gelten über die Beendigung dieser Bedingungen hinaus fort.',
      ],
    },
    {
      id: 'modifications',
      title: '13. Änderungen dieser Bedingungen',
      paragraphs: [
        '<strong class="text-white">13.1. Mitteilung von Änderungen.</strong> Hook0 kann diese Bedingungen jederzeit ändern. Bei wesentlichen Änderungen kündigt Hook0 diese mindestens 30 Tage im Voraus per E-Mail an die Ihrem Konto zugeordnete Adresse an und veröffentlicht die aktualisierten Bedingungen auf der Website unter Angabe des neuen Wirksamkeitsdatums.',
        '<strong class="text-white">13.2. Annahme.</strong> Die fortgesetzte Nutzung des Dienstes nach Wirksamwerden der geänderten Bedingungen gilt als Annahme der Änderungen. Sind Sie mit den geänderten Bedingungen nicht einverstanden, haben Sie die Nutzung des Dienstes einzustellen und Ihr Abonnement vor dem Wirksamkeitsdatum zu kündigen.',
        '<strong class="text-white">13.3. Geringfügige Änderungen.</strong> Hook0 kann diese Bedingungen ohne Vorankündigung ändern, wenn es sich um rein administrative Änderungen handelt (etwa Korrektur typografischer Fehler oder aktualisierte Kontaktdaten) oder die Änderung gesetzlich vorgeschrieben ist. Solche Änderungen werden durch ein aktualisiertes « Letzte Aktualisierung »-Datum am Anfang der Bedingungen gekennzeichnet.',
      ],
    },
    {
      id: 'general',
      title: '14. Allgemeine Bestimmungen',
      paragraphs: [
        '<strong class="text-white">14.1. Anwendbares Recht.</strong> Diese Bedingungen und alle daraus oder im Zusammenhang damit entstehenden Streitigkeiten oder Ansprüchen (einschließlich außervertraglicher) unterliegen französischem Recht. Das UN-Übereinkommen über Verträge über den internationalen Warenkauf (CISG) findet keine Anwendung.',
        '<strong class="text-white">14.2. Gerichtsstand.</strong> Gemäß Artikel 48 des französischen Code de procédure civile, der zwischen Kaufleuten anwendbar ist, vereinbaren die Parteien für alle Streitigkeiten aus oder im Zusammenhang mit diesen Bedingungen die ausschließliche Zuständigkeit der Gerichte von La Roche-sur-Yon, Frankreich, in deren Bezirk Hook0 seinen Sitz hat, vorbehaltlich zwingender Gerichtsstandsregeln.',
        '<strong class="text-white">14.3. Höhere Gewalt.</strong> Keine Partei haftet für ein Versäumnis oder einen Verzug bei der Leistung, der auf Ursachen außerhalb ihres zumutbaren Einflussbereichs zurückzuführen ist, insbesondere höhere Gewalt, Krieg, Terrorismus, Aufruhr, Brand, Überschwemmung, Naturkatastrophen, behördliche Maßnahmen, Streiks, Aussperrungen oder Ausfälle von Telekommunikationsnetzen oder -infrastrukturen Dritter. Die betroffene Partei unterrichtet die andere Partei unverzüglich und unternimmt wirtschaftlich angemessene Anstrengungen, die Leistung baldmöglichst wiederaufzunehmen.',
        '<strong class="text-white">14.4. Abtretung.</strong> Sie dürfen Ihre Rechte oder Pflichten aus diesen Bedingungen ohne vorherige schriftliche Zustimmung von Hook0 weder abtreten noch übertragen. Hook0 kann diese Bedingungen ganz oder teilweise abtreten, insbesondere im Rahmen einer Verschmelzung, einer Übernahme oder eines Verkaufs aller oder im Wesentlichen aller Vermögenswerte, mit schriftlicher Mitteilung an Sie.',
        '<strong class="text-white">14.5. Salvatorische Klausel.</strong> Sollte eine Bestimmung dieser Bedingungen von einem zuständigen Gericht für unwirksam, rechtswidrig oder nicht durchsetzbar befunden werden, gilt sie in dem strikt erforderlichen Maß als eingeschränkt oder entfallen; die übrigen Bestimmungen bleiben in vollem Umfang in Kraft.',
        '<strong class="text-white">14.6. Verzicht.</strong> Verzichtet eine Partei darauf, ein Recht oder eine Bestimmung dieser Bedingungen durchzusetzen, gilt dies nicht als Verzicht auf dieses Recht oder diese Bestimmung für die Zukunft.',
        '<strong class="text-white">14.7. Verhältnis der Parteien.</strong> Keine Bestimmung dieser Bedingungen begründet eine Partnerschaft, ein Joint Venture, eine Stellvertretung oder ein Arbeitsverhältnis zwischen den Parteien und ist nicht so auszulegen. Jede Partei handelt als unabhängiger Vertragspartner.',
        '<strong class="text-white">14.8. Mitteilungen.</strong> Rechtliche Mitteilungen an Hook0 sind schriftlich per E-Mail an <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a> oder per Post an FGRibreau SARL, 3 rue de l\'Aubépine, 85110 Chantonnay, Frankreich, zu übermitteln. Hook0 kann Mitteilungen an Sie per E-Mail an die Ihrem Konto zugeordnete Adresse übermitteln. Elektronische Mitteilungen gelten am Tag der Übermittlung als zugegangen, sofern keine Zustellfehlermeldung eingeht.',
      ],
    },
    {
      id: 'contact',
      title: '15. Kontakt',
      lead: 'Für Fragen, Anmerkungen oder Anliegen zu diesen Bedingungen oder zum Dienst erreichen Sie uns wie folgt:',
      contactItems: [
        '<strong class="text-white">Rechtliche Anliegen:</strong> <a href="mailto:legal@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">legal@hook0.com</a>',
        '<strong class="text-white">Support:</strong> <a href="mailto:support@hook0.com" class="text-green-400 hover:text-green-300 transition-colors">support@hook0.com</a>',
        '<strong class="text-white">Postanschrift:</strong> FGRibreau SARL, 3 rue de l\'Aubépine, 85110 Chantonnay, Frankreich',
      ],
    },
  ],
};
