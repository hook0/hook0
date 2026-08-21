//! What a snippet is allowed to leave out, said in the snippet's own language.
//!
//! A documentation example is written for a reader: it skips the imports, assumes a client is
//! already built, and stops before the boilerplate. A harness is the rest of that file, written
//! once per language beside the pages, with a hole where the snippet goes. The page names the
//! region on the fence; the harness says what that region provides. Both are readable, and both
//! are compiled, so a harness that drifts from the client fails exactly like an example would.
//!
//! The markers are words inside whatever a language spells a comment with, which is what lets one
//! parser read a harness written in eleven languages without knowing any of them.

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::Error;
use crate::limits::{MAX_HARNESS_BYTES, MAX_REGIONS};
use crate::page::read_bounded;

/// The word that opens a region, followed by the name the page asks for it by.
pub const OPEN: &str = "HARNESS";

/// The words that close one.
pub const CLOSE: &str = "END HARNESS";

/// The word standing where the snippet is written.
pub const HOLE: &str = "EXAMPLE";

/// One region: the lines above the hole, the indentation of the hole, and the lines below it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub above: Vec<String>,
    pub indent: String,
    pub below: Vec<String>,
}

impl Region {
    /// The region with the snippet in its hole, indented to where the hole was.
    ///
    /// A blank line is left blank rather than filled with spaces, because trailing whitespace is
    /// what several of these languages' formatters exist to remove.
    pub fn fill(&self, snippet: &str) -> String {
        let mut out = String::new();
        for line in &self.above {
            out.push_str(line);
            out.push('\n');
        }
        for line in snippet.lines() {
            if !line.trim().is_empty() {
                out.push_str(&self.indent);
                out.push_str(line);
            }
            out.push('\n');
        }
        for line in &self.below {
            out.push_str(line);
            out.push('\n');
        }
        out
    }
}

/// Every region one harness file declares, by name.
pub type Regions = BTreeMap<String, Region>;

/// Reads one harness file.
pub fn read(path: &Path) -> Result<Regions, Error> {
    let body = read_bounded(path, "harness", MAX_HARNESS_BYTES)?;
    parse(&body, &path.display().to_string())
}

/// The regions the text declares.
pub fn parse(body: &str, path: &str) -> Result<Regions, Error> {
    let mut regions = Regions::new();
    let mut open: Option<(String, Vec<String>)> = None;

    for (index, line) in body.lines().enumerate() {
        if line.contains(CLOSE) {
            let Some((name, collected)) = open.take() else {
                return Err(Error::RegionNotOpened {
                    path: path.to_owned(),
                    line: index + 1,
                });
            };
            if regions.len() == MAX_REGIONS {
                return Err(Error::TooMany {
                    path: path.to_owned(),
                    what: "harness regions",
                    found: regions.len() + 1,
                    maximum: MAX_REGIONS,
                });
            }
            regions.insert(name.clone(), split(&collected, path, &name)?);
            continue;
        }

        if let Some(name) = opens(line) {
            if let Some((already, _)) = &open {
                return Err(Error::RegionNotClosed {
                    path: path.to_owned(),
                    region: already.clone(),
                });
            }
            if regions.contains_key(&name) {
                return Err(Error::RegionDeclaredTwice {
                    path: path.to_owned(),
                    region: name,
                });
            }
            open = Some((name, Vec::new()));
            continue;
        }

        if let Some((_, collected)) = &mut open {
            collected.push(line.to_owned());
        }
    }

    if let Some((name, _)) = open {
        return Err(Error::RegionNotClosed {
            path: path.to_owned(),
            region: name,
        });
    }
    if regions.is_empty() {
        return Err(Error::HarnessWithoutRegion {
            path: path.to_owned(),
        });
    }
    Ok(regions)
}

/// The name a line opens a region with, when it opens one.
fn opens(line: &str) -> Option<String> {
    let at = line.find(OPEN)?;
    let rest = line[at + OPEN.len()..].trim();
    // `END HARNESS` also carries the word, and is handled before this is reached; a line naming
    // nothing after it opens nothing either.
    let name = rest.split_whitespace().next()?;
    Some(name.to_owned())
}

/// The collected lines cut at the hole.
fn split(collected: &[String], path: &str, region: &str) -> Result<Region, Error> {
    let holes: Vec<usize> = collected
        .iter()
        .enumerate()
        .filter(|(_, line)| holds_hole(line))
        .map(|(index, _)| index)
        .collect();

    let [at] = holes[..] else {
        return Err(Error::RegionWithoutHole {
            path: path.to_owned(),
            region: region.to_owned(),
            found: holes.len(),
        });
    };

    let hole = &collected[at];
    let indent = hole[..hole.len() - hole.trim_start().len()].to_owned();

    Ok(Region {
        above: collected[..at].to_vec(),
        indent,
        below: collected[at + 1..].to_vec(),
    })
}

/// Whether the line is the hole, rather than a line that merely mentions the word.
///
/// The hole is a line whose only content besides a comment marker is the word, which is what keeps
/// a sentence about examples in a harness comment from being mistaken for one.
fn holds_hole(line: &str) -> bool {
    let trimmed = line.trim();
    let Some(at) = trimmed.find(HOLE) else {
        return false;
    };
    let after = trimmed[at + HOLE.len()..].trim();
    if !after.is_empty() {
        return false;
    }
    let before = trimmed[..at].trim();
    before.chars().all(|character| !character.is_alphanumeric())
}
