// Per-page strings for pricing (DE).
// /humanizer pro + legal-reviewer applied. DSGVO als Prozess-Claim; SSPL = quelloffen.
// Verbotene Strings (CLAUDE.local.md): «100% souverän», «kein US-Konzern im
// Stack», «keine Daten verlassen die EU», «CLOUD Act free». NIS2/DORA nur als
// Kunden-Kontext, niemals als Zertifizierung.
module.exports = {
  pageTitle: 'Hook0 Preise: kostenloser Tarif, EU-Cloud | Webhooks',
  pageDescription: 'Developer-Tarif dauerhaft kostenlos. Cloud ab 59 €/Monat, quelloffen (SSPL-1.0), selbst hostbar. Keine versteckten Kosten.',
  "pageModified": "2026-06-27",
  "track": "de-preise",
  "hero": {
    "h1": "Hook0 Preise",
    "tagline": "Wähl den Tarif, der zu deinem Team passt. Kostenlos starten, in Produktion skalieren."
  },
  "differentiators": {
    "h2": "Warum diese Preisstruktur anders ist",
    "cards": [
      {
        "title": "Bootstrapped, kein VC",
        "body": "Kein Druck, die Preise zu erhöhen. Wir wachsen mit dir, niemals gegen dich."
      },
      {
        "title": "Quelloffen, kein Lock-in",
        "body": "Auditiere jede Zeile Code. Self-Hosting für Compliance. Starte mit Cloud für den schnellsten Weg in die Produktion."
      },
      {
        "title": "Keine versteckten Kosten",
        "body": "Wiederholungsversuche sind kostenlos. HMAC-Signaturen inklusive. Keine Kosten pro Endpoint. Aufpreise stehen auf jedem Tarif."
      }
    ]
  },
  "faq": {
    "h2": "Preis-FAQ",
    "items": [
      {
        "q": "Was passiert, wenn ich mein tägliches Event-Limit überschreite?",
        "a": "Im kostenlosen Developer-Tarif werden zusätzliche Events blockiert (HTTP 429). In den bezahlten Tarifen (Startup und Pro) werden zusätzliche Events <strong>niemals blockiert</strong>. Sie werden pro Event abgerechnet (0,003 € pro Event in Startup, 0,0001 € pro Event in Pro). Wir haben uns gegen die Unterbrechung entschieden, damit Kunden, die Produkte auf Hook0 bauen, keine Probleme bekommen."
      },
      {
        "q": "Wie überwache ich meinen Verbrauch?",
        "a": "Das Organization-Dashboard in der Hook0-App zeigt deinen Event-Verbrauch für den aktuellen Tag und die vergangenen Tage. Für Abrechnungsdetails und die Rechnungs-Historie öffne dein Stripe-Billing-Portal."
      },
      {
        "q": "Ist Hook0 kostenlos?",
        "a": "Ja. Hook0 hat einen kostenlosen Developer-Tarif mit 100 Webhook-Events pro Tag, HMAC-Signaturen und Zustell-Monitoring. Keine Kreditkarte nötig. Hook0 ist außerdem quelloffen und selbst hostbar, falls du Datensouveränität oder spezifische Infrastruktur-Anforderungen hast. Hook0 Cloud läuft auf <a href=\"/eu-webhook-infrastructure\">EU-Webhook-Infrastruktur</a> (Details auf Englisch), standardmäßig in Frankreich gehostet."
      },
      {
        "q": "Kann ich Hook0 kostenlos selbst hosten?",
        "a": "Ja. Hook0 ist vollständig quelloffen unter der SSPL-1.0-Lizenz. Du kannst es per Docker Compose oder Kubernetes selbst hosten. Beim Self-Hosting verwaltest du deine eigene Infrastruktur, dein Scaling, deine Updates und dein Monitoring. Die meisten Teams starten mit Hook0 Cloud für den schnellsten Weg in die Produktion."
      },
      {
        "q": "Wie vergleicht sich der Hook0-Preis mit Svix und Hookdeck?",
        "a": "Hook0 Cloud startet bei 59 € netto/Monat gegenüber Svix bei 490 $/Monat für vergleichbare Funktionen. Svix versteckt Self-Hosting hinter Enterprise-Preisen. Hookdeck hat keine selbst gehostete Option. Hook0 ist außerdem vollständig quelloffen unter SSPL, du kannst also selbst hosten, wenn du Datensouveränität brauchst. Detaillierte Zahlen von 100k bis 10M Events pro Monat findest du im <a href=\"/webhook-cost-comparison\">Webhook-Kostenvergleich</a> (auf Englisch)."
      }
    ]
  }
};
