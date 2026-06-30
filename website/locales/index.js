// i18n engine helpers — pure, side-effect-free. Consumed by:
//  - the Rust orchestrator (scripts/build-i18n) via `configLines`/`slugs`
//  - per-locale `.ejsrc.js` (regenerated per build) via `getLocale`
//  - the data-driven page templates via `getPageLocals`/`getPath`/`getAlternates`
//
// EN is the base language and renders at the site root. FR/DE render under
// /fr/ and /de/ with translated slugs (see slugs.js). This module is plain JS
// to match the existing build glue (data.js, scripts/fix-sitemap.js).

const slugs = require('./slugs');

// Locale routing config. `dir` is the dist subdirectory ('' = site root for EN).
// SITE_URL is absolute and drives only content URLs (canonical, hreflang,
// og:url, JSON-LD @id, sitemap entries). It is overridable via LOCAL_PREVIEW_URL
// (e.g. http://localhost:4000) for the local-preview workflow.
//
// publicUrl is host-relative on purpose: Parcel receives it as `--public-url`,
// so every asset href Parcel emits (CSS bundle, hashed favicons, JS) resolves
// from the host that serves the HTML — the same dist/ works on www.hook0.com,
// on a Netlify preview origin (deploy-id--site.netlify.app), and on localhost.
// Per-locale prefix keeps the FR/DE subdir asset paths correct (assets for FR
// live under dist/fr/ → /fr/*.css).
const SITE_URL = process.env.LOCAL_PREVIEW_URL || 'https://www.hook0.com';
const LOCALES = [
  { lang: 'en', dir: '',   publicUrl: '/',    ejsLocale: 'en_US' },
  { lang: 'fr', dir: 'fr', publicUrl: '/fr',  ejsLocale: 'fr_FR' },
  { lang: 'de', dir: 'de', publicUrl: '/de',  ejsLocale: 'de_DE' },
];
const BASE_LANG = 'en';

// Locale string overlay. EN's prose lives in data.js + en.js; the base lang
// returns an empty overlay so EN output stays byte-identical until extracted.
function getLocale(lang) {
  if (lang === BASE_LANG) {
    try { return require('./en'); } catch (e) { return {}; }
  }
  try { return require('./' + lang); } catch (e) { return {}; }
}

// Translated slug for a page in a locale; '' (falsy → caller skips) when the
// page is not localized for that locale. EN always returns its own slug.
function getSlug(enSlug, lang) {
  if (lang === BASE_LANG) return enSlug;
  const m = slugs[enSlug];
  return (m && m[lang]) ? m[lang] : '';
}

// Public path for a page in a locale: '/webhook-platform', '/fr/plateforme-webhook'.
function getPath(enSlug, lang) {
  const s = getSlug(enSlug, lang);
  if (!s) return '';
  if (lang === BASE_LANG) return enSlug === 'index' ? '/' : '/' + s;
  return enSlug === 'index' ? '/' + lang + '/' : '/' + lang + '/' + s;
}

// Reciprocal hreflang set: base lang + every locale this page is translated to.
function getAlternates(enSlug) {
  const out = [{ lang: BASE_LANG, href: getPath(enSlug, BASE_LANG) }];
  for (const loc of LOCALES) {
    if (loc.lang === BASE_LANG) continue;
    if (getSlug(enSlug, loc.lang)) out.push({ lang: loc.lang, href: getPath(enSlug, loc.lang) });
  }
  return out;
}

// Non-base langs a page is localized to — drives per-locale page selection.
function langsForPage(enSlug) {
  return Object.keys(slugs[enSlug] || {});
}

// Build a FAQPage schema from a [{q,a}] list (single source: same array renders
// the visible <details> and this JSON-LD → byte-identical visible↔schema).
function faqToSchema(faq, lang) {
  return {
    '@context': 'https://schema.org',
    '@type': 'FAQPage',
    inLanguage: lang,
    mainEntity: faq.map(function (x) {
      return { '@type': 'Question', name: x.q, acceptedAnswer: { '@type': 'Answer', text: x.a } };
    }),
  };
}

// COMPLETE page-meta object, every key defaulted. Each per-locale entry calls
// Object.assign(locals, getPageLocals(enSlug, lang)) so no page key bleeds from
// the previously-rendered page (parcel-transformer-ejs shares one mutable
// `locals` across a build run). `t` carries the page's localized strings AND
// merges chrome.includes / chrome.features / chrome.footerLinks so converted
// page templates and their includes can read `locals.t.includes.<name>`,
// `locals.t.features`, `locals.t.footerLinks` uniformly. Page-scoped keys win
// over chrome keys on name conflict.
function getPageLocals(enSlug, lang) {
  const loc = getLocale(lang);
  const page = (loc.pages && loc.pages[enSlug]) ? loc.pages[enSlug] : {};
  const chrome = loc.chrome || {};
  const t = Object.assign({}, page);
  if (chrome.includes && !t.includes) t.includes = chrome.includes;
  if (chrome.features && !t.features) t.features = chrome.features;
  if (chrome.footerLinks && !t.footerLinks) t.footerLinks = chrome.footerLinks;
  // Also flatten chrome.features / chrome.footerLinks onto locals so the
  // existing `locals.features.filter(...)` / `locals.footerLinks.about.items`
  // call sites work without rewriting — when the locale-aware copy is present,
  // it shadows the EN base loaded from data.js.
  const extra = {};
  if (chrome.features) extra.features = chrome.features;
  if (chrome.footerLinks) extra.footerLinks = chrome.footerLinks;
  return Object.assign({
    pageLang: lang,
    pageCanonical: getPath(enSlug, lang),
    pageHreflang: getAlternates(enSlug),
    pageTitle: page.pageTitle || '',
    pageDescription: page.pageDescription || '',
    pageImage: page.pageImage || null,
    pageType: page.pageType || 'website',
    pageModified: page.pageModified || null,
    pageNoindex: page.pageNoindex || false,
    pageFAQSchema: (page.faq && page.faq.items) ? faqToSchema(page.faq.items, lang) : null,
    pageSchema: page.pageSchema || null,
    pageBreadcrumb: page.breadcrumb || null,
    chrome,
    t,
  }, extra);
}

// Tab-separated locale config for the Rust orchestrator: lang\tdir\tpublicUrl.
function configLines() {
  return LOCALES.map(function (l) { return [l.lang, l.dir, l.publicUrl].join('\t'); }).join('\n');
}

module.exports = {
  LOCALES, BASE_LANG, SITE_URL, slugs,
  getLocale, getSlug, getPath, getAlternates, langsForPage, faqToSchema, getPageLocals, configLines,
};
