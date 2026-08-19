//! Invariants an identifier holds whatever name it was shaped from.

use hook0_sdkgen::identifier::{Case, Escape, ReservedWords, checked_words, escape, render, words};
use hook0_sdkgen::{Error, Limits};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

/// Seeds of past failures, replayed before anything random is drawn.
const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/proptest-regressions/identifier_properties.txt"
);

/// The fallback a generated keyword list falls back on. It carries a digit, which the generated
/// keywords cannot, so it is never one of them.
const PLACEHOLDER: &str = "value0";

/// Largest number of keywords a generated vocabulary carries.
const KEYWORD_SLOTS: usize = 12;

/// Widest an identifier can grow past the name it was rendered from: casing a character can spell
/// it as up to three others, and each word costs one separator on top.
const RENDERING_FACTOR: usize = 4;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSIONS))),
        ..ProptestConfig::default()
    }
}

/// Names spanning what a snapshot spells and what it should not.
fn name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(String::new()),
        Just("...".to_owned()),
        Just("_".to_owned()),
        "[a-z]{1,8}",
        "[a-z]{1,6}\\.[a-z]{1,6}",
        "[a-zA-Z]{1,12}",
        "[a-zA-Z0-9_. -]{0,24}",
        "[\\p{L}\\p{N}_. -]{0,24}",
    ]
}

/// Whether text is a name every one of the twelve languages reads.
///
/// The same rule the crate applies, written again rather than reached for, so the property is
/// answering the question rather than asking the code under test to agree with itself.
fn is_an_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    characters
        .all(|character| character.is_ascii_alphanumeric() || character == '_' || character == '-')
}

fn case() -> impl Strategy<Value = Case> {
    prop_oneof![
        Just(Case::Snake),
        Just(Case::LowerCamel),
        Just(Case::UpperCamel),
        Just(Case::ScreamingSnake),
        Just(Case::Kebab),
        Just(Case::Lower),
    ]
}

/// Vocabularies a language could plausibly keep for itself, sorted and deduplicated as the list
/// requires, and escaped one way or the other.
fn reserved() -> impl Strategy<Value = ReservedWords> {
    (
        prop::collection::vec("[a-z_]{1,6}", 0..KEYWORD_SLOTS),
        any::<bool>(),
    )
        .prop_map(|(mut keywords, prefixes)| {
            keywords.sort();
            keywords.dedup();

            let escape = if prefixes {
                Escape::Prefix('@')
            } else {
                Escape::Suffix('_')
            };
            let borrowed: Vec<&str> = keywords.iter().map(String::as_str).collect();

            ReservedWords::build(&borrowed, escape, PLACEHOLDER)
                .expect("a sorted, deduplicated vocabulary with a fallback outside it is usable")
        })
}

proptest! {
    #![proptest_config(config())]

    /// Whatever a name is, it either yields an identifier that can be written down and compiled, or
    /// it is refused. It never yields something in between.
    ///
    /// The three assertions this used to carry were that the identifier spells something, that it
    /// is not a keyword, and that escaping it again leaves it alone. None of them can fail on a
    /// rendering that is no identifier at all, which is what the docstring was already claiming
    /// they covered: `2fa` spells something, is nobody's keyword, and is stable under escaping, and
    /// it compiles in none of the twelve languages. What is asserted below is the claim itself,
    /// character by character.
    #[test]
    fn a_name_always_yields_an_identifier_the_language_accepts(
        name in name(),
        case in case(),
        reserved in reserved(),
    ) {
        let rendered = render(&words(&name), case);
        let Ok(identifier) = escape(&rendered, &reserved) else {
            // A refusal is an answer, and it is only allowed to be the answer when the rendering
            // really is no identifier. Checked here rather than trusted, so a refusal cannot become
            // the easy way out of a name that was perfectly usable.
            prop_assert!(
                !is_an_identifier(&rendered),
                "`{}` rendered `{}`, which is an identifier, and it was refused anyway",
                name,
                rendered
            );
            return Ok(());
        };

        prop_assert!(!identifier.is_empty(), "`{}` yielded no identifier", name);
        // C# spells an escaped keyword `@type`, so a vocabulary that prefixes is allowed one
        // marker in front of what is otherwise a name. Dropped by characters rather than by bytes,
        // since a marker is a `char` and slicing one off by index splits it.
        let mut past_marker = identifier.chars();
        past_marker.next();
        let unmarked: String = past_marker.collect();
        prop_assert!(
            is_an_identifier(&identifier) || is_an_identifier(&unmarked),
            "`{}` yielded `{}`, which no language reads as a name",
            name,
            identifier
        );
        prop_assert!(
            !reserved.contains(&identifier),
            "`{}` yielded the keyword `{}`",
            name,
            identifier
        );
        prop_assert_eq!(
            escape(&identifier, &reserved).ok(),
            Some(identifier.clone()),
            "escaping `{}` again moved it",
            identifier
        );
    }

    /// Shaping a name is a function of the name alone, so two emitters of one model write the same
    /// identifier.
    #[test]
    fn a_name_is_shaped_the_same_way_every_time(name in name(), case in case()) {
        prop_assert_eq!(words(&name), words(&name));
        prop_assert_eq!(render(&words(&name), case), render(&words(&name), case));
    }

    /// A name the ceilings accept renders to an identifier the ceilings still bound: nothing
    /// downstream has to bound it again.
    #[test]
    fn an_accepted_name_renders_to_a_bounded_identifier(name in name(), case in case()) {
        let limits = Limits::default();

        let Ok(words) = checked_words(&name, &limits) else {
            return Ok(());
        };
        let rendered = render(&words, case);

        prop_assert!(
            rendered.len() <= RENDERING_FACTOR * limits.max_identifier_bytes,
            "`{}` rendered to {} bytes",
            name,
            rendered.len()
        );
        prop_assert!(words.len() <= limits.max_words_per_identifier);
    }

    /// A name past a ceiling is refused with the count it reached and the ceiling it crossed, and
    /// one under it is never refused for that reason.
    #[test]
    fn a_name_past_a_ceiling_is_refused_rather_than_cut_down(
        name in name(),
        ceiling in 0usize..8,
    ) {
        let counted = words(&name).len();
        let limits = Limits { max_words_per_identifier: ceiling, ..Limits::DEFAULT };

        let result = checked_words(&name, &limits);
        if counted > ceiling {
            prop_assert_eq!(
                result.map(|words| words.len()).map_err(|error| match error {
                    Error::TooManyWords { count, limit, .. } => (count, limit),
                    other => panic!("unexpected error: {other}"),
                }),
                Err((counted, ceiling))
            );
        } else {
            let accepted = result.map_err(|error| TestCaseError::fail(error.to_string()))?;
            prop_assert_eq!(accepted.len(), counted);
        }
    }

    /// A name longer than the ceiling is refused with the ceiling, never shortened to fit it.
    #[test]
    fn a_name_above_the_length_ceiling_is_refused_rather_than_shortened(
        name in name(),
        ceiling in 0usize..16,
    ) {
        let limits = Limits { max_identifier_bytes: ceiling, ..Limits::DEFAULT };
        let result = checked_words(&name, &limits);

        if name.len() > ceiling {
            prop_assert_eq!(
                result.map(|words| words.len()).map_err(|error| match error {
                    Error::IdentifierTooLong { size, limit, .. } => (size, limit),
                    other => panic!("unexpected error: {other}"),
                }),
                Err((name.len(), ceiling))
            );
        } else {
            let refused_for_length = matches!(result, Err(Error::IdentifierTooLong { .. }));
            prop_assert!(!refused_for_length, "a name under the ceiling was refused");
        }
    }

    /// A vocabulary that is not sorted and deduplicated is refused where it is written, since a
    /// keyword sitting past the point a search gives up on would go through unescaped.
    #[test]
    fn a_vocabulary_that_could_let_a_keyword_through_is_refused(
        keywords in prop::collection::vec("[a-z_]{0,6}", 0..KEYWORD_SLOTS),
    ) {
        let borrowed: Vec<&str> = keywords.iter().map(String::as_str).collect();
        let usable = keywords.windows(2).all(|pair| match pair {
            [previous, word] => word > previous,
            _ => true,
        }) && keywords.iter().all(|word| !word.is_empty());

        let result = ReservedWords::build(&borrowed, Escape::Suffix('_'), PLACEHOLDER);
        prop_assert_eq!(result.is_ok(), usable, "vocabulary: {:?}", keywords);
    }
}
