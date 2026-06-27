// French locale.
//
// Per-page strings live in `locales/fr/<enSlug>.js` (keyed by EN slug for
// cross-locale lookup, even though the rendered URL uses the translated
// slug from `locales/slugs.js`). Each file MUST be passed through
// `/humanizer pro` + `legal-reviewer` before commit (legal: «code source
// ouvert (SSPL-1.0)», never «open source» — L121-1 risk on SSPL).

const fs = require('fs');
const path = require('path');

const pages = {};
const dir = path.join(__dirname, 'fr');
if (fs.existsSync(dir)) {
  for (const f of fs.readdirSync(dir)) {
    if (!f.endsWith('.js') || f.startsWith('_')) continue;
    const slug = f.replace(/\.js$/, '');
    pages[slug] = require(path.join(dir, f));
  }
}

module.exports = { lang: 'fr', pages };
