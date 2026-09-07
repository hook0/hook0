// Locale comes from the environment so build-i18n never has to rewrite (and
// restore) this tracked file: a build killed mid-run used to leave it pinned to
// whichever locale was in flight. Every locale (EN included) gets `lang` +
// `i18nHelpers` so a data-driven template can self-inject its per-page locals
// (Object.assign(locals, getPageLocals(enSlug, lang))). Legacy passthrough pages
// ignore the two extra keys, so their EN output stays byte-identical; the
// converted pages read locals.t.* instead.
module.exports = {
  locals: Object.assign({}, require('./data'), {
    lang: process.env.BUILD_I18N_LANG || 'en',
    i18nHelpers: require('./locales'),
  }),
};
