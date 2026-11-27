// Per-page strings for open-source-webhooks (DE).
// /humanizer pro + legal-reviewer applied.
// Hook0 selbst = « Open Source (SSPL-1.0) », Klammerzusatz obligatorisch, NIE
// « Open Source » allein (SSPL nicht von der OSI anerkannt, UWG §5
// DACH-Risiko). Der Titel/Slug «open-source-webhooks»
// bleibt als KATEGORIE/SEO-Begriff für das Ökosystem (Svix open-core, Convoy
// Elastic License 2.0, Hook0 SSPL) erhalten, die Regel gilt für Claims über Hook0 selbst.
// Souveränität: CDN Cloudflare (USA) offen genannt, Anwendungsdaten bei
// Clever Cloud (Frankreich). NIE «kein US-Konzern im Stack /
// keine Daten verlassen die EU / 100 % souverän / CLOUD Act free».
// DSGVO als Prozess-Claim («auf DSGVO-Konformität ausgelegt»),
// keine absoluten Zertifizierungsaussagen.
module.exports = {
  "tldr": {
    "h2": "Welchen Open-Source-Webhook-Server solltest du wählen?",
    "body": "Nimm Hook0, wenn der Code, den du selbst hostest, derselbe sein soll wie in der gemanagten Cloud. Hook0 ist Open Source unter SSPL-1.0, läuft auf Docker oder Kubernetes mit allen Funktionen, und Hook0 Cloud wird ab 59 €/Monat in der EU gehostet. Die anderen Projekte unterscheiden sich vor allem bei der Lizenz und darin, wo das Kundenportal liegt. Svix hat einen MIT-Kern, sein App Portal ist für den gehosteten Dienst dokumentiert. Convoy steht unter der Elastic License 2.0 und packt Portal Links in einen kostenpflichtigen Self-Hosting-Tarif. Hookdeck Outpost (Apache-2.0) bringt ein Portal pro Tenant mit, braucht aber PostgreSQL, Redis und eine Message Queue. WebhookX (Apache-2.0) hält Multi-Tenancy hinter einer kommerziellen Lizenz und dokumentiert keine gemanagte Cloud."
  },
  "pageTitle": "Bester Open-Source-Webhook-Server (2026) | Hook0",
  "pageDescription": "Open-Source-Webhook-Server im Vergleich: Hook0 (SSPL-1.0), Svix, Convoy, Hookdeck Outpost und WebhookX. Lizenz, Stack, Portal, Cloud.",
  "pageModified": "2026-11-25",
  "track": "de-oss-webhooks",
  "hero": {
    "eyebrow": "Open Source",
    "titleLine1": "Bester Open Source",
    "titleLine2": "Webhook-Server",
    "subtitle": "Hook0 ist Open Source unter SSPL-1.0, prüfe jede Zeile, hoste selbst für deine Compliance oder nimm Hook0 Cloud für gemanagte Infrastruktur, automatische Updates und Anwendungs-Hosting in Frankreich (CDN Cloudflare in den USA offen genannt). Bootstrapped, ohne Open-Core-Tricks.",
    "ctaPrimary": "Kostenlos in der Cloud starten",
    "ctaPrimaryTrack": "de-oss-webhooks-hero-cloud-signup",
    "ctaSecondary": "Playground testen",
    "ctaSecondaryHref": "https://play.hook0.com",
    "ctaSecondaryTrack": "de-oss-webhooks-hero-playground",
    "trustIndicators": [
      "Open Source SSPL-1.0",
      "Selbst hosten möglich (Docker / K8s)",
      "Bootstrapped, ohne VC"
    ]
  },
  "socialProof": true,
  "whyOss": {
    "eyebrow": "Warum Open Source",
    "h2": "Warum dein Webhook-Server Open Source sein sollte",
    "cards": [
      {
        "icon": "audit",
        "title": "Jede Codezeile prüfen",
        "body": "Webhooks transportieren sensible Payloads. Mit offenem Quellcode prüft dein Security-Team genau, wie Daten verarbeitet, signiert und zugestellt werden. Keine Black Boxes."
      },
      {
        "icon": "lock",
        "title": "Kein Anbieter-Lock-in",
        "body": "Wenn der Anbieter verschwindet, die Preise anhebt oder pivotiert, hast du den Code weiterhin. Forke ihn, pflege ihn, oder migriere in deinem Tempo. Deine Webhook-Infrastruktur gehört dir."
      },
      {
        "icon": "selfhost",
        "title": "Überall selbst hosten",
        "body": "Deploye auf deinen Servern, in deiner Cloud oder in Air-gapped-Netzen. Open Source heißt, du entscheidest, wo deine Daten leben, nicht der Anbieter."
      },
      {
        "icon": "community",
        "title": "Community und Beiträge",
        "body": "Melde Bugs, schicke PRs, schlage Funktionen vor. Open-Source-Projekte richten die Interessen aus, das Produkt wird besser, weil die Nutzer es direkt formen."
      }
    ]
  },
  "projects": {
    "eyebrow": "Projekte",
    "h2": "Fünf Open-Source-Webhook-Server im Vergleich",
    "intro": "Jede Zeile stammt aus dem öffentlichen Repository, der Dokumentation oder der Preisseite des Projekts, geprüft am 24. und 25. November 2026. Die Quellen stehen am Ende dieser Seite.",
    "columns": {
      "project": "Projekt",
      "license": "Lizenz",
      "stack": "Stack",
      "selfHost": "Self-Hosting",
      "portal": "Kundenportal",
      "managed": "Gemanagte Cloud"
    },
    "rows": [
      {
        "project": "Hook0",
        "license": "SSPL-1.0",
        "stack": "Rust, PostgreSQL (Pulsar + S3 optional)",
        "selfHost": "Docker Compose, Kubernetes",
        "portal": "Abonnement-API und Dashboard",
        "managed": "Hook0 Cloud, in der EU gehostet, kostenloser Tarif, dann ab 59 €/Monat"
      },
      {
        "project": "Svix",
        "license": "MIT-Kern, geschlossene Enterprise-Funktionen",
        "stack": "Rust, PostgreSQL, Redis optional",
        "selfHost": "Docker Compose oder ein einzelner Container",
        "portal": "App Portal, für den gehosteten Dienst dokumentiert",
        "managed": "Svix Cloud, kostenlos bis 50.000 Nachrichten/Monat"
      },
      {
        "project": "Convoy",
        "license": "Elastic License 2.0",
        "stack": "Go, PostgreSQL, Redis",
        "selfHost": "Docker Compose, Helm",
        "portal": "Portal Links, im kostenpflichtigen Self-Hosting-Tarif Premium",
        "managed": "Convoy Cloud"
      },
      {
        "project": "Hookdeck Outpost",
        "license": "Apache-2.0",
        "stack": "Go, PostgreSQL, Redis, RabbitMQ oder SQS",
        "selfHost": "Docker Compose, Helm, Railway",
        "portal": "Portal pro Tenant enthalten",
        "managed": "Hookdeck, 10 $ pro Million Events, die ersten 50.000 kostenlos"
      },
      {
        "project": "WebhookX",
        "license": "Apache-2.0; Multi-Tenancy, Secret-Management und connect-auth brauchen eine kommerzielle Lizenz",
        "stack": "Go, PostgreSQL 13+, Redis 6.2+",
        "selfHost": "Docker Compose oder Binary",
        "portal": "Keines dokumentiert",
        "managed": "Keine dokumentiert"
      }
    ]
  },
  "comparison": {
    "eyebrow": "Lizenzen",
    "h2": "Webhook-Lizenzmodelle im Vergleich",
    "columns": {
      "criteria": "Kriterium",
      "sspl": "SSPL (Hook0)",
      "openCore": "Open Core (Svix)",
      "mit": "MIT (permissiv)",
      "proprietary": "Proprietär (geschlossenes SaaS)"
    },
    "rows": [
      {
        "criteria": "Quellcode verfügbar",
        "sspl": "Ja, die gesamte Codebasis auf GitHub und GitLab",
        "openCore": "Teilweise (nur Kern)",
        "mit": "Ja, auf GitHub",
        "proprietary": "Nein"
      },
      {
        "criteria": "Code prüfbar",
        "sspl": "Jede Zeile, inklusive Infrastruktur",
        "openCore": "Nur Kern, Enterprise geschlossen",
        "mit": "Ja",
        "proprietary": "Nein"
      },
      {
        "criteria": "Selbst hosten möglich",
        "sspl": "Ja, kostenlos (Docker / K8s)",
        "openCore": "Ja, MIT-Kernserver; On-Prem-Support in Enterprise",
        "mit": "Ja, kostenlos",
        "proprietary": "Nein"
      },
      {
        "criteria": "Funktionsparität (Cloud = Self-Host)",
        "sspl": "Gleiche Codebasis, alle Funktionen",
        "openCore": "Verschiedene Editionen, Funktionen abgeschottet",
        "mit": "Cloud ist ein eigenes Produkt",
        "proprietary": "Entfällt (nur Cloud)"
      },
      {
        "criteria": "Risiko von Anbieter-Lock-in",
        "sspl": "Gering, jederzeit forkbar, Standard-PostgreSQL",
        "openCore": "Mittel, Enterprise-Funktionen entfallen beim Wechsel",
        "mit": "Gering, MIT erlaubt Forks",
        "proprietary": "Hoch, kein Quellcode, kein Self-Host"
      },
      {
        "criteria": "Datensouveränität",
        "sspl": "Volle Kontrolle (Self-Host oder EU-Cloud)",
        "openCore": "US- oder EU-Cloud oder Self-Hosting des Kerns",
        "mit": "Nur Self-Host",
        "proprietary": "US-Cloud, keine Self-Host-Option"
      },
      {
        "criteria": "Community-Beiträge",
        "sspl": "PRs auf die gesamte Codebasis willkommen",
        "openCore": "PRs nur auf den Kern",
        "mit": "PRs willkommen",
        "proprietary": "Kein Community-Zugang"
      },
      {
        "criteria": "Lizenzbeschränkungen",
        "sspl": "Darf nicht als gemanagter Dienst weiterverkauft werden",
        "openCore": "Enterprise-Funktionen erfordern eine kostenpflichtige Lizenz",
        "mit": "Keine (permissiv)",
        "proprietary": "Jede Nutzung unterliegt den Anbieterbedingungen"
      }
    ]
  },
  "differentiators": {
    "eyebrow": "Hook0-Unterschied",
    "h2": "Was Hook0 abhebt",
    "cards": [
      {
        "icon": "audit",
        "title": "Jede Zeile prüfen",
        "body": "Webhooks transportieren sensible Payloads. Deine Security- und Compliance-Teams prüfen die gesamte Codebasis, API, Worker, Datenbankschema, vor dem Produktiveinsatz. Keine geschlossenen Black Boxes."
      },
      {
        "icon": "lock",
        "title": "Kein Anbieter-Lock-in",
        "body": "Migriere jederzeit. Keine proprietären APIs, keine proprietären Datenformate. Hook0 speichert alles in Standard-PostgreSQL. Wenn du wechselst, gehen deine Daten und das Wissen über deine Infrastruktur mit dir."
      },
      {
        "icon": "cloud",
        "title": "Cloud, wenn du sie willst",
        "body": "Starte mit Hook0 Cloud für den schnellsten Weg in die Produktion. Wechsle später auf Self-Host für Compliance oder Datensouveränität, oder umgekehrt. Gleiche Codebasis, kein Migrationsaufwand."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Häufige Fragen",
    "items": [
      {
        "q": "Ist Hook0 Open Source?",
        "a": "Ja. Hook0 ist Open Source unter der SSPL-1.0-Lizenz. Jede Codezeile liegt auf GitHub und GitLab. Es gibt keine proprietäre Enterprise-Edition."
      },
      {
        "q": "Welche Lizenz nutzt Hook0?",
        "a": "SSPL-1.0 (Server Side Public License). Du kannst den Code frei selbst hosten, anpassen und prüfen. Die einzige Einschränkung ist, Hook0 als gemanagten Dienst an Dritte anzubieten, ohne deine eigene Stack Open Source zu machen."
      },
      {
        "q": "Was braucht das Selbst-Hosten von Hook0?",
        "a": "Das Selbst-Hosten von Hook0 verlangt Docker Compose oder Kubernetes und eine PostgreSQL-Datenbank. Du verantwortest deine Infrastruktur, Skalierung, Backups, Updates und Monitoring. Das selbst gehostete Binary stammt aus derselben Codebasis wie Hook0 Cloud, keine Funktionen werden entfernt. Hook0 Cloud erledigt all das für dich, wenn du den gemanagten Weg bevorzugst."
      },
      {
        "q": "Welche Risiken haben Open-Core-Webhook-Tools?",
        "a": "Open-Core-Webhook-Tools spalten ihre Codebasis in eine kostenlose Community-Edition und eine kostenpflichtige Enterprise-Edition. Das Risiko: Funktionen, auf die du dich heute verlässt (SSO, erweitertes Monitoring, Self-Host-Support), können jederzeit hinter die Paywall verschoben werden. Die geschlossenen Teile lassen sich nicht auf Sicherheit prüfen. Und wenn du selbst hostest, läuft eine abgespeckte Version. Hook0 vermeidet das, die gesamte Codebasis ist unter SSPL verfügbar, ohne Enterprise-Edition."
      },
      {
        "q": "Ist Hook0 wirklich kostenlos selbst hostbar?",
        "a": "Ja. Hook0 ist Open Source und ohne Lizenzkosten selbst hostbar. Hook0 Cloud ergänzt gemanagte Infrastruktur, automatische Updates, EU-Hosting, vorrangigen Support und ein SLA, damit du dich auf dein Produkt konzentrierst statt auf den Betrieb einer Webhook-Infrastruktur. Starte im kostenlosen Cloud-Tarif (100 Events/Tag, keine Kreditkarte)."
      },
      { "q": "Gibt es eine Open-Source-Webhook-Plattform, die auch in der EU gehostet wird?", "a": "Ja. Hook0 ist Open Source (SSPL-1.0), und Hook0 Cloud betreibt seine Datenebene in jedem Tarif auf Clever Cloud in Frankreich (innerhalb der EU). Viele in der EU gehostete Webhook-Dienste sind proprietär und rein cloudbasiert, sodass Sie den Code weder lesen noch selbst betreiben können. Mit Hook0 können Sie den Code prüfen, ihn selbst hosten oder die EU-Cloud nutzen; das vorgelagerte CDN der Cloud ist Cloudflare (US), offengelegt in unserer öffentlichen Unterauftragsverarbeiter-Liste." },
      {
        "q": "Ist WebhookX eine gute Open-Source-Alternative zum Versenden von Webhooks?",
        "a": "WebhookX ist ein Webhook-Gateway in Go unter Apache-2.0, das Events empfängt, validiert, transformiert und zustellt. Es braucht PostgreSQL 13+ und Redis 6.2+, die neueste Version am 25. November 2026 ist v1.2.0. Multi-Tenancy, Secret-Management und das connect-auth-Plugin erfordern eine kommerzielle Lizenz, und die README dokumentiert weder eine gemanagte Cloud noch ein Kundenportal. Wenn du Webhooks an viele Kunden sendest und Tenants ohne Zusatzlizenz trennen willst, ist das bei Hook0 im SSPL-1.0-Code enthalten."
      },
      {
        "q": "Was ist Hookdeck Outpost, und wie schlägt es sich gegen Hook0?",
        "a": "Outpost ist die Zustell-Engine von Hookdeck für ausgehende Events: Go, Apache-2.0, mandantenfähig, mit einem Portal pro Tenant. Selbst gehostet läuft es auf PostgreSQL, Redis und einer RabbitMQ- oder SQS-Queue; gemanagt berechnet Hookdeck 10 $ pro Million Events nach den ersten 50.000. Hook0 läuft in den meisten Setups allein auf PostgreSQL, Pulsar und S3 kommen nur für hohen Durchsatz dazu, und die gemanagte Cloud wird in der EU gehostet."
      }
    ]
  },
  "comparisonSources": {
    "competitors": [
      "Svix",
      "Convoy",
      "Hookdeck Outpost",
      "WebhookX"
    ],
    "scope": "Diese Seite vergleicht Open-Source- und Source-Available-Webhook-Server nach Lizenz, Stack, Self-Hosting, Kundenportal und gemanagtem Angebot. Die Fakten stammen aus dem öffentlichen Repository, der Dokumentation und der Preisseite jedes Projekts.",
    "sources": [
      {
        "label": "Svix: Quellcode, Lizenz und Laufzeitabhängigkeiten auf GitHub",
        "url": "https://github.com/svix/svix-webhooks",
        "consulted": "2026-11-25"
      },
      {
        "label": "Svix: Dokumentation des App Portal",
        "url": "https://docs.svix.com/app-portal",
        "consulted": "2026-11-25"
      },
      {
        "label": "Svix: Preise",
        "url": "https://www.svix.com/pricing/",
        "consulted": "2026-11-24"
      },
      {
        "label": "Convoy: Quellcode und Lizenz auf GitHub",
        "url": "https://github.com/frain-dev/convoy",
        "consulted": "2026-11-25"
      },
      {
        "label": "Convoy: Kubernetes-Installationsanleitung",
        "url": "https://getconvoy.io/docs/deployment/install-convoy/kubernetes",
        "consulted": "2026-11-25"
      },
      {
        "label": "Convoy: Preise",
        "url": "https://www.getconvoy.io/pricing",
        "consulted": "2026-11-25"
      },
      {
        "label": "Hookdeck Outpost: Quellcode und Lizenz auf GitHub",
        "url": "https://github.com/hookdeck/outpost",
        "consulted": "2026-11-25"
      },
      {
        "label": "Hookdeck Outpost: Preise",
        "url": "https://hookdeck.com/outpost/pricing",
        "consulted": "2026-11-25"
      },
      {
        "label": "WebhookX: Quellcode und Lizenz auf GitHub",
        "url": "https://github.com/webhookx-io/webhookx",
        "consulted": "2026-11-25"
      },
      {
        "label": "WebhookX: Release Notes v1.2.0",
        "url": "https://github.com/webhookx-io/webhookx/releases/tag/v1.2.0",
        "consulted": "2026-11-25"
      }
    ],
    "updated": "2026-11-25"
  },
  "related": {
    "h2": "Weiterführend (auf Englisch)",
    "links": [
      { "label": "Self-Hosted Webhooks", "href": "/self-hosted-webhooks" },
      { "label": "Hook0 vs Svix", "href": "/hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "/hook0-vs-hookdeck" },
      { "label": "Hook0 vs Convoy", "href": "/hook0-vs-convoy" },
      { "label": "Build vs Buy Webhooks", "href": "/build-vs-buy-webhooks" },
      { "label": "Hook0 Alternatives", "href": "/hook0-alternatives" }
    ]
  }
};
