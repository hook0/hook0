// Per-page strings for webhook-playground (DE).
// /humanizer pro angewendet. Duzen. Kein Em-Dash, kein Pivot-Doppelpunkt.
// SSPL = « quelloffen (SSPL-1.0) » in pageDescription.
module.exports = {
  pageTitle: 'Kostenloser Webhook-Tester online | Hook0',
  pageDescription: 'Kostenloser Webhook-Tester ohne Anmeldung. Sende Test-Events, prüfe Payloads, verifiziere HMAC. Stripe, GitHub, Shopify. Quelloffen.',
  hero: {
    badge: 'Kostenlos, ohne Anmeldung',
    titleBefore: 'Teste deine Webhooks',
    titleAccent: 'in Sekunden',
    subtitle: 'Sende Events, prüfe Payloads, verifiziere HMAC-Signaturen und debugge die Zustellung, alles im Browser. Der schnellste Weg, deine Webhook-Integration zu testen.',
    ctaPrimary: 'Playground öffnen',
    ctaSecondary: 'Preise ansehen, kostenloser Tarif inklusive',
  },
  features: {
    eyebrow: 'Was du machen kannst',
    h2: 'Alles, was du zum Testen brauchst',
    cards: [
      { title: 'Test-Events senden', body: 'Schick Webhook-Events mit eigenen JSON-Payloads an jeden Endpoint. Sieh die Antwort in Echtzeit.' },
      { title: 'HMAC-Signaturen prüfen', body: 'Verifizier, dass dein Webhook-Empfänger HMAC-SHA256-Signaturen korrekt validiert und manipulierte Payloads ablehnt.' },
      { title: 'Payloads inspizieren', body: 'Sieh HTTP-Header, Request-Body, Status-Codes und Latenz für jeden Zustellversuch.' },
      { title: 'Wiederholungen testen', body: 'Simulier Endpoint-Ausfälle und beobachte die konfigurierbare Zwei-Phasen-Logik von Hook0 (schnelle und langsame Wiederholungen).' },
      { title: 'Code-Beispiele', body: 'Funktionierender Copy-Paste-Code für Python, Node.js, Go und Rust, Integration in Minuten.' },
      { title: 'Keine Anmeldung, keine Installation', body: 'Läuft sofort im Browser. Kein Account, kein CLI-Tool, kein Docker-Setup nötig zum Starten.' },
    ],
  },
  toProduction: {
    h2: 'Vom Test zur Produktion in 5 Minuten',
    subtitle: 'Wenn du bereit für Webhooks in Produktion bist, übernimmt Hook0 die Wiederholungsversuche, HMAC-Signaturen, Zustell-Monitoring und Multi-Tenant-Routing. Start mit dem kostenlosen Tarif, keine Kreditkarte nötig.',
    ctaPrimary: 'Playground ausprobieren',
    ctaSecondary: 'Preise ansehen',
  },
  faq: {
    items: [
      { q: 'Ist Hook0 Playground kostenlos?', a: 'Ja. Hook0 Playground ist komplett kostenlos und braucht keine Anmeldung. Du kannst Webhook-Events senden, Payloads prüfen, HMAC-Signaturen verifizieren und Zustellprobleme debuggen, alles im Browser.' },
      { q: 'Was kann ich mit dem Webhook-Playground testen?', a: 'Du kannst Test-Webhook-Events mit eigenen Payloads senden, HTTP-Header und Response-Codes prüfen, HMAC-SHA256-Signaturen verifizieren, Wiederholungsverhalten testen und Endpoint-Konnektivität debuggen. Alle Standard-Webhook-Patterns von Stripe, GitHub, Shopify und anderen Plattformen werden unterstützt.' },
      { q: 'Brauche ich einen Account?', a: 'Nein. Der Playground läuft sofort ohne Anmeldung. Wenn du deine Konfigurationen speichern oder mehr als 100 Events/Tag senden willst, kannst du einen kostenlosen Hook0-Account anlegen.' },
      { q: 'Gibt es einen kostenlosen Webhook-Dienst für die Produktion?', a: 'Ja. Hook0 Cloud skaliert vom kostenlosen Tarif bis zur Produktion, der Developer-Tarif enthält 100 Events/Tag, HMAC-Signaturen, Zustell-Monitoring und 7 Tage Datenaufbewahrung. Ohne Kreditkarte. Selbst-Hosting ist ebenfalls möglich für spezielle Infra-Anforderungen.' },
    ],
  },
};
