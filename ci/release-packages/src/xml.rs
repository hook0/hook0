//! Enough of XML to read a `pom.xml` and a `.csproj`, and to say where in the bytes an answer came
//! from.
//!
//! Depth is the whole point. A `pom.xml` declares `<version>` for the artefact it *is* and again
//! for every artefact it depends on, and the two are told apart by nothing but how deep they sit,
//! so a scan that matched on the element name alone would publish a package under the version of
//! whichever dependency happened to be listed first. What is here therefore tracks nesting rather
//! than searching text, and answers with a byte range so the writer can replace exactly what the
//! reader saw.

use std::ops::Range;

/// What an element the scan matched holds, and where those bytes are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub text: String,
    pub span: Range<usize>,
}

/// The elements at `path` under the document's root element, in the order they appear.
///
/// `path` is matched against the element names below the root — `["version"]` reads the artefact's
/// own version out of a `pom.xml`, and `["PropertyGroup", "Version"]` reads a project property out
/// of a `.csproj`. Only elements holding text are answered; one holding children is skipped, since
/// nothing this reads is ever both.
///
/// Attributes, namespace prefixes, comments, processing instructions, doctypes and CDATA are
/// stepped over rather than interpreted: none of them carries anything a package is named or
/// versioned by, and reading them would be a parser rather than the reader this is.
pub fn elements_at(document: &str, path: &[&str]) -> Vec<Element> {
    let bytes = document.as_bytes();
    let mut found = Vec::new();
    // What is currently open, root included, so `stack[1..]` is what `path` is compared against.
    let mut stack: Vec<&str> = Vec::new();
    let mut at = 0usize;

    while at < bytes.len() {
        let Some(open) = document[at..].find('<').map(|i| at + i) else {
            break;
        };

        if let Some(after) = skip_non_element(document, open) {
            at = after;
            continue;
        }

        let Some(close) = tag_end(document, open) else {
            break;
        };
        let tag = &document[open + 1..close];
        at = close + 1;

        if let Some(name) = tag.strip_prefix('/') {
            if stack.last() == Some(&local_name(name.trim())) {
                stack.pop();
            }
            continue;
        }

        // `<foo/>` opens and closes at once, so it can hold no text and changes no depth.
        if tag.ends_with('/') {
            continue;
        }

        let name = local_name(tag);
        stack.push(name);

        // The root is the frame `path` is relative to, so only what sits below it is compared.
        let below_root = &stack[1.min(stack.len())..];
        if below_root != path {
            continue;
        }

        // The text runs to the next tag; a child there means this element holds elements rather
        // than a value, and there is nothing to read.
        let Some(next) = document[at..].find('<').map(|i| at + i) else {
            break;
        };
        if !document[next..].starts_with(&format!("</{name}")) {
            continue;
        }
        found.push(Element {
            text: document[at..next].trim().to_string(),
            span: at..next,
        });
    }

    found
}

/// Where the tag opened at `open` closes, stepping over any `>` sitting inside an attribute value.
fn tag_end(document: &str, open: usize) -> Option<usize> {
    let mut quote = None;
    for (offset, c) in document[open..].char_indices() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '>') => return Some(open + offset),
            (None, _) => {}
        }
    }
    None
}

/// Where a comment, a processing instruction, a doctype or a CDATA section opened at `at` ends, or
/// nothing if what opened there is an element.
fn skip_non_element(document: &str, at: usize) -> Option<usize> {
    for (opener, closer) in [
        ("<!--", "-->"),
        ("<![CDATA[", "]]>"),
        ("<?", "?>"),
        ("<!", ">"),
    ] {
        if document[at..].starts_with(opener) {
            let from = at + opener.len();
            return Some(match document[from..].find(closer) {
                Some(i) => from + i + closer.len(),
                None => document.len(),
            });
        }
    }
    None
}

/// An element's name without its attributes and without the namespace prefix it may carry.
fn local_name(tag: &str) -> &str {
    let name = tag
        .split(|c: char| c.is_whitespace() || c == '/')
        .next()
        .unwrap_or(tag);
    match name.rsplit_once(':') {
        Some((_, local)) => local,
        None => name,
    }
}
