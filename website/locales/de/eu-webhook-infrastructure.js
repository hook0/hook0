// Per-page strings für eu-webhook-infrastructure (DE, Slug: eu-webhook-infrastruktur).
// /humanizer pro + rechtliche Grenzen (siehe website/CLAUDE.local.md und locales/de.js):
//   - Data Plane = Clever Cloud SAS (Frankreich, EWR). CDN Cloudflare, Inc. (USA)
//     OFFENGELEGT, abgesichert durch die Standardvertragsklauseln 2021 + Transfer-
//     Folgenabschätzung und, sofern zutreffend, das EU-US Data Privacy Framework.
//   - NIE «100 % souverän», «keine Daten verlassen die EU», «CLOUD-Act-frei».
//   - DSGVO/NIS2/DORA = Prozess-/Kontext-Claims («auf … ausgelegt», «unterstützt
//     deine Anforderungen»), nie «zertifiziert».
//   - Lizenz = «Open Source (SSPL-1.0)», Klammerzusatz obligatorisch, NIE «Open
//     Source» allein (SSPL nicht von der OSI anerkannt, UWG §5).
// Konkurrenzfakten aus den business.md-Snapshots vom 2026-07-08; verwaltete
// On-Premise-Preise gegen src/includes/_pricing.ejs geprüft (1.000 € Einrichtung +
// 500 €/Monat zzgl. MwSt., oder 0 € Einrichtung + 6.000 €/Jahr zzgl. MwSt.).
// Der Text von faq.items[].a MUSS Byte für Byte mit dem sichtbaren Karten-Body
// übereinstimmen; das FAQPage-JSON-LD wird aus derselben Liste generiert.
module.exports = {
  "pageTitle": "EU-Webhook-Infrastruktur: standardmäßig in Frankreich gehostet | Hook0",
  "pageDescription": "Hook0 betreibt seine Webhook-Data-Plane bei Clever Cloud (Frankreich) ab der kostenlosen Stufe. Französische Gesellschaft, öffentliche Liste der Unterauftragsverarbeiter, jederzeit Self-Hosting oder On-Premise.",
  "pageModified": "2026-09-24",
  "track": "de-eu-webhook-infrastruktur",
  // Direktantwort-Block (AEO). Jede Zahl ist in der Seitenkopie unten verankert:
  // Clever Cloud SAS (Frankreich, EWR) auf jedem Tarif, Gesellschaft nach
  // französischem Recht, Cloudflare Inc. (USA) offengelegt unter SVK 2021 + TIA +
  // DPF, DSGVO/NIS2/DORA als Prozess-Claims, Reversibilität Open Source (SSPL-1.0),
  // 100 Events/Tag gratis, ab 59 €/Monat. Setzt bewusst EU-als-Standard OHNE
  // Souveränitätsversprechen (nie «keine Daten verlassen die EU»).
  "tldr": {
    "h2": "Kurz gesagt",
    "body": "Hook0 betreibt seine Webhook-Data-Plane (Payloads, Datenbank und Backups) bei Clever Cloud SAS in Frankreich, innerhalb des Europäischen Wirtschaftsraums, auf jedem Tarif einschließlich der kostenlosen Stufe. Das Unternehmen hinter Hook0 ist nach französischem Recht gegründet, ohne US-Muttergesellschaft. Die CDN- und DDoS-Schicht vor Website und API ist Cloudflare, Inc. (USA), offengelegt in einer öffentlichen Liste der Unterauftragsverarbeiter und abgesichert durch die Standardvertragsklauseln von 2021, eine dokumentierte Transfer-Folgenabschätzung und, sofern zutreffend, das EU-US Data Privacy Framework. Das ist EU als Standard, ohne zu behaupten, dass keine Daten je ein US-Unternehmen erreichen. Hook0 ist auf DSGVO-, NIS2- und DORA-Anforderungen ausgelegt, statt dagegen zertifiziert zu sein, und bleibt reversibel: Cloud, Self-Hosting (Open Source SSPL-1.0, Docker oder Kubernetes) und verwaltetes On-Premise laufen auf derselben Codebasis. Kostenlos für 100 Events pro Tag ohne Kreditkarte; kostenpflichtige Tarife ab 59 €/Monat."
  },
  "hero": {
    "eyebrow": "EU-Webhook-Infrastruktur",
    "titleLine1": "EU als Standard,",
    "titleLine2": "nicht als kostenpflichtige Option",
    "subtitle": "Die Webhook-Data-Plane von Hook0 läuft bei Clever Cloud, in Frankreich, ab der kostenlosen Stufe. Das Unternehmen dahinter ist nach französischem Recht gegründet, ohne US-Muttergesellschaft. Und falls du irgendwann aussteigen willst, hostet sich dieselbe Codebasis selbst, Open Source (SSPL-1.0).",
    "ctaPrimary": "Kostenlos starten",
    "ctaSecondary": "Preise ansehen",
    "ctaSecondaryHref": "/de/preise",
    "microcopy": "100 Events/Tag gratis. Keine Kreditkarte. EU-Data-Plane auf jedem Tarif."
  },
  "socialProof": true,
  "pillars": {
    "eyebrow": "Datenresidenz",
    "h2": "Wo deine Webhook-Daten wirklich liegen",
    "cards": [
      {
        "title": "Data Plane in Frankreich, ab der kostenlosen Stufe",
        "bodyHtml": "Webhook-Payloads, Datenbank und Backups laufen auf der Infrastruktur von Clever Cloud SAS in Frankreich, innerhalb des Europäischen Wirtschaftsraums. Das ist kein Enterprise-Add-on: Die kostenlose Stufe und jeder kostenpflichtige Tarif nutzen dieselbe EU-Data-Plane."
      },
      {
        "title": "Eine Gesellschaft nach französischem Recht",
        "bodyHtml": "Hook0 wird von einer nach französischem Recht gegründeten Gesellschaft entwickelt und betrieben, ohne US-Muttergesellschaft. Dein Vertrag, dein AVV und deine Datenschutzfragen werden unter EU-Gerichtsbarkeit bearbeitet."
      },
      {
        "title": "Ein transparenter Edge: Cloudflare, offengelegt",
        "bodyHtml": "Unsere CDN- und DDoS-Schutzschicht ist Cloudflare, Inc. (USA). Wir legen das offen, statt es zu verstecken: Diese Übermittlungen sind durch die Standardvertragsklauseln von 2021, eine dokumentierte Transfer-Folgenabschätzung und, sofern zutreffend, das EU-US Data Privacy Framework abgesichert. Die vollständige <a href=\"/de/dsgvo-unterauftragsverarbeiter\" class=\"text-green-400 hover:text-green-300 transition-colors\">Liste der Unterauftragsverarbeiter</a> ist öffentlich."
      },
      {
        "title": "Ein AVV, den man wirklich lesen kann",
        "bodyHtml": "Der <a href=\"/de/auftragsverarbeitungsvertrag\" class=\"text-green-400 hover:text-green-300 transition-colors\">Auftragsverarbeitungsvertrag</a> von Hook0 listet jeden Unterauftragsverarbeiter und seinen Übermittlungsmechanismus auf. Kein Vertriebsgespräch nötig, um herauszufinden, wohin deine Daten gehen."
      }
    ]
  },
  "residency": {
    "eyebrow": "Vergleich",
    "h2": "EU-Datenresidenz, Anbieter für Anbieter",
    "subtitle": "Was nötig ist und was es kostet, um Webhook-Daten bei jedem verwalteten Anbieter in der EU zu halten. Öffentliche Preise, geprüft im Juli 2026.",
    "headers": {
      "provider": "Anbieter",
      "residency": "EU-Residenz in der verwalteten Cloud",
      "price": "Wo die Preise beginnen"
    },
    "rows": [
      {
        "highlight": true,
        "provider": "Hook0",
        "residencyHtml": "Standard. Data Plane bei Clever Cloud (Frankreich) auf jedem Tarif, inklusive der kostenlosen Stufe.",
        "priceHtml": "0 € (kostenlose Stufe); kostenpflichtige Tarife ab 59 €/Monat"
      },
      {
        "highlight": false,
        "provider": "Hookdeck",
        "residencyHtml": "US-, EU- und Asien-Regionen auf der verwalteten Plattform; die genauen EU-Regionen werden nicht veröffentlicht.",
        "priceHtml": "Kostenlose Stufe verfügbar; der SLA-gestützte Growth-Tarif kostet 499 $/Monat"
      },
      {
        "highlight": false,
        "provider": "Svix",
        "residencyHtml": "Keine in der EU gehostete verwaltete Cloud beworben; Datenresidenz wird ohne explizite EU-Region erwähnt.",
        "priceHtml": "Managed Pro beginnt bei 490 $/Monat; Self-Hosting (MIT), um die Residenz selbst zu steuern"
      },
      {
        "highlight": false,
        "provider": "Convoy",
        "residencyHtml": "Keine verwaltete geografische Residenz; die Wahl deiner Region erfordert Self-Hosting.",
        "priceHtml": "Self-hosted Community ist kostenlos (Elastic License v2); der verwaltete Premium-Tarif kostet 999 $/Monat"
      }
    ],
    "footnote": "Quellen: die öffentlichen Preis- und Dokumentationsseiten jedes Anbieters, zuletzt geprüft am 16. Juli 2026. Etwas veraltet entdeckt? Sag uns Bescheid, wir korrigieren es."
  },
  "reversibility": {
    "eyebrow": "Reversibilität",
    "h2": "Reversibilität ist die andere Hälfte der Souveränität",
    "intro": "EU-Hosting zählt weniger, wenn ein Wechsel unmöglich ist. Manche in der EU gehosteten Webhook-Dienste sind proprietär und nur als Cloud verfügbar, sodass dein einziger Ausweg ein Export und ein Neuaufbau ist. Die Cloud-, Self-Hosting- und On-Premise-Deployments von Hook0 teilen sich eine Codebasis, ein Wechsel bedeutet also, dieselbe Software woanders zu betreiben.",
    "cards": [
      {
        "title": "Hoste es selbst",
        "bodyHtml": "Die vollständige Hook0-Codebasis ist Open Source (SSPL-1.0). Docker Compose oder Kubernetes, PostgreSQL darunter. Deine Webhook-Payloads bleiben in deinem eigenen Netzwerk. Siehe <a href=\"/de/selbst-gehostete-webhooks\" class=\"text-green-400 hover:text-green-300 transition-colors\">selbst-gehostete Webhooks</a>."
      },
      {
        "title": "Verwaltetes On-Premise",
        "bodyHtml": "Wir stellen eine dedizierte Hook0-Instanz in deiner Umgebung bereit und halten sie gewartet und aktuell: 1.000 € Einrichtung + 500 €/Monat (zzgl. MwSt.), oder 0 € Einrichtung + 6.000 €/Jahr (zzgl. MwSt.). Deine Infrastruktur, unsere Wartung."
      }
    ]
  },
  "compliance": {
    "eyebrow": "Compliance",
    "h2": "Gebaut für Teams mit DSGVO-, NIS2- oder DORA-Anforderungen",
    "intro": "Hook0 verkauft dir kein Compliance-Siegel. Es liefert dir die konkreten Eigenschaften, nach denen deine Auditoren fragen.",
    "cards": [
      {
        "title": "DSGVO",
        "bodyHtml": "Die Data Plane bleibt in Frankreich (EWR), Unterauftragsverarbeiter sind mit ihren Übermittlungsmechanismen dokumentiert, und ein AVV steht zur Verfügung, bevor du irgendetwas unterschreibst. Hook0 ist auf DSGVO-Konformität ausgelegt: konkrete Nachweise statt eines Abzeichens."
      },
      {
        "title": "NIS2",
        "bodyHtml": "NIS2 ist eine Richtlinie, die für deine Organisation gilt, keine Produktzertifizierung. Hook0 unterstützt deine Anforderungen mit EU-Datenresidenz als Standard, einer öffentlichen Liste der Unterauftragsverarbeiter, Zustelllogs pro Versuch und einer dokumentierten <a href=\"/de/sicherheit\" class=\"text-green-400 hover:text-green-300 transition-colors\">Sicherheitsseite</a>."
      },
      {
        "title": "DORA",
        "bodyHtml": "Finanzunternehmen unter DORA prüfen IKT-Drittparteienrisiko und Ausstiegsstrategien genau. Hook0 unterstützt diese Analyse: EU-Data-Plane, dokumentierte Unterauftragsverarbeiter und ein echter Ausstiegsweg, nämlich Self-Hosting oder On-Premise auf derselben Codebasis."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Fragen zur EU-Webhook-Infrastruktur",
    "items": [
      {
        "q": "Wo werden die Webhook-Daten von Hook0 gehostet?",
        "a": "Die Webhook-Data-Plane von Hook0 (Payloads, Datenbank und Backups) läuft auf der Infrastruktur von Clever Cloud SAS in Frankreich, innerhalb des Europäischen Wirtschaftsraums. Die CDN- und DDoS-Schutzschicht vor Website und API ist Cloudflare, Inc. (USA), offengelegt in unserer öffentlichen Liste der Unterauftragsverarbeiter und abgesichert durch die Standardvertragsklauseln von 2021, eine dokumentierte Transfer-Folgenabschätzung und, sofern zutreffend, das EU-US Data Privacy Framework."
      },
      {
        "q": "Ist Hook0 DSGVO-konform?",
        "a": "Hook0 ist auf DSGVO-Konformität ausgelegt: eine EU-Data-Plane, eine öffentliche Liste der Unterauftragsverarbeiter mit Übermittlungsmechanismen und ein Auftragsverarbeitungsvertrag, den du prüfen kannst, bevor du etwas unterschreibst. Es gibt kein offizielles DSGVO-Abzeichen für Webhook-Anbieter, also frage bei jedem, den du bewertest, nach dem AVV und der Liste der Unterauftragsverarbeiter. Unsere sind öffentlich."
      },
      {
        "q": "Unterstützt Hook0 NIS2- oder DORA-Anforderungen?",
        "a": "NIS2 und DORA gelten für deine Organisation; kein Webhook-Anbieter kann dagegen „zertifiziert“ sein. Was Hook0 liefert, ist das Material, das dein Compliance-Team braucht: EU-Datenresidenz als Standard, dokumentierte Unterauftragsverarbeiter, Zustelllogs für jeden Versuch und eine Ausstiegsstrategie (Self-Hosting oder verwaltetes On-Premise, beide auf derselben Codebasis wie die Cloud)."
      },
      {
        "q": "Kann ich die Hook0-Cloud später verlassen?",
        "a": "Ja. Die Cloud-, Self-Hosting- und On-Premise-Versionen teilen sich eine Codebasis, Open Source unter SSPL-1.0. Du kannst mit Docker Compose oder Kubernetes selbst hosten oder uns bitten, eine dedizierte Instanz in deiner Umgebung zu betreiben, für 1.000 € Einrichtung + 500 €/Monat (zzgl. MwSt.). So oder so behältst du dieselbe API, dein Integrationscode ändert sich also nicht."
      },
      {
        "q": "Welche Webhook-Anbieter bieten EU-Datenresidenz in ihrer verwalteten Cloud?",
        "a": "Stand Juli 2026: Hook0 hostet seine Data Plane in Frankreich auf jedem Tarif. Hookdeck bewirbt US-, EU- und Asien-Regionen auf seiner verwalteten Plattform, ohne die genauen EU-Regionen zu veröffentlichen. Svix bewirbt keine in der EU gehostete verwaltete Cloud. Convoy bietet keine verwaltete geografische Residenz; du wählst deine Region per Self-Hosting. Anbieter ändern ihre Angebote, prüfe daher auch deren aktuelle Dokumentation."
      }
    ]
  },
  "related": {
    "h2": "Weiterführend",
    "links": [
      { "label": "Selbst-gehostete Webhooks", "href": "/de/selbst-gehostete-webhooks" },
      { "label": "Preise", "href": "/de/preise" },
      { "label": "DSGVO-Unterauftragsverarbeiter", "href": "/de/dsgvo-unterauftragsverarbeiter" },
      { "label": "Sicherheit", "href": "/de/sicherheit" },
      { "label": "Auftragsverarbeitungsvertrag", "href": "/de/auftragsverarbeitungsvertrag" }
    ]
  }
};
