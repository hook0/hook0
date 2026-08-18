//! What a language is held to when it says it drove an operation.
//!
//! The set on one side of the bijection is the API document read through the generator, and it is
//! read here for real rather than fabricated: a set built by the test would only hold the test's
//! own idea of the API. The set on the other side is what a smoke printed, and every case below is
//! a line a smoke could print — including the ones nobody means to.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use hook0_sdkgen::PUBLIC_TAG;
use hook0_sdkgen::targets::Decoding;
use hook0_sdkgen::{ApiModel, Limits, Snapshot};
use hook0_smoke::surface::{
    MAX_REPORTS, Models, Outcome, Report, THROTTLED, declared, held, models, report, reported,
};

/// A target name no client answers to. The bijection is about a set of operations, and holding it
/// to a real client's name would suggest it is about the client.
const A_TARGET: &str = "brainfuck";

/// The API document this repository commits, which is what every client is generated from.
fn document() -> PathBuf {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repository = crate_root.parent().expect("a directory above this crate");
    repository.join("api").join("openapi.snapshot.json")
}

/// The model types the rust client is generated from.
fn types() -> Models {
    models(&document(), PUBLIC_TAG, Decoding::Modelled).expect("the document declares models")
}

/// Everything the document declares, reported as driven and decoded — a language that did all of
/// it. What the cases below then do is take one thing away.
fn every(operations: &BTreeSet<String>, types: &Models) -> Vec<String> {
    operations
        .iter()
        .map(|operation| format!("exercised {operation} accepted"))
        .chain(
            types
                .answered
                .iter()
                .map(|model| format!("decoded {model}")),
        )
        .collect()
}

#[test]
fn the_operations_come_from_the_document_rather_than_from_here() {
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");

    // Not a count and not a list: what is held is that the set is the document's, so the only
    // claims here are ones that stay true as the API grows.
    assert!(
        operations.contains("events.ingest"),
        "the surface an SDK exposes carries ingestion: {operations:?}"
    );
    assert!(
        !operations.iter().any(|operation| operation.is_empty()),
        "every operation is named: {operations:?}"
    );
}

#[test]
fn a_document_that_is_not_one_is_refused_naming_where_it_was_looked_for() {
    let missing = Path::new("/where/no/api/document/is/openapi.snapshot.json");

    let refused = declared(missing, PUBLIC_TAG).expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains("openapi.snapshot.json"), "{said}");
}

#[test]
fn every_target_the_registry_declares_is_generated_from_operations_the_document_declares() {
    // Not every target selects the same tag — the SDKs are generated from the public surface and
    // the MCP server from its own — so the set a language is held to is the one its own client was
    // generated from, read off the registry rather than decided anywhere.
    for target in hook0_sdkgen::targets::targets() {
        let operations = declared(&document(), target.tag)
            .unwrap_or_else(|refused| panic!("{}: {refused}", target.name));
        assert!(
            !operations.is_empty(),
            "{} is generated from nothing",
            target.name
        );
    }
}

#[test]
fn a_language_that_drove_everything_is_held_to_have_driven_everything() {
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");

    let types = types();

    let held = held(
        A_TARGET,
        true,
        &operations,
        &types,
        &every(&operations, &types),
    )
    .expect("the bijection");

    assert_eq!(held.operations, operations.len());
    assert_eq!(held.models, types.answered.len());
}

#[test]
fn an_operation_that_was_never_driven_is_named_and_so_is_every_other_one() {
    // Two left out rather than one: the refusal has to name every one of them, since a language
    // told about the first would otherwise come back for the second.
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let left_out: Vec<String> = operations.iter().take(2).cloned().collect();
    let types = types();
    let reports: Vec<String> = every(&operations, &types)
        .into_iter()
        .filter(|line| {
            !left_out
                .iter()
                .any(|operation| line == &format!("exercised {operation} accepted"))
        })
        .collect();

    let refused = held(A_TARGET, true, &operations, &types, &reports).expect_err("a refusal");

    let said = format!("{refused}");
    for operation in &left_out {
        assert!(said.contains(operation), "{operation} is not named: {said}");
    }
    assert!(said.contains(A_TARGET), "{said}");
}

#[test]
fn a_report_naming_an_operation_the_document_does_not_declare_is_refused() {
    // What a typo looks like from here. Left unrefused it would satisfy nothing while looking like
    // work, and the operation it was meant to name would be reported as never driven — which sends
    // whoever reads it looking at the wrong call site.
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let types = types();
    let mut reports = every(&operations, &types);
    reports.push("exercised events.injest accepted".to_owned());

    let refused = held(A_TARGET, true, &operations, &types, &reports).expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains("events.injest"), "{said}");
}

#[test]
fn one_operation_answered_two_ways_in_one_run_is_refused() {
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let types = types();
    let mut reports = every(&operations, &types);
    reports.push("exercised events.ingest refused:EventAlreadyIngested".to_owned());

    let refused = held(A_TARGET, true, &operations, &types, &reports).expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains("events.ingest"), "{said}");
    assert!(said.contains("EventAlreadyIngested"), "{said}");
}

#[test]
fn one_operation_reported_twice_the_same_way_is_a_flow_that_read_a_list_twice() {
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let types = types();
    let mut reports = every(&operations, &types);
    reports.push("exercised events.list accepted".to_owned());

    let held = held(A_TARGET, true, &operations, &types, &reports).expect("the bijection");

    assert_eq!(held.operations, operations.len());
}

#[test]
fn a_language_that_reports_nothing_and_says_nothing_about_that_is_refused() {
    // The hole this closes: without it, deleting every report from a language that had them would
    // leave the run green and the client untested against a real instance.
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");

    let refused = held(A_TARGET, true, &operations, &types(), &[]).expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains("drives_surface"), "{said}");
    assert!(said.contains(A_TARGET), "{said}");
}

#[test]
fn a_language_whose_manifest_says_it_drives_nothing_yet_is_allowed_to_drive_nothing() {
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");

    let held = held(A_TARGET, false, &operations, &types(), &[]).expect("no bijection to hold");

    assert_eq!(held.operations, 0);
    assert_eq!(held.models, 0);
}

#[test]
fn a_language_that_drives_the_surface_while_saying_it_does_not_is_refused() {
    // The mirror of the case above, and what keeps the manifest honest: a smoke ported without its
    // manifest being updated would otherwise be held to nothing at all.
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");

    let types = types();
    let refused = held(
        A_TARGET,
        false,
        &operations,
        &types,
        &every(&operations, &types),
    )
    .expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains("drives_surface"), "{said}");
}

#[test]
fn more_reports_than_the_ceiling_allows_is_refused_at_the_ceiling() {
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let reports: Vec<String> = (0..MAX_REPORTS + 2)
        .map(|_| "exercised events.list accepted".to_owned())
        .collect();

    let refused = held(A_TARGET, true, &operations, &types(), &reports).expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains(&MAX_REPORTS.to_string()), "{said}");
}

#[test]
fn a_line_that_opens_with_the_word_and_is_malformed_is_refused_rather_than_passed_over() {
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let types = types();
    let mut reports = every(&operations, &types);
    reports.push("exercised events.list".to_owned());

    let refused = held(A_TARGET, true, &operations, &types, &reports).expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains("exercised events.list"), "{said}");
}

#[test]
fn what_counts_as_a_report_is_the_word_a_line_opens_with() {
    assert!(reported("exercised events.list accepted"));
    assert!(reported("  exercised events.list accepted"));
    // Malformed, and still a report: it is refused by name rather than mistaken for prose.
    assert!(reported("exercised"));

    assert!(!reported("the client exercised events.list"));
    assert!(!reported("exercisedevents.list accepted"));
    assert!(!reported(""));
}

#[test]
fn a_report_says_the_operation_and_which_of_the_two_outcomes_it_had() {
    assert_eq!(
        report("exercised events.list accepted").expect("a report"),
        Report::Exercised("events.list".to_owned(), Outcome::Accepted)
    );
    assert_eq!(
        report("exercised applications.create refused:Forbidden").expect("a report"),
        Report::Exercised(
            "applications.create".to_owned(),
            Outcome::Refused("Forbidden".to_owned())
        )
    );
    assert_eq!(
        report("decoded Subscription").expect("a report"),
        Report::Decoded("Subscription".to_owned())
    );

    for wrong in [
        "exercised",
        "exercised events.list",
        "exercised events.list maybe",
        "exercised events.list refused:",
        "exercised events.list accepted and then some",
        "decoded",
        "decoded Subscription and then some",
    ] {
        assert!(report(wrong).is_err(), "`{wrong}` is not a report");
    }
}

#[test]
fn an_operation_the_instance_only_paced_is_refused_rather_than_counted() {
    // The one way this whole exercise could pass and mean nothing: an instance pacing a flow that
    // asks for three dozen operations in a row answers every one of them without looking at any of
    // them, and a smoke reporting that would look exactly like a smoke that drove them.
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let types = types();
    let reports: Vec<String> = every(&operations, &types)
        .into_iter()
        .map(|line| {
            line.replace(
                "exercised events.list accepted",
                &format!("exercised events.list refused:{THROTTLED}"),
            )
        })
        .collect();

    let refused = held(A_TARGET, true, &operations, &types, &reports).expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains(THROTTLED), "{said}");
    assert!(said.contains("Retry-After"), "{said}");
}

#[test]
fn a_model_no_operation_answers_is_not_one_a_language_is_held_to() {
    // The narrowing is the document's, not this crate's. A type only a request body carries is
    // never decoded by anybody — what would catch a wrong field name in one is the operation that
    // sends it being refused, which the other bijection holds.
    let types = types();

    assert!(types.emitted.contains("SubscriptionPost"), "it is emitted");
    assert!(
        !types.answered.contains("SubscriptionPost"),
        "and no operation answers it"
    );
    assert!(types.answered.contains("Subscription"));
    assert!(types.answered.is_subset(&types.emitted));
}

#[test]
fn a_model_the_generator_emits_and_nothing_answers_may_still_be_reported() {
    // An instance that populates one of the optional ones is right to say so, and a language that
    // decoded it should not be refused for being on a richer instance than the next one.
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let types = types();
    let mut reports = every(&operations, &types);
    reports.push("decoded InstanceConfigMatomo".to_owned());

    let held = held(A_TARGET, true, &operations, &types, &reports).expect("the bijection");

    assert_eq!(held.models, types.answered.len() + 1);
}

#[test]
fn a_model_that_was_never_decoded_is_named_and_so_is_every_other_one() {
    // The hole the model bijection closes: every operation below is reported, and reported as
    // refused, which the operation bijection is perfectly happy with. A client that decoded nothing
    // at all would pass it.
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let types = types();
    let reports: Vec<String> = operations
        .iter()
        .map(|operation| format!("exercised {operation} refused:Forbidden"))
        .collect();

    let refused = held(A_TARGET, true, &operations, &types, &reports).expect_err("a refusal");

    let said = format!("{refused}");
    for model in &types.answered {
        assert!(said.contains(model), "{model} is not named: {said}");
    }
}

#[test]
fn a_decoded_line_naming_something_that_is_not_a_model_is_refused() {
    let operations = declared(&document(), PUBLIC_TAG).expect("the document declares operations");
    let types = types();
    let mut reports = every(&operations, &types);
    reports.push("decoded Subscriptions".to_owned());

    let refused = held(A_TARGET, true, &operations, &types, &reports).expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains("Subscriptions"), "{said}");
}

/// What each target is held to in the way of models is what it writes, not what its tag selects.
///
/// The two readings coincide for every target that writes one type per schema, which is why holding
/// all of them to the tag alone looked right for as long as all of them did. A target that writes
/// none is held to nothing: the tag it selects still declares twenty-nine types, and demanding them
/// of a client that hands its answers on untouched would be demanding the smoke re-implement the
/// model it is meant to be checking.
#[test]
fn what_a_target_is_held_to_follows_from_whether_it_writes_the_types_at_all() {
    let mut modelled = 0;
    let mut pass_through = 0;

    for target in hook0_sdkgen::targets::targets() {
        let types = models(&document(), target.tag, target.decoding)
            .unwrap_or_else(|refused| panic!("{}: {refused}", target.name));

        // What the target really writes, emitted for this test rather than read off the tree it
        // landed in, so the claim is about the generator and not about whatever is on disk.
        let read = Snapshot::from_path(&document(), target.tag, &Limits::DEFAULT)
            .unwrap_or_else(|refused| panic!("{}: {refused}", target.name));
        let model = ApiModel::from_snapshot(&read, &Limits::DEFAULT)
            .unwrap_or_else(|refused| panic!("{}: {refused}", target.name));
        let written: String = (target.emit)(&target.language, &model)
            .unwrap_or_else(|refused| panic!("{}: {refused}", target.name))
            .files()
            .iter()
            .map(|file| file.contents.as_str())
            .collect();

        // Whether the emitted source names the API's types at all. Text rather than structure,
        // because twelve languages spell a declaration twelve ways and all of them spell the name
        // the same. `declared` here is the set the tag selects, taken independently of `types` so
        // that a pass-through target is still measured against something.
        let declared = models(&document(), target.tag, Decoding::Modelled)
            .unwrap_or_else(|refused| panic!("{}: {refused}", target.name))
            .emitted;
        let named = declared
            .iter()
            .filter(|model| written.contains(model.as_str()))
            .count();

        match target.decoding {
            Decoding::Modelled => {
                modelled += 1;
                assert!(
                    named > 0,
                    "{} says it writes the API's types and names none of the {} its tag selects",
                    target.name,
                    declared.len()
                );
                assert!(
                    !types.answered.is_empty(),
                    "{} writes the API's types and yet answers none of them",
                    target.name
                );
                assert!(types.answered.is_subset(&types.emitted), "{}", target.name);
            }
            Decoding::PassThrough => {
                pass_through += 1;
                assert_eq!(
                    named, 0,
                    "{} says it writes no types and yet its emitted source names {named} of them",
                    target.name
                );
                assert!(
                    types.emitted.is_empty() && types.answered.is_empty(),
                    "{} writes no types, so nothing can hold it to one",
                    target.name
                );
            }
        }
    }

    // Both arms asserted non-vacuously. A registry that lost its one pass-through target, or that
    // marked every target as one, would otherwise satisfy this by never entering the arm that
    // matters.
    assert!(
        modelled > 0 && pass_through > 0,
        "{modelled} / {pass_through}"
    );
}
