// Per-page strings for open-source-webhooks (DE).
// /humanizer pro + legal-reviewer applied.
// Hook0 selbst = « quelloffen (SSPL-1.0) », NIE « Open Source » (SSPL von der
// OSI abgelehnt, UWG §5 DACH-Risiko). Der Titel/Slug «open-source-webhooks»
// bleibt als KATEGORIE/SEO-Begriff für das Ökosystem (Svix open-core, Convoy
// MIT, Hook0 SSPL) erhalten, die Regel gilt für Claims über Hook0 selbst.
// Souveränität: CDN Cloudflare (USA) offen genannt, Anwendungsdaten bei
// Clever Cloud (Frankreich). NIE «kein US-Konzern im Stack /
// keine Daten verlassen die EU / 100 % souverän / CLOUD Act free».
// DSGVO als Prozess-Claim («auf DSGVO-Konformität ausgelegt»),
// keine absoluten Zertifizierungsaussagen.
module.exports = {
  "pageTitle": "Bester quelloffener Webhook-Server (2026) | Hook0",
  "pageDescription": "Quelloffene Webhook-Server im Vergleich: Hook0 (SSPL, voll funktional), Svix (Open Core), Convoy (MIT). Cloud ab 59 €/Monat oder selbst gehostet für Compliance.",
  "pageModified": "2026-07-16",
  "track": "de-oss-webhooks",
  "hero": {
    "eyebrow": "Quelloffen",
    "titleLine1": "Bester quelloffener",
    "titleLine2": "Webhook-Server",
    "subtitle": "Hook0 ist vollständig quelloffen unter SSPL-1.0, prüfe jede Zeile, hoste selbst für deine Compliance oder nimm Hook0 Cloud für gemanagte Infrastruktur, automatische Updates und Anwendungs-Hosting in Frankreich (CDN Cloudflare in den USA offen genannt). Bootstrapped, ohne Open-Core-Tricks.",
    "ctaPrimary": "Kostenlos in der Cloud starten",
    "ctaPrimaryTrack": "de-oss-webhooks-hero-cloud-signup",
    "ctaSecondary": "Playground testen",
    "ctaSecondaryHref": "https://play.hook0.com",
    "ctaSecondaryTrack": "de-oss-webhooks-hero-playground",
    "trustIndicators": [
      "Quelloffen SSPL-1.0",
      "Selbst hosten möglich (Docker / K8s)",
      "Bootstrapped, ohne VC"
    ]
  },
  "socialProof": true,
  "whyOss": {
    "eyebrow": "Warum quelloffen",
    "h2": "Warum dein Webhook-Server quelloffen sein sollte",
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
        "body": "Deploye auf deinen Servern, in deiner Cloud oder in Air-gapped-Netzen. Quelloffen heißt, du entscheidest, wo deine Daten leben, nicht der Anbieter."
      },
      {
        "icon": "community",
        "title": "Community und Beiträge",
        "body": "Melde Bugs, schicke PRs, schlage Funktionen vor. Quelloffene Projekte richten die Interessen aus, das Produkt wird besser, weil die Nutzer es direkt formen."
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
      "mit": "MIT (Convoy)",
      "proprietary": "Proprietär (Hookdeck)"
    },
    "rows": [
      {
        "criteria": "Quellcode verfügbar",
        "sspl": "Ja, 100 % auf GitHub und GitLab",
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
        "openCore": "Nur im Enterprise-Plan",
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
        "openCore": "US-Cloud oder Enterprise-Self-Host",
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
        "q": "Ist Hook0 quelloffen?",
        "a": "Ja. Hook0 ist vollständig quelloffen unter der SSPL-1.0-Lizenz. Jede Codezeile liegt auf GitHub und GitLab. Es gibt keine proprietäre Enterprise-Edition."
      },
      {
        "q": "Welche Lizenz nutzt Hook0?",
        "a": "SSPL-1.0 (Server Side Public License). Du kannst den Code frei selbst hosten, anpassen und prüfen. Die einzige Einschränkung ist, Hook0 als gemanagten Dienst an Dritte anzubieten, ohne deine eigene Stack quelloffen zu machen."
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
        "a": "Ja. Hook0 ist quelloffen und ohne Lizenzkosten selbst hostbar. Hook0 Cloud ergänzt gemanagte Infrastruktur, automatische Updates, EU-Hosting, vorrangigen Support und ein SLA, damit du dich auf dein Produkt konzentrierst statt auf den Betrieb einer Webhook-Infrastruktur. Starte im kostenlosen Cloud-Tarif (100 Events/Tag, keine Kreditkarte)."
      },
      { "q": "Gibt es eine quelloffene Webhook-Plattform, die auch in der EU gehostet wird?", "a": "Ja. Hook0 ist quelloffen (SSPL-1.0), und Hook0 Cloud betreibt seine Datenebene in jedem Tarif auf Clever Cloud in Frankreich (innerhalb der EU). Viele in der EU gehostete Webhook-Dienste sind proprietär und rein cloudbasiert, sodass Sie den Code weder lesen noch selbst betreiben können. Mit Hook0 können Sie den Code prüfen, ihn selbst hosten oder die EU-Cloud nutzen; das vorgelagerte CDN der Cloud ist Cloudflare (US), offengelegt in unserer öffentlichen Unterauftragsverarbeiter-Liste." }
    ]
  },
  "related": {
    "h2": "Weiterführend (auf Englisch)",
    "links": [
      { "label": "Self-Hosted Webhooks", "href": "/self-hosted-webhooks" },
      { "label": "Hook0 vs Svix", "href": "/hook0-vs-svix" },
      { "label": "Hook0 vs Hookdeck", "href": "/hook0-vs-hookdeck" },
      { "label": "Build vs Buy Webhooks", "href": "/build-vs-buy-webhooks" },
      { "label": "Hook0 Alternatives", "href": "/hook0-alternatives" }
    ]
  }
};
