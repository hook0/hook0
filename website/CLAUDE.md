# Website — Hook0

## Build

- **Stack** : EJS templates + Parcel bundler, orchestrated per-locale by a Rust binary (`scripts/build-i18n`). Locales declared in `locales/` (`slugs.js`, `en.js`, `fr.js`, `de.js`, `index.js`).
- **Build** : `npm run build` (from `website/`) → compiles + runs `scripts/build-i18n` → mirrors `src/` into `temp/<lang>/`, regenerates `.ejsrc.js` per locale (injects `lang` + `i18nHelpers`), runs ONE Parcel build per locale into `dist/` (EN), `dist/fr/`, `dist/de/`. EN templates render byte-identically to the legacy direct-Parcel build (gated by `scripts/i18n-gate`).
- **Gate** : `npm run check:i18n` → compiles + runs `scripts/i18n-gate` (Rust). Asserts on every page in `dist/`: canonical is a real https URL (never a Parcel bundle hash, R2 universal); on converted pages (those in `locales/slugs.js`): canonical matches own URL AND `<html lang>` matches locale (R1 strict); sitemap reciprocity (every localized URL listed with `<xhtml:link hreflang>` triples + `x-default`); llms.txt link integrity (internal links resolve to a file in `dist/`). **Blocking in CI**.
- **Static files** : `parcel-reporter-static-files-copy` copies `static/` to each `dist/<dir>/` per Parcel invocation. The orchestrator then dedupes origin-scoped files (`robots.txt`, `llms.txt`, `sw.js`, `manifest.json`, `favicon.ico`, fonts/...) out of `dist/{fr,de}/` — they live only at the site root.
- **Parcel path resolution** : Parcel tries to resolve `href="/path"` as imports. For files in `static/` (served at runtime, not bundled), use absolute URLs via EJS: `href="<%= locals.seo.siteUrl %>/path"`. Example: `/fonts/inter.css` fails, `<%= locals.seo.siteUrl %>/fonts/inter.css` works.
- **Shared-locals bleed (KNOWN, PHASE-1)** : `parcel-transformer-ejs` shares one mutable `locals` object across every page in a Parcel run, so non-converted pages inherit the previous page's `pageX` keys (`pageCanonical`, `pageFAQSchema`, etc.). Phase 0 fixes this for `webhook-platform` only (its template runs `Object.assign(locals, i18nHelpers.getPageLocals(enSlug, lang))` BEFORE `<!DOCTYPE>` — full reset). Phase 1 extends the per-page reset to the other 27 templates. `_head.ejs` carries a hreflang bleed defense (skips emission when bled `pageHreflang` doesn't match own canonical) so non-converted pages don't ship Parcel-rewritten canonical hashes.

## Legal pages structure

- **Mentions lgales** : `mentions-legales.ejs` (art. 6 LCEN)
- **Privacy Policy** : `privacy-policy.ejs` (bilingue EN+FR, art. 13 RGPD)
- **Terms of Service (CGU)** : `terms.ejs`
- **Terms of Sale (CGV)** : `terms-of-sale.ejs` (B2B, Code de commerce)
- **DPA** : `data-processing-addendum.ejs` (art. 28 RGPD)
- **GDPR Subprocessors** : `gdpr-subprocessors.ejs`

## Legal / RGPD rules

- **Entity** : FGRibreau SARL (pas SAS). Capital 2 000 EUR, RCS La Roche-sur-Yon 850 824 350, TVA FR27850824350
- **Director of publication** : David Sferruzza
- **Hosting** : Clever Cloud SAS (France), CDN Cloudflare (USA)
- **B2B only** : pas de droit de rtractation consommateur
- **Prices** : always HT (excl. VAT). Mention "excl. VAT" obligatoire sur tous les prix affichs
- **SLA** : pas de page SLA ddie. Utiliser "Custom SLA" (pas "Guaranteed SLA") pour viter un engagement contractuel implicite
- **Late payment** : pnalits 3x taux lgal BCE + indemnit forfaitaire 40 EUR (art. L441-10 Code de commerce)
- **Cookie consent TTL** : 13 mois max (recommandation CNIL). Stock dans localStorage avec date, vrifi  chaque visite
- **Crisp Chat** : DOIT tre conditionn au consentement cookies (RGPD). Ne charger que si `getConsent() === 'granted'`
- **Google Fonts** : INTERDIT de charger depuis googleapis.com (collecte IP = transfert hors UE). Police Inter auto-hberge dans `static/fonts/`
- **"No data sharing" claim** : FAUX et trompeur (art. L121-1 C. conso). Toujours lister honntement les sous-traitants
- **Suppression de page** : JAMAIS supprimer une page lgale sans redirection 301

## ISMS coherence

Les documents juridiques du site (`privacy-policy`, `terms`, `DPA`) et l'ISMS (`documentation/hook0-cloud/`) DOIVENT rester cohrents. Points de synchronisation critiques :

| Document site | Fichier ISMS | Points de cohrence |
|---|---|---|
| privacy-policy.ejs (dures de conservation) | information-retention-policy.md | Dures identiques |
| DPA Annexe 2 (mesures de scurit) | secure-development-policy.md, secure-engineering-policy.md | Outils lists (Argon2, Trivy, etc.) |
| DPA Annexe 2 (backup) | backup-policy.md | 30j rtention, quotidien |
| DPA (notification 72h) | business-continuity-disaster-recovery.md | Procdure notification violation |
| DPA (MFA) | access-control-policy.md | MFA infra (fait) vs MFA clients (pas fait) |
| DPA (password hashing) | password-policy.md | Argon2 |

**Rgle** : toute modification d'un document juridique du site doit tre vrifie contre l'ISMS, et inversement.

## Footer

- **Structure** : 6 colonnes dans `_footer.ejs` (Logo + About + Compare + Guides + Developers + Community)
- **Data** : liens dclars dans `data.js` sous `locals.footerLinks`
- **Rgle** : la colonne About ne doit PAS dpasser ~10 liens. Si elle grandit, scinder en sous-catgories.

## Fonts

- **Police** : Inter (weights 400, 500, 600, 700, 800)
- **Fichiers** : `static/fonts/inter.css` + `Inter-latin.woff2` + `Inter-latin-ext.woff2`
- **Chargement** : via `<link rel="stylesheet" href="<%= locals.seo.siteUrl %>/fonts/inter.css">` dans `_head.ejs`
