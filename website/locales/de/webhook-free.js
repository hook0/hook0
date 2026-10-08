// Per-page strings for webhook-free (DE).
// /humanizer pro applied. Du-Form. Strikte Parität mit locales/en/webhook-free.js
// (gleiche Abschnitte, gleiche 8 FAQ-Fragen, gleiche Tabelle).
// Geprüfte Fakten: kostenloser Cloud-Tarif = src/includes/_pricing.ejs (1 Entwickler,
// 1 Anwendung, 10 Event-Typen, 10 Subscriptions, 100 Events/Tag, überzählige Events
// blockiert, 7 Tage Aufbewahrung); Lizenz SSPL-1.0 (LICENSE.txt); play.hook0.com ohne
// Konto. Wettbewerberzellen wie auf svix-alternatives / webhook-service.
// Natives deutsches Korrekturlesen steht vor jeder bezahlten DE-Kampagne noch aus.
// faq.items[].a muss dem sichtbaren Kartentext entsprechen.
module.exports = {
  "pageTitle": "Webhooks kostenlos: Cloud, Self-Hosting oder Tester | Hook0",
  "pageDescription": "Drei kostenlose Wege zu Hook0: Webhooks im Browser ohne Konto testen, den Open-Source-Server (SSPL-1.0) ohne Volumenlimit selbst hosten oder den kostenlosen Cloud-Tarif ohne Kreditkarte nutzen.",
  "pageModified": "2026-10-08",
  "track": "de-webhook-kostenlos",
  "selfHostHref": "https://documentation.hook0.com/self-hosting/docker-compose",
  "hero": {
    "eyebrow": "Kostenlose Webhooks",
    "titleLine1": "Kostenlose Webhooks, auf drei ehrliche Arten",
    "titleLine2": "Teste jetzt einen, oder betreibe den Server kostenlos",
    "subtitleHtml": "Drei Dinge sind bei Hook0 wirklich kostenlos, ohne versteckte Paywall. <strong class=\"text-gray-200\">play.hook0.com</strong> prüft und testet jeden Webhook, ohne Konto. Der <strong class=\"text-gray-200\">Open-Source-Server</strong> (SSPL-1.0) läuft selbst gehostet auf deiner Infrastruktur, ohne Volumenlimit. Und <strong class=\"text-gray-200\">Hook0 Cloud</strong> hat einen kostenlosen Tarif: 100 Events pro Tag, keine Kreditkarte, keine gesperrten Funktionen. Nimm, was du wirklich brauchst.",
    "ctaPlay": "Webhook jetzt testen",
    "ctaSelfHost": "Server kostenlos selbst hosten",
    "microcopy": "Testen ohne Konto. Self-Hosting ohne Limit. Kostenloser Cloud-Tarif, keine Kreditkarte."
  },
  "flow": {
    "title": "Wie Hook0 einen Webhook zustellt",
    "desc": "Ein Event wird per HMAC signiert, zugestellt, nach einem 503-Fehler automatisch wiederholt und dann mit einer 200-Antwort angenommen.",
    "event": "Event",
    "sign": "signieren",
    "deliver": "zustellen",
    "retry": "Retry",
    "caption": "Jedes Event wird signiert (HMAC), zugestellt, bei Fehlern automatisch wiederholt und protokolliert, in der Cloud wie selbst gehostet."
  },
  "router": {
    "eyebrow": "Hier starten",
    "h2": "Empfangen oder senden? Wähl deinen kostenlosen Weg",
    "subtitle": "„Kostenloser Webhook“ meint zwei verschiedene Aufgaben. Das Diagramm zeigt, welche du brauchst. Beide sind kostenlos.",
    "diagram": {
      "title": "Webhooks mit Hook0 empfangen oder senden",
      "desc": "Links empfängst und prüfst du einen Webhook, den dir ein Dienst schickt, mit dem kostenlosen Tool play.hook0.com. Rechts sendest du deine eigenen HMAC-signierten Webhooks an deine Nutzer, mit dem kostenlosen Cloud-Tarif oder dem selbst gehosteten Open-Source-Server.",
      "receiveEyebrow": "EMPFANGEN & DEBUGGEN",
      "receiveSub": "play.hook0.com, ohne Konto",
      "service": "ein Dienst",
      "serviceSub": "Stripe, GitHub…",
      "playSub": "prüfen, erneut senden",
      "sendEyebrow": "SIGNIERT SENDEN",
      "sendSub": "Cloud oder Self-Hosting, Produktion",
      "hook0Sub": "signieren, Retry",
      "signed": "HMAC-signiert",
      "autoRetry": "Auto-Retry bei 5xx",
      "yourUser": "dein Nutzer"
    },
    "captionHtml": "<strong class=\"text-gray-300\">Links</strong> empfängst und prüfst du einen Webhook mit dem kostenlosen Tool <a href=\"https://play.hook0.com\" target=\"_blank\" rel=\"noopener\" class=\"text-indigo-400 hover:text-indigo-300 underline\" onclick=\"trackEvent('CTA', 'Click', 'de-webhook-kostenlos-roles-play')\">play.hook0.com</a>. <strong class=\"text-gray-300\">Rechts</strong> sendest du deine eigenen signierten Webhooks an deine Nutzer, kostenlos auf dem <a href=\"/de/selbst-gehostete-webhooks\" class=\"text-indigo-400 hover:text-indigo-300 underline\" onclick=\"trackEvent('CTA', 'Click', 'de-webhook-kostenlos-roles-selfhost')\">selbst gehosteten Server</a> oder im kostenlosen Cloud-Tarif.",
    "test": {
      "title": "Ich will nur einen Webhook testen",
      "body": "Erzeug eine öffentliche URL, prüf die Payloads, die ein Dienst dir schickt, und sende sie erneut. Im Browser, nichts zu installieren, keine Registrierung.",
      "cta": "play.hook0.com öffnen →",
      "note": "Für immer kostenlos. Öffentliches Tool, Open Source."
    },
    "infra": {
      "title": "Ich brauche Webhook-Infrastruktur für mein Produkt",
      "body": "Signiere, wiederhole, verteile und überwache die Webhooks, die du an deine eigenen Nutzer sendest. Hoste den Open-Source-Server selbst oder starte mit dem kostenlosen Cloud-Tarif.",
      "ctaSelfHost": "Kostenlos selbst hosten (SSPL)",
      "ctaCloud": "Kostenlos in der Cloud starten",
      "note": "Open Source ohne Limit, oder 100 Events pro Tag in der Cloud."
    }
  },
  "ways": {
    "eyebrow": "Drei kostenlose Wege",
    "h2": "Drei Wege, Hook0 kostenlos zu nutzen, und wo jeder endet",
    "subtitle": "Keine Paywall-Überraschung. Hier steht genau, was kostenlos ist und wo jede Option endet.",
    "cards": [
      {
        "id": "play",
        "title": "Playground",
        "badge": "Für immer kostenlos, ohne Konto",
        "body": "Webhook-Payloads im Browser empfangen, prüfen, debuggen und erneut senden. Der schnellste Weg zu sehen, was ein Dienst tatsächlich schickt.",
        "bestFor": "Ideal, um eine Integration sofort zu debuggen.",
        "cta": "Playground öffnen →",
        "href": "https://play.hook0.com",
        "external": true
      },
      {
        "id": "selfhost",
        "title": "Selbst gehostet (Open Source)",
        "badge": "Für immer kostenlos, unbegrenztes Volumen",
        "body": "Betreibe den kompletten Hook0-Server auf deiner Infrastruktur unter SSPL-1.0. Derselbe Code wie in der Cloud, alle Zustellfunktionen, deine Daten bleiben bei dir. Keine Lizenzkosten, kein Volumenlimit von Hook0.",
        "bestFor": "Ideal für Produktion mit voller Kontrolle und ohne Zähler.",
        "cta": "Self-Hosting-Anleitung →",
        "href": "https://documentation.hook0.com/self-hosting/docker-compose",
        "external": false
      },
      {
        "id": "cloud",
        "title": "Kostenloser Cloud-Tarif",
        "badge": "Für immer kostenlos, keine Kreditkarte",
        "body": "Von uns betrieben, du musst nichts betreiben. 1 Anwendung, 10 Event-Typen, 10 Subscriptions, bis zu 100 Events pro Tag, 7 Tage Aufbewahrung. Alle Funktionen enthalten. Events über 100 pro Tag werden blockiert.",
        "bestFor": "Ideal für Nebenprojekte und einen ernsthaften Test, ohne eigene Infrastruktur.",
        "cta": "Kostenlos in der Cloud starten →",
        "href": "https://app.hook0.com/register",
        "external": false
      }
    ]
  },
  "compare": {
    "eyebrow": "Vergleich",
    "h2": "Was „kostenlos“ bei Webhook-Tools bedeutet",
    "subtitle": "Die meisten „kostenlosen“ Webhook-Angebote enden bei einem gedeckelten Cloud-Tarif. Hook0 lässt sich kostenlos selbst hosten, mit denselben Funktionen wie in der Cloud.",
    "headers": {
      "criteria": "Kriterium",
      "hook0": "Hook0",
      "svix": "Svix",
      "hookdeck": "Hookdeck",
      "webhooksite": "webhook.site"
    },
    "rows": [
      { "criteria": "Kostenloser Cloud-Tarif", "hook0": "Ja, ohne Kreditkarte", "svix": "Ja", "hookdeck": "Ja", "webhooksite": "Ja (nur Tester)" },
      { "criteria": "Kostenlos selbst hosten", "hook0": "Ja, voller Funktionsumfang (SSPL)", "svix": "MIT-Server, einige Cloud-Funktionen fehlen", "hookdeck": "Nur Outpost (Apache-2.0)", "webhooksite": "Nein" },
      { "criteria": "Webhooks an deine Nutzer senden", "hook0": "Ja", "svix": "Ja", "hookdeck": "Ja", "webhooksite": "Nein (nur Empfang)" },
      { "criteria": "Kostenloses Tool zum Testen von Webhooks", "hook0": "Ja (play.hook0.com)", "svix": "Ja", "hookdeck": "Console", "webhooksite": "Ja" },
      { "criteria": "Vollständiger Quellcode verfügbar", "hook0": "Ja, zu 100 % (GitHub und GitLab)", "svix": "Nur Kernserver", "hookdeck": "Nur Outpost", "webhooksite": "Nein" },
      { "criteria": "Wo die Daten laufen", "hook0": "EU-Datenebene auf jedem Tarif, oder selbst gehostet", "svix": "Regionen USA und EU, oder selbst gehostet", "hookdeck": "Regionen USA, EU und Asien", "webhooksite": "Unterschiedlich" }
    ],
    "footnote": "Vergleich öffentlich dokumentierter Angebote, zuletzt geprüft im Oktober 2026. Prüf die aktuellen Tarife jedes Anbieters, bevor du entscheidest."
  },
  "limits": {
    "eyebrow": "Keine Überraschungen",
    "h2": "Wo „kostenlos“ endet",
    "intro": "Offen sagen ist besser als eine versteckte Paywall. Hier steht genau, wann du zahlst:",
    "itemsHtml": [
      "<strong class=\"text-gray-200\">Self-Hosting bleibt für immer kostenlos</strong>, ohne Lizenzkosten und ohne Volumenlimit. Du zahlst nur deine eigene Infrastruktur.",
      "<strong class=\"text-gray-200\">Bezahlte Cloud-Tarife</strong> (Startup für 59 €/Monat zzgl. MwSt., dann Pro) schalten mehr Volumen, mehr Anwendungen und dedizierten Support frei. Keine Funktion ist dahinter gesperrt: Der kostenlose Tarif hat bereits HMAC-Signaturen, Retries, Protokolle und das Portal.",
      "<strong class=\"text-gray-200\">Du zahlst für Volumen und Support.</strong> Wenn du mehr als 100 Events pro Tag in der Cloud brauchst oder ein verwaltetes SLA, beginnt dort der bezahlte Bereich."
    ],
    "cta": "Alle Preise ansehen →",
    "ctaHref": "/de/preise"
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Fragen zu kostenlosen Webhooks",
    "items": [
      {
        "q": "Ist Hook0 wirklich kostenlos?",
        "a": "Ja, auf drei Arten. play.hook0.com ist ein kostenloses Tool, um Webhooks im Browser zu testen und zu prüfen, ohne Konto. Der Open-Source-Server lässt sich für immer kostenlos selbst hosten, ohne Volumenlimit von Hook0 (SSPL-1.0). Und Hook0 Cloud hat einen kostenlosen Developer-Tarif: 100 Events pro Tag, keine Kreditkarte, keine gesperrten Funktionen. Kein Lockangebot."
      },
      {
        "q": "Gibt es einen kostenlosen Open-Source-Webhook-Server?",
        "a": "Ja. Hook0 ist ein kostenloser Open-Source-Webhook-Server unter SSPL-1.0, veröffentlicht auf GitHub und GitLab. Er bringt Docker-Compose- und Kubernetes-Manifeste mit und läuft auf PostgreSQL. Self-Hosting ist kostenlos, ohne Lizenzkosten und ohne Volumenlimit von Hook0. Es ist derselbe Code wie in der Cloud, mit denselben Funktionen."
      },
      {
        "q": "Welche Limits hat der kostenlose Tarif von Hook0 Cloud?",
        "a": "Der kostenlose Developer-Tarif umfasst 1 Entwickler, 1 Anwendung, 10 Event-Typen, 10 Subscriptions, bis zu 100 Events pro Tag und 7 Tage Datenaufbewahrung. Alle Zustellfunktionen sind enthalten: HMAC-Signaturen, Retries, Zustellprotokolle und das Subscriber-Portal. Eine Kreditkarte ist nicht nötig, und der Tarif bleibt für Nebenprojekte für immer kostenlos."
      },
      {
        "q": "Kann ich einen Webhook kostenlos ohne Konto testen?",
        "a": "Ja. play.hook0.com gibt dir eine öffentliche URL, um Webhook-Payloads direkt im Browser zu empfangen, zu prüfen und erneut zu senden. Nichts zu installieren, keine Registrierung. Das Tool ist kostenlos und selbst Open Source."
      },
      {
        "q": "Ist Self-Hosting von Hook0 kostenlos?",
        "a": "Ja. Hook0 verlangt beim Self-Hosting keine Lizenzkosten und setzt kein Volumenlimit. Du stellst die Infrastruktur (Docker oder Kubernetes plus PostgreSQL) und bekommst Community-Support auf Discord. Verwaltete Skalierung, ein Verfügbarkeits-SLA und verwaltete Updates übernimmst du beim Self-Hosting selbst."
      },
      {
        "q": "Was ist der Unterschied zwischen play.hook0.com und Hook0 Cloud?",
        "a": "play.hook0.com ist ein kostenloses Tool, um Webhooks zu empfangen, zu prüfen und zu debuggen, die dir andere Dienste schicken. Hook0 Cloud ist die Infrastruktur, um signierte Webhooks in Produktion an deine eigenen Nutzer zu senden, mit Retries, Dead Letter Queue und Subscriber-Portal. Play zum Debuggen, Cloud oder Self-Hosting zum Ausliefern."
      },
      {
        "q": "Läuft der kostenlose Cloud-Tarif ab oder braucht er eine Kreditkarte?",
        "a": "Nein. Der kostenlose Developer-Tarif hat kein Zeitlimit und braucht keine Kreditkarte. Er bleibt für Nebenprojekte für immer kostenlos."
      },
      {
        "q": "Was passiert, wenn ich den kostenlosen Cloud-Tarif überschreite?",
        "a": "Events über 100 pro Tag werden im kostenlosen Tarif bis zum nächsten Tag blockiert. Um mehr zu senden, wechselst du in einen bezahlten Tarif (Startup für 59 €/Monat zzgl. MwSt., oder Pro). Bezahlte Tarife schalten nur mehr Volumen und Support frei, keine Funktion ist dahinter gesperrt. Wenn du gar kein Volumenlimit willst, hoste den Open-Source-Server kostenlos selbst."
      }
    ]
  },
  "related": {
    "h2": "Verwandte Seiten",
    "links": [
      { "label": "play.hook0.com", "href": "https://play.hook0.com", "external": true },
      { "label": "Webhook-Tester", "href": "/de/webhook-tester" },
      { "label": "Von webhook.site migrieren", "href": "/de/von-webhook-site-migrieren" },
      { "label": "Open-Source-Webhooks", "href": "/de/quelloffene-webhooks" },
      { "label": "Selbst gehostete Webhooks", "href": "/de/selbst-gehostete-webhooks" },
      { "label": "Webhook-Plattform", "href": "/de/webhook-plattform" },
      { "label": "Webhook Service", "href": "/de/webhook-service" },
      { "label": "Preise", "href": "/de/preise" },
      { "label": "Hook0 vs Svix", "href": "/de/hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "/de/hook0-vs-hookdeck" }
    ]
  }
};
