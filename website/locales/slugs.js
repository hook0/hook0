// URL slug map: English slug (source) -> translated slugs per locale.
//
// EN is the base language: every page renders at the site root with its English
// slug. A page is rendered for FR/DE ONLY if it has an entry here for that
// locale. Add a key when a page's data-driven template + localized strings are
// ready (see locales/<lang>.js). No hardcoded page list lives anywhere else:
// the build discovers templates from src/*.ejs and consults this map per locale.
module.exports = {
  'webhook-platform': { fr: 'plateforme-webhook', de: 'webhook-plattform' },
  'pricing': { fr: 'tarifs', de: 'preise' },
};
