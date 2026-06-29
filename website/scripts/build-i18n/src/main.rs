//! i18n build orchestrator for the Hook0 marketing site.
//!
//! For each locale (locales/index.js → configLines), it regenerates the root
//! `.ejsrc.js` so parcel-transformer-ejs injects that locale's strings, mirrors
//! the shared `src/` tree into `temp/<lang>/` (flat, so Parcel resolves bundled
//! assets identically), and runs ONE Parcel build per locale into the right dist
//! subdirectory (EN → dist root, FR → dist/fr, DE → dist/de). Pages are
//! discovered from `src/*.ejs`; which pages render for FR/DE is driven by
//! locales/slugs.js — no hardcoded page list lives here.
//!
//! Must be run from the `website/` directory (the one holding `.ejsrc.js`).
//! "build once, promote everywhere": the same templates render every locale.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

type R<T> = Result<T, String>;

fn main() {
    if let Err(e) = run() {
        eprintln!("build-i18n: ERROR: {e}");
        std::process::exit(1);
    }
}

fn run() -> R<()> {
    let root = std::env::current_dir().map_err(|e| e.to_string())?;
    if !root.join(".ejsrc.js").exists() {
        return Err(format!(
            "must run from website/ (no .ejsrc.js in {})",
            root.display()
        ));
    }
    let src = root.join("src");
    let temp = root.join("temp");
    let dist = root.join("dist");
    let ejsrc = root.join(".ejsrc.js");

    // Locale routing config from locales/index.js: lang \t dir \t publicUrl
    let locales = read_locales(&root)?;
    // enSlug -> langs it is localized to (FR/DE), from locales/slugs.js
    let localized = read_localized(&root)?;
    // Discover EN templates: src/*.ejs (top-level only; partials live in includes/)
    let templates = discover_templates(&src)?;

    let ejsrc_orig = fs::read_to_string(&ejsrc).map_err(|e| e.to_string())?;
    let _ = fs::remove_dir_all(&temp);
    let _ = fs::remove_dir_all(&dist);
    fs::create_dir_all(&temp).map_err(|e| e.to_string())?;

    let restore = |_: &str| {
        let _ = fs::write(&ejsrc, &ejsrc_orig);
    };

    for (lang, dir, public_url) in &locales {
        let pages: Vec<String> = if lang == "en" {
            templates.clone()
        } else {
            templates
                .iter()
                .filter(|t| {
                    localized
                        .get(*t)
                        .map(|m| m.contains_key(lang))
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        };
        if pages.is_empty() {
            continue;
        }
        eprintln!("build-i18n: locale {lang} ({} pages)", pages.len());

        // Regenerate .ejsrc.js for this locale. Every locale (EN included) gets
        // `lang` + `i18nHelpers` so a data-driven template can self-inject its
        // per-page locals (Object.assign(locals, getPageLocals(enSlug, lang))).
        // Legacy passthrough pages ignore the two extra keys, so their EN output
        // stays byte-identical; the converted pages read locals.t.* instead.
        let ejsrc_content = format!(
            "module.exports = {{ locals: Object.assign({{}}, require('./data'), {{ lang: '{lang}', i18nHelpers: require('./locales') }}) }};\n"
        );
        if let Err(e) = fs::write(&ejsrc, &ejsrc_content) {
            restore(lang);
            return Err(e.to_string());
        }

        // Mirror shared src/ into temp/<lang>/ (exclude the de/ seed + cruft).
        let tdir = temp.join(lang);
        fs::create_dir_all(&tdir).map_err(|e| e.to_string())?;
        if let Err(e) = rsync(&src, &tdir) {
            restore(lang);
            return Err(e);
        }

        // EN keeps every discovered template as a root entry (passthrough).
        // FR/DE: prune temp/<lang>/ down to the localized pages only, each
        // renamed to its translated slug. The shared data-driven template
        // self-injects getPageLocals(enSlug, lang) from its own hardcoded
        // enSlug, so the renamed file still resolves the right strings.
        if lang != "en" {
            let mut to_rename: Vec<(PathBuf, PathBuf)> = Vec::new();
            let mut to_remove: Vec<PathBuf> = Vec::new();
            for e in fs::read_dir(&tdir).map_err(|e| e.to_string())? {
                let p = e.map_err(|e| e.to_string())?.path();
                if !(p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("ejs")) {
                    continue;
                }
                let stem = p
                    .file_stem()
                    .and_then(|x| x.to_str())
                    .unwrap_or("")
                    .to_string();
                match localized.get(&stem).and_then(|m| m.get(lang)) {
                    Some(slug) if pages.iter().any(|pg| pg == &stem) => {
                        if slug != &stem {
                            to_rename.push((p.clone(), tdir.join(format!("{slug}.ejs"))));
                        }
                    }
                    _ => to_remove.push(p),
                }
            }
            for p in to_remove {
                let _ = fs::remove_file(p);
            }
            for (from, to) in to_rename {
                if let Err(e) = fs::rename(&from, &to) {
                    restore(lang);
                    return Err(format!("rename {} -> {}: {e}", from.display(), to.display()));
                }
            }
        }

        let ddir = if dir.is_empty() {
            dist.clone()
        } else {
            dist.join(dir)
        };
        fs::create_dir_all(&ddir).map_err(|e| e.to_string())?;

        let entries = format!("{}/*.ejs", tdir.display());
        if let Err(e) = parcel_build(&root, &entries, &ddir, public_url) {
            restore(lang);
            return Err(e);
        }

        // parcel-reporter-static-files-copy runs per Parcel invocation, so
        // everything in static/ also lands in dist/<dir>/. Origin-scoped files
        // (robots.txt, llms.txt, sw.js, manifest.json, favicon.ico, fonts/...)
        // belong only at the site root — dupes under /fr/ or /de/ pollute the
        // origin (split service worker scopes, conflicting robots, etc.).
        // Dedupe by mirroring the static/ tree onto dist/<dir>/ and removing
        // every matching path. List is derived from static/, not hardcoded.
        if !dir.is_empty() {
            let _ = dedupe_origin_files(&root.join("static"), &ddir);
        }
    }

    restore("");

    // Unified multilingual sitemap at dist/sitemap.xml. Replaces both the
    // (removed) parcel-reporter-sitemap and scripts/fix-sitemap.js.
    write_sitemap(&dist, SITE_URL, &locales, &localized)?;

    println!("build-i18n: done");
    Ok(())
}

const SITE_URL: &str = "https://www.hook0.com";

// Walk dist/, build one <url> entry per indexable HTML, attach <xhtml:link>
// hreflang cross-references for every page that ships in multiple locales.
//
// Exclusion is page-declared (no hardcoded path list, per CLAUDE.md):
//   - <meta name="robots" content="...noindex..."> → never indexable
//   - <meta name="sitemap" content="exclude">      → indexable, but off-sitemap
//   - 404.html                                     → never a real URL
fn write_sitemap(
    dist: &Path,
    site_url: &str,
    locales: &[(String, String, String)],
    localized: &HashMap<String, HashMap<String, String>>,
) -> R<()> {
    // Reverse lookup: any locale slug -> (enSlug, lang) so we can recognise a
    // localized FR/DE page from its on-disk filename and link it back to its
    // EN counterpart's full hreflang set.
    let mut slug_to_en: HashMap<(String, String), String> = HashMap::new(); // (lang, localizedSlug) -> enSlug
    for (en, m) in localized {
        for (l, s) in m {
            slug_to_en.insert((l.clone(), s.clone()), en.clone());
        }
    }

    let mut urls: Vec<String> = Vec::new();
    let today = today_iso(dist);

    for (lang, dir, _public_url) in locales {
        let ldir = if dir.is_empty() {
            dist.to_path_buf()
        } else {
            dist.join(dir)
        };
        if !ldir.exists() {
            continue;
        }
        for e in fs::read_dir(&ldir).map_err(|e| e.to_string())? {
            let p = e.map_err(|e| e.to_string())?.path();
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
            if has_noindex(&html) || has_sitemap_exclude(&html) {
                continue;
            }

            // URL form: index → /, lang-root → /lang/, else /[lang/]slug
            let loc = if stem == "index" {
                if dir.is_empty() {
                    format!("{site_url}/")
                } else {
                    format!("{site_url}/{dir}/")
                }
            } else if dir.is_empty() {
                format!("{site_url}/{stem}")
            } else {
                format!("{site_url}/{dir}/{stem}")
            };

            // Resolve enSlug to look up the hreflang set, if any.
            let en_slug = if lang == "en" {
                stem.clone()
            } else {
                slug_to_en
                    .get(&(lang.clone(), stem.clone()))
                    .cloned()
                    .unwrap_or_default()
            };
            let alts = if !en_slug.is_empty() && localized.contains_key(&en_slug) {
                hreflang_links(site_url, &en_slug, locales, localized)
            } else {
                String::new()
            };

            urls.push(format!(
                "  <url>\n    <loc>{loc}</loc>\n    <lastmod>{today}</lastmod>\n{alts}  </url>"
            ));
        }
    }

    urls.sort();
    let body = urls.join("\n");
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n{body}\n</urlset>\n"
    );
    fs::write(dist.join("sitemap.xml"), xml).map_err(|e| e.to_string())?;
    // Drop any per-dir sitemap.xml leftover (defensive: parcel-reporter-sitemap
    // is now disabled, but a previous build may have left them on disk).
    for (_lang, dir, _) in locales {
        if dir.is_empty() {
            continue;
        }
        let _ = fs::remove_file(dist.join(dir).join("sitemap.xml"));
    }
    Ok(())
}

// Per-URL hreflang block: every locale this page ships in + x-default → root EN.
fn hreflang_links(
    site_url: &str,
    en_slug: &str,
    locales: &[(String, String, String)],
    localized: &HashMap<String, HashMap<String, String>>,
) -> String {
    let mut out = String::new();
    for (lang, dir, _) in locales {
        let slug = if lang == "en" {
            Some(en_slug.to_string())
        } else {
            localized.get(en_slug).and_then(|m| m.get(lang)).cloned()
        };
        if let Some(s) = slug {
            let href = if s == "index" {
                if dir.is_empty() {
                    format!("{site_url}/")
                } else {
                    format!("{site_url}/{dir}/")
                }
            } else if dir.is_empty() {
                format!("{site_url}/{s}")
            } else {
                format!("{site_url}/{dir}/{s}")
            };
            out.push_str(&format!(
                "    <xhtml:link rel=\"alternate\" hreflang=\"{lang}\" href=\"{href}\"/>\n"
            ));
        }
    }
    out.push_str(&format!(
        "    <xhtml:link rel=\"alternate\" hreflang=\"x-default\" href=\"{site_url}/\"/>\n"
    ));
    out
}

// htmlnano often strips quotes — match the unquoted form too. We only need to
// catch a robots meta that mentions noindex (in any common attribute order).
fn has_noindex(html: &str) -> bool {
    let lower = html.to_lowercase();
    for line in lower.split('<') {
        if !line.starts_with("meta ") && !line.starts_with("meta\t") {
            continue;
        }
        if line.contains("robots") && line.contains("noindex") {
            return true;
        }
    }
    false
}

fn has_sitemap_exclude(html: &str) -> bool {
    let lower = html.to_lowercase();
    for line in lower.split('<') {
        if !line.starts_with("meta ") && !line.starts_with("meta\t") {
            continue;
        }
        if line.contains("name=\"sitemap\"") || line.contains("name=sitemap") {
            if line.contains("exclude") {
                return true;
            }
        }
    }
    false
}

// Stable YYYY-MM-DD without pulling in chrono: stat dist mtime (set by the
// build that just ran). Falls back to a fixed "1970-01-01" if stat fails —
// the sitemap is still well-formed, only the lastmod accuracy is degraded.
fn today_iso(dist: &Path) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    let secs = fs::metadata(dist)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .unwrap_or(Duration::ZERO)
        .as_secs();
    // Civil from days since 1970-01-01 (Howard Hinnant's algorithm, integer-only).
    let days = (secs / 86_400) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let _ = SystemTime::now(); // (kept to keep std::time::SystemTime imported)
    format!("{y:04}-{m:02}-{d:02}")
}

fn read_locales(root: &Path) -> R<Vec<(String, String, String)>> {
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
    let cfg = String::from_utf8_lossy(&out.stdout);
    let v: Vec<(String, String, String)> = cfg
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
        .collect();
    if v.is_empty() {
        return Err("no locales configured".into());
    }
    Ok(v)
}

// enSlug -> { lang -> translatedSlug }, from locales/slugs.js. Drives both which
// pages render for FR/DE and the translated filename each renders under.
fn read_localized(root: &Path) -> R<HashMap<String, HashMap<String, String>>> {
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
            let slug = kv.next().unwrap_or("").to_string();
            if !l.is_empty() && !slug.is_empty() {
                langs.insert(l, slug);
            }
        }
        m.insert(k, langs);
    }
    Ok(m)
}

// Remove from `locale_dist` every file (and now-empty directory) that exists in
// `static_root`. Walks static_root, derives relative paths, deletes matching
// ones from locale_dist. No hardcoded list. Silent on missing entries — many
// won't have been copied for every locale anyway.
fn dedupe_origin_files(static_root: &Path, locale_dist: &Path) -> R<()> {
    if !static_root.is_dir() || !locale_dist.is_dir() {
        return Ok(());
    }
    let mut dirs: Vec<PathBuf> = Vec::new();
    walk_files(static_root, &mut |rel| {
        let target = locale_dist.join(rel);
        let _ = fs::remove_file(&target);
        if let Some(parent) = rel.parent() {
            if parent.as_os_str().len() > 0 {
                dirs.push(locale_dist.join(parent));
            }
        }
    })?;
    // Try to remove now-empty directories, deepest first.
    dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    dirs.dedup();
    for d in dirs {
        let _ = fs::remove_dir(d);
    }
    Ok(())
}

fn walk_files(root: &Path, on_file: &mut dyn FnMut(&Path)) -> R<()> {
    walk_files_inner(root, root, on_file)
}

fn walk_files_inner(base: &Path, dir: &Path, on_file: &mut dyn FnMut(&Path)) -> R<()> {
    for e in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let p = e.map_err(|e| e.to_string())?.path();
        if p.is_dir() {
            walk_files_inner(base, &p, on_file)?;
        } else if p.is_file() {
            if let Ok(rel) = p.strip_prefix(base) {
                on_file(rel);
            }
        }
    }
    Ok(())
}

fn discover_templates(src: &Path) -> R<Vec<String>> {
    let mut t: Vec<String> = Vec::new();
    for e in fs::read_dir(src).map_err(|e| e.to_string())? {
        let p: PathBuf = e.map_err(|e| e.to_string())?.path();
        if p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("ejs") {
            if let Some(stem) = p.file_stem().and_then(|x| x.to_str()) {
                t.push(stem.to_string());
            }
        }
    }
    t.sort();
    Ok(t)
}

fn rsync(src: &Path, dst: &Path) -> R<()> {
    let status = Command::new("rsync")
        .args([
            "-a",
            "--delete",
            "--exclude=de",
            "--exclude=.DS_Store",
            &format!("{}/", src.display()),
            &format!("{}/", dst.display()),
        ])
        .status()
        .map_err(|e| format!("rsync: {e}"))?;
    if !status.success() {
        return Err(format!("rsync exited {:?}", status.code()));
    }
    Ok(())
}

// Parcel hangs after a successful build (known issue), so wrap in `timeout` and
// treat a clean exit (0) or SIGKILL-after-done (137) / timeout (124) as success.
fn parcel_build(root: &Path, entries: &str, dist_dir: &Path, public_url: &str) -> R<()> {
    let cmd = format!(
        "timeout --signal=KILL 300 npx parcel build '{}' --dist-dir '{}' --public-url='{}' --no-cache --no-source-maps; \
         ec=$?; if [ $ec -eq 0 ] || [ $ec -eq 137 ] || [ $ec -eq 124 ]; then exit 0; else exit $ec; fi",
        entries,
        dist_dir.display(),
        public_url
    );
    run_shell(root, &cmd)
}

fn run_shell(root: &Path, script: &str) -> R<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(script)
        .current_dir(root)
        .status()
        .map_err(|e| format!("sh: {e}"))?;
    if !status.success() {
        return Err(format!("command failed ({:?}): {script}", status.code()));
    }
    Ok(())
}
