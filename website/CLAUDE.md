# Website — Hook0

## Build

- **Stack** : EJS templates + Parcel bundler
- **Build** : `node_modules/.bin/parcel build 'src/*.ejs' --no-cache` (from `website/`)
- **Static files** : `parcel-reporter-static-files-copy` copies `static/` to `dist/` at build time
- **Parcel path resolution** : Parcel tries to resolve `href="/path"` as imports. For files in `static/` (served at runtime, not bundled), use absolute URLs via EJS: `href="<%= locals.seo.siteUrl %>/path"`. Example: `/fonts/inter.css` fails, `<%= locals.seo.siteUrl %>/fonts/inter.css` works.

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
