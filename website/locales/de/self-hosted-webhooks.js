// Per-page strings for self-hosted-webhooks (DE).
// /humanizer pro + legal-reviewer applied.
// Hook0 selbst = « quelloffen (SSPL-1.0) », NIE « Open Source » (SSPL von der
// OSI abgelehnt, UWG §5 DACH-Risiko).
// Souveränität: CDN Cloudflare (USA) offen genannt, Anwendungsdaten bei
// Clever Cloud (Frankreich). NIE «kein US-Konzern im Stack /
// keine Daten verlassen die EU / 100 % souverän / CLOUD Act free».
// DSGVO als Prozess-Claim («auf DSGVO-Konformität ausgelegt»),
// keine absoluten Zertifizierungsaussagen.
module.exports = {
  "pageTitle": "Selbst gehostete Webhooks Docker und Kubernetes | Hook0",
  "pageDescription": "Hoste Hook0 auf deiner Infrastruktur mit Docker oder Kubernetes. Quelloffen unter SSPL-1.0, gleiche Codebasis wie die Cloud. Deine Webhook-Payloads bleiben in deinem Netz.",
  "pageModified": "2026-06-27",
  "track": "de-self-hosted",
  "hero": {
    "eyebrow": "Selbst gehostet",
    "titleLine1": "Selbst gehostete",
    "titleLine2": "Webhook-Plattform",
    "subtitle": "Hoste deine Webhooks selbst, mit derselben Codebasis wie unsere Cloud. Deine Webhook-Payloads verlassen dein Netz nie. Docker Compose oder Kubernetes. Quelloffen unter SSPL-1.0, kein Anbieter-Lock-in.",
    "ctaPrimary": "Kostenlos starten",
    "ctaPrimaryTrack": "de-self-hosted-hero-register",
    "ctaSecondary": "Installations-Guide",
    "ctaSecondaryHref": "https://documentation.hook0.com/self-hosting/docker-compose",
    "ctaSecondaryTrack": "de-self-hosted-hero-docs",
    "trustIndicators": [
      "Gleiche Codebasis wie die Cloud",
      "SSPL-1.0-Lizenz",
      "Keine Telemetrie"
    ]
  },
  "socialProof": true,
  "whySelfHost": {
    "eyebrow": "Warum selbst hosten",
    "h2": "Deine Daten, deine Infrastruktur",
    "cards": [
      {
        "icon": "shield",
        "title": "Datensouveränität",
        "body": "On-Premise-Webhooks, bei denen Payloads in deinem Perimeter bleiben. Punkt. Kein Dritter sieht deine Daten. CISO-freundlich für Gesundheitswesen, Finanzen, Behörden, und auf DSGVO-Konformität ausgelegt."
      },
      {
        "icon": "code",
        "title": "Quelloffen unter SSPL-1.0",
        "body": "Unter SSPL-1.0 lizenziert. Keine Open-Core-Tricks, keine abgeschotteten Funktionen. Jede Codezeile liegt auf GitHub und GitLab. Du kannst sie prüfen, forken oder eine PR schicken."
      },
      {
        "icon": "server",
        "title": "Docker und Kubernetes",
        "body": "Docker Compose für Entwicklung und kleine Deployments. Helm-Chart für produktive Kubernetes-Cluster. Beides läuft sofort."
      },
      {
        "icon": "sync",
        "title": "Gleicher Code, gleiche Funktionen",
        "body": "Eine Codebasis. Das Binary, das du deployst, stammt aus demselben Repo wie unsere Cloud. Wiederholungsversuche, Signaturen, Monitoring, Subscription-Verwaltung, nichts wird entfernt."
      }
    ]
  },
  "deployment": {
    "eyebrow": "Deployment",
    "h2": "Zwei Wege zu deployen",
    "options": [
      {
        "kind": "docker",
        "title": "Docker Compose",
        "body": "Gut für Entwicklung, Tests und kleine Produktivumgebungen. Drei Befehle, alles startet.",
        "code": "git clone https://github.com/hook0/hook0.git<br>cd hook0<br>docker compose up -d",
        "docsHref": "https://documentation.hook0.com/self-hosting/docker-compose",
        "docsLabel": "Docker-Compose-Guide",
        "docsTrack": "de-self-hosted-docker-docs"
      },
      {
        "kind": "kubernetes",
        "title": "Kubernetes",
        "body": "Für die Produktion. Horizontale Skalierung, Health-Checks, Rolling-Updates über Helm.",
        "code": "helm repo add hook0 https://charts.hook0.com<br>helm install hook0 hook0/hook0",
        "docsHref": "https://documentation.hook0.com/self-hosting/kubernetes",
        "docsLabel": "Kubernetes-Guide",
        "docsTrack": "de-self-hosted-k8s-docs"
      }
    ]
  },
  "whoSelfHosts": {
    "eyebrow": "Anwendungsfälle",
    "h2": "Wer hostet Hook0 selbst?",
    "cards": [
      {
        "icon": "industry",
        "title": "Regulierte Branchen",
        "body": "Gesundheitswesen, Finanzen, Behörden. Wenn dein Compliance-Team sagt « kein externes SaaS für diese Daten », brauchst du trotzdem Webhooks."
      },
      {
        "icon": "globe",
        "title": "Datensouveränität",
        "body": "Europäische Unternehmen unter DSGVO, oder alle, die genau belegen müssen, wo Daten verarbeitet und gespeichert werden."
      },
      {
        "icon": "lock",
        "title": "Air-gapped-Netze",
        "body": "Kein Internet? Kein Problem. Hook0 hat null Phone-Home, null Telemetrie, null externe Abhängigkeiten."
      }
    ]
  },
  "faq": {
    "eyebrow": "FAQ",
    "h2": "Häufige Fragen",
    "items": [
      {
        "q": "Ist die selbst gehostete Version identisch mit der Cloud-Version?",
        "a": "Ja. Eine Codebasis, keine « Community-Edition ». Was auf unserer Cloud läuft, deployst du auf deiner."
      },
      {
        "q": "Welche Infrastruktur brauche ich?",
        "a": "Docker Compose für einfache Setups, Kubernetes (Helm) für die Produktion. PostgreSQL zur Speicherung. Ein Knoten schafft Tausende Events pro Minute."
      },
      {
        "q": "Verlassen meine Daten mein Netz?",
        "a": "Nein. Alles bleibt auf deiner Infrastruktur. Keine Telemetrie, kein Phone-Home, keine externen Calls. Wenn du den gemanagten Weg bevorzugst, läuft Hook0 Cloud auf französischer Infrastruktur (Clever Cloud), deine Daten bleiben in der EU und ausserhalb des US-CLOUD-Acts."
      },
      {
        "q": "Gibt es Support für selbst gehostete Deployments?",
        "a": "Ja. Der kommerzielle Support deckt Installationshilfe, Konfigurations-Review und vorrangige Bugfixes ab."
      },
      {
        "q": "Kann ich Hook0 vorher testen?",
        "a": "Ja. Unsere Cloud bietet einen kostenlosen Tarif mit 100 Events pro Tag, ohne Kreditkarte. Teste sie, dann hoste on-premise, wenn du bereit bist."
      }
    ]
  },
  "deepDive": {
    "prefix": "Mehr Details gewünscht?",
    "linkLabel": "Lies den kompletten Self-Hosting-Guide in der Doku",
    "linkHref": "https://documentation.hook0.com/self-hosting/docker-compose",
    "suffix": "."
  },
  "related": {
    "h2": "Weiterführend (auf Englisch)",
    "links": [
      { "enSlug": "hook0-vs-svix", "label": "Hook0 vs Svix" },
      { "enSlug": "hook0-vs-hookdeck", "label": "Hook0 vs Hookdeck" },
      { "enSlug": "build-vs-buy-webhooks", "label": "Build vs Buy Webhooks" }
    ]
  }
};
