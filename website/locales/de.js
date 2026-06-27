// German locale.
//
// Per-page strings live in `locales/de/<enSlug>.js` (keyed by EN slug for
// cross-locale lookup, even though the rendered URL uses the translated
// slug from `locales/slugs.js`). Each file MUST be passed through
// `/humanizer pro` + `legal-reviewer` before commit. Hard constraints
// (CLAUDE.local.md): SSPL = «quelloffen» (never «Open Source»); DSGVO as
// process claim only («auf DSGVO-Konformität ausgelegt»); forbidden:
// «100% souverän», «kein US-Konzern im Stack», «keine Daten verlassen die EU»,
// «CLOUD Act free»; NIS2/DORA only as client context («unterstützt deine
// Anforderungen»), never certified.

const fs = require('fs');
const path = require('path');

const pages = {};
const dir = path.join(__dirname, 'de');
if (fs.existsSync(dir)) {
  for (const f of fs.readdirSync(dir)) {
    if (!f.endsWith('.js') || f.startsWith('_')) continue;
    const slug = f.replace(/\.js$/, '');
    pages[slug] = require(path.join(dir, f));
  }
}

module.exports = { lang: 'de', pages };
