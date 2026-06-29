// Per-page strings for webhook-platform (DE).
// /humanizer pro + legal-reviewer applied. DSGVO claims as process; SSPL = quelloffen.
module.exports = {
  "pageTitle": "Webhook-Service mit EU-Hosting, DSGVO-konform betrieben | Hook0",
  "pageDescription": "Hook0 ist eine quelloffene Webhook-Plattform mit automatischen Retries, HMAC-Signaturen und vollständigen Zustellprotokollen. Anwendungsdaten in Frankreich (Clever Cloud), auf DSGVO-Konformität ausgelegt. Kostenlos starten, ohne Kreditkarte.",
  "pageModified": "2026-06-25",
  "track": "de-webhook-plattform",
  "hero": {
    "eyebrow": "Webhook-Plattform",
    "titleLine1": "Webhooks, die ankommen",
    "titleLine2": "zuverlässig, prüfbar, in Europa",
    "subtitle": "Automatische Wiederholungsversuche, HMAC-Signaturen und vollständige Logs. Die Anwendungsdaten werden in Frankreich verarbeitet (Clever Cloud). Quelloffen, selbst hostbar. Herausgeber ohne US-Mutterkonzern.",
    "ctaPrimary": "Kostenlos starten",
    "ctaSecondary": "Playground testen",
    "microcopy": "100 Events/Tag kostenlos. Keine Kreditkarte. Quelloffen."
  },
  "socialProof": false,
  "capabilities": {
    "eyebrow": "Funktionen",
    "h2": "Kein verlorenes Webhook",
    "subtitle": "Produktionsreife Webhook-Zustellung bedeutet rund drei Sprints Infrastruktur. Die sparst du dir.",
    "cards": [
      {
        "title": "HMAC-Signaturen",
        "body": "Alle Payloads werden signiert, damit deine Empfänger prüfen können, dass eine Anfrage wirklich von dir stammt. Zeitstempel schützen vor Replay-Angriffen."
      },
      {
        "title": "Zwei-Phasen-Retries",
        "body": "Schnelle Wiederholungen für kurzzeitige Aussetzer, langsame über Tage bei echten Ausfällen. Pro Subscription einstellbar, mit Dead-Letter-Queue am Ende."
      },
      {
        "title": "Zustellprotokolle",
        "body": "Anfrage, Antwort, Statuscode, Latenz. Jedes Event lässt sich aus dem Dashboard erneut zustellen. Dein Support rät nicht mehr."
      },
      {
        "title": "Subscriber-Portal",
        "body": "Eine fertige Oberfläche, in der deine Nutzer ihre Endpoints, Secrets und Event-Filter selbst verwalten. Keine Tickets mehr, um einen Schlüssel zu rotieren."
      },
      {
        "title": "SSPL, kein Open-Core",
        "body": "Derselbe Code in Cloud und Self-Hosting. Docker Compose oder Kubernetes. Keine Enterprise-Stufe, die nützliche Funktionen versteckt."
      },
      {
        "title": "In Europa gehostet",
        "body": "Anwendungsdaten in Frankreich bei Clever Cloud, auf DSGVO-Konformität ausgelegt. CDN Cloudflare USA im <a href=\"/de/auftragsverarbeitungsvertrag\">DPA</a> offengelegt. Wichtig, wenn deine Kunden Wert darauf legen, wo ihre Events landen."
      }
    ]
  },
  "howItWorks": {
    "eyebrow": "Code",
    "h2": "Ein API-Aufruf, um ein Event zuzustellen",
    "code": "// Löse ein Event aus, von überall in deinem Backend.\nawait hook0.message.create(\"&lt;application_id&gt;\", {\n  event_type: \"invoice.paid\",\n  event_id:   \"evt_Wqb1k73rXprtTm7Qdlr38G\",\n  payload: {\n    invoice_id: \"in_8X9aBcDeFgHiJk\",\n    status:     \"paid\",\n    amount_eur: 4990\n  }\n});\n\n// Was Hook0 nach diesem Aufruf tut:\n// signiert den Payload per HMAC, verteilt an jeden passenden\n// Subscriber, wiederholt fehlgeschlagene Zustellungen nach einem\n// Zwei-Phasen-Plan und speichert Anfrage und Antwort für den erneuten Versand.\n",
    "docsLabel": "Zum Schnellstart-Guide (auf Englisch) →",
    "docsHref": "https://documentation.hook0.com/docs/getting-started",
    "docsTrack": "de-webhook-plattform-docs"
  },
  "dataResidency": {
    "eyebrow": "Datenresidenz & DSGVO",
    "h2": "Deine Daten bleiben in der EU, ehrlich erklärt",
    "paras": [
      {
        "html": "Die Anwendungsdaten von Hook0 Cloud werden ausschließlich in der EU verarbeitet, in Frankreich bei <a href=\"https://www.clever-cloud.com\" target=\"_blank\" rel=\"noopener\" class=\"text-indigo-400 hover:text-indigo-300 underline\">Clever Cloud</a>. Herausgeber ist die FGRibreau SARL, eine französische Gesellschaft ohne US-Muttergesellschaft."
      },
      {
        "html": "Als CDN setzen wir Cloudflare ein, einen US-Konzern, der grundsätzlich dem US&nbsp;CLOUD&nbsp;Act unterliegt. Welche Daten über das CDN laufen und auf welcher Rechtsgrundlage, steht in unserem Auftragsverarbeitungsvertrag (DPA). Wir behaupten nicht „100&nbsp;% souverän&#8220;. Wir legen offen, wo deine Daten liegen und wer sie verarbeitet. Ein DPA mit vollständiger Liste der Unterauftragsverarbeiter ist auf Anfrage verfügbar."
      },
      {
        "html": "<strong>Für regulierte Umgebungen:</strong> Eine in der EU betriebene Webhook-Zustellung mit klarer Unterauftragsverarbeiter-Kette unterstützt dich bei deinen Pflichten zum Lieferkettenrisiko (NIS2) und zu IKT-Drittparteien (DORA). Hook0 ist kein zertifizierter Anbieter dieser Rahmenwerke, aber ein dokumentierbarer, in der EU ansässiger Baustein. Self-Hosting gibt dir zusätzlich maximale Kontrolle.",
        "emphasis": true
      }
    ]
  },
  "openSource": {
    "eyebrow": "Quelloffen",
    "h2": "Du behältst die Kontrolle",
    "html": "Hook0 ist quelloffen unter der <a href=\"https://spdx.org/licenses/SSPL-1.0.html\" target=\"_blank\" rel=\"noopener\" class=\"text-indigo-400 hover:text-indigo-300 underline\">SSPL-1.0</a> (Server Side Public License). Du kannst dieselbe Plattform jederzeit auf deiner eigenen Infrastruktur in der EU betreiben. Das ist deine garantierte Ausstiegsoption. Der Quellcode ist jederzeit auf <a href=\"{{GITHUB}}\" target=\"_blank\" rel=\"noopener\" class=\"text-indigo-400 hover:text-indigo-300 underline\">GitHub</a> einsehbar."
  },
  "buildVsBuy": {
    "eyebrow": "Selbst bauen vs. Hook0",
    "h2": "Was der Eigenbau wirklich kostet",
    "head": [
      "Aspekt",
      "Selbst bauen",
      "Hook0"
    ],
    "rows": [
      [
        "Zeit bis zum ersten Webhook",
        "3+ Sprints",
        "30 Minuten"
      ],
      [
        "Retry-Logik",
        "Backoff und DLQ selbst entwerfen",
        "Zwei-Phasen, konfigurierbar"
      ],
      [
        "HMAC-Signaturen",
        "selbst implementieren und dokumentieren",
        "enthalten, dokumentiert"
      ],
      [
        "Subscriber-Oberfläche",
        "ein weiterer Sprint",
        "fertiges Portal"
      ],
      [
        "Replay und Debugging",
        "eigene Dashboards",
        "integrierte Logs und Replay"
      ],
      [
        "Laufende Wartung",
        "für immer, dein Team",
        "unsere"
      ],
      [
        "Lock-in-Risiko",
        "keins, aber alles auf dir",
        "Keins. Voller Quellcode, selbst hostbar."
      ]
    ],
    "footHtml": "Transparente Preise findest du in unserer <a href=\"/pricing\" class=\"text-indigo-400 hover:text-indigo-300 underline\">Preisübersicht</a> (auf Englisch)."
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Häufige Fragen zur Webhook-Plattform",
    "items": [
      {
        "q": "Was ist eine Webhook-Plattform?",
        "a": "Eine Webhook-Plattform ist die produktionsreife Infrastruktur zwischen deiner Anwendung und den Endpoints deiner Kunden. Sie signiert Payloads per HMAC, wiederholt fehlgeschlagene Zustellungen mit konfigurierbarem Backoff, speichert Zustellprotokolle und stellt ein Subscriber-Portal bereit, in dem deine Nutzer ihre Endpoints selbst verwalten. Hook0 liefert all das als Managed Service oder als selbst gehosteten Docker-Stack."
      },
      {
        "q": "Warum eine Webhook-Plattform nutzen, statt selbst zu bauen?",
        "a": "Ein produktionsreifes Webhook-System bedeutet mehr als drei Entwicklungs-Sprints für Retries, Dead-Letter-Queues, HMAC-Signaturen, Monitoring und eine Subscriber-Oberfläche, dazu die laufende Wartung. Mit einer Webhook-Plattform wie Hook0 bist du in 30 Minuten startklar und gewinnst diese Entwicklungszeit für dein Produkt zurück."
      },
      {
        "q": "Wo werden meine Daten gehostet? Ist Hook0 auf DSGVO-Konformität ausgelegt?",
        "a": "Die Anwendungsdaten von Hook0 Cloud werden in Frankreich bei Clever Cloud verarbeitet. Herausgeber ist eine französische SARL ohne US-Mutterkonzern. Als CDN nutzen wir Cloudflare; das ist samt Rechtsgrundlage der Übermittlung in unserem Auftragsverarbeitungsvertrag (DPA) aufgeführt. Hook0 ist auf DSGVO-Konformität ausgelegt; ein DPA mit vollständiger Liste der Unterauftragsverarbeiter ist auf Anfrage verfügbar. Wer maximale Kontrolle braucht, hostet Hook0 selbst in der EU."
      },
      {
        "q": "Ist Hook0 kostenlos?",
        "a": "Ja. Hook0 hat eine dauerhaft kostenlose Stufe, ohne Kreditkarte. Du kannst denselben Code zusätzlich kostenlos auf deiner eigenen Infrastruktur selbst hosten, unter der SSPL-1.0. Bezahlte Pläne schalten nur höheres Volumen und dedizierten Support frei; es werden keine Funktionen zurückgehalten."
      },
      {
        "q": "Kann ich Hook0 selbst hosten?",
        "a": "Ja. Die gesamte Webhook-Plattform ist quelloffen unter der SSPL-1.0 und kommt mit Docker Compose und Kubernetes-Manifesten. Self-Hosting ist kostenlos, ohne Enterprise-Stufe, und die selbst gehostete Edition hat denselben Funktionsumfang wie die Cloud."
      }
    ]
  },
  "related": {
    "h2": "Weiterführend (auf Englisch)",
    "links": [
      {
        "label": "Webhook Platform",
        "href": "/webhook-platform"
      },
      {
        "label": "Open-Source Webhooks",
        "href": "/open-source-webhooks"
      },
      {
        "label": "Self-Hosted Webhooks",
        "href": "/self-hosted-webhooks"
      },
      {
        "label": "Security & Compliance",
        "href": "/security"
      },
      {
        "label": "Preise",
        "href": "/pricing"
      }
    ]
  },
  "cta": {
    "h2": "Du hast Wichtigeres zu bauen",
    "subtitle": "Hör auf, Webhook-Infrastruktur zu bauen. Liefere Features. In wenigen Minuten bist du startklar, und du kannst jederzeit zum Self-Hosting migrieren.",
    "ctaPrimary": "Kostenlos starten",
    "ctaSecondary": "Schnellstart-Guide",
    "ctaSecondaryHref": "https://documentation.hook0.com/docs/getting-started",
    "track": "de-webhook-plattform-cta-register",
    "badges": [
      "Keine Kreditkarte nötig",
      "In 5 Minuten startklar",
      "Quelloffen, jederzeit migrierbar"
    ]
  }
};
