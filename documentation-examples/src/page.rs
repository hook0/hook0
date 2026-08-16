//! What a documentation page says about itself and what it shows.
//!
//! Two things are read off a page and nothing else: the target it claims in its front matter, and
//! its fenced blocks. A block is an *example* when its language is one a generated target answers
//! to — which is a question asked of the generator's registry, never of a list kept here — and an
//! example says which harness region it drops into on the fence itself, where the person editing
//! the page can see it.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::limits::{MAX_EXAMPLE_BYTES, MAX_EXAMPLES_PER_PAGE, MAX_PAGE_BYTES};

/// The front-matter key a page names its target with.
pub const TARGET_KEY: &str = "sdkTarget";

/// The value that says, on the page, that the page documents no single client.
pub const NO_TARGET: &str = "none";

/// The fence attribute an example names its harness region with.
pub const REGION_ATTRIBUTE: &str = "example";

/// One fenced example, and where a reader would find it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    /// The page it is shown on, relative to the documentation root.
    pub page: String,
    /// The line the fence opens on, 1-based, so the message is a place to click.
    pub line: usize,
    /// The language of the fence, which is the name of the target that claims it.
    pub language: String,
    /// The harness region it is assembled into.
    pub region: String,
    pub body: String,
}

impl Example {
    /// How the example is spelled in a report: a place, then what it is.
    pub fn locate(&self) -> String {
        format!(
            "{}:{} ({}, {}={})",
            self.page, self.line, self.language, REGION_ATTRIBUTE, self.region
        )
    }
}

/// One page of the SDK reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub path: PathBuf,
    /// How the page is named in a report, relative to the documentation root.
    pub name: String,
    /// The target the page documents, absent when the page says it documents no single client.
    pub target: Option<String>,
    pub examples: Vec<Example>,
}

/// Reads one page, given the languages a target answers to.
///
/// `languages` decides what counts as an example, so a page is read differently the day a target
/// is added — which is the point: the twelfth client's examples are proven without this file
/// changing.
pub fn read(path: &Path, name: &str, languages: &[String]) -> Result<Page, Error> {
    let body = read_bounded(path, "page", MAX_PAGE_BYTES)?;
    let target = read_target(&body);
    let examples = read_examples(&body, name, languages)?;

    Ok(Page {
        path: path.to_path_buf(),
        name: name.to_owned(),
        target,
        examples,
    })
}

/// The file, refused rather than truncated when it is above the ceiling.
pub fn read_bounded(path: &Path, what: &'static str, maximum: u64) -> Result<String, Error> {
    let size = fs::metadata(path)
        .map_err(|cause| Error::ReadFile {
            path: path.display().to_string(),
            cause,
        })?
        .len();
    if size > maximum {
        return Err(Error::FileTooLarge {
            path: path.display().to_string(),
            what,
            size,
            maximum,
        });
    }
    fs::read_to_string(path).map_err(|cause| Error::ReadFile {
        path: path.display().to_string(),
        cause,
    })
}

/// The target the front matter claims, if it claims one.
///
/// The front matter is read for one key rather than parsed as a document: this crate has one
/// question to ask of it, and a YAML parser here would be a dependency carried for a single line.
fn read_target(body: &str) -> Option<String> {
    let mut lines = body.lines();
    if lines.next().map(str::trim) != Some("---") {
        return None;
    }
    for line in lines {
        if line.trim() == "---" {
            return None;
        }
        let Some(value) = line.strip_prefix(TARGET_KEY) else {
            continue;
        };
        let Some(value) = value.strip_prefix(':') else {
            continue;
        };
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if value.is_empty() {
            continue;
        }
        return Some(value.to_owned());
    }
    None
}

/// Every fenced block of the page that is an example, refusing the ones that would go unproven.
fn read_examples(body: &str, page: &str, languages: &[String]) -> Result<Vec<Example>, Error> {
    let lines: Vec<&str> = body.lines().collect();
    let mut examples = Vec::new();

    let mut at = 0;
    while at < lines.len() {
        let Some(fence) = opening_fence(lines[at]) else {
            at += 1;
            continue;
        };
        let opened = at + 1;
        let mut collected: Vec<&str> = Vec::new();
        let mut closed = false;
        at += 1;
        while at < lines.len() {
            if is_closing_fence(lines[at], fence.marker) {
                closed = true;
                at += 1;
                break;
            }
            collected.push(lines[at]);
            at += 1;
        }
        if !closed {
            return Err(Error::UnclosedFence {
                page: page.to_owned(),
                line: opened,
            });
        }

        let claimed = languages.iter().any(|known| known == fence.language);
        let region = fence.region(page, opened)?;

        let region = match (claimed, region) {
            (true, Some(region)) => region,
            (true, None) => {
                return Err(Error::ExampleWithoutRegion {
                    page: page.to_owned(),
                    line: opened,
                    language: fence.language.to_owned(),
                });
            }
            (false, Some(_)) => {
                return Err(Error::ExampleInUnclaimedLanguage {
                    page: page.to_owned(),
                    line: opened,
                    language: fence.language.to_owned(),
                    known: languages.join(", "),
                });
            }
            (false, None) => continue,
        };

        let body = collected.join("\n");
        if body.len() > MAX_EXAMPLE_BYTES {
            return Err(Error::ExampleTooLarge {
                page: page.to_owned(),
                line: opened,
                language: fence.language.to_owned(),
                size: body.len(),
                maximum: MAX_EXAMPLE_BYTES,
            });
        }
        if examples.len() == MAX_EXAMPLES_PER_PAGE {
            return Err(Error::TooMany {
                path: page.to_owned(),
                what: "examples",
                found: examples.len() + 1,
                maximum: MAX_EXAMPLES_PER_PAGE,
            });
        }

        examples.push(Example {
            page: page.to_owned(),
            line: opened,
            language: fence.language.to_owned(),
            region,
            body,
        });
    }

    Ok(examples)
}

/// The opening line of a fence, taken apart.
struct Fence<'a> {
    marker: char,
    language: &'a str,
    attributes: &'a str,
}

impl Fence<'_> {
    /// The harness region the fence names, refusing an attribute that means nothing.
    fn region(&self, page: &str, line: usize) -> Result<Option<String>, Error> {
        let mut region = None;
        for word in self.attributes.split_whitespace() {
            let Some(value) = word.strip_prefix(REGION_ATTRIBUTE) else {
                return Err(Error::UnknownAttribute {
                    page: page.to_owned(),
                    line,
                    attribute: word.to_owned(),
                });
            };
            let Some(value) = value.strip_prefix('=') else {
                return Err(Error::UnknownAttribute {
                    page: page.to_owned(),
                    line,
                    attribute: word.to_owned(),
                });
            };
            if value.is_empty() {
                return Err(Error::UnknownAttribute {
                    page: page.to_owned(),
                    line,
                    attribute: word.to_owned(),
                });
            }
            region = Some(value.to_owned());
        }
        Ok(region)
    }
}

/// The line, when it opens a fence.
fn opening_fence(line: &str) -> Option<Fence<'_>> {
    let marker = if line.starts_with("```") {
        '`'
    } else if line.starts_with("~~~") {
        '~'
    } else {
        return None;
    };
    let info = line.trim_start_matches(marker).trim();
    let (language, attributes) = match info.split_once(char::is_whitespace) {
        Some((language, attributes)) => (language, attributes.trim()),
        None => (info, ""),
    };
    Some(Fence {
        marker,
        language,
        attributes,
    })
}

fn is_closing_fence(line: &str, marker: char) -> bool {
    let trimmed = line.trim_end();
    trimmed.len() >= 3 && trimmed.chars().all(|character| character == marker)
}
