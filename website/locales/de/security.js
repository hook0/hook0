// Per-page strings for security (DE).
// /humanizer pro + legal-reviewer applied.
// Process-Claim für die DSGVO (« auf DSGVO-Konformität ausgelegt »), keine absoluten Zertifizierungsaussagen.
// Anwendungsdaten in Frankreich bei Clever Cloud, CDN Cloudflare (USA) offen genannt.
// Links auf /data-processing-addendum und /gdpr-subprocessors behalten den EN-Slug
// (Seiten noch nicht übersetzt).
module.exports = {
  pageTitle: 'Hook0 Sicherheit: HMAC, Audits, Mandantentrennung',
  pageDescription: 'Hook0 Sicherheit: HMAC-Signaturen, Mandantentrennung, ISO-27001-inspirierte Audits, Hosting in Frankreich (Clever Cloud).',
  "pageModified": "2026-06-27",
  "hero": {
    "eyebrow": "Vertrauen und Sicherheit",
    "h1": "Sicherheit und Compliance",
    "subtitle": "Unser umfassender Ansatz zum Schutz deiner Daten, im Detail."
  },
  "sections": [
    {
      "h2": "DSGVO, Compliance und Zertifizierung",
      "cards": [
        {
          "bodyHtml": "Sobald du Daten aus der Europäischen Union über einen Dienstleister wie Hook0 verarbeitest, brauchst du mit jedem Anbieter einen vertraglichen Rahmen. So weiß die EU, dass du nur mit Unternehmen arbeitest, deren Praktiken auf die Konformität mit der Datenschutz-Grundverordnung (DSGVO) ausgelegt sind."
        }
      ]
    },
    {
      "h2": "Auftragsverarbeitungsvertrag (DPA)",
      "cards": [
        {
          "bodyHtml": "Ein Auftragsverarbeitungsvertrag (Data Processing Agreement, DPA), auch Data Processing Addendum genannt, ist ein Vertrag zwischen Verantwortlichen und Auftragsverarbeitern oder zwischen Auftragsverarbeitern und weiteren Unterauftragsverarbeitern.\n                        <a href=\"/data-processing-addendum\" class=\"text-green-400 hover:text-green-300 transition-colors ml-1\">Mehr erfahren (auf Englisch)</a>."
        }
      ]
    },
    {
      "h2": "Unterauftragsverarbeiter",
      "cards": [
        {
          "bodyHtml": "Im Sinne der DSGVO ist ein Unterauftragsverarbeiter jedes Unternehmen oder jeder Dienstleister, durch den Kundendaten als Nebeneffekt der Nutzung von Hook0 fließen können.\n                        <a href=\"/gdpr-subprocessors\" class=\"text-green-400 hover:text-green-300 transition-colors ml-1\">Mehr erfahren (auf Englisch)</a>."
        }
      ]
    },
    {
      "h2": "PCI DSS",
      "cards": [
        {
          "bodyHtml": "Zahlungs- und Kreditkartendaten werden über <a href=\"https://stripe.com/docs/security\" class=\"text-green-400 hover:text-green-300 transition-colors\">Stripe</a> abgewickelt. Stripe wird von einem unabhängigen PCI Qualified Security Assessor auditiert und ist als PCI Level 1 Service Provider zertifiziert, der strengsten Stufe der Branche. Hook0 erhält im Normalbetrieb keine Kreditkartendaten und ist damit in den meisten Konstellationen mit den Payment Card Industry Data Security Standards (PCI DSS) konform."
        }
      ]
    },
    {
      "h2": "Meldung von Sicherheitslücken",
      "cards": [
        {
          "paragraphs": [
            "Um eine Sicherheitslücke oder andere Sicherheitsbedenken zu einem Hook0-Produkt zu melden, schreib an <a href=\"mailto:security@hook0.com\" class=\"text-green-400 hover:text-green-300 transition-colors\">security@hook0.com</a>.",
            "Lege einen Proof of Concept bei, eine Liste der verwendeten Tools (mit Versionen) und die vollständige Ausgabe dieser Tools. Jede Meldung wird sehr ernst genommen. Sobald eine Meldung eingeht, wird die Sicherheitslücke zügig geprüft, bevor die nötigen Schritte zur Behebung folgen. Nach Bestätigung gehen regelmäßige Status-Updates raus, während die Behebung läuft.",
            "Um sensible Informationen verschlüsselt zu übermitteln, ist der PGP-Schlüssel <a href=\"https://keybase.io/fgribreau\" class=\"text-green-400 hover:text-green-300 transition-colors\">auf Keybase verfügbar</a>.",
            "Für kritische Sicherheitslücken in der Hook0-API (https://app.hook0.com/api/v1/) gibt es ein offenes Bug-Bounty-Programm."
          ]
        }
      ]
    },
    {
      "h2": "Infrastruktur- und Netzwerksicherheit",
      "cards": [
        {
          "h3": "Physische Zugangskontrolle",
          "bodyHtml": "Hook0 läuft auf der <a href=\"https://www.clever-cloud.com/\" class=\"text-green-400 hover:text-green-300 transition-colors\">Clever-Cloud-Plattform</a> in Frankreich. Die Rechenzentren von Clever Cloud setzen auf ein mehrschichtiges Sicherheitsmodell mit umfassenden Schutzmaßnahmen, etwa:",
          "bullets": [
            "Speziell angefertigte elektronische Zugangskarten",
            "Alarmanlagen und Perimeterzäune",
            "Fahrzeugschranken und Metalldetektoren",
            "Biometrische Authentifizierung"
          ],
          "footHtml": "Mitarbeitende von Hook0 haben keinen physischen Zugang zu Rechenzentren, Servern, Netzwerkequipment oder Speichersystemen von Clever Cloud."
        },
        {
          "h3": "Logische Zugangskontrolle",
          "bodyHtml": "Hook0 ist der eingetragene Administrator seiner Infrastruktur bei Clever Cloud. Nur namentlich autorisierte Mitglieder des Hook0-Ops-Teams konfigurieren die Infrastruktur bei Bedarf, hinter einem per Zwei-Faktor-Authentifizierung gesicherten VPN. Für einzelne Server sind spezifische private Schlüssel erforderlich, die an einem sicheren und verschlüsselten Ort liegen."
        },
        {
          "h3": "Unabhängige Audits",
          "bodyHtml": "Clever Cloud durchläuft regelmäßig diverse unabhängige Audits Dritter und kann die Prüfung der Compliance-Kontrollen für Rechenzentren, Infrastruktur und Betrieb belegen. Dazu zählen unter anderem die SOC-2-Zertifizierung nach SSAE 16 und die ISO-27001-Zertifizierung."
        }
      ]
    },
    {
      "h2": "Geschäftskontinuität und Disaster Recovery",
      "cards": [
        {
          "h3": "Hohe Verfügbarkeit",
          "bodyHtml": "Jeder Baustein des Hook0-Dienstes läuft auf sauber dimensionierten, redundanten Servern (mehrere Load Balancer, Webserver, Replikat-Datenbanken), um Ausfälle abzufangen. Im Rahmen der regulären Wartung werden Server aus dem Betrieb genommen, ohne die Verfügbarkeit zu beeinträchtigen."
        },
        {
          "h3": "Geschäftskontinuität",
          "bodyHtml": "Hook0 hält stündliche, verschlüsselte Backups in mehreren Regionen bei Clever Cloud vor. Auch wenn dieser Fall nie erwartet wird, werden bei Verlust von Produktionsdaten (Verlust der primären Datenspeicher) die Organisationsdaten aus diesen Backups wiederhergestellt."
        },
        {
          "h3": "Disaster Recovery",
          "bodyHtml": "Bei einem regionsweiten Ausfall baut Hook0 eine gleichwertige Umgebung in einer anderen Clever-Cloud-Region auf. Das Hook0-Ops-Team verfügt über ausgiebige Erfahrung mit vollständigen Regionsmigrationen."
        }
      ]
    },
    {
      "h2": "Unternehmenssicherheit",
      "cards": [
        {
          "h3": "Schutz vor Schadsoftware",
          "bodyHtml": "Bei Hook0 fangen gute Sicherheitspraktiken im eigenen Team an. Wir gehen ausdrücklich über das Übliche hinaus, um uns gegen interne Bedrohungen und lokale Sicherheitslücken zu schützen."
        },
        {
          "h3": "Risikomanagement",
          "paragraphs": [
            "Hook0 folgt den Risikomanagement-Verfahren aus <a href=\"http://csrc.nist.gov/publications/PubsSPs.html\" class=\"text-green-400 hover:text-green-300 transition-colors\">NIST SP 800-30</a> mit neun Schritten der Risikobewertung und sieben Schritten der Risikominimierung.",
            "Alle Produktänderungen durchlaufen Code-Review, CI und Build-Pipeline, bevor sie die Produktionsserver erreichen. Nur namentlich benannte Mitarbeitende des Hook0-Ops-Teams haben SSH-Zugang zu den Produktionsservern.",
            "Hook0 führt über den gesamten Produktlebenszyklus Risikobewertungen nach den Standards der <a href=\"https://www.law.cornell.edu/cfr/text/45/164.308\" class=\"text-green-400 hover:text-green-300 transition-colors\">HIPAA Security Rule, 45 CFR 164.308</a> durch."
          ]
        },
        {
          "h3": "Sicherheitsrichtlinien und Schulungen",
          "bodyHtml": "Hook0 pflegt ein internes Wiki der Sicherheitsrichtlinien, das laufend aktualisiert und jährlich auf Lücken überprüft wird. Alle neuen Mitarbeitenden erhalten ein Onboarding und eine Systemeinweisung mit Review der Sicherheitsrichtlinien."
        },
        {
          "h3": "Offenlegungsrichtlinie",
          "paragraphs": [
            "Hook0 folgt dem vom <a href=\"https://www.sans.org/reading-room/whitepapers/incident/incident-handlers-handbook-33901\" class=\"text-green-400 hover:text-green-300 transition-colors\">SANS</a> empfohlenen Prozess zur Behandlung von Sicherheitsvorfällen, mit Identifikation, Eindämmung, Beseitigung, Wiederherstellung, Kommunikation und Dokumentation der Sicherheitsereignisse.",
            "Hook0 veröffentlicht den operativen Status und Vorfälle live auf der <a href=\"https://status.hook0.com/\" class=\"text-green-400 hover:text-green-300 transition-colors\">Status-Seite</a>. Bekannte Vorfälle werden dort sowie im <a href=\"https://twitter.com/hook0_\" class=\"text-green-400 hover:text-green-300 transition-colors\">Twitter-Feed</a> kommuniziert."
          ]
        }
      ]
    }
  ]
};
