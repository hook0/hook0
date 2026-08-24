//! A security fix is filed as one because its commit says so, not because its prose mentions it.
//!
//! `cliff.toml` sorts commits into the sections a user reads, and a `fix` that closes a
//! vulnerability reads as an ordinary fix unless something separates the two. What separated them
//! was `body = ".*security"`, and running the pinned git-cliff over a repository written for it
//! showed that rule getting both halves wrong at once:
//!
//! - it is case-sensitive, so a body opening `Security impact: …` was filed under Fixed, and so was
//!   the reset-token fix, whose body never says the word at all;
//! - it is unanchored and reads prose, so `feat(api): answer every request with a set of security
//!   headers` — a commit whose own body says it is not a security fix — was the single entry the
//!   section had, taken out of Added to get there.
//!
//! A footer is what a commit uses to say something to a reader that is not human; it is how the
//! same file already reads a breaking change. So `Security:` is the mark, and this holds the shape
//! of that: one parser, reading the footer, forgiving the casing, and sitting where it does.
//!
//! Where it sits is not a preference. Moved below `^fix` the section empties and every commit
//! carrying the trailer goes back to Fixed. Moved above the two breaking parsers the Breaking
//! Changes section empties instead, a breaking commit that also carries the trailer landing here —
//! while `bump.rs` still demands a major for it, which is the disagreement `breaking_changes.rs`
//! exists to refuse. Both were measured, not reasoned about; editing these parsers means measuring
//! them again.

use std::fs;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

const SECURITY: &str = "Security";
const BREAKING: &str = "Breaking Changes";

fn parsers() -> Vec<Value> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("two directories above this crate")
        .join("cliff.toml");
    let body = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{} cannot be read: {err}", path.display()));
    let document: Table = body
        .parse()
        .unwrap_or_else(|err| panic!("{} is unreadable: {err}", path.display()));

    document["git"]["commit_parsers"]
        .as_array()
        .expect("the commit parsers are a list")
        .to_vec()
}

/// Where a parser filing into `group` sits, the first one if there is more than one.
fn position(parsers: &[Value], group: &str) -> usize {
    parsers
        .iter()
        .position(|parser| parser.get("group").and_then(Value::as_str) == Some(group))
        .unwrap_or_else(|| panic!("a parser files a commit under `{group}`"))
}

#[test]
fn a_security_fix_is_filed_by_a_trailer_rather_than_by_its_prose() {
    let parsers = parsers();
    let security: Vec<&Value> = parsers
        .iter()
        .filter(|parser| parser.get("group").and_then(Value::as_str) == Some(SECURITY))
        .collect();

    assert_eq!(
        security.len(),
        1,
        "`{SECURITY}` is filed by {} parsers. One mark keeps what lands there decided by the \
         commit's author; a second one is a second way in that nobody who writes a commit knows \
         about.",
        security.len()
    );

    let parser = security[0];
    assert!(
        parser.get("body").is_none(),
        "a parser files a security fix by its body. A body is prose written to explain a change to \
         a person, and reading it caught `feat(api): answer every request with a set of security \
         headers` while missing the vulnerability fix of the same release."
    );

    let footer = parser
        .get("footer")
        .and_then(Value::as_str)
        .expect("the parser reads the footer, which is where a commit says something deliberately");
    assert!(
        footer.contains("(?i)"),
        "the footer parser is `{footer}`, which is case-sensitive. That is the half of \
         `.*security` that made the section empty: a commit says `Security:` and a rule spelled \
         `security` does not see it."
    );
    assert!(
        footer.to_lowercase().contains("security"),
        "the footer parser is `{footer}`, which does not read a `Security:` trailer, so nothing \
         a commit can write reaches this section."
    );
}

#[test]
fn the_security_parser_sits_under_breaking_and_over_the_types() {
    let parsers = parsers();
    let security = position(&parsers, SECURITY);
    let breaking = position(&parsers, BREAKING);
    // Not simply the first parser reading a subject: the breaking one reads a subject too, and
    // taking it for the first type parser puts this boundary at the top of the list, where every
    // arrangement is on the wrong side of it and the assertion below holds nothing.
    let first_type = parsers
        .iter()
        .position(|parser| {
            parser.get("message").is_some()
                && parser.get("group").and_then(Value::as_str) != Some(BREAKING)
        })
        .expect("a parser files a commit by the type its subject declares");

    assert!(
        security > breaking,
        "the `{SECURITY}` parser is reached before the `{BREAKING}` ones. git-cliff stops at the \
         first parser that matches, so a breaking commit whose commit also carries the trailer is \
         filed as a security fix and the `{BREAKING}` section empties — while `bump.rs` reads the \
         same history and still demands a major, which is the release announcing one thing and \
         being numbered as another."
    );
    assert!(
        security < first_type,
        "the `{SECURITY}` parser is reached after the parsers that read the commit's type. `^fix` \
         matches first, so every security fix is filed under Fixed and this section stays empty — \
         which is the state it was already in."
    );
}
