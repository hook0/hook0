//! How a name read off the snapshot is shaped into the identifier a target writes.

use hook0_sdkgen::identifier::{Case, Escape, ReservedWords, checked_words, escape, render, words};
use hook0_sdkgen::{Error, Limits};

/// A keyword list holding both `type` and the name a single escape of it lands on, so escaping has
/// to step twice.
const RESERVED: [&str; 3] = ["const", "type", "type_"];

fn reserved() -> ReservedWords {
    ReservedWords::build(&RESERVED, Escape::Suffix('_'), "value")
        .expect("the keyword list is sorted, deduplicated and free of its own placeholder")
}

#[test]
fn a_name_reads_as_the_words_it_is_built_from() {
    let cases: [(&str, &[&str]); 14] = [
        ("applications.list", &["applications", "list"]),
        ("application_secret", &["application", "secret"]),
        ("application-secret", &["application", "secret"]),
        ("application secret", &["application", "secret"]),
        ("applicationSecret", &["application", "secret"]),
        ("ApplicationSecret", &["application", "secret"]),
        // Nothing in a run of capitals says where one word ends and the next begins.
        ("HTTPServer", &["httpserver"]),
        ("FooTests.cs", &["foo", "tests", "cs"]),
        ("foo2Bar", &["foo2", "bar"]),
        ("__tests__", &["tests"]),
        ("événementCréé", &["événement", "créé"]),
        ("", &[]),
        ("...", &[]),
        ("   ", &[]),
    ];

    for (name, expected) in cases {
        assert_eq!(words(name), expected, "reading `{name}`");
    }
}

#[test]
fn a_name_above_the_identifier_ceiling_is_refused_rather_than_shortened() {
    let limits = Limits {
        max_identifier_bytes: 8,
        ..Limits::DEFAULT
    };

    assert_eq!(
        checked_words("applications", &limits),
        Err(Error::IdentifierTooLong {
            identifier: "applications".to_owned(),
            size: 12,
            limit: 8,
        })
    );
    assert_eq!(checked_words("apps", &limits), Ok(vec!["apps".to_owned()]));
}

#[test]
fn words_are_written_back_out_in_the_case_a_target_spells_them_in() {
    let words = ["application".to_owned(), "secret".to_owned()];
    let cases = [
        (Case::Snake, "application_secret"),
        (Case::LowerCamel, "applicationSecret"),
        (Case::UpperCamel, "ApplicationSecret"),
        (Case::ScreamingSnake, "APPLICATION_SECRET"),
        (Case::Kebab, "application-secret"),
        (Case::Lower, "applicationsecret"),
    ];

    for (case, expected) in cases {
        assert_eq!(render(&words, case), expected, "rendering in {case:?}");
    }
}

#[test]
fn rendering_recases_whatever_case_the_words_arrived_in() {
    let shouted = ["APPLICATION".to_owned(), "Secret".to_owned()];

    assert_eq!(render(&shouted, Case::Snake), "application_secret");
    assert_eq!(render(&shouted, Case::LowerCamel), "applicationSecret");
    assert_eq!(render(&shouted, Case::UpperCamel), "ApplicationSecret");
}

#[test]
fn an_empty_word_is_dropped_rather_than_spelled_as_a_bare_separator() {
    let padded = [String::new(), "list".to_owned(), String::new()];

    assert_eq!(render(&padded, Case::Snake), "list");
    assert_eq!(render(&padded, Case::LowerCamel), "list");
    assert_eq!(render(&[], Case::Snake), "");
}

#[test]
fn a_name_landing_on_a_keyword_is_stepped_around_until_it_no_longer_does() {
    let reserved = reserved();

    assert_eq!(escape("application", &reserved), "application");
    assert_eq!(escape("const", &reserved), "const_");
    // `type_` is reserved too, so one step is not enough.
    assert_eq!(escape("type", &reserved), "type__");
}

#[test]
fn a_name_that_rendered_to_nothing_comes_back_as_the_placeholder() {
    assert_eq!(escape("", &reserved()), "value");
}

#[test]
fn the_escape_strategy_travels_with_the_list_it_escapes_against() {
    let prefixing = ReservedWords::build(&RESERVED, Escape::Prefix('@'), "value")
        .expect("the keyword list is usable");

    assert_eq!(escape("const", &prefixing), "@const");
    assert_eq!(escape("", &prefixing), "value");
}

#[test]
fn a_keyword_list_that_could_let_a_keyword_through_is_refused_where_it_is_written() {
    const UNSORTED: [&str; 2] = ["type", "const"];
    const REPEATED: [&str; 2] = ["const", "const"];
    const EMPTY_WORD: [&str; 2] = ["", "const"];

    let refusals = [
        ReservedWords::build(&UNSORTED, Escape::Suffix('_'), "value"),
        ReservedWords::build(&REPEATED, Escape::Suffix('_'), "value"),
        ReservedWords::build(&EMPTY_WORD, Escape::Suffix('_'), "value"),
        ReservedWords::build(&RESERVED, Escape::Suffix('_'), ""),
        ReservedWords::build(&RESERVED, Escape::Suffix('_'), "const"),
    ];

    for refusal in refusals {
        let error = refusal.expect_err("an unusable keyword list is refused");
        let named_as_unusable = matches!(error, Error::UnusableReservedWords { .. });
        assert!(named_as_unusable, "unexpected error: {error}");
    }
}
