//! What happens to a value somebody typed on its way into a snippet.
//!
//! The payload is whatever was written in the form, and it lands inside a string in code the reader
//! is told to copy. Before this, it landed raw inside a Rust raw string: a payload carrying the
//! closing sequence produced a snippet that does not compile, on the reader's own data, in code
//! Hook0 handed them. So each language declares how a string of it is opened, closed and escaped,
//! and the property below is what those declarations are held to — not examples of hostile
//! payloads, which only ever prove the examples.

use hook0_dashboard_examples::{StringLiteral, manifest, shown};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

mod common;
mod declaration;

/// How a literal reads back.
#[derive(Debug, PartialEq, Eq)]
enum Reading {
    /// It runs to the end of what was put in it, which is the only acceptable answer.
    Whole,
    /// It closed early, at this byte: everything after it is code the reader never wrote.
    ClosedAt(usize),
    /// It ends on an escape with nothing to consume, so the delimiter closing it is swallowed and
    /// the literal never ends at all.
    Dangling,
}

/// Reads an escaped body back the way the language that declared the rules would.
///
/// Whether a backslash escapes anything is read off the rules themselves rather than assumed: a
/// language whose replacements introduce one is a language whose literals are escaped with one, and
/// a language whose replacements introduce none has no escape character to honour.
fn read_back(literal: &StringLiteral, body: &str) -> Reading {
    let escaped = literal.escape.iter().any(|(_, to)| to.contains('\\'));
    let mut at = 0;

    while at < body.len() {
        let rest = &body[at..];
        if escaped && rest.starts_with('\\') {
            let mut following = rest[1..].chars();
            return match following.next() {
                None => Reading::Dangling,
                Some(consumed) => match read_back(literal, &rest[1 + consumed.len_utf8()..]) {
                    Reading::Whole => Reading::Whole,
                    Reading::Dangling => Reading::Dangling,
                    Reading::ClosedAt(later) => {
                        Reading::ClosedAt(at + 1 + consumed.len_utf8() + later)
                    }
                },
            };
        }
        if rest.starts_with(literal.close.as_str()) {
            return Reading::ClosedAt(at);
        }
        at += rest.chars().next().map_or(1, char::len_utf8);
    }
    Reading::Whole
}

/// A value that is nothing but the characters a literal is most likely to be broken by.
fn hostile() -> impl Strategy<Value = String> {
    prop_oneof![
        ".{0,64}",
        r#"[\\"'`\n\r\t\u{0000}{}$#]{0,32}"#.prop_map(String::from),
    ]
}

/// Every declared rule, with the language it was declared for.
fn declared() -> Vec<(String, StringLiteral)> {
    let tree = common::tree();
    shown()
        .expect("the registry is unreadable")
        .into_iter()
        .map(|target| {
            let path = tree.join(target.manifest());
            let read = manifest::read(&path).unwrap_or_else(|cause| {
                panic!("`{}` declares nothing usable: {cause}", target.target)
            });
            (target.target, read.string)
        })
        .collect()
}

proptest! {
    // A counterexample is written where the crate keeps them rather than beside this file, which is
    // where proptest falls back to when it cannot find a `lib.rs` from an integration suite — it
    // says so in a warning nobody reads, and a file written somewhere unexpected is a file that
    // gets tidied away instead of committed. The point of persisting one is that it outlives the
    // fix: proptest replays it before any new case, so a value that broke this once cannot escape
    // again.
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/proptest-regressions/escaping.txt"
        )))),
        ..ProptestConfig::default()
    })]

    /// Whatever a reader types, the literal it lands in ends where the snippet says it ends.
    ///
    /// This is what the order of the replacements is for. Escaping the quote before the backslash
    /// leaves `\"` where the reader typed `"`, and the backslash rule then turns it into `\\"` —
    /// which reads back as an escaped backslash followed by a quote that closes the literal, and
    /// everything after it is code nobody wrote.
    #[test]
    fn no_value_closes_the_literal_it_lands_in(value in hostile()) {
        for (target, literal) in declared() {
            let escaped = literal.escaped(&value);
            prop_assert_eq!(
                read_back(&literal, &escaped),
                Reading::Whole,
                "`{}` renders {:?} as {:?}, which does not stay inside {}…{}",
                target,
                value,
                escaped,
                literal.open,
                literal.close
            );
        }
    }
}

/// The rules a language declares are what the property holds; these are what it would catch.
///
/// Two shapes, because the order goes wrong in two ways. Leaving the backslash unescaped ends a
/// value on one with nothing to consume, so the delimiter after it is swallowed. Escaping it last
/// doubles the backslash the quote rule had just written, so the quote closes the literal.
#[test]
fn the_property_catches_a_rule_written_in_the_wrong_order() {
    let quotes = ("\"".to_owned(), "\\\"".to_owned());
    let backslash = ("\\".to_owned(), "\\\\".to_owned());

    let unescaped = StringLiteral {
        open: "\"".to_owned(),
        close: "\"".to_owned(),
        escape: vec![quotes.clone()],
    };
    assert_eq!(
        read_back(&unescaped, &unescaped.escaped("\\")),
        Reading::Dangling,
        "a value ending on a backslash was read as staying inside its literal"
    );

    let last = StringLiteral {
        open: "\"".to_owned(),
        close: "\"".to_owned(),
        escape: vec![quotes, backslash],
    };
    assert_eq!(
        read_back(&last, &last.escaped("\"")),
        Reading::ClosedAt(2),
        "a quote escaped before the backslash was read as staying inside its literal"
    );
}

/// The declaration itself is refused before it can be applied.
#[test]
fn a_rule_written_in_the_wrong_order_is_refused() {
    let mut declared = declaration::Declaration::new("escaping");

    declared.escape = "[['\"', '\\\"']]".to_owned();
    let refused = manifest::read(&declared.written())
        .expect_err("a rule introducing an unescaped backslash was accepted");
    assert!(
        refused.to_string().contains("backslash"),
        "the refusal does not say what is wrong: {refused}"
    );

    declared.escape = "[['\\', '\\\\'], ['\"', '\\\"']]".to_owned();
    manifest::read(&declared.written()).expect("the rule every language declares was refused");
}
