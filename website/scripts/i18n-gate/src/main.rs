//! i18n gate.
//!
//! Catches the shared-locals bleed bug (the SEO regression in
//! `parcel-transformer-ejs` that mutably shares one `locals` across every
//! page in a run) by asserting a one-page-one-canonical invariant over every
//! rendered HTML in `dist/`:
//!
//!   For every indexable page X, <link rel="canonical"> on dist/X.html MUST
//!   point to X's own URL — never to another page's slug, never to a Parcel
//!   bundle hash. A bled canonical from a previous page is the canonical
//!   symptom of the bug.
//!
//! Also asserts, for every page Hook0 ships in multiple locales (via
//! locales/slugs.js), that the hreflang set in `dist/sitemap.xml` is
//! reciprocal (every member of the localized set is listed) and that the
//! `<html lang>` attribute on disk matches the URL's locale.
//!
//! Must run from `website/`. Exit 0 on PASS, 1 on FAIL.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

const SITE_URL_DEFAULT: &str = "https://www.hook0.com";

// Mirror of build-i18n's site_url(): LOCAL_PREVIEW_URL overrides for
// `npm run build:local` so the gate validates the same origin the orchestrator
// actually emitted.
fn site_url() -> String {
    std::env::var("LOCAL_PREVIEW_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| SITE_URL_DEFAULT.to_string())
}

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let site_url = site_url();

    let root = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("i18n-gate: cwd: {e}");
            exit(1);
        }
    };
    let dist = root.join("dist");
    if !dist.exists() {
        eprintln!("i18n-gate: no dist/ — run `npm run build` first");
        exit(1);
    }

    let locales = match read_locales(&root) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("i18n-gate: {e}");
            exit(1);
        }
    };
    let localized = match read_localized(&root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("i18n-gate: {e}");
            exit(1);
        }
    };

    // 1a. Strict canonical + html-lang gate for CONVERTED pages only (only
    //     data-driven templates carry the per-page reset that guarantees
    //     their own meta). Legacy non-converted pages still suffer the
    //     pre-existing shared-locals bleed.
    // 1b. Universal canonical gate: every canonical that *is* present MUST be
    //     a real URL (https://...), never a Parcel bundle hash. All pages.
    for (lang, dir, _) in &locales {
        let ldir = if dir.is_empty() {
            dist.clone()
        } else {
            dist.join(dir)
        };
        if !ldir.exists() {
            continue;
        }
        let entries = match fs::read_dir(&ldir) {
            Ok(it) => it,
            Err(e) => {
                failures.push(format!("read {}: {e}", ldir.display()));
                continue;
            }
        };
        for e in entries {
            let p = match e {
                Ok(de) => de.path(),
                Err(_) => continue,
            };
            if !(p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("html")) {
                continue;
            }
            let stem = p
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_string();
            if stem == "404" {
                continue;
            }
            let html = match fs::read_to_string(&p) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // R2 (universal): canonical, if present, must be a real https URL.
            if let Some(c) = extract_canonical(&html) {
                if !c.starts_with("https://") && !c.starts_with("http://") {
                    failures.push(format!(
                        "R2: {} canonical = {c} (not a real URL — Parcel bundle hash?)",
                        p.display()
                    ));
                }
            }

            // Is this a converted page? (in slugs.js for any locale, OR the EN
            // entry whose enSlug is a slugs.js key.)
            let is_converted = if lang == "en" {
                localized.contains_key(&stem)
            } else {
                localized
                    .values()
                    .any(|m| m.get(lang).map(|s| s == &stem).unwrap_or(false))
            };
            if !is_converted {
                continue;
            }

            // R1 strict (converted pages only): canonical + html-lang correct.
            let expected = expected_url(&site_url, dir, &stem);
            match extract_canonical(&html) {
                Some(c) if c == expected => {}
                Some(c) => failures.push(format!(
                    "BLEED (converted): {} canonical = {c} (expected {expected})",
                    p.display()
                )),
                None => failures
                    .push(format!("MISSING: {} has no <link rel=canonical>", p.display())),
            }
            match extract_html_lang(&html) {
                Some(l) if &l == lang => {}
                Some(l) => failures.push(format!(
                    "LANG: {} <html lang={l}> (expected {lang})",
                    p.display()
                )),
                None => failures.push(format!("LANG: {} no <html lang>", p.display())),
            }
        }
    }

    // 1c. llms.txt link integrity: every markdown link in dist/llms.txt must
    //     resolve. Internal links (same SITE_URL) must point to a file that
    //     actually shipped in dist/. External links must be well-formed https.
    //     No HTTP fetch — gate stays offline-deterministic.
    let llms_path = dist.join("llms.txt");
    if llms_path.exists() {
        let llms = fs::read_to_string(&llms_path).unwrap_or_default();
        for url in extract_markdown_links(&llms) {
            if let Some(rest) = url.strip_prefix(site_url.as_str()) {
                // Internal link → derive dist path and assert it exists.
                let path = rest.trim_start_matches('/');
                let candidate = if path.is_empty() {
                    dist.join("index.html")
                } else if path.ends_with('/') {
                    dist.join(path).join("index.html")
                } else if dist.join(format!("{path}.html")).exists() {
                    dist.join(format!("{path}.html"))
                } else {
                    dist.join(path).join("index.html")
                };
                if !candidate.exists() {
                    failures.push(format!(
                        "LLMSTXT: {url} → {} does not exist in dist/",
                        candidate.display()
                    ));
                }
            } else if !url.starts_with("https://") {
                failures.push(format!("LLMSTXT: {url} is not an https:// URL"));
            } else if url.contains("localhost") || url.contains("127.0.0.1") {
                failures.push(format!("LLMSTXT: {url} points at localhost"));
            }
        }
    } else {
        failures.push("LLMSTXT: dist/llms.txt missing".into());
    }

    // 1d. FAQ schema integrity: for every (lang, enSlug) where the locale
    //     strings declare faq.items, the rendered HTML at the page's URL MUST
    //     contain a FAQPage JSON-LD with inLanguage matching the locale AND
    //     mainEntity length == faq.items.length. Non-regression check for the
    //     data-driven chain (locales/{lang}.js faq.items → faqToSchema →
    //     getPageLocals → _head.ejs pageFAQSchema block → rendered HTML).
    //     Property-test/fuzz: not applicable here, input space is fully bounded
    //     by slugs.js × locales — this deterministic sweep IS the property.
    let faq_counts = read_faq_counts(&root).unwrap_or_default();
    for ((lang, en_slug), expected) in &faq_counts {
        let dir = locales
            .iter()
            .find(|(l, _, _)| l == lang)
            .map(|(_, d, _)| d.clone())
            .unwrap_or_default();
        let stem = if lang == "en" {
            en_slug.clone()
        } else {
            localized
                .get(en_slug)
                .and_then(|m| m.get(lang))
                .cloned()
                .unwrap_or_default()
        };
        if stem.is_empty() {
            continue;
        }
        let html_path = if dir.is_empty() {
            dist.join(format!("{stem}.html"))
        } else {
            dist.join(&dir).join(format!("{stem}.html"))
        };
        if !html_path.exists() {
            continue;
        }
        let html = match fs::read_to_string(&html_path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let where_at = html_path.display();
        match extract_faqpage_block(&html) {
            None => failures.push(format!(
                "FAQ: {where_at} declares faq.items in locales/{lang}.js but rendered HTML has no FAQPage JSON-LD"
            )),
            Some(block) => {
                let want_lang = format!("\"inLanguage\":\"{lang}\"");
                if !block.contains(&want_lang) {
                    failures.push(format!(
                        "FAQ: {where_at} FAQPage block missing {want_lang}"
                    ));
                }
                let got = block.matches("\"@type\":\"Question\"").count();
                if got != *expected {
                    failures.push(format!(
                        "FAQ: {where_at} FAQPage has {got} Question(s), expected {expected} (from locales/{lang}.js faq.items)"
                    ));
                }
            }
        }
    }

    // 2. sitemap reciprocity: every localized page must appear in dist/sitemap.xml
    //    with the full hreflang set (every lang it ships in + x-default).
    let sitemap_path = dist.join("sitemap.xml");
    let sitemap = fs::read_to_string(&sitemap_path).unwrap_or_default();
    if sitemap.is_empty() {
        failures.push("SITEMAP: dist/sitemap.xml missing or empty".into());
    } else {
        for (en_slug, m) in &localized {
            // Build the full expected URL set for this page (every locale +
            // x-default). The sitemap MUST contain every URL, and the entry
            // for each URL MUST list every other localized URL via xhtml:link.
            // Base lang = the entry in `locales` whose dir is empty (renders at
            // site root). Derived, never hardcoded.
            let base_lang = locales
                .iter()
                .find(|(_, dir, _)| dir.is_empty())
                .map(|(l, _, _)| l.clone())
                .unwrap_or_default();
            let urls: HashMap<String, String> = locales
                .iter()
                .filter_map(|(lang, dir, _)| {
                    let slug = if lang == &base_lang {
                        Some(en_slug.clone())
                    } else {
                        m.get(lang).cloned()
                    };
                    slug.map(|s| (lang.clone(), expected_url(&site_url, dir, &s)))
                })
                .collect();

            for (lang, url) in &urls {
                if !sitemap.contains(url) {
                    failures.push(format!(
                        "SITEMAP: {url} missing from dist/sitemap.xml (locale {lang}, page {en_slug})"
                    ));
                    continue;
                }
                // Every other locale's URL must be reachable as an xhtml:link
                // from this <url> entry. Cheap heuristic: the sitemap contains
                // the xhtml:link line anywhere (we already know it carries
                // this <loc>; the writer emits the same alt set everywhere).
                for (other_lang, other_url) in &urls {
                    if other_lang == lang {
                        continue;
                    }
                    let needle = format!(
                        "hreflang=\"{other_lang}\" href=\"{other_url}\""
                    );
                    if !sitemap.contains(&needle) {
                        failures.push(format!(
                            "SITEMAP: hreflang reciprocity broken for {en_slug}: missing {other_lang}→{other_url}"
                        ));
                    }
                }
                let x_default = format!(
                    "hreflang=\"x-default\" href=\"{site_url}/\""
                );
                if !sitemap.contains(&x_default) {
                    failures.push(format!(
                        "SITEMAP: x-default missing (page {en_slug})"
                    ));
                }
            }
        }
    }

    // Testimonial wall MUST animate under prefers-reduced-motion (explicit
    // owner decision, matching fgribreau.github.io — cf. commented note in
    // `src/style.scss`). The nuclear `*{animation-duration:0.001ms!important}`
    // in the reduced-motion @media would otherwise kill the marquee, so an
    // explicit `!important` override that RESTORES the animation is required.
    // Guard fails on either signal:
    //   (a) a rule inside `@media (prefers-reduced-motion...)` KILLS the
    //       testimonial animation (contains `animation:none` or a zero-ish
    //       `animation-duration:0*ms` while targeting `.testimonial-wall*` /
    //       `.wall-quote*`);
    //   (b) no restore override — no reduced-motion rule sets
    //       `.testimonial-wall__track--left { animation:...!important }`
    //       to beat the universal `*` blanket.
    check_wall_survives_reduced_motion(&dist, &mut failures);

    // EntityMap (brand knowledge graph) must ship valid and its HTML companion
    // must stay in sync with the JSON — the companion is generator-emitted, never
    // hand-edited, so a mismatch means someone edited one without the other.
    check_entitymap(&dist, &mut failures);

    if failures.is_empty() {
        println!("i18n-gate: PASS");
        exit(0);
    }
    // Dedup so the same finding from N urls counts once.
    let mut seen: HashSet<String> = HashSet::new();
    let mut uniq: Vec<&String> = Vec::new();
    for f in &failures {
        if seen.insert(f.clone()) {
            uniq.push(f);
        }
    }
    eprintln!("i18n-gate: FAIL ({} finding(s))", uniq.len());
    for f in uniq {
        eprintln!("  - {f}");
    }
    exit(1);
}

// Locate the first FAQPage JSON-LD block in the document. Walks every
// `<script type="application/ld+json">...</script>` (tolerant to attr-order
// and unquoted-value variants htmlnano produces) and returns the inner JSON
// of the first one that contains `"@type":"FAQPage"`. Returns None if absent.
fn extract_faqpage_block(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let mut cursor = 0;
    while let Some(pos) = lower[cursor..].find("<script") {
        let abs = cursor + pos;
        let tag_end = match html[abs..].find('>') {
            Some(p) => abs + p + 1,
            None => return None,
        };
        let tag = &lower[abs..tag_end];
        if !tag.contains("application/ld+json") {
            cursor = tag_end;
            continue;
        }
        let close = match lower[tag_end..].find("</script>") {
            Some(p) => tag_end + p,
            None => return None,
        };
        let body = &html[tag_end..close];
        if body.contains("\"@type\":\"FAQPage\"") || body.contains("\"@type\": \"FAQPage\"") {
            return Some(body.to_string());
        }
        cursor = close + "</script>".len();
    }
    None
}

// (lang, enSlug) -> faq.items.length for every locale that ships a faq for the
// page. Asks node directly so the gate doesn't reimplement the JS structure;
// failure to read is non-fatal (returns empty → no FAQ assertions fire).
fn read_faq_counts(root: &Path) -> Result<HashMap<(String, String), usize>, String> {
    let script = "const idx=require('./locales');\
        const out=[];\
        for(const loc of idx.LOCALES){\
          const data=idx.getLocale(loc.lang);\
          if(!data||!data.pages)continue;\
          for(const slug of Object.keys(data.pages)){\
            const p=data.pages[slug];\
            if(p&&p.faq&&Array.isArray(p.faq.items)&&p.faq.items.length>0){\
              out.push(loc.lang+'\\t'+slug+'\\t'+p.faq.items.length);\
            }\
          }\
        }\
        process.stdout.write(out.join('\\n'));";
    let out = Command::new("node")
        .arg("-e")
        .arg(script)
        .current_dir(root)
        .output()
        .map_err(|e| format!("node (faq counts): {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "faq counts failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let mut m: HashMap<(String, String), usize> = HashMap::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() != 3 {
            continue;
        }
        if let Ok(n) = parts[2].parse::<usize>() {
            m.insert((parts[0].to_string(), parts[1].to_string()), n);
        }
    }
    Ok(m)
}

// Extract URLs from every `[text](url)` markdown link. Stops the URL at the
// first whitespace or closing paren (no nested parens support — fine for our
// llms.txt where URLs never contain `)`).
fn extract_markdown_links(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'[' {
            i += 1;
            continue;
        }
        let close_br = match text[i..].find("](") {
            Some(p) => i + p,
            None => break,
        };
        let url_start = close_br + 2;
        if url_start >= text.len() {
            break;
        }
        let url_end = text[url_start..]
            .find(|c: char| c == ')' || c.is_whitespace())
            .map(|p| url_start + p)
            .unwrap_or(text.len());
        out.push(text[url_start..url_end].to_string());
        i = url_end;
    }
    out
}

fn expected_url(site: &str, dir: &str, stem: &str) -> String {
    if stem == "index" {
        if dir.is_empty() {
            format!("{site}/")
        } else {
            format!("{site}/{dir}/")
        }
    } else if dir.is_empty() {
        format!("{site}/{stem}")
    } else {
        format!("{site}/{dir}/{stem}")
    }
}

// Tolerates both quoted (legacy) and unquoted (htmlnano-minified) attribute
// forms: rel="canonical" / rel=canonical.
fn extract_canonical(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let i = lower.find("rel=\"canonical\"").or_else(|| lower.find("rel=canonical"))?;
    // Look ahead for the href= within the same tag (next 200 chars max).
    let win = &html[i..i.saturating_add(200).min(html.len())];
    extract_attr(win, "href")
}

fn extract_html_lang(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let i = lower.find("<html")?;
    let win = &html[i..i.saturating_add(200).min(html.len())];
    extract_attr(win, "lang")
}

// attr value parser tolerating: foo="bar" | foo='bar' | foo=bar (no whitespace).
fn extract_attr(window: &str, name: &str) -> Option<String> {
    let lw = window.to_lowercase();
    let key = format!("{name}=");
    let i = lw.find(&key)?;
    let rest = &window[i + key.len()..];
    let bytes = rest.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let (delim, off) = match bytes[0] {
        b'"' => (b'"', 1),
        b'\'' => (b'\'', 1),
        _ => (b' ', 0),
    };
    let tail = &rest[off..];
    let end = if delim == b' ' {
        // HTML5 unquoted attribute value: terminated by whitespace, > < " ' = `
        // — but NOT by '/' (URLs contain slashes!).
        tail.find(|c: char| {
            c.is_whitespace()
                || c == '>'
                || c == '<'
                || c == '"'
                || c == '\''
                || c == '='
                || c == '`'
        })
        .unwrap_or(tail.len())
    } else {
        tail.bytes().position(|b| b == delim).unwrap_or(tail.len())
    };
    Some(tail[..end].to_string())
}

fn read_locales(root: &Path) -> Result<Vec<(String, String, String)>, String> {
    let out = Command::new("node")
        .arg("-e")
        .arg("process.stdout.write(require('./locales').configLines())")
        .current_dir(root)
        .output()
        .map_err(|e| format!("node (locales config): {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "locales config failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let p: Vec<&str> = l.splitn(3, '\t').collect();
            (
                p.first().copied().unwrap_or("").to_string(),
                p.get(1).copied().unwrap_or("").to_string(),
                p.get(2).copied().unwrap_or("").to_string(),
            )
        })
        .collect())
}

fn read_localized(root: &Path) -> Result<HashMap<String, HashMap<String, String>>, String> {
    let script = "const s=require('./locales').slugs;\
        process.stdout.write(Object.keys(s).map(k=>k+'\\t'+Object.keys(s[k]).map(l=>l+'='+s[k][l]).join(',')).join('\\n'))";
    let out = Command::new("node")
        .arg("-e")
        .arg(script)
        .current_dir(root)
        .output()
        .map_err(|e| format!("node (slugs): {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "slugs read failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let mut m: HashMap<String, HashMap<String, String>> = HashMap::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut it = line.splitn(2, '\t');
        let k = it.next().unwrap_or("").to_string();
        let mut langs: HashMap<String, String> = HashMap::new();
        for pair in it.next().unwrap_or("").split(',').filter(|x| !x.is_empty()) {
            let mut kv = pair.splitn(2, '=');
            let l = kv.next().unwrap_or("").to_string();
            let s = kv.next().unwrap_or("").to_string();
            if !l.is_empty() && !s.is_empty() {
                langs.insert(l, s);
            }
        }
        m.insert(k, langs);
    }
    Ok(m)
}

// Testimonial wall must keep animating even when the user has enabled
// prefers-reduced-motion. Rationale (see /Users/.../fgribreau.github.io/src/scss/layout/_site.scss):
// the marquee is the whole visual signal of the section — killing it collapses
// the section to a static column that looks broken. Trades W3C-purity for a
// design decision the owner has already documented on fgribreau.github.io.
//
// The bundle carries a nuclear `*{animation-duration:0.001ms!important}` inside
// its reduced-motion @media (WCAG 2.3.3 blanket). That blanket would kill the
// marquee unless a class-specific `!important` rule restores it. This check
// enforces BOTH:
//   (a) no rule inside a reduced-motion @media KILLS the wall/quote animation
//       (patterns: `animation:none`, `animation-duration:0`, `0ms`, `0.001ms`
//        while targeting `.testimonial-wall*` / `.wall-quote*`);
//   (b) at least one restore rule exists — a reduced-motion selector matching
//       `.testimonial-wall__track--left` sets an animation with `!important`.
// EntityMap conformance-lite + drift guard (no JSON parser; uses stable markers).
// No-op if the site ships no EntityMap. Fails on: missing version 1.0, zero
// entities, or an HTML companion whose per-entity JSON-LD block count diverges
// from the JSON entity count (== hand-edited or stale — regenerate the companion).
fn check_entitymap(dist: &Path, failures: &mut Vec<String>) {
    let json = match fs::read_to_string(dist.join("entitymap.json")) {
        Ok(s) => s,
        Err(_) => return, // no EntityMap on this site
    };
    if !(json.contains("\"version\": \"1.0\"") || json.contains("\"version\":\"1.0\"")) {
        failures.push("entitymap.json: missing version \"1.0\"".to_string());
    }
    let entities = json.matches("\"entityId\"").count();
    if entities == 0 {
        failures.push("entitymap.json: no entities".to_string());
    }
    match fs::read_to_string(dist.join("entitymap.html")) {
        Ok(html) => {
            let blocks = html.matches("application/ld+json").count();
            if blocks != entities {
                failures.push(format!(
                    "entitymap.html out of sync with entitymap.json: {blocks} JSON-LD block(s) vs {entities} entit(y/ies) — regenerate via `validate-entitymap --emit-html`"
                ));
            }
            if !html.contains("entitymap.json") {
                failures.push(
                    "entitymap.html: missing rel=alternate link to entitymap.json".to_string(),
                );
            }
        }
        Err(_) => failures.push(
            "entitymap.json ships but entitymap.html is missing — regenerate the companion"
                .to_string(),
        ),
    }
}

fn check_wall_survives_reduced_motion(dist: &Path, failures: &mut Vec<String>) {
    let mut css_paths: Vec<PathBuf> = Vec::new();
    collect_css_bundles(dist, &mut css_paths);
    for p in &css_paths {
        let css = match fs::read_to_string(p) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let name = p
            .strip_prefix(dist)
            .map(|r| r.to_string_lossy().into_owned())
            .unwrap_or_else(|_| p.display().to_string());
        let (kills, has_restore) = scan_reduced_motion_offenses(&css);
        for offense in kills {
            failures.push(format!(
                "REDUCED_MOTION: {name} — @media (prefers-reduced-motion) rule `{offense}` cancels the testimonial-wall animation (must survive reduced-motion, cf. explicit note in src/style.scss)"
            ));
        }
        if !has_restore {
            failures.push(format!(
                "REDUCED_MOTION: {name} — missing restore override. The universal `*{{animation-duration:0.001ms!important}}` in @media (prefers-reduced-motion) kills the marquee. Add a class-specific rule that sets `animation:wall-scroll-left ...!important` on `.testimonial-wall__track--left` inside the same @media block (cf. src/style.scss)."
            ));
        }
    }
}

fn collect_css_bundles(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            // Skip vendored font dir — third-party CSS we can't rewrite.
            if p.file_name().and_then(|x| x.to_str()) == Some("fonts") {
                continue;
            }
            collect_css_bundles(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("css") {
            // Same skip at file level (fonts/inter.css lives at dist root too).
            if p.parent()
                .and_then(|d| d.file_name())
                .and_then(|x| x.to_str())
                == Some("fonts")
            {
                continue;
            }
            out.push(p);
        }
    }
}

// Scan the compiled CSS for testimonial-wall regressions inside every
// `@media (prefers-reduced-motion...)` block.
//
// Returns `(kills, has_restore)`:
//   - `kills`  — selectors of any rule that cancels the wall/quote animation
//                (contains `animation:none` or `animation-duration:0*` while
//                targeting `.testimonial-wall*` or `.wall-quote*`).
//   - `has_restore` — true when at least one reduced-motion rule targets
//                     `.testimonial-wall__track--left` and sets an
//                     `animation:...!important` (the class-specific override
//                     that beats the universal `*` blanket).
fn scan_reduced_motion_offenses(css: &str) -> (Vec<String>, bool) {
    const WALL_SELECTORS: &[&str] = &[".testimonial-wall", ".wall-quote"];
    const KILL_PATTERNS: &[&str] = &[
        "animation:none",
        "animation-duration:0;",
        "animation-duration:0ms",
        "animation-duration:0s",
        "animation-duration:0.001ms",
    ];
    let mut kills: Vec<String> = Vec::new();
    let mut has_restore = false;
    let bytes = css.as_bytes();
    let mut cursor = 0;
    while let Some(pos) = css[cursor..].find("@media") {
        let start = cursor + pos;
        let brace_open = match css[start..].find('{') {
            Some(b) => start + b,
            None => break,
        };
        let condition = &css[start..brace_open];
        if !condition.contains("prefers-reduced-motion") {
            cursor = brace_open + 1;
            continue;
        }
        // Match the enclosing braces of this @media block.
        let mut depth: i32 = 1;
        let mut i = brace_open + 1;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        let body_end = i.saturating_sub(1);
        let body = &css[brace_open + 1..body_end];

        // Walk individual rules inside this @media (naive split: each `}`
        // terminates a rule at depth 1, so we scan char-by-char). Rules that
        // themselves open a nested `{` (unlikely in minified reduced-motion
        // blocks but handled defensively) are treated as opaque.
        let rb = body.as_bytes();
        let mut ri = 0;
        while ri < rb.len() {
            // Find next rule opening brace.
            let open = match body[ri..].find('{') {
                Some(o) => ri + o,
                None => break,
            };
            let selector = body[ri..open].trim().trim_start_matches(',').trim();
            // Match rule body (balance braces).
            let mut d: i32 = 1;
            let mut j = open + 1;
            while j < rb.len() && d > 0 {
                match rb[j] {
                    b'{' => d += 1,
                    b'}' => d -= 1,
                    _ => {}
                }
                j += 1;
            }
            let decls = &body[open + 1..j - 1];
            let targets_wall = WALL_SELECTORS
                .iter()
                .any(|needle| selector.contains(needle));
            if targets_wall {
                if KILL_PATTERNS.iter().any(|pat| decls.contains(pat)) {
                    kills.push(selector.to_string());
                }
                let restores_animation = decls.contains("animation:")
                    && decls.contains("wall-scroll")
                    && decls.contains("!important");
                if restores_animation && selector.contains(".testimonial-wall__track--left") {
                    has_restore = true;
                }
            }
            ri = j;
        }
        cursor = i;
    }
    (kills, has_restore)
}
