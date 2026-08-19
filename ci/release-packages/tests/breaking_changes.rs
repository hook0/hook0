//! The changelog and the version number agree on what broke.
//!
//! `bump.rs` reads the history to decide how small a release may be, and Conventional Commits give
//! it two ways to say something broke. One is a `!` immediately before the colon, the other a
//! `BREAKING CHANGE:` footer in either of its two spellings. `cliff.toml` reads the same history to write the changelog
//! a user reads. When the two disagree, a release announces one thing and is numbered as another.
//!
//! They did disagree, in both directions, and both were found by running the pinned git-cliff over
//! a repository written for it rather than by reading the file:
//!
//! - `^.*!` matched an exclamation mark anywhere in the subject, so `feat(api): let a caller say no!`
//!   was published under Breaking Changes while the release stayed a minor.
//! - `body = "BREAKING CHANGE"` matched nothing at all. With `conventional_commits = true` a footer
//!   is not part of the body, so a commit that broke the wire through its footer was published under
//!   Changed while `required-bump` correctly demanded a major for it.
//!
//! What is asserted below is the shape of the fix rather than its behaviour, because git-cliff is
//! not in the image this test runs in. It is deliberately narrow, catching the two mistakes that
//! were actually made and saying what to do about a third. Editing these parsers means running
//! git-cliff over the four cases again.

use std::fs;
use std::path::{Path, PathBuf};

use toml::{Table, Value};

/// The group a parser puts a breaking commit in.
const BREAKING: &str = "Breaking Changes";

fn cliff() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("two directories above this crate")
        .join("cliff.toml")
}

/// The parsers that file a commit as breaking.
fn breaking_parsers() -> Vec<Value> {
    let path = cliff();
    let body = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("{} cannot be read: {err}", path.display()));
    let document: Table = body
        .parse()
        .unwrap_or_else(|err| panic!("{} is unreadable: {err}", path.display()));

    document["git"]["commit_parsers"]
        .as_array()
        .expect("the commit parsers are a list")
        .iter()
        .filter(|parser| parser.get("group").and_then(Value::as_str) == Some(BREAKING))
        .cloned()
        .collect()
}

#[test]
fn a_breaking_commit_is_filed_by_the_two_marks_that_make_one() {
    let parsers = breaking_parsers();
    assert_eq!(
        parsers.len(),
        2,
        "`{BREAKING}` is filed by {} parsers rather than the two the specification gives. If a \
         third way to declare a break has been added, `ci/release-packages/src/bump.rs` has to \
         read it too, or a release will be numbered as though it never happened.",
        parsers.len()
    );

    let subject = parsers
        .iter()
        .filter_map(|parser| parser.get("message").and_then(Value::as_str))
        .next()
        .expect("one parser reads the subject");
    assert!(
        subject.contains("!:"),
        "the subject parser is `{subject}`, which does not bind the `!` to the colon. `^.*!` \
         matched an exclamation mark anywhere, so `feat(api): let a caller say no!` was published \
         as a breaking change while the release stayed a minor."
    );

    let footer = parsers
        .iter()
        .filter_map(|parser| parser.get("footer").and_then(Value::as_str))
        .next()
        .expect(
            "one parser reads the footer. `body` does not: with `conventional_commits = true` a \
             footer is not part of the body, so a body parser matches nothing and a commit that \
             breaks the wire through its footer is published as an ordinary change.",
        );
    for spelling in ["BREAKING CHANGE", "BREAKING-CHANGE"] {
        let separator = &spelling["BREAKING".len().."BREAKING".len() + 1];
        assert!(
            footer.contains(separator) || footer.contains(spelling),
            "the footer parser is `{footer}`, which does not read `{spelling}`. `bump.rs` reads \
             both spellings, so a release can be a major for a footer the changelog never mentions."
        );
    }

    assert!(
        !parsers.iter().any(|parser| parser.get("body").is_some()),
        "a parser files a breaking commit by its body. That matches nothing here, since \
         `conventional_commits = true` keeps the footer out of the body, and a parser that matches \
         nothing is a rule nobody notices is gone."
    );
}
