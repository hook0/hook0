// DE-Strings für sovereign-webhooks. Humanizer pro (Hooky-Stimme, «du»,
// Entwickler-Vokabular, kein AI-Slop). Gleiche Leitplanken wie die EN-Basis:
//   - keine Zertifizierung für Hook0 behaupten (SecNumCloud, HDS, SEAL-Stufe,
//     ISO 27001, SOC 2); keine Badges; keine negative Zertifizierungs-Erwähnung;
//   - «Open Source (SSPL-1.0)», nie «100% Open Source» oder «source-available»;
//   - keine standardmäßige At-Rest-Verschlüsselung behaupten;
//   - der CLOUD-Act-Abschnitt ist erklärend + generische Checkliste; er behauptet
//     NICHT, dass Hook0 außerhalb des CLOUD Act liegt (juristische Schlussfolgerung
//     ist /legal-gated). Er wiederholt nur bereits veröffentlichte Fakten: Herausgeber
//     nach französischem Recht ohne US-Mutter, Data Plane in Frankreich, Cloudflare-Edge
//     (USA) offengelegt unter SVK 2021 + TIA + DPF. Nie «keine Daten verlassen die EU»;
//   - kein Wettbewerber wird hier genannt; die Links verweisen auf die bestehenden
//     Vergleichsseiten.
// Der Text von faq.items[].a muss wortgleich zur sichtbaren Karte sein; das FAQPage-
// JSON-LD wird aus derselben Liste generiert.
module.exports = {
  "pageTitle": "Souveräne Webhooks: digitale Souveränität by Design | Hook0",
  "pageDescription": "Souveränität heißt: wer kann rechtlich gezwungen werden, deine Daten herauszugeben, nicht nur wo sie liegen. Hook0: Herausgeber nach französischem Recht, EU-Data-Plane, Open Source, reversibel.",
  "pageModified": "2026-10-06",
  "track": "de-souveraene-webhooks",
  "tldr": {
    "h2": "Kurz gesagt",
    "body": "Digitale Souveränität entscheidet sich nicht am Standort eines Servers, sondern an der Frage, wer rechtlich gezwungen werden kann, deine Daten herauszugeben. Hook0 beantwortet das mit überprüfbaren Eigenschaften statt mit einem Badge. Der Herausgeber ist eine Gesellschaft nach französischem Recht, ohne US-Muttergesellschaft. Die Webhook-Data-Plane läuft in Frankreich (Clever Cloud), auf jedem Tarif einschließlich der kostenlosen Stufe. Die gesamte Plattform ist Open Source: Server unter SSPL-1.0, SDKs unter MIT, und der selbstgehostete Code ist derselbe, der die verwaltete Cloud betreibt. Damit holst du deine Webhook-Infrastruktur ins Haus, ohne einem Anbieter vertrauen zu müssen. Die CDN- und DDoS-Schicht ist Cloudflare, Inc. (USA), offengelegt in einer öffentlichen Liste der Unterauftragsverarbeiter. Hook0 ist auf DSGVO-, NIS2- und DORA-Anforderungen ausgelegt, statt dagegen zertifiziert zu sein. Kostenlos für 100 Events pro Tag, ohne Kreditkarte."
  },
  "hero": {
    "eyebrow": "Digitale Souveränität",
    "titleLine1": "Souveräne Webhooks,",
    "titleLine2": "nicht nur EU-gehostet",
    "subtitle": "Daten zu lokalisieren ist der leichte Teil. Souveränität heißt: wer kann rechtlich gezwungen werden, sie herauszugeben. Hook0 wird von einer Gesellschaft nach französischem Recht ohne US-Mutter herausgegeben, hostet seine Data Plane in Frankreich und ist durchgängig Open Source. Aussteigen heißt, denselben Code einfach selbst zu betreiben.",
    "ctaPrimary": "Kostenlos starten",
    "ctaSecondary": "Wo deine Daten liegen",
    "ctaSecondaryHref": "/de/eu-webhook-infrastruktur",
    "microcopy": "100 Events/Tag gratis. Keine Kreditkarte. EU-Data-Plane auf jedem Tarif."
  },
  "socialProof": true,
  "dimensions": {
    "eyebrow": "Lokalisieren ≠ souverän",
    "h2": "Souveränität sind drei Fragen, nicht eine",
    "intro": "Ein Rechenzentrum in Paris ist notwendig, nicht hinreichend. Die EU-Souveränitätsrahmen zerlegen das Wort in drei getrennte Fragen, und der Datenstandort beantwortet nur die erste.",
    "cards": [
      {
        "title": "Rechtlich",
        "bodyHtml": "Welches Recht gilt für den Anbieter, und welches Recht kann ihn zwingen? Ein Rechenzentrum in der EU, betrieben von einer Gesellschaft unter Nicht-EU-Recht, bleibt extraterritorialen Zugriffsanfragen ausgesetzt. Die Rechtsordnung der Gesellschaft zählt so viel wie der Standort der Festplatte."
      },
      {
        "title": "Operativ",
        "bodyHtml": "Wer betreibt, unterstützt, aktualisiert und kann den Dienst abschalten? Wo sitzen die Bereitschaftsteams, und wer hält die Verschlüsselungsschlüssel? Souveränität erodiert, sobald die tägliche Kontrolle aus der gewählten Rechtsordnung herausfällt."
      },
      {
        "title": "Technologisch",
        "bodyHtml": "Woraus besteht der Stack (Software, Datenbank, Identität, DNS) und ist er offen, prüfbar, austauschbar? Ein geschlossener, reiner Cloud-Dienst lässt dir nur Export und Neuaufbau als Ausgang. Open Source hält den Stack austauschbar."
      }
    ]
  },
  "compulsion": {
    "eyebrow": "Die eigentliche Frage",
    "h2": "Wer kann rechtlich gezwungen werden, deine Daten herauszugeben?",
    "intro": "Der US-amerikanische CLOUD Act (2018) erlaubt US-Behörden, von einem dem US-Recht unterliegenden Anbieter die Herausgabe von Daten zu verlangen, die er kontrolliert, egal wo sie gespeichert sind, auch in einer EU-Region. Die Frage an jeden Webhook-Anbieter lautet also nicht nur, wo die Server stehen, sondern wer angewiesen werden könnte, die Daten herauszugeben.",
    "checklistTitle": "Sieben Fragen an jeden Webhook-Anbieter",
    "checklist": [
      "Wem gehört und wer kontrolliert die Gesellschaft, und unter welchem Recht?",
      "Wo laufen die Daten <strong>und</strong> die Operationen tatsächlich?",
      "Wer hält die Verschlüsselungsschlüssel?",
      "Welche versteckten Abhängigkeiten fahren mit: Identität, DNS, CDN, E-Mail, Container-Registries, KI?",
      "Wie reversibel ist es: kannst du exportieren und anderswo betreiben, und wurde das getestet?",
      "Welche Stufe wird behauptet, und mit welchen tatsächlich lesbaren Audit-Nachweisen?",
      "Welcher Plan greift, wenn der Anbieter verschwindet oder abgeschnitten wird?"
    ],
    "hook0Title": "Wie Hook0 antwortet, in überprüfbaren Fakten",
    "hook0Html": "Die Gesellschaft hinter Hook0 ist nach französischem Recht gegründet, ohne US-Muttergesellschaft. Ihre Webhook-Data-Plane (Payloads, Datenbank und Backups) läuft in Frankreich (Clever Cloud), auf jedem Tarif. Die Plattform ist Open Source (SSPL-1.0): die Schlüssel, der Stack und der Ausgang gehören dir, zum Prüfen und zum Mitnehmen. Die einzige offengelegte US-Abhängigkeit ist die Cloudflare-CDN- und DDoS-Schicht, gelistet in einer öffentlichen <a href=\"/de/dsgvo-unterauftragsverarbeiter\" class=\"text-green-400 hover:text-green-300 transition-colors\">Liste der Unterauftragsverarbeiter</a> und abgesichert durch die Standardvertragsklauseln 2021, eine dokumentierte Transfer-Folgenabschätzung und, sofern zutreffend, das EU-US Data Privacy Framework. Lies den <a href=\"/de/auftragsverarbeitungsvertrag\" class=\"text-green-400 hover:text-green-300 transition-colors\">DPA</a> und entscheide selbst, ohne Vertriebsgespräch.",
    // legalTitle/legalHtml: von /legal validierte, eng gefasste Aussage (wortgleich,
    // nicht umformulieren, nicht durch den Humanizer). Bindet die CLOUD-Act-Analyse
    // nur an die herausgebende Einheit und die von ihr kontrollierten Daten; nennt
    // Cloudflare (USA) als weiterhin im Anwendungsbereich; behält den Vorbehalt
    // «keine absolute Garantie». Eine Änderung erfordert eine neue /legal-Freigabe.
    "legalTitle": "Hook0 und der US-CLOUD-Act",
    "legalHtml": "Der Herausgeber von Hook0, die FGRibreau SARL, ist ein französisches Unternehmen ohne US-Tochtergesellschaft, Muttergesellschaft oder Niederlassung. Der CLOUD Act (18 U.S.C. § 2713, zur Änderung des Stored Communications Act) verpflichtet einen der US-Gerichtsbarkeit unterliegenden Anbieter, Daten in seiner „possession, custody, or control\" herauszugeben. Mangels jeglicher US-Anknüpfung ist der Herausgeber nach unserem Kenntnisstand und nach geltendem Recht kein Anbieter, den US-Behörden nach dem CLOUD Act zur Herausgabe der von ihm kontrollierten Daten zwingen könnten. Diese Einschätzung betrifft die herausgebende Einheit und die von ihr kontrollierten Daten; sie erstreckt sich nicht auf dem US-Recht unterliegende Dritte — insbesondere Cloudflare (USA), das am Edge eingesetzt wird und für die von ihm verarbeiteten Daten im Anwendungsbereich des CLOUD Act bleibt. Die Datenebene wird standardmäßig in Frankreich gehostet (Clever Cloud). Da Hook0 Open Source und selbst hostbar ist, ist bei einer selbst gehosteten Installation kein Dritter beteiligt, der gezwungen werden könnte."
  },
  "levers": {
    "eyebrow": "Die Hebel",
    "h2": "Was Hook0 dir konkret gibt",
    "intro": "Das sind die anerkannten Hebel für digitale Resilienz und Reversibilität. Hook0 gibt dir die Eigenschaften, nicht ein Etikett.",
    "cards": [
      {
        "title": "EU-Rechtsordnung und Hosting",
        "bodyHtml": "Eine Gesellschaft nach französischem Recht, ohne US-Mutter, mit der Webhook-Data-Plane in Frankreich (Clever Cloud) auf jedem Tarif einschließlich der kostenlosen Stufe, nicht als Enterprise-Zusatz. Siehe <a href=\"/de/eu-webhook-infrastruktur\" class=\"text-green-400 hover:text-green-300 transition-colors\">EU-Webhook-Infrastruktur</a> für das vollständige Bild zur Datenresidenz."
      },
      {
        "title": "Open Source von Anfang bis Ende",
        "bodyHtml": "Der Server ist Open Source unter SSPL-1.0, die SDKs unter MIT. Keine geschlossene Enterprise-Edition: der Code, den du selbst hostest, ist der Code, der die verwaltete Cloud betreibt. Nichts hinter einer Paywall, die du nicht prüfen kannst. Siehe <a href=\"/de/quelloffene-webhooks\" class=\"text-green-400 hover:text-green-300 transition-colors\">quelloffene Webhooks</a>."
      },
      {
        "title": "Reversibilität, getestet",
        "bodyHtml": "Betreibe dieselbe Plattform auf deiner eigenen Infrastruktur mit Docker Compose oder Kubernetes, oder lass uns eine dedizierte Instanz on-premise betreiben. Selbstgehostet bleibt kein Drittanbieter übrig, der gezwungen werden kann: dir gehören Code und Server. Siehe <a href=\"/de/selbst-gehostete-webhooks\" class=\"text-green-400 hover:text-green-300 transition-colors\">selbst-gehostete Webhooks</a>."
      },
      {
        "title": "Offengelegte Abhängigkeiten",
        "bodyHtml": "Jeder Unterauftragsverarbeiter und sein Transfermechanismus sind veröffentlicht, nicht vergraben. Die Cloudflare-Edge (USA) wird benannt, und der <a href=\"/de/auftragsverarbeitungsvertrag\" class=\"text-green-400 hover:text-green-300 transition-colors\">Auftragsverarbeitungsvertrag</a> steht vor jeder Unterschrift bereit. Transparenz ist der Teil der Souveränität, den du schon heute prüfen kannst."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Fragen zu souveränen Webhooks",
    "items": [
      {
        "q": "Was macht eine Webhook-Plattform souverän?",
        "a": "Souveränität ist mehr als EU-Hosting. Es geht darum, wer rechtlich gezwungen werden kann, deine Daten herauszugeben, wer den Dienst betreibt und die Schlüssel hält, und ob der Stack offen und reversibel ist. Die überprüfbaren Hebel sind die Rechtsordnung der Gesellschaft, der Standort von Daten und Operationen, Open-Source-Code, offengelegte Abhängigkeiten und ein getesteter Ausgang. Hook0 liefert diese Hebel statt eines Zertifizierungs-Badges."
      },
      {
        "q": "Was ist der US-amerikanische CLOUD Act und was hat er mit Webhook-Daten zu tun?",
        "a": "Der CLOUD Act (2018) erlaubt US-Behörden, von einem dem US-Recht unterliegenden Anbieter die Herausgabe von Daten zu verlangen, die er kontrolliert, egal wo sie gespeichert sind, auch in einer EU-Region. Deshalb klärt der Datenstandort allein die Souveränitätsfrage nicht. Die Gesellschaft hinter Hook0 ist nach französischem Recht gegründet, ohne US-Mutter, ihre Webhook-Data-Plane läuft in Frankreich, und ihre einzige offengelegte US-Abhängigkeit ist die Cloudflare-CDN- und DDoS-Schicht, veröffentlicht in der Liste der Unterauftragsverarbeiter mit ihren Transfergarantien. Lies den DPA und die Liste, um deine eigenen Anforderungen zu bewerten."
      },
      {
        "q": "Ist Hook0 Open Source?",
        "a": "Ja. Der Server ist Open Source unter SSPL-1.0 und die SDKs unter MIT. Keine geschlossene Enterprise-Edition: der Code, den du selbst hostest, ist derselbe, der die verwaltete Cloud betreibt, also bleibt der gesamte Stack prüfbar und reversibel."
      },
      {
        "q": "Wo hostet Hook0 Webhook-Daten?",
        "a": "Die Webhook-Data-Plane von Hook0 (Payloads, Datenbank und Backups) läuft auf Clever-Cloud-Infrastruktur in Frankreich, innerhalb des Europäischen Wirtschaftsraums, auf jedem Tarif einschließlich der kostenlosen Stufe. Die CDN- und DDoS-Schicht vor Website und API ist Cloudflare, Inc. (USA), offengelegt in einer öffentlichen Liste der Unterauftragsverarbeiter mit ihren Transfermechanismen."
      },
      {
        "q": "Kann ich Hook0 ganz ohne Drittanbieter betreiben?",
        "a": "Ja. Hoste den Open-Source-Server (SSPL-1.0) selbst mit Docker Compose oder Kubernetes: deine Webhook-Payloads bleiben in deinem eigenen Netzwerk, ohne Drittanbieter, der gezwungen werden kann. Du behältst dieselbe API, dein Integrationscode ändert sich also nicht, wenn du später zwischen Self-Hosting, On-Premise und Cloud wechselst."
      },
      {
        "q": "Unterliegt Hook0 dem CLOUD Act?",
        "a": "Der Herausgeber (französisches Unternehmen ohne US-Anknüpfung) ist nach unserem Kenntnisstand und geltendem Recht kein durch eine CLOUD-Act-Anordnung erreichbarer Anbieter für die von ihm kontrollierten Daten. Unser Edge-Dienstleister Cloudflare, ein US-Unternehmen, fällt für die von ihm verarbeiteten Daten unter den CLOUD Act."
      },
      {
        "q": "Kann eine US-Behörde meine Daten erlangen?",
        "a": "Nicht durch Zwang gegen den Herausgeber nach dem CLOUD Act, solange dieser keine US-Anknüpfung hat. Eine absolute Garantie ist jedoch nicht möglich: Daten, die über unser Cloudflare-Edge (USA) laufen, unterliegen US-Recht, und andere Wege (internationale Rechtshilfe) bestehen unabhängig vom CLOUD Act. Selbst-Hosting entfernt jeden Dritten, der gezwungen werden könnte."
      },
      {
        "q": "Ändert die künftige Luxemburger Struktur diese Einschätzung?",
        "a": "Nein, unter einer Bedingung: Die Luxemburger Einheit bleibt eine EU-Einheit. Die Argumentation gilt, solange sie keine US-Niederlassung, -Tochter, -Muttergesellschaft oder sonstige US-Anknüpfung hat."
      }
    ]
  },
  "related": {
    "h2": "Weiterführend",
    "links": [
      { "enSlug": "eu-webhook-infrastructure", "label": "EU-Webhook-Infrastruktur" },
      { "enSlug": "open-source-webhooks", "label": "Quelloffene Webhooks" },
      { "enSlug": "self-hosted-webhooks", "label": "Selbst-gehostete Webhooks" },
      { "enSlug": "gdpr-subprocessors", "label": "DSGVO-Unterauftragsverarbeiter" },
      { "enSlug": "security", "label": "Sicherheit" },
      { "enSlug": "pricing", "label": "Preise" },
      { "enSlug": "webhook-cost-comparison", "label": "Webhook-Kostenvergleich (auf Englisch)" }
    ]
  }
};
