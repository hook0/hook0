//! Markdown twins of the pages `llms.txt` lists, plus `llms-full.txt`.
//!
//! Answer engines cite what they can read cheaply. A rendered page is ~100 kB
//! of HTML for ~5 kB of text; the same text as Markdown is what they fetch when
//! a site offers it (the llms.txt convention: a page's Markdown lives at its URL
//! plus `.md`, and `llms-full.txt` carries every listed page in one file).
//!
//! Source of truth: the rendered HTML in `dist/`, i.e. the EJS templates fed by
//! the locale files. Nothing here holds page copy, so the Markdown cannot drift
//! from the page: every build regenerates it from what Parcel just wrote.
//!
//! Which pages: the ones `dist/llms.txt` links on this origin, in its order.
//! llms.txt is already the curated list of pages worth an assistant's time, so
//! there is no second list to keep in sync. An EN page listed there also gets
//! the `.md` twin of each FR/DE version that locales/slugs.js declares;
//! `llms-full.txt` itself stays English, like llms.txt.
//!
//! Only the `<main>` landmark is converted: header, footer and the mobile menu
//! sit outside it on every page (see includes/_header.ejs and _footer.ejs).
//! Inside it, decoration an assistant cannot use is dropped (inline SVG,
//! buttons, forms, breadcrumbs, `aria-hidden` subtrees, scripts).
//!
//! Std-only on purpose, like the rest of this crate: the HTML comes from our own
//! minified build, so a small tolerant tokenizer is enough and keeps the CI
//! install free of a parser dependency tree. It is not a general HTML parser:
//! unclosed `<p>`/`<li>` are tolerated because block starts flush anyway, but
//! nested tables are flattened into the outer cell, and table cells must be
//! closed (the minifier keeps `</td>`/`</th>`; the unit tests pin that shape).
//!
//! Each `.md` file also gets a Netlify `_headers` rule (Markdown content type
//! and a canonical `Link` to the HTML page), so search engines fold the twin
//! into the page instead of indexing it as a duplicate.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Writes `dist/<slug>.md` for every EN page `dist/llms.txt` links (and each
/// FR/DE version of it), `dist/llms-full.txt`, and the matching `_headers`
/// rules. Returns the number of Markdown files written.
///
/// No `dist/llms.txt` means nothing to mirror: returns `Ok(0)` and writes
/// nothing. A listed URL whose HTML is not in `dist/` is skipped here; the
/// i18n-gate is what fails the build on a dangling llms.txt link.
pub fn write_markdown_twins(
    dist: &Path,
    site_url: &str,
    locales: &[(String, String, String)],
    localized: &HashMap<String, HashMap<String, String>>,
) -> Result<usize, String> {
    let llms_path = dist.join("llms.txt");
    let Ok(llms) = fs::read_to_string(&llms_path) else {
        return Ok(0);
    };
    let locale_dirs: Vec<&str> = locales
        .iter()
        .map(|(_, d, _)| d.as_str())
        .filter(|d| !d.is_empty())
        .collect();
    let slugs = listed_en_slugs(&llms, site_url, &locale_dirs, dist);

    let mut full_sections: Vec<String> = Vec::new();
    let mut header_rules = String::new();
    let mut written = 0usize;

    for slug in &slugs {
        let url = page_url(site_url, "", slug);
        let md = page_file_markdown(&dist.join(format!("{slug}.html")), site_url, &url)?;
        let rel = format!("{slug}.md");
        fs::write(dist.join(&rel), &md).map_err(|e| format!("write {rel}: {e}"))?;
        header_rules.push_str(&header_rule(&rel, &url));
        full_sections.push(md);
        written += 1;

        // Same page in the other locales, under their translated slugs.
        let Some(per_lang) = localized.get(slug) else {
            continue;
        };
        let mut langs: Vec<(&String, &String)> = per_lang.iter().collect();
        langs.sort();
        for (lang, lslug) in langs {
            let Some((_, dir, _)) = locales.iter().find(|(l, d, _)| l == lang && !d.is_empty())
            else {
                continue;
            };
            let html = dist.join(dir).join(format!("{lslug}.html"));
            if !html.is_file() {
                continue;
            }
            let lurl = page_url(site_url, dir, lslug);
            let lmd = page_file_markdown(&html, site_url, &lurl)?;
            let lrel = format!("{dir}/{lslug}.md");
            fs::write(dist.join(&lrel), lmd).map_err(|e| format!("write {lrel}: {e}"))?;
            header_rules.push_str(&header_rule(&lrel, &lurl));
            written += 1;
        }
    }

    if slugs.is_empty() {
        return Ok(0);
    }

    let full = llms_full(&llms, site_url, &slugs, &full_sections);
    fs::write(dist.join("llms-full.txt"), full).map_err(|e| format!("write llms-full.txt: {e}"))?;

    // Appended, not overwritten: static/_headers (security headers, cache
    // rules) is already in dist/ at this point.
    let headers_path = dist.join("_headers");
    let mut headers = fs::read_to_string(&headers_path).unwrap_or_default();
    if !headers.is_empty() && !headers.ends_with('\n') {
        headers.push('\n');
    }
    headers.push_str(
        "\n# Generated by build-i18n (markdown.rs): Markdown twins of the pages llms.txt lists.\n",
    );
    headers.push_str(&header_rules);
    fs::write(&headers_path, headers).map_err(|e| format!("write _headers: {e}"))?;

    Ok(written)
}

/// EN page slugs linked from llms.txt on this origin, in order, deduplicated.
/// Skips other locales (`/fr/…`, `/de/…`), files (`/llms-full.txt`) and any
/// link whose `dist/<slug>.html` does not exist (the homepage maps to `index`).
fn listed_en_slugs(llms: &str, site_url: &str, locale_dirs: &[&str], dist: &Path) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for url in markdown_links(llms) {
        let Some(rest) = url.strip_prefix(site_url) else {
            continue;
        };
        let path = rest.split(['#', '?']).next().unwrap_or("").trim_matches('/');
        let first = path.split('/').next().unwrap_or("");
        if locale_dirs.contains(&first) || path.contains('.') {
            continue;
        }
        let slug = if path.is_empty() { "index" } else { path };
        if dist.join(format!("{slug}.html")).is_file() && !out.iter().any(|s| s == slug) {
            out.push(slug.to_string());
        }
    }
    out
}

/// Every `](url)` target in a Markdown document, in order.
fn markdown_links(md: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = md;
    while let Some(p) = rest.find("](") {
        let after = &rest[p + 2..];
        let Some(end) = after.find(')') else { break };
        out.push(after[..end].trim());
        rest = &after[end..];
    }
    out
}

fn page_url(site_url: &str, dir: &str, slug: &str) -> String {
    let slug = if slug == "index" { "" } else { slug };
    match (dir.is_empty(), slug.is_empty()) {
        (true, true) => format!("{site_url}/"),
        (true, false) => format!("{site_url}/{slug}"),
        (false, true) => format!("{site_url}/{dir}"),
        (false, false) => format!("{site_url}/{dir}/{slug}"),
    }
}

fn header_rule(rel: &str, canonical: &str) -> String {
    format!(
        "/{rel}\n  Content-Type: text/markdown; charset=utf-8\n  Link: <{canonical}>; rel=\"canonical\"\n"
    )
}

fn page_file_markdown(html_path: &Path, site_url: &str, page_url: &str) -> Result<String, String> {
    let html = fs::read_to_string(html_path)
        .map_err(|e| format!("read {}: {e}", html_path.display()))?;
    Ok(page_markdown(&html, site_url, page_url))
}

/// One page as Markdown: a `Source:` line with the canonical URL, then the
/// converted `<main>`.
pub fn page_markdown(html: &str, site_url: &str, page_url: &str) -> String {
    let body = to_markdown(main_region(html), site_url);
    format!("Source: {page_url}\n\n{body}")
}

/// `llms-full.txt`: a short header (reusing llms.txt's own title and summary
/// lines, so no copy is written twice), then each page separated by `---`.
fn llms_full(llms: &str, site_url: &str, slugs: &[String], sections: &[String]) -> String {
    let title = llms
        .lines()
        .find_map(|l| l.strip_prefix("# "))
        .unwrap_or("Hook0");
    let summary: Vec<&str> = llms
        .lines()
        .skip_while(|l| !l.starts_with("> "))
        .take_while(|l| l.starts_with('>'))
        .collect();
    let example = page_url(site_url, "", &slugs[0]);
    let mut s = format!("# {title}: full text\n\n");
    if !summary.is_empty() {
        s.push_str(&summary.join("\n"));
        s.push_str("\n\n");
    }
    s.push_str(&format!(
        "The English pages listed in {site_url}/llms.txt, as Markdown, regenerated from the live pages on every deploy. \
Each one is also served on its own at its URL plus .md ({example}.md). \
If this file and a page ever disagree, the page wins.\n"
    ));
    for sec in sections {
        s.push_str("\n---\n\n");
        s.push_str(sec.trim_end());
        s.push('\n');
    }
    s
}

/// The inside of `<main …>…</main>`, or the whole input when the page has no
/// main landmark.
pub fn main_region(html: &str) -> &str {
    let mut from = 0;
    while let Some(p) = html[from..].find("<main") {
        let at = from + p;
        let next = html.as_bytes().get(at + 5).copied().unwrap_or(b'>');
        if next == b'>' || next.is_ascii_whitespace() {
            let Some(gt) = html[at..].find('>') else { break };
            let start = at + gt + 1;
            let end = html.rfind("</main>").filter(|e| *e >= start).unwrap_or(html.len());
            return &html[start..end];
        }
        from = at + 5;
    }
    html
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

struct Tag {
    name: String,
    closing: bool,
    self_closing: bool,
    attrs: Vec<(String, String)>,
}

impl Tag {
    fn attr(&self, k: &str) -> Option<&str> {
        self.attrs.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str())
    }
}

const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// Elements whose text is never page copy; their content is skipped raw
/// (they may contain `<` that is not markup).
const RAW_SKIP: &[&str] = &["script", "style", "template", "textarea", "noscript"];

/// Elements dropped with their whole subtree: decoration or controls that
/// read as noise out of context ("Copy", menu toggles, form fields, icons,
/// the breadcrumb already given by the Source line).
const SUBTREE_SKIP: &[&str] = &["svg", "button", "form", "select", "iframe", "nav", "canvas", "video", "audio", "picture"];

/// Parses the tag starting at `html[i] == '<'`. Returns the tag and the index
/// just past its `>`, or `None` when this `<` does not open a tag. Attribute
/// values may be double-quoted, single-quoted or bare (the minifier drops
/// quotes where it can: `href=/pricing`).
fn parse_tag(html: &str, i: usize) -> Option<(Tag, usize)> {
    let b = html.as_bytes();
    let mut j = i + 1;
    let closing = b.get(j) == Some(&b'/');
    if closing {
        j += 1;
    }
    let name_start = j;
    while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'-') {
        j += 1;
    }
    if j == name_start || !b[name_start].is_ascii_alphabetic() {
        return None;
    }
    let name = html[name_start..j].to_ascii_lowercase();
    let mut attrs = Vec::new();
    let mut self_closing = false;
    loop {
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= b.len() {
            return None;
        }
        match b[j] {
            b'>' => return Some((Tag { name, closing, self_closing, attrs }, j + 1)),
            b'/' => {
                self_closing = true;
                j += 1;
                continue;
            }
            _ => {}
        }
        let ks = j;
        while j < b.len() && !b[j].is_ascii_whitespace() && !matches!(b[j], b'=' | b'>' | b'/') {
            j += 1;
        }
        let key = html[ks..j].to_ascii_lowercase();
        while j < b.len() && b[j].is_ascii_whitespace() {
            j += 1;
        }
        let mut val = String::new();
        if b.get(j) == Some(&b'=') {
            j += 1;
            while j < b.len() && b[j].is_ascii_whitespace() {
                j += 1;
            }
            match b.get(j) {
                Some(&q) if q == b'"' || q == b'\'' => {
                    let vs = j + 1;
                    let ve = html[vs..].find(q as char).map(|p| vs + p)?;
                    val = decode_entities(&html[vs..ve]);
                    j = ve + 1;
                }
                _ => {
                    let vs = j;
                    while j < b.len() && !b[j].is_ascii_whitespace() && b[j] != b'>' {
                        j += 1;
                    }
                    val = decode_entities(&html[vs..j]);
                }
            }
        }
        if !key.is_empty() {
            attrs.push((key, val));
        }
    }
}

/// Converts an HTML fragment to Markdown. `site_url` absolutizes root-relative
/// links so the output still works once copied out of the site.
pub fn to_markdown(html: &str, site_url: &str) -> String {
    let mut w = Writer::new(site_url);
    let b = html.as_bytes();
    let mut i = 0;
    while i < html.len() {
        if b[i] == b'<' {
            if html[i..].starts_with("<!--") {
                i = html[i + 4..].find("-->").map_or(html.len(), |p| i + 4 + p + 3);
                continue;
            }
            if html[i..].starts_with("<!") || html[i..].starts_with("<?") {
                i = html[i..].find('>').map_or(html.len(), |p| i + p + 1);
                continue;
            }
            if let Some((tag, end)) = parse_tag(html, i) {
                i = end;
                if !tag.closing && RAW_SKIP.contains(&tag.name.as_str()) {
                    let close = format!("</{}", tag.name);
                    i = find_ci(html, i, &close)
                        .and_then(|p| html[p..].find('>').map(|g| p + g + 1))
                        .unwrap_or(html.len());
                    continue;
                }
                w.tag(&tag);
                continue;
            }
            w.text("<");
            i += 1;
            continue;
        }
        let next = html[i..].find('<').map_or(html.len(), |p| i + p);
        w.text(&decode_entities(&html[i..next]));
        i = next;
    }
    w.finish()
}

fn find_ci(hay: &str, from: usize, needle: &str) -> Option<usize> {
    let h = hay.as_bytes();
    let n = needle.as_bytes();
    (from..h.len().saturating_sub(n.len() - 1)).find(|&k| h[k..k + n.len()].eq_ignore_ascii_case(n))
}

fn decode_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(p) = rest.find('&') {
        out.push_str(&rest[..p]);
        let tail = &rest[p..];
        let semi = tail[..tail.len().min(12)].find(';');
        let decoded = semi.and_then(|e| entity(&tail[1..e]).map(|c| (c, e)));
        match decoded {
            Some((c, e)) => {
                out.push(c);
                rest = &tail[e + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn entity(name: &str) -> Option<char> {
    if let Some(num) = name.strip_prefix('#') {
        let n = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse().ok()?,
        };
        return char::from_u32(n);
    }
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => ' ',
        "rsquo" => '’',
        "lsquo" => '‘',
        "rdquo" => '”',
        "ldquo" => '“',
        "mdash" => '—',
        "ndash" => '–',
        "hellip" => '…',
        "euro" => '€',
        "copy" => '©',
        "reg" => '®',
        "trade" => '™',
        "times" => '×',
        "rarr" => '→',
        "larr" => '←',
        "middot" => '·',
        "bull" => '•',
        "laquo" => '«',
        "raquo" => '»',
        "check" => '✓',
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Writer
// ---------------------------------------------------------------------------

/// Accumulates Markdown blocks. Text goes to the innermost capture (an open
/// link or table cell) when there is one, else to the current block line.
/// A block is written out only when it has text, so empty wrappers vanish.
struct Writer<'a> {
    site_url: &'a str,
    out: String,
    line: String,
    prefix: String,
    line_is_li: bool,
    last_was_li: bool,
    caps: Vec<String>,
    links: Vec<Option<String>>,
    lists: Vec<(bool, u32)>,
    li_depth: usize,
    rows: Vec<Vec<String>>,
    row: Vec<String>,
    caption: Option<String>,
    table_depth: usize,
    pre: usize,
    skip_name: String,
    skip_depth: usize,
}

impl<'a> Writer<'a> {
    fn new(site_url: &'a str) -> Self {
        Writer {
            site_url,
            out: String::new(),
            line: String::new(),
            prefix: String::new(),
            line_is_li: false,
            last_was_li: false,
            caps: Vec::new(),
            links: Vec::new(),
            lists: Vec::new(),
            li_depth: 0,
            rows: Vec::new(),
            row: Vec::new(),
            caption: None,
            table_depth: 0,
            pre: 0,
            skip_name: String::new(),
            skip_depth: 0,
        }
    }

    fn target(&mut self) -> &mut String {
        match self.caps.last_mut() {
            Some(c) => c,
            None => &mut self.line,
        }
    }

    fn text(&mut self, s: &str) {
        if self.skip_depth > 0 {
            return;
        }
        if self.pre > 0 {
            self.line.push_str(s);
            return;
        }
        let t = self.target();
        for ch in s.chars() {
            if ch.is_whitespace() {
                if !t.is_empty() && !t.ends_with([' ', '\n']) {
                    t.push(' ');
                }
            } else {
                t.push(ch);
            }
        }
    }

    fn space(&mut self) {
        self.text(" ");
    }

    /// Ends the current block. Inside a link, a cell or a list item, a block
    /// boundary is only a space: the enclosing construct owns the line.
    fn soft_block(&mut self) {
        if !self.caps.is_empty() || self.li_depth > 0 || self.table_depth > 0 {
            self.space();
        } else {
            self.flush();
        }
    }

    fn flush(&mut self) {
        let text = self.line.trim().to_string();
        if !text.is_empty() {
            self.push_block(&format!("{}{}", self.prefix, text), self.line_is_li);
        }
        self.line.clear();
        self.prefix.clear();
        self.line_is_li = false;
    }

    fn push_block(&mut self, block: &str, is_li: bool) {
        if !self.out.is_empty() {
            self.out.push_str(if is_li && self.last_was_li { "\n" } else { "\n\n" });
        }
        self.out.push_str(block);
        self.last_was_li = is_li;
    }

    fn resolve(&self, href: &str) -> Option<String> {
        let h = href.trim();
        if h.is_empty() || h.starts_with('#') || h.starts_with("javascript:") {
            None
        } else if h.starts_with("https://") || h.starts_with("http://") || h.starts_with("mailto:") {
            Some(h.to_string())
        } else if let Some(r) = h.strip_prefix("//") {
            Some(format!("https://{r}"))
        } else if h.starts_with('/') {
            Some(format!("{}{h}", self.site_url))
        } else {
            Some(format!("{}/{h}", self.site_url))
        }
    }

    fn tag(&mut self, t: &Tag) {
        let name = t.name.as_str();
        let void = VOID.contains(&name) || t.self_closing;

        // Inside a dropped subtree: only track nesting of the same element.
        if self.skip_depth > 0 {
            if name == self.skip_name && !void {
                if t.closing {
                    self.skip_depth -= 1;
                } else {
                    self.skip_depth += 1;
                }
            }
            return;
        }
        if !t.closing && !void {
            let hidden = t.attr("aria-hidden") == Some("true") || t.attr("hidden").is_some();
            if hidden || SUBTREE_SKIP.contains(&name) {
                self.skip_name = name.to_string();
                self.skip_depth = 1;
                return;
            }
        }

        if self.pre > 0 && name != "pre" {
            if name == "br" {
                self.line.push('\n');
            }
            return;
        }

        match (name, t.closing) {
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", false) => {
                self.soft_block();
                if self.caps.is_empty() && self.li_depth == 0 && self.table_depth == 0 {
                    let level = (name.as_bytes()[1] - b'0') as usize;
                    self.prefix = format!("{} ", "#".repeat(level));
                }
            }
            ("h1" | "h2" | "h3" | "h4" | "h5" | "h6", true) => self.soft_block(),

            ("ul" | "ol", false) => {
                if self.table_depth == 0 && self.caps.is_empty() {
                    self.flush();
                }
                self.lists.push((name == "ol", 1));
            }
            ("ul" | "ol", true) => {
                self.lists.pop();
                if self.table_depth == 0 && self.caps.is_empty() {
                    self.flush();
                }
            }
            ("li", false) => {
                if self.table_depth > 0 || !self.caps.is_empty() {
                    self.space();
                    return;
                }
                self.flush();
                self.li_depth += 1;
                let depth = self.lists.len().max(1);
                let marker = match self.lists.last_mut() {
                    Some((true, n)) => {
                        let m = format!("{n}. ");
                        *n += 1;
                        m
                    }
                    _ => "- ".to_string(),
                };
                self.prefix = format!("{}{marker}", "  ".repeat(depth - 1));
                self.line_is_li = true;
            }
            ("li", true) => {
                if self.table_depth > 0 || !self.caps.is_empty() {
                    self.space();
                    return;
                }
                self.flush();
                self.li_depth = self.li_depth.saturating_sub(1);
            }

            ("table", false) => {
                if self.table_depth == 0 {
                    self.flush();
                    self.rows.clear();
                    self.row.clear();
                    self.caption = None;
                }
                self.table_depth += 1;
            }
            ("table", true) if self.table_depth > 0 => {
                self.table_depth -= 1;
                if self.table_depth == 0 {
                    self.emit_table();
                }
            }
            ("caption", false) if self.table_depth == 1 => self.caps.push(String::new()),
            ("caption", true) if self.table_depth == 1 => {
                self.caption = self.caps.pop().map(|c| c.trim().to_string());
            }
            ("tr", false) if self.table_depth == 1 => self.row.clear(),
            ("tr", true) if self.table_depth == 1 => {
                let row = std::mem::take(&mut self.row);
                if !row.is_empty() {
                    self.rows.push(row);
                }
            }
            ("td" | "th", false) if self.table_depth == 1 => self.caps.push(String::new()),
            ("td" | "th", true) if self.table_depth == 1 => {
                let cell = self.caps.pop().unwrap_or_default();
                self.row.push(cell.trim().replace('|', "\\|").replace('\n', " "));
            }

            ("a", false) => {
                let href = t.attr("href").and_then(|h| self.resolve(h));
                self.links.push(href);
                self.caps.push(String::new());
            }
            ("a", true) => {
                let (Some(href), Some(text)) = (self.links.pop(), self.caps.pop()) else {
                    return;
                };
                let text = text.trim().replace('[', "\\[").replace(']', "\\]");
                if text.is_empty() {
                    return;
                }
                let piece = match href {
                    Some(h) => format!("[{text}]({h})"),
                    None => text,
                };
                let tgt = self.target();
                tgt.push_str(&piece);
            }

            ("pre", false) => {
                self.soft_block();
                if self.caps.is_empty() && self.li_depth == 0 && self.table_depth == 0 {
                    self.pre += 1;
                }
            }
            ("pre", true) if self.pre > 0 => {
                self.pre -= 1;
                let code = std::mem::take(&mut self.line);
                let code = code.trim_matches('\n');
                if !code.trim().is_empty() {
                    self.push_block(&format!("```\n{code}\n```"), false);
                }
            }
            ("code", _) => self.target().push('`'),

            ("br", _) => {
                if self.caps.is_empty() && self.li_depth == 0 && self.table_depth == 0 {
                    self.line.push('\n');
                } else {
                    self.space();
                }
            }
            ("img", _) => {
                if let Some(alt) = t.attr("alt").map(str::trim).filter(|a| !a.is_empty()) {
                    let alt = alt.to_string();
                    self.space();
                    self.text(&alt);
                    self.space();
                }
            }
            (
                "p" | "div" | "section" | "article" | "header" | "footer" | "aside" | "figure"
                | "figcaption" | "blockquote" | "dl" | "dt" | "dd" | "details" | "summary"
                | "address" | "fieldset" | "legend" | "hr",
                _,
            ) => self.soft_block(),
            _ => {}
        }
    }

    fn emit_table(&mut self) {
        let rows = std::mem::take(&mut self.rows);
        let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
        if cols == 0 {
            return;
        }
        let fmt_row = |r: &Vec<String>| {
            let mut cells: Vec<&str> = r.iter().map(String::as_str).collect();
            cells.resize(cols, "");
            format!("| {} |", cells.join(" | "))
        };
        let mut s = String::new();
        if let Some(c) = self.caption.take().filter(|c| !c.is_empty()) {
            s.push_str(&c);
            s.push_str("\n\n");
        }
        s.push_str(&fmt_row(&rows[0]));
        s.push('\n');
        s.push_str(&format!("|{}", " --- |".repeat(cols)));
        for r in &rows[1..] {
            s.push('\n');
            s.push_str(&fmt_row(r));
        }
        self.push_block(&s, false);
    }

    fn finish(mut self) -> String {
        self.flush();
        let mut s = self.out.trim().to_string();
        s.push('\n');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SITE: &str = "https://www.hook0.com";

    #[test]
    fn main_region_takes_only_the_main_landmark() {
        let html = "<header>menu</header><main id=main class=x><p>body</p></main><footer>f</footer>";
        assert_eq!(main_region(html), "<p>body</p>");
        assert_eq!(main_region("<p>no main</p>"), "<p>no main</p>");
        // <mainfoo> is not <main>
        assert_eq!(main_region("<mainfoo>x</mainfoo>"), "<mainfoo>x</mainfoo>");
    }

    #[test]
    fn headings_paragraphs_and_whitespace() {
        let md = to_markdown(
            "<section><h1 class=t>Hook0 vs Svix <span class=block>Comparison</span></h1>\n  <p>Open   source,\n EU-hosted.</p></section>",
            SITE,
        );
        assert_eq!(md, "# Hook0 vs Svix Comparison\n\nOpen source, EU-hosted.\n");
    }

    #[test]
    fn links_are_absolutized_and_anchors_dropped() {
        let md = to_markdown(
            "<p>See <a href=/pricing class=x>pricing</a>, <a href='#faq'>FAQ</a> and <a href=\"https://svix.com/\">Svix</a>.</p>",
            SITE,
        );
        assert_eq!(
            md,
            "See [pricing](https://www.hook0.com/pricing), FAQ and [Svix](https://svix.com/).\n"
        );
    }

    #[test]
    fn decoration_and_controls_are_dropped() {
        let md = to_markdown(
            "<nav aria-label=Breadcrumb><a href=/>Home</a></nav>\
             <script type=application/ld+json>{\"a\":\"<b>\"}</script>\
             <p>Start<svg viewBox=\"0 0 1 1\"><path d=x /></svg></p>\
             <button>Copy</button><div aria-hidden=true><p>deco</p><div>x</div></div><p>kept</p>",
            SITE,
        );
        assert_eq!(md, "Start\n\nkept\n");
    }

    #[test]
    fn lists_nested_and_ordered() {
        let md = to_markdown(
            "<ul><li><p>One</p></li><li>Two<ol><li>a</li><li>b</li></ol></li></ul><p>after</p>",
            SITE,
        );
        assert_eq!(md, "- One\n- Two\n  1. a\n  2. b\n\nafter\n");
    }

    #[test]
    fn tables_become_pipe_tables() {
        let md = to_markdown(
            "<table><thead><tr><th>Feature</th><th>Hook0</th><th>Svix</th></tr></thead>\
             <tbody><tr><td>License</td><td><a href=/open-source-webhooks>SSPL-1.0</a></td><td>MIT | EE</td></tr>\
             <tr><td>EU</td><td><svg></svg>Yes</td></tr></tbody></table>",
            SITE,
        );
        assert_eq!(
            md,
            "| Feature | Hook0 | Svix |\n| --- | --- | --- |\n\
             | License | [SSPL-1.0](https://www.hook0.com/open-source-webhooks) | MIT \\| EE |\n\
             | EU | Yes |  |\n"
        );
    }

    #[test]
    fn entities_and_code() {
        let md = to_markdown(
            "<p>R&amp;D &#8212; &euro;19&nbsp;/mo &bogus; <code>X-Hook0-Signature</code></p><pre><code>curl -X POST\n  https://x</code></pre>",
            SITE,
        );
        assert_eq!(
            md,
            "R&D — €19 /mo &bogus; `X-Hook0-Signature`\n\n```\ncurl -X POST\n  https://x\n```\n"
        );
    }

    #[test]
    fn image_alt_text_is_kept() {
        let md = to_markdown(
            "<p>Trusted by</p><div><img src=/a.png alt=Acme><img src=/b.png alt=\"\"><img src=/c.png alt=Globex></div>",
            SITE,
        );
        assert_eq!(md, "Trusted by\n\nAcme Globex\n");
    }

    #[test]
    fn listed_slugs_follow_llms_order_and_skip_locales_and_files() {
        let dir = std::env::temp_dir().join(format!("md-twins-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("fr")).unwrap();
        for f in ["hook0-vs-svix.html", "pricing.html", "fr/tarifs.html"] {
            fs::write(dir.join(f), "<main><h1>T</h1></main>").unwrap();
        }
        let llms = "# Hook0\n\n> Summary line.\n\n- [Svix](https://www.hook0.com/hook0-vs-svix): x\n\
                    - [Pricing](https://www.hook0.com/pricing)\n- [Again](https://www.hook0.com/hook0-vs-svix)\n\
                    - [FR](https://www.hook0.com/fr/tarifs)\n- [Full](https://www.hook0.com/llms-full.txt)\n\
                    - [Gone](https://www.hook0.com/missing)\n- [Docs](https://documentation.hook0.com/)\n";
        let slugs = listed_en_slugs(llms, SITE, &["fr", "de"], &dir);
        assert_eq!(slugs, vec!["hook0-vs-svix", "pricing"]);

        fs::write(dir.join("llms.txt"), llms).unwrap();
        fs::write(dir.join("_headers"), "/*\n  X-Frame-Options: SAMEORIGIN\n").unwrap();
        let locales = vec![
            ("en".to_string(), String::new(), "/".to_string()),
            ("fr".to_string(), "fr".to_string(), "/fr".to_string()),
        ];
        let mut localized: HashMap<String, HashMap<String, String>> = HashMap::new();
        localized.insert(
            "pricing".into(),
            HashMap::from([("fr".to_string(), "tarifs".to_string())]),
        );
        let n = write_markdown_twins(&dir, SITE, &locales, &localized).unwrap();
        assert_eq!(n, 3);
        assert_eq!(
            fs::read_to_string(dir.join("fr/tarifs.md")).unwrap(),
            "Source: https://www.hook0.com/fr/tarifs\n\n# T\n"
        );
        let full = fs::read_to_string(dir.join("llms-full.txt")).unwrap();
        assert!(full.starts_with("# Hook0: full text\n\n> Summary line.\n\n"));
        assert!(full.contains("(https://www.hook0.com/hook0-vs-svix.md)"));
        assert!(full.contains("\n---\n\nSource: https://www.hook0.com/pricing\n\n# T\n"));
        assert!(!full.contains("fr/tarifs"));
        let headers = fs::read_to_string(dir.join("_headers")).unwrap();
        assert!(headers.starts_with("/*\n  X-Frame-Options: SAMEORIGIN\n"));
        assert!(headers.contains(
            "/fr/tarifs.md\n  Content-Type: text/markdown; charset=utf-8\n  Link: <https://www.hook0.com/fr/tarifs>; rel=\"canonical\"\n"
        ));
        let _ = fs::remove_dir_all(&dir);
    }
}
