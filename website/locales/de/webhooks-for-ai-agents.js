// Per-page strings for webhooks-for-ai-agents (DE, Slug: mcp-server-fuer-webhooks).
// Slug-Wahl: Google Suggest DE liefert zu «webhook ki» nichts Brauchbares,
// zu «mcp server webhook» / «mcp server webhooks» dagegen schon — deutsche
// Entwickler behalten die englischen Fachwörter und suchen den MCP-Server.
// Daher «mcp-server-fuer-webhooks» statt einer wörtlichen Übersetzung von
// «webhooks for ai agents».
// /humanizer pro + legal-reviewer applied.
// Hook0 selbst = «quelloffen (SSPL-1.0)», NIE «Open Source» (SSPL von der
// OSI abgelehnt, UWG §5 DACH-Risiko).
// Souveränität: CDN Cloudflare, Inc. (USA) offen genannt, Anwendungsdaten bei
// Clever Cloud SAS (Frankreich, EWR). NIE «kein US-Konzern im Stack /
// keine Daten verlassen die EU / 100 % souverän / CLOUD Act free».
// DSGVO als Prozess-Claim, keine absoluten Zertifizierungsaussagen.
// Weder SOC 2 noch ISO: Hook0 hat beides nicht.
// Register du, developer-first. Englisch bleibt: Webhook, Endpoint, Payload,
// Event, HMAC, Dashboard. Übersetzt: Signatur, Zustellung, Wiederholungsversuche.
// Fähigkeiten aus documentation/reference/mcp.md (9 Lese-, 8 Schreibwerkzeuge,
// 17 gesamt) — nichts darüber hinaus, und keine Aussage darüber, was
// Wettbewerber nicht können.
// DE-Gate (website/CLAUDE.local.md): vor bezahlter Promotion dieser Seite ist
// ein muttersprachliches Lektorat erforderlich.
// Der Text von faq.items[].a MUSS zeichengleich zum sichtbaren Kartentext sein;
// das FAQPage-JSON-LD wird aus demselben Array erzeugt. Reiner Text, kein HTML.
module.exports = {
  "pageTitle": "Webhook-MCP-Server für KI-Agenten | Hook0",
  "pageDescription": "Der MCP-Server von Hook0 gibt Claude, Cursor und Windsurf 17 Werkzeuge für die Webhooks, die dein Produkt sendet: Subscriptions anlegen, Events auslösen, fehlgeschlagene Zustellungen erneut senden.",
  "pageModified": "2026-08-25",
  "track": "de-mcp-webhooks",
  "hero": {
    "eyebrow": "Webhook-MCP-Server",
    "titleLine1": "Dein Agent kann Webhooks lesen.",
    "titleLine2": "Senden kann er sie auch.",
    "subtitle": "Einen Assistenten an Webhooks zu hängen heißt meistens: Events konsumieren, die andere aussenden. Hook0 dreht die Richtung um. Ein MCP-Server, mit dem Claude, Cursor oder Windsurf die Webhooks bedienen, die dein eigenes Produkt sendet: Event-Typen anlegen, Subscriptions einrichten, Events auslösen, Fehlgeschlagenes erneut senden.",
    "ctaPrimary": "Kostenlos starten",
    "ctaSecondary": "MCP-Doku lesen",
    "ctaSecondaryHref": "https://documentation.hook0.com/reference/mcp",
    "microcopy": "100 Events/Tag kostenlos. Ohne Kreditkarte. Der MCP-Server läuft auf deiner Maschine."
  },
  "answer": {
    "h2": "Kurz gefasst",
    "lead": "Der MCP-Server von Hook0 ist ein lokaler Prozess, der dein Hook0-Konto einem MCP-fähigen Assistenten als 17 Werkzeuge zugänglich macht. Neun lesen: Applications, Event-Typen, Subscriptions, Events und Zustellversuche. Acht schreiben: Event-Typ anlegen, Subscription einrichten, Event auslösen, fehlgeschlagene Zustellung erneut senden.",
    "factsLabel": "Das Wichtigste",
    "facts": [
      { "k": "Installation", "v": "<code class=\"text-green-400\">cargo install hook0-mcp</code>" },
      { "k": "Werkzeuge", "v": "17 (9 lesend, 8 schreibend)" },
      { "k": "Clients", "v": "Claude Desktop, Cursor, Windsurf, Cline und jeder MCP-Client" },
      { "k": "Transport", "v": "stdio als Standard, SSE im Dienstbetrieb" },
      { "k": "Ziel", "v": "Hook0 Cloud oder deine Instanz über <code class=\"text-green-400\">HOOK0_API_URL</code>" },
      { "k": "Einschränken", "v": "<code class=\"text-green-400\">HOOK0_READ_ONLY=true</code> plus abgeschwächtes Service-Token" },
      { "k": "Lizenz", "v": "Quelloffen (SSPL-1.0), ohne Open-Core-Rückhalt" }
    ]
  },
  "socialProof": true,
  "sides": {
    "eyebrow": "Zwei Richtungen",
    "h2": "Webhooks und Agenten zeigen in zwei Richtungen",
    "subtitle": "Beides wird oft als ein Thema verhandelt. Es ist nicht dasselbe Problem, und dasselbe Werkzeug löst es nicht.",
    "cards": [
      {
        "title": "Ein Agent, der Webhooks empfängt",
        "bodyHtml": "In Stripe, GitHub oder einem CRM passiert etwas, und ein Agent soll darauf reagieren. Die Arbeit ist eingehend: ein öffentlicher Endpoint oder ein Tunnel, Signaturprüfung, Deduplizierung, und denselben Agentenlauf nicht zweimal starten. Für diese Richtung gibt es reichlich Werkzeuge."
      },
      {
        "title": "Ein Agent, der bedient, was du sendest",
        "bodyHtml": "Hier sendest du. Deine Kunden abonnieren <code class=\"text-green-400\">order.completed</code> und erwarten, dass es ankommt. Die Arbeit ist ausgehend: Event-Typen anlegen, Subscriptions einrichten, nachlesen, warum ein Versuch scheiterte, ihn erneut senden, sobald der Empfänger repariert ist. Genau diese Seite legt der MCP-Server von Hook0 deinem Assistenten offen."
      }
    ],
    "note": "Wenn du ein Produkt betreibst, das andere Systeme abonnieren, kostet dich die zweite Richtung die Support-Tickets."
  },
  "versus": {
    "eyebrow": "MCP oder Webhook",
    "h2": "Zwei Protokolle, zwei Ebenen",
    "intro": "Verglichen werden sie, weil beide Events transportieren. Sie sitzen auf verschiedenen Ebenen, und ein tragfähiges Setup nutzt beide.",
    "rows": [
      {
        "label": "Ein Webhook",
        "bodyHtml": "Eine HTTP-Anfrage, die dein Produkt an eine vom Kunden hinterlegte URL sendet, wenn bei dir etwas passiert. Einseitig, von Maschine zu Maschine, signiert, damit der Empfänger ihr trauen kann, und wiederholt, wenn er nicht erreichbar ist. Das ist die Datenebene: sie bewegt das Event."
      },
      {
        "label": "MCP",
        "bodyHtml": "Das Model Context Protocol, über das ein Assistent Werkzeuge findet und aufruft. Anfrage/Antwort, von einer Person im Gespräch geführt, lokal ausgeführt. Das ist eine Steuerungsebene: sie bedient das, was das Event bewegt."
      },
      {
        "label": "Beides zusammen",
        "bodyHtml": "Dein Produkt sendet seine Events weiterhin über die REST-API oder ein SDK, unverändert. MCP greifst du, wenn du einen neuen Event-Typ anlegen, eine Subscription umhängen oder herausfinden musst, warum die Zustellung von gestern scheiterte, ohne das Dashboard zu öffnen."
      }
    ]
  },
  "tools": {
    "eyebrow": "MCP-Server",
    "h2": "Was der Assistent tatsächlich kann",
    "subtitle": "Siebzehn Werkzeuge, getrennt nach Lesen und Schreiben. Das ist die ausgelieferte Liste, keine Roadmap. Jedes ist mit Beispiel-Prompt in der MCP-Referenz dokumentiert.",
    "headers": {
      "tool": "Werkzeug",
      "does": "Was es tut",
      "say": "Was du tippst"
    },
    "groups": [
      {
        "label": "Lesen (9 Werkzeuge)",
        "rows": [
          { "tool": "list_organizations", "does": "Erreichbare Organisationen auflisten", "say": "&laquo;&nbsp;Zeig meine Organisationen&nbsp;&raquo;" },
          { "tool": "list_applications", "does": "Applications einer Organisation auflisten", "say": "&laquo;&nbsp;Welche Apps habe ich&nbsp;?&nbsp;&raquo;" },
          { "tool": "get_application", "does": "Details einer Application lesen", "say": "&laquo;&nbsp;Zeig Details zu App X&nbsp;&raquo;" },
          { "tool": "list_event_types", "does": "Event-Typen einer Application auflisten", "say": "&laquo;&nbsp;Welche Event-Typen sind angelegt&nbsp;?&nbsp;&raquo;" },
          { "tool": "list_subscriptions", "does": "Webhook-Subscriptions und ihre Konfiguration auflisten", "say": "&laquo;&nbsp;Zeig alle meine Webhooks&nbsp;&raquo;" },
          { "tool": "get_subscription", "does": "Konfiguration einer Subscription lesen", "say": "&laquo;&nbsp;Zeig die Webhook-Konfiguration von&hellip;&nbsp;&raquo;" },
          { "tool": "list_events", "does": "Von einer Application gesendete Events auflisten", "say": "&laquo;&nbsp;Zeig die letzten Events&nbsp;&raquo;" },
          { "tool": "get_event", "does": "Ein Event samt Payload lesen", "say": "&laquo;&nbsp;Zeig Event abc123&nbsp;&raquo;" },
          { "tool": "list_request_attempts", "does": "Zustellversuche zu einem Event auflisten", "say": "&laquo;&nbsp;Zeig die Zustellhistorie zu Event X&nbsp;&raquo;" }
        ]
      },
      {
        "label": "Schreiben (8 Werkzeuge)",
        "highlight": true,
        "rows": [
          { "tool": "create_application", "does": "Eine Application anlegen", "say": "&laquo;&nbsp;Leg eine App Order Service an&nbsp;&raquo;" },
          { "tool": "delete_application", "does": "Eine Application löschen", "say": "&laquo;&nbsp;Lösch die Test-Application&nbsp;&raquo;" },
          { "tool": "create_event_type", "does": "Einen neuen Event-Typ anlegen", "say": "&laquo;&nbsp;Füg den Typ order.completed hinzu&nbsp;&raquo;" },
          { "tool": "create_subscription", "does": "Eine Webhook-Subscription auf eine URL anlegen", "say": "&laquo;&nbsp;Leg einen Webhook auf https://&hellip; an&nbsp;&raquo;" },
          { "tool": "update_subscription", "does": "Eine bestehende Subscription ändern oder abschalten", "say": "&laquo;&nbsp;Schalt den Webhook von&hellip; ab&nbsp;&raquo;" },
          { "tool": "delete_subscription", "does": "Eine Subscription löschen", "say": "&laquo;&nbsp;Entfern den Staging-Webhook&nbsp;&raquo;" },
          { "tool": "ingest_event", "does": "Ein Event auslösen, die Sendeseite, aus dem Assistenten", "say": "&laquo;&nbsp;Schick ein Test-Event user.created&nbsp;&raquo;" },
          { "tool": "retry_delivery", "does": "Eine fehlgeschlagene Zustellung erneut senden", "say": "&laquo;&nbsp;Sende die fehlgeschlagene Zustellung zu Event X erneut&nbsp;&raquo;" }
        ]
      }
    ],
    "footnote": "Dazu acht Ressourcen-URIs unter hook0:// für direkte Abfragen und drei geführte Prompts für die wiederkehrenden Abläufe: Subscription anlegen, Zustellung debuggen, Application aufsetzen.",
    "footHref": "https://documentation.hook0.com/reference/mcp",
    "footLabel": "Vollständige Werkzeugreferenz, mit der Konfiguration für Claude Desktop, Cursor, Windsurf und Cline"
  },
  "setup": {
    "eyebrow": "Einrichtung",
    "h2": "Drei Schritte bis zum ersten Prompt",
    "intro": "Der Server ist eine Rust-Binary, die du einmal installierst. Auf Hook0-Seite läuft nichts Zusätzliches.",
    "steps": [
      {
        "n": "1",
        "title": "Server installieren",
        "bodyHtml": "Er liegt auf crates.io und baut zu einer einzigen Binary.",
        "code": "cargo install hook0-mcp"
      },
      {
        "n": "2",
        "title": "Service-Token anlegen",
        "bodyHtml": "Im Hook0-Dashboard unter den Service-Tokens deiner Organisation. Schwäche es auf die Applications ab, die der Assistent erreichen soll, bevor du es irgendwo einfügst.",
        "code": ""
      },
      {
        "n": "3",
        "title": "Im Assistenten eintragen",
        "bodyHtml": "Claude Desktop liest <code class=\"text-green-400\">claude_desktop_config.json</code>. Cursor, Windsurf und Cline nehmen denselben Block in ihrer eigenen Konfigurationsdatei.",
        "code": "{\n  \"mcpServers\": {\n    \"hook0\": {\n      \"command\": \"hook0-mcp\",\n      \"env\": {\n        \"HOOK0_API_TOKEN\": \"dein-service-token-hier\"\n      }\n    }\n  }\n}"
      }
    ],
    "outro": "Starte den Assistenten neu und frag ihn etwas, das du sonst zusammenklickst: &laquo;&nbsp;Warum ist meine letzte Webhook-Zustellung fehlgeschlagen&nbsp;?&nbsp;&raquo;",
    "docsHref": "https://documentation.hook0.com/reference/mcp",
    "docsLabel": "Pfade der Konfigurationsdateien je Assistent, Umgebungsvariablen und SSE-Modus"
  },
  "guardrails": {
    "eyebrow": "Leitplanken",
    "h2": "Einem Agenten deine Zustellinfrastruktur überlassen",
    "intro": "Schreibzugriff auf Produktions-Webhooks vergibt man nicht nebenbei. Drei Kontrollen kommen mit dem Server, und sie lassen sich kombinieren.",
    "cards": [
      {
        "title": "Nur-Lesen-Modus",
        "bodyHtml": "Setz <code class=\"text-green-400\">HOOK0_READ_ONLY=true</code>, dann meldet der Server nur die neun Lesewerkzeuge. Der Assistent kann einer fehlgeschlagenen Zustellung nachgehen und dabei nichts verändern."
      },
      {
        "title": "Abgeschwächte Tokens",
        "bodyHtml": "Der Nur-Lesen-Modus verkürzt die Werkzeugliste; das Token selbst behält, was ihm zugeteilt wurde. Die Abschwächung begrenzt ein Token auf bestimmte Applications und kann ein Ablaufdatum tragen, durchgesetzt an der API statt im Client. Nimm beides. Sie decken verschiedene Fehlerfälle ab."
      },
      {
        "title": "Er läuft, wo du ihn startest",
        "bodyHtml": "Der MCP-Server ist ein lokaler Prozess, der mit der Hook0-API spricht. Der Assistent sieht, was die aufgerufenen Werkzeuge zurückgeben, und sonst wird nichts irgendwohin weitergereicht. Richte <code class=\"text-green-400\">HOOK0_API_URL</code> auf deine Instanz, und dieselben Werkzeuge bedienen eine selbst gehostete Installation."
      }
    ]
  },
  "platform": {
    "eyebrow": "Darunter",
    "h2": "Der Agent ist die Oberfläche, nicht die Garantie",
    "intro": "Natürliche Sprache ändert, wie du Webhook-Zustellung bedienst. Zustellen tut sie nichts. Das hier tut es.",
    "cards": [
      {
        "title": "Signiert, wiederholt, protokolliert",
        "bodyHtml": "Jeder Versuch trägt eine HMAC-SHA256-Signatur. Die Wiederholungsversuche laufen zweistufig und konfigurierbar: ein Abonnent, der eine Stunde ausfällt, kostet dich nicht diese Stunde an Events. Jeder Versuch wird protokolliert, und das macht aus &laquo;&nbsp;warum ist das fehlgeschlagen&nbsp;&raquo; eine Abfrage statt einer Vermutung."
      },
      {
        "title": "Eine EU-Datenebene, in jedem Tarif",
        "bodyHtml": "Payloads, Datenbank und Backups laufen auf Infrastruktur von Clever Cloud SAS in Frankreich, im EWR, auch im kostenlosen Tarif. Das CDN davor ist Cloudflare, Inc. (USA), offengelegt in der öffentlichen <a href=\"/de/dsgvo-unterauftragsverarbeiter\" class=\"text-green-400 hover:text-green-300 transition-colors\">Liste der Unterauftragsverarbeiter</a> samt Übermittlungsmechanismus. Mehr dazu unter <a href=\"/de/sicherheit\" class=\"text-green-400 hover:text-green-300 transition-colors\">Sicherheit</a>."
      },
      {
        "title": "Code, den du mitnehmen kannst",
        "bodyHtml": "Hook0 ist quelloffen (SSPL-1.0), ohne Open-Core-Rückhalt: der gehostete Dienst führt den Code aus, den du selbst ausführen kannst. MCP-Server, API und Abonnenten-Portal verhalten sich gegenüber einer selbst gehosteten Instanz gleich."
      }
    ]
  },
  "faq": {
    "eyebrow": "Fragen",
    "h2": "Bevor du einen Assistenten auf die Produktion richtest",
    "items": [
      {
        "q": "Wie nutze ich Webhooks mit MCP-Werkzeugen?",
        "a": "Installiere hook0-mcp, leg ein Hook0-Service-Token an und trag den Server in die Konfigurationsdatei deines Assistenten ein. Ab dann hat er siebzehn Werkzeuge auf deinem Konto: Vorhandenes auflisten, einen Event-Typ anlegen, eine Subscription auf eine URL einrichten, ein Test-Event auslösen, eine fehlgeschlagene Zustellung erneut senden. Deine Anwendung sendet ihre echten Events weiterhin über die REST-API oder ein SDK, und dieser Weg bleibt unberührt."
      },
      {
        "q": "Was ist der Unterschied zwischen MCP und einem Webhook?",
        "a": "Ein Webhook ist eine HTTP-Anfrage, die dein Produkt an eine vom Kunden hinterlegte URL sendet, einseitig und von Maschine zu Maschine, signiert, damit der Empfänger ihr trauen kann. MCP ist die Art, wie ein Assistent Werkzeuge findet und aufruft, in Anfrage/Antwort, geführt von einer Person im Gespräch. Der Webhook bewegt das Event; MCP bedient das System, das es bewegt. Setups mit beidem lassen den Webhook-Weg unangetastet und nutzen MCP für die Betriebsarbeit."
      },
      {
        "q": "Sieht der Assistent meine Event-Payloads?",
        "a": "Er sieht, was die aufgerufenen Werkzeuge zurückgeben, und ein Event zu lesen gibt dessen Payload zurück. Hook0 reicht nichts an Dritte weiter: der MCP-Server ist ein lokaler Prozess, der direkt mit der Hook0-API spricht. Sollen Payloads ganz aus dem Gespräch bleiben, schwäch das Token auf Applications ab, die keine sensiblen Daten führen."
      },
      {
        "q": "Welche Assistenten funktionieren damit?",
        "a": "Claude Desktop, Cursor, Windsurf und Cline sind mit ihrer Konfigurationsdatei dokumentiert, und jeder MCP-fähige Client funktioniert genauso. ChatGPT unterstützt MCP derzeit nicht nativ."
      },
      {
        "q": "Was hindert einen Agenten daran, eine Produktions-Subscription zu löschen?",
        "a": "Zwei Dinge, und sie ergänzen sich. Der Nur-Lesen-Modus nimmt die Schreibwerkzeuge aus der Liste, die der Assistent überhaupt sieht. Die Token-Abschwächung begrenzt, was das Token selbst anfassen darf, durchgesetzt an der API, sodass ein Fehler im Client nicht darüber hinausgehen kann. Gelöschte Ressourcen sind nicht automatisch wiederherstellbar. Deshalb setzt man beides, bevor man einen Assistenten auf die Produktion richtet."
      },
      {
        "q": "Was passiert mit Events, wenn der Endpoint eines Abonnenten eine Weile ausfällt?",
        "a": "Hook0 wiederholt die Zustellung nach einem zweistufigen, konfigurierbaren Plan, statt sie beim ersten Fehlschlag zu verwerfen, und hält jeden Versuch mit seiner Antwort fest. Sobald der Empfänger repariert ist, sendest du das Fehlgeschlagene erneut, aus dem Dashboard, über die API oder indem du den Assistenten darum bittest. Das Event geht nicht verloren, während der Endpoint nicht erreichbar ist."
      },
      {
        "q": "Wie prüft die Empfängerseite ein von Hook0 gesendetes Payload?",
        "a": "Jeder Versuch trägt eine HMAC-SHA256-Signatur, berechnet aus dem Payload und dem Secret der Subscription. Der Empfänger berechnet sie nach und vergleicht, bevor er auf das Event reagiert. Das verhindert, dass eine gefälschte Anfrage einen Ablauf auslöst. Signaturverfahren und ein Prüf-Snippet stehen in der Hook0-Dokumentation."
      },
      {
        "q": "Funktioniert das mit selbst gehostetem Hook0?",
        "a": "Ja. Setz HOOK0_API_URL auf deine Instanz, dann verhalten sich die siebzehn Werkzeuge identisch. Das gesamte Produkt ist quelloffen (SSPL-1.0) ohne Open-Core-Rückhalt: die selbst gehostete Installation führt dieselbe Software aus wie die Cloud."
      },
      {
        "q": "Ist das ein Agent Skill oder ein Plugin?",
        "a": "Nein. Es ist ein MCP-Server, installiert mit cargo install hook0-mcp und eingetragen in der Konfiguration deines Assistenten. Er läuft standardmäßig über stdio, oder über SSE, wenn du ihn lieber als Dienst betreibst."
      },
      {
        "q": "Brauche ich den MCP-Server, um Webhooks zu senden?",
        "a": "Nein. Die REST-API und die SDKs bleiben der normale Weg, auf dem deine Anwendung Events sendet, und nichts davon ändert sich. Der MCP-Server richtet sich an die Person, die das Setup betreibt: einen Event-Typ anlegen, eine Subscription einrichten, herausfinden, warum eine Zustellung um vier Uhr nachmittags fehlschlug."
      }
    ]
  },
  "related": {
    "h2": "Weiterlesen",
    "links": [
      { "href": "/de/webhook-api", "label": "Webhook-API" },
      { "href": "/de/webhook-plattform", "label": "Webhook-Plattform" },
      { "href": "/de/selbst-gehostete-webhooks", "label": "Selbst gehostete Webhooks" },
      { "href": "/de/sicherheit", "label": "Sicherheit" },
      { "href": "https://documentation.hook0.com/reference/mcp", "label": "Dokumentation des MCP-Servers" }
    ]
  }
};
