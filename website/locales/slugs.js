// URL slug map: English slug (source) -> translated slugs per locale.
//
// EN is the base language: every page renders at the site root with its English
// slug. A page is rendered for FR/DE ONLY if it has an entry here for that
// locale. Add a key when a page's data-driven template + localized strings are
// ready (see locales/<lang>.js). No hardcoded page list lives anywhere else:
// the build discovers templates from src/*.ejs and consults this map per locale.
module.exports = {
  // Homepage. Special-cased by getPath (lang root → `/`, `/fr/`, `/de/`) and
  // by the orchestrator's URL/hreflang builders (stem === 'index'). The slug
  // value stays 'index' for FR/DE so the orchestrator keeps the file name as
  // `index.ejs` and Parcel emits `dist/<lang>/index.html`.
  'index': { fr: 'index', de: 'index' },
  'webhook-platform': { fr: 'plateforme-webhook', de: 'webhook-plattform' },
  'pricing': { fr: 'tarifs', de: 'preise' },
  'webhook-api': { fr: 'api-webhook', de: 'webhook-api' },
  'oss-friends': { fr: 'amis-open-source', de: 'open-source-freunde' },
  'security': { fr: 'securite', de: 'sicherheit' },
  'webhook-playground': { fr: 'testeur-webhook', de: 'webhook-tester' },
  'built-to-last': { fr: 'construit-pour-durer', de: 'gebaut-um-zu-bleiben' },
  'migrate-from-webhook-site': { fr: 'migrer-depuis-webhook-site', de: 'von-webhook-site-migrieren' },
  'open-source-webhooks': { fr: 'webhooks-open-source', de: 'quelloffene-webhooks' },
  'hook0-vs-convoy': { fr: 'hook0-vs-convoy', de: 'hook0-vs-convoy' },
  'self-hosted-webhooks': { fr: 'webhooks-auto-heberges', de: 'selbst-gehostete-webhooks' },
  'build-vs-buy-webhooks': { fr: 'build-vs-buy-webhooks', de: 'build-vs-buy-webhooks' },
  'hook0-vs-svix': { fr: 'hook0-vs-svix', de: 'hook0-vs-svix' },
  'svix-alternatives': { fr: 'alternatives-a-svix', de: 'svix-alternativen' },
  'hook0-vs-hookdeck': { fr: 'hook0-vs-hookdeck', de: 'hook0-vs-hookdeck' },
  // EN-only pilot: per-provider "<Provider> Webhook Tester" pages (WR2). No fr/de
  // variants — the empty map renders only the English page. Localize only if these
  // rank (test-and-scale).
  'stripe-webhook-tester': {},
  'github-webhook-tester': {},
  'shopify-webhook-tester': {},
  // EN + FR only (DE gate: no second DE cluster until paid DE proves demand).
  'webhook-cost-comparison': { fr: 'comparatif-cout-webhook' },
  'eu-webhook-infrastructure': { fr: 'infrastructure-webhook-europeenne' },
  'hookdeck-alternatives': { fr: 'alternatives-a-hookdeck', de: 'hookdeck-alternativen' },
  'hook0-alternatives': { fr: 'alternatives-a-hook0', de: 'hook0-alternativen' },
  'mediakit': { fr: 'kit-presse', de: 'pressekit' },
  'mentions-legales': { fr: 'mentions-legales', de: 'impressum' },
  'terms': { fr: 'conditions-utilisation', de: 'nutzungsbedingungen' },
  'gdpr-subprocessors': { fr: 'sous-traitants-rgpd', de: 'dsgvo-unterauftragsverarbeiter' },
  'terms-of-sale': { fr: 'conditions-vente', de: 'verkaufsbedingungen' },
  'data-processing-addendum': { fr: 'accord-traitement-donnees', de: 'auftragsverarbeitungsvertrag' },
  'privacy-policy': { fr: 'politique-confidentialite', de: 'datenschutzerklaerung' },
};
