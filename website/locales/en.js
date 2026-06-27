// English (base language) locale.
//
// Per-page strings live in `locales/en/<enSlug>.js` so multiple agents can
// edit different pages without write-conflict (Phase-1 mass extraction).
// EN is the live, ranking copy: page strings are VERBATIM extractions from
// the live src/*.ejs templates and must stay byte-identical to what ships
// today (the EN-identity build gate enforces it). EN is therefore NOT
// humanized — the /humanizer pro pass applies to FR/DE only.

const fs = require('fs');
const path = require('path');

const pages = {};
const dir = path.join(__dirname, 'en');
if (fs.existsSync(dir)) {
  for (const f of fs.readdirSync(dir)) {
    if (!f.endsWith('.js') || f.startsWith('_')) continue;
    const slug = f.replace(/\.js$/, '');
    pages[slug] = require(path.join(dir, f));
  }
}

module.exports = { lang: 'en', pages };
