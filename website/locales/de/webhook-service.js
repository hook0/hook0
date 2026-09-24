// Per-page strings for webhook-service (DE).
// /humanizer pro + legal-reviewer applied. Strikte Parität mit locales/en/webhook-service.js.
// Recht: Datenebene Clever Cloud SAS (Frankreich, EWR); CDN Cloudflare, Inc. (USA) offengelegt
// (SCC 2021 + TIA + ggf. DPF); nie „100 % souverän"; DSGVO = „auf DSGVO-Konformität ausgelegt",
// nie „zertifiziert"; Lizenz „Open Source (SSPL-1.0)", nie „Open Source" allein.
// Wettbewerberpreise erhoben am 2026-07-16 (Hookdeck Growth 499 $/Monat, Svix Pro 490 $/Monat,
// Convoy Premium 999 $/Monat). faq.items[].a muss dem sichtbaren Kartentext entsprechen.
module.exports = {
  "tldr": {
    "h2": "Kurz gesagt",
    "body": "Hook0 ist ein Managed Webhook Service: Er signiert jeden Payload per HMAC-SHA256, wiederholt in zwei Phasen, wenn ein Empfänger ausfällt, protokolliert jeden Versuch, sodass eine strittige Zustellung zur Suchabfrage statt zur Diskussion wird, und gibt deinen Kunden ein einsatzfertiges Portal, um ihre eigenen Endpoints zu verwalten. Die Datenebene, also Payloads, Datenbank und Backups, läuft bei Clever Cloud in Frankreich, auf jedem Tarif inklusive der kostenlosen Stufe, mit Cloudflare, Inc. (USA) als offengelegtem CDN. Kostenlos bis 100 Events/Tag, bezahlte Tarife ab 59 €/Monat, gegenüber einem Growth-Tarif zu 499 $/Monat bei Hookdeck, 490 $/Monat bei Svix und 999 $/Monat bei Convoy zum Stand Juli 2026. Der gesamte Code ist Open Source (SSPL-1.0), ohne Open-Core-Rückhalt, sodass du an dem Tag, an dem ein Managed Service nicht mehr die richtige Antwort ist, dieselbe Software selbst hostest und dein Integrationscode gleich bleibt."
  },
  "pageTitle": "Managed Webhook Service, mit Ausstiegsoption | Hook0",
  "pageDescription": "Ein Managed Webhook Service mit HMAC-Signaturen, Zwei-Phasen-Retries und Protokollen pro Versuch. EU-Datenebene auf jedem Tarif, und derselbe Open-Source-Code (SSPL-1.0) lässt sich selbst hosten, wenn du aussteigen willst.",
  "pageModified": "2026-09-24",
  "track": "de-webhook-service",
  "hero": {
    "eyebrow": "Webhook Service",
    "titleLine1": "Ein Managed Webhook Service,",
    "titleLine2": "den du mitnehmen kannst",
    "subtitle": "Hook0 stellt deine Events zu, signiert sie, wiederholt sie, wenn der Empfänger ausfällt, und protokolliert jeden Versuch. Er läuft ab der kostenlosen Stufe auf einer EU-Datenebene. Und an dem Tag, an dem ein Managed Webhook Service nicht mehr die richtige Antwort ist, hostet sich derselbe Code selbst, Open Source unter SSPL-1.0.",
    "ctaPrimary": "Kostenlos starten",
    "ctaSecondary": "Preise ansehen",
    "ctaSecondaryHref": "/de/preise",
    "microcopy": "100 Events/Tag kostenlos. Keine Kreditkarte. EU-Datenebene auf jedem Tarif."
  },
  "socialProof": false,
  "checklist": {
    "eyebrow": "Bewertung",
    "h2": "Was ein Webhook Service dir schuldet",
    "subtitle": "Die Teile, die Teams erst entdecken, nachdem sie ihre eigene Zustell-Schleife bereits ausgeliefert haben. Hier steht, wo Hook0 bei jedem davon steht.",
    "cards": [
      {
        "title": "Signierte Payloads, bei jedem Versuch",
        "bodyHtml": "Jede Zustellung trägt eine HMAC-SHA256-Signatur, sodass der Empfänger deine Events von allem anderen unterscheidet, das seinen Endpoint gefunden hat. Die Prüfschritte sind dokumentiert, statt als Übung überlassen."
      },
      {
        "title": "Retries, die einen schlechten Nachmittag überstehen",
        "bodyHtml": "Konfigurierbare Zwei-Phasen-Retries. Ein Abonnent, der eine Stunde ausfällt, kostet dich nicht die Events aus dieser Stunde, und du schreibst den Backoff-Plan nicht selbst."
      },
      {
        "title": "Beleg für das, was wirklich passiert ist",
        "bodyHtml": "Zustellprotokolle pro Versuch. Wenn ein Kunde sagt, er habe das Event nie erhalten, ist die Antwort eine Abfrage statt einer Diskussion."
      },
      {
        "title": "Replay, sobald der Bug behoben ist",
        "bodyHtml": "Zustellungen lassen sich aus den Protokollen erneut abspielen. Der Ausfall des Empfängers oder sein Parsing-Bug ist kein Datenverlust-Vorfall mehr, sondern ein Replay."
      },
      {
        "title": "Etwas, das deine Kunden allein nutzen",
        "bodyHtml": "Ein einsatzfertiges Abonnenten-Portal, in dem deine Nutzer ihre eigenen Endpoints registrieren und ihre eigene Zustellhistorie lesen, statt für jeden 500er ein Support-Ticket zu öffnen."
      },
      {
        "title": "Eine Datenebene, auf die du zeigen kannst",
        "bodyHtml": "Payloads, Datenbank und Backups laufen auf der Infrastruktur von Clever Cloud SAS in Frankreich, im EWR, auf jedem Tarif inklusive der kostenlosen Stufe. Das CDN davor ist Cloudflare, Inc. (USA), offengelegt in der <a href=\"/de/dsgvo-unterauftragsverarbeiter\" class=\"text-green-400 hover:text-green-300 transition-colors\">öffentlichen Liste der Unterauftragsverarbeiter</a> samt Übermittlungsmechanismus. Siehe <a href=\"/eu-webhook-infrastructure\" class=\"text-green-400 hover:text-green-300 transition-colors\">EU-Webhook-Infrastruktur</a> (auf Englisch)."
      },
      {
        "title": "Ein Ausstieg, der keine Neuentwicklung ist",
        "bodyHtml": "Das gesamte Produkt ist Open Source unter SSPL-1.0, ohne Open-Core-Rückhalt, sodass die Cloud den Code betreibt, den du selbst betreiben kannst. Aussteigen heißt, dieselbe Software zu hosten, statt gegen eine neue API neu zu bauen."
      }
    ]
  },
  "cost": {
    "eyebrow": "Kosten",
    "h2": "Was ein Managed Webhook Service kostet",
    "subtitle": "Wo der öffentliche Preis jedes Anbieters beginnt, sobald du einen unterstützten, gemanagten Dienst statt einer Testphase willst.",
    "headers": {
      "provider": "Anbieter",
      "offer": "Wie du ihn betreibst",
      "price": "Wo der Preis beginnt"
    },
    "rows": [
      {
        "highlight": true,
        "provider": "Hook0",
        "offerHtml": "Managed Cloud, selbst gehostet oder gemanagt on-premise, alles auf einem Code.",
        "priceHtml": "0 € (kostenlose Stufe, 100 Events/Tag); bezahlte Tarife ab 59 €/Monat"
      },
      {
        "highlight": false,
        "provider": "Hookdeck",
        "offerHtml": "Managed-Plattform, Regionen US, EU und Asien.",
        "priceHtml": "Kostenlose Stufe verfügbar; der SLA-gestützte Growth-Tarif kostet 499 $/Monat"
      },
      {
        "highlight": false,
        "provider": "Svix",
        "offerHtml": "Managed Cloud, oder Self-Hosting unter MIT.",
        "priceHtml": "Managed Pro beginnt bei 490 $/Monat"
      },
      {
        "highlight": false,
        "provider": "Convoy",
        "offerHtml": "Selbst gehostetes Community, oder der gemanagte Premium-Tarif.",
        "priceHtml": "Community ist kostenlos (Elastic License v2); Managed Premium kostet 999 $/Monat"
      }
    ],
    "footnote": "Quellen: die öffentlichen Preisseiten der Anbieter, zuletzt geprüft am 2026-07-16. Anbieter ändern ihre Preise, prüfe also auch deren Seiten. Etwas veraltet entdeckt? Sag uns Bescheid, wir korrigieren es.",
    "footLabel": "Den vollständigen Webhook-Kostenvergleich lesen (auf Englisch) →",
    "footHref": "/webhook-cost-comparison"
  },
  "ownership": {
    "eyebrow": "Eigentum",
    "h2": "Heute gemanagt, morgen deins",
    "intro": "Ein Webhook Service ist eine Abhängigkeit auf deinem Zustellweg. Es lohnt sich, vor der Unterschrift zu wissen, was an dem Tag passiert, an dem du ihn wieder loswerden willst.",
    "cards": [
      {
        "title": "Hoste genau dasselbe selbst",
        "bodyHtml": "Docker Compose oder Kubernetes, PostgreSQL darunter, Open Source unter SSPL-1.0. Deine Payloads bleiben in deinem eigenen Netzwerk. Siehe <a href=\"/de/selbst-gehostete-webhooks\" class=\"text-green-400 hover:text-green-300 transition-colors\">selbst gehostete Webhooks</a>."
      },
      {
        "title": "Oder behalte es gemanagt, in deiner Umgebung",
        "bodyHtml": "Wir stellen eine dedizierte Instanz in deiner Infrastruktur bereit und halten sie gewartet: 1.000 € Einrichtung + 500 €/Monat (zzgl. USt.), oder 0 € Einrichtung + 6.000 €/Jahr (zzgl. USt.)."
      },
      {
        "title": "Die API ändert sich nicht",
        "bodyHtml": "Cloud, Self-Hosting und On-Premise betreiben dieselbe <a href=\"/de/webhook-api\" class=\"text-green-400 hover:text-green-300 transition-colors\">Webhook API</a>. Der Wechsel dazwischen ändert dein Deployment, während dein Integrationscode bleibt, wie er ist."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Fragen zum Webhook Service",
    "items": [
      {
        "q": "Was ist ein Webhook Service?",
        "a": "Ein Webhook Service nimmt ein Event aus deinem Backend und stellt es an die HTTP-Endpoints zu, die deine Kunden registriert haben. Er übernimmt die vier Teile, die mühsam selbst zu tragen sind: jeden Payload signieren, damit der Empfänger ihm vertraut, wiederholen, wenn der Empfänger ausfällt, ein Protokoll jedes Versuchs führen und deinen Kunden einen Ort geben, um ihre eigenen Endpoints zu verwalten. Du machst einen API-Aufruf; den Rest der Zustellung erledigt der Dienst."
      },
      {
        "q": "Was ist der Unterschied zwischen einem Webhook Service und einer Webhook-Plattform?",
        "a": "In der Praxis beschreiben beide Wörter dasselbe Produkt, und Anbieter verwenden sie austauschbar. „Service\" beschreibt eher die gemanagte Seite (jemand anderes betreibt die Zustellung für dich), während „Plattform\" eher die Oberfläche beschreibt, auf der du baust: API, Portal, Event-Typen, Protokolle. Hook0 ist beides, weshalb dieselben Funktionen auf dieser Seite und auf der Webhook-Plattform-Seite auftauchen."
      },
      {
        "q": "Wie viel kostet ein Webhook Service?",
        "a": "Hook0 ist kostenlos bis 100 Events/Tag, mit bezahlten Tarifen ab 59 €/Monat. Unter den anderen gemanagten Anbietern kostet, zuletzt geprüft im Juli 2026, Hookdecks SLA-gestützter Growth-Tarif 499 $/Monat, Svix Managed Pro beginnt bei 490 $/Monat und Convoy Managed Premium kostet 999 $/Monat. Self-Hosting verlagert die Kosten von einem Abonnement auf deine eigene Infrastruktur und deine Wartungszeit."
      },
      {
        "q": "Wo hostet Hook0 die Webhook-Daten?",
        "a": "Die Webhook-Datenebene, also Payloads, Datenbank und Backups, läuft auf der Infrastruktur von Clever Cloud SAS in Frankreich, im Europäischen Wirtschaftsraum, auf jedem Tarif inklusive der kostenlosen Stufe. Das CDN und die Anti-DDoS-Schicht vor Website und API stellt Cloudflare, Inc. (USA), offengelegt in unserer öffentlichen Liste der Unterauftragsverarbeiter und eingerahmt durch die Standardvertragsklauseln 2021, eine dokumentierte Transfer-Folgenabschätzung und, wo einschlägig, das EU-US Data Privacy Framework."
      },
      {
        "q": "Kann ich den Managed Service später verlassen?",
        "a": "Ja, und genau deshalb ist der Code Open Source unter SSPL-1.0, ohne Open-Core-Rückhalt. Du kannst dieselbe Software mit Docker Compose oder Kubernetes selbst hosten, oder uns eine dedizierte Instanz in deiner eigenen Umgebung betreiben lassen. Die API bleibt in allen drei Fällen gleich, sodass dein Integrationscode sich nicht ändert."
      },
      {
        "q": "Brauche ich noch einen Webhook Service, wenn ich bereits HTTP-Anfragen sende?",
        "a": "Die erste Anfrage zu senden ist der einfache Teil. Die Sprints stecken in allem danach: ein Retry-Plan, der einen strauchelnden Empfänger nicht überrollt, Signaturen, die deine Kunden prüfen können, Protokolle pro Versuch für das Support-Gespräch, Replay nach einem Ausfall und ein Portal, damit Abonnenten ihre eigenen Endpoints verwalten. Wenn du das alles bereits gebaut hast und die Wartung magst, brauchst du keinen Webhook Service."
      }
    ]
  },
  "related": {
    "h2": "Weiterführend",
    "links": [
      { "label": "Webhook-Plattform", "href": "/de/webhook-plattform" },
      { "label": "Webhook API", "href": "/de/webhook-api" },
      { "label": "MCP-Server für Webhooks", "href": "/de/mcp-server-fuer-webhooks" },
      { "label": "Build vs. Buy Webhooks", "href": "/de/build-vs-buy-webhooks" },
      { "label": "Webhook-Kostenvergleich (auf Englisch)", "href": "/webhook-cost-comparison" },
      { "label": "EU-Webhook-Infrastruktur (auf Englisch)", "href": "/eu-webhook-infrastructure" },
      { "label": "Selbst gehostete Webhooks", "href": "/de/selbst-gehostete-webhooks" },
      { "label": "Preise", "href": "/de/preise" }
    ]
  }
};
