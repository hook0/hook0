//! Black-box suite over the shared conformance corpus and the API it describes.
//!
//! The corpus is hand-authored data every target reads from its own suite. What it cannot do on its
//! own is notice that the API grew a problem nobody classified, or lost one it still names — which
//! is what this checks, against the very snapshot the targets are generated from.
//!
//! Nothing here writes down a problem identifier or a status: a case that needs a corpus saying
//! something else builds one by altering the committed documents, so a rename in the API moves both
//! sides of an assertion at once.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use hook0_sdkgen::conformance::ConformanceError;
use hook0_sdkgen::{ApiModel, Corpus, CorpusLimits, Limits, SDK_TAG, Snapshot};
use serde_json::{Value, json};
use tempfile::TempDir;

mod common;

/// Where the corpus every target reads sits, from the crate this suite runs out of.
const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../conformance");

/// The API contract the committed snapshot describes.
fn api() -> ApiModel {
    let limits = Limits::default();
    let snapshot = Snapshot::from_bytes(&common::fixture_bytes(), SDK_TAG, &limits)
        .expect("the committed snapshot parses");
    ApiModel::from_snapshot(&snapshot, &limits).expect("the committed snapshot yields a model")
}

fn corpus() -> Corpus {
    Corpus::read(Path::new(CORPUS), &CorpusLimits::default()).expect("the committed corpus is read")
}

/// The committed documents, read back as values a case can alter before writing them out again.
///
/// The directory is walked rather than the documents named, so a document added to the corpus is
/// carried here without this suite being told about it.
fn committed() -> BTreeMap<String, Value> {
    let mut documents = BTreeMap::new();
    for entry in fs::read_dir(CORPUS).expect("the corpus directory is readable") {
        let entry = entry.expect("every entry of the corpus is readable");
        let name = entry.file_name().to_string_lossy().into_owned();
        let bytes = fs::read(entry.path()).expect("every document of the corpus is readable");
        documents.insert(
            name,
            serde_json::from_slice(&bytes).expect("every document of the corpus is JSON"),
        );
    }
    documents
}

/// A corpus laid out in a directory of its own, kept for the lifetime of the case.
fn laid_out(documents: &BTreeMap<String, Value>) -> TempDir {
    let directory = TempDir::new().expect("a directory to lay the corpus out in");
    for (name, document) in documents {
        fs::write(
            directory.path().join(name),
            serde_json::to_vec_pretty(document).expect("a document written back out"),
        )
        .expect("a document laid out");
    }
    directory
}

fn read(directory: &TempDir) -> Result<Corpus, ConformanceError> {
    Corpus::read(directory.path(), &CorpusLimits::default())
}

/// The list of that name, wherever in the corpus it is written.
fn list<'a>(documents: &'a mut BTreeMap<String, Value>, subject: &str) -> &'a mut Vec<Value> {
    documents
        .values_mut()
        .find_map(|document| declared(document, subject, MAX_SEARCH_DEPTH))
        .unwrap_or_else(|| panic!("no document of the corpus declares `{subject}`"))
        .as_array_mut()
        .unwrap_or_else(|| panic!("`{subject}` is not a list"))
}

/// How deep under a document a case looks for the list it is about to alter.
const MAX_SEARCH_DEPTH: usize = 4;

/// The member of that name, at whatever depth of a document it sits.
fn declared<'a>(holder: &'a mut Value, subject: &str, depth: usize) -> Option<&'a mut Value> {
    if depth == 0 {
        return None;
    }
    let members = holder.as_object_mut()?;
    if members.contains_key(subject) {
        return members.get_mut(subject);
    }
    members
        .values_mut()
        .find_map(|nested| declared(nested, subject, depth - 1))
}

/// The corpus classifies the API's whole problem catalogue, rules on every status it answers, and
/// names nothing the API does not.
#[test]
fn the_committed_corpus_describes_the_api_the_snapshot_declares() {
    let api = api();
    let corpus = corpus();

    corpus
        .agrees_with(&api.errors)
        .expect("the committed corpus describes the committed API");

    assert_eq!(
        corpus.problems.len(),
        api.errors.catalogue.len(),
        "the corpus and the catalogue do not carry the same number of problems"
    );
    assert_eq!(
        corpus
            .statuses
            .iter()
            .map(|rule| rule.status)
            .collect::<Vec<u16>>(),
        api.errors.statuses,
        "the corpus does not rule on exactly the statuses the API answers"
    );
}

/// The reason the corpus exists: a status alone does not always say whether to try again, so at
/// least one status carries both verdicts depending on the problem named under it.
#[test]
fn one_status_carries_opposite_verdicts_depending_on_the_problem() {
    let corpus = corpus();

    let split = corpus.statuses.iter().any(|rule| {
        let under: Vec<bool> = corpus
            .problems
            .iter()
            .filter(|problem| problem.status == rule.status)
            .map(|problem| problem.retryable)
            .collect();
        under.iter().any(|retryable| *retryable) && under.iter().any(|retryable| !*retryable)
    });

    assert!(
        split,
        "no status of the corpus carries opposite verdicts, so nothing exercises the problem \
         winning over the status"
    );
}

/// The same, one layer down: the failures that never produced an answer do not share one verdict
/// either, so a client deciding by which of its own types carries the failure fails at least one.
#[test]
fn the_transport_causes_do_not_all_share_one_verdict() {
    let corpus = corpus();

    assert!(
        corpus.transport.iter().any(|cause| cause.retryable),
        "no transport cause is worth repeating, so a send would never ride out a reset connection"
    );
    assert!(
        corpus.transport.iter().any(|cause| !cause.retryable),
        "every transport cause is worth repeating, so nothing exercises a client telling a failure \
         that could end differently from one that could not"
    );
}

/// A header carried on an occasion the corpus does not declare says nothing a target could act on.
#[test]
fn a_header_carried_on_an_undeclared_occasion_is_refused() {
    let mut documents = committed();
    let carried = list(&mut documents, "headers");
    let name = carried[0]["name"]
        .as_str()
        .expect("every header is named")
        .to_owned();
    carried[0]["when"] = json!("whenever it feels like it");

    assert_eq!(
        read(&laid_out(&documents)).expect_err("an undeclared occasion is refused"),
        ConformanceError::UnknownOccasion {
            header: name,
            occasion: "whenever it feels like it".to_owned(),
        }
    );
}

/// An occasion nothing is carried on is one no target would ever check.
#[test]
fn an_occasion_no_header_is_carried_on_is_refused() {
    let mut documents = committed();
    list(&mut documents, "occasions").push(json!("a request nobody makes"));

    assert_eq!(
        read(&laid_out(&documents)).expect_err("an occasion nothing exercises is refused"),
        ConformanceError::UnexercisedOccasion {
            occasion: "a request nobody makes".to_owned(),
        }
    );
}

/// Two rules for one header differ only in the case somebody wrote it in, and HTTP does not tell
/// them apart, so which of the two a target honoured would be its own business.
#[test]
fn two_rules_for_one_header_are_refused_whatever_case_they_are_written_in() {
    let mut documents = committed();
    let carried = list(&mut documents, "headers");
    let mut repeated = carried[0].clone();
    repeated["name"] = json!(
        carried[0]["name"]
            .as_str()
            .expect("every header is named")
            .to_uppercase()
    );
    carried.push(repeated);

    assert!(
        matches!(
            read(&laid_out(&documents)),
            Err(ConformanceError::Duplicated { .. })
        ),
        "one header declared twice was accepted"
    );
}

/// A problem the corpus classifies and the API does not declare is a rename or a typo: the verdict
/// would sit there being right about nothing.
#[test]
fn a_problem_the_api_does_not_declare_is_refused_by_name() {
    let mut documents = committed();
    let classified = list(&mut documents, "problems");
    let mut invented = classified[0].clone();
    invented["problem"] = json!("NoSuchProblemIsEverAnswered");
    classified.push(invented);

    let corpus = read(&laid_out(&documents)).expect("the altered corpus is still readable");
    let refused = corpus
        .agrees_with(&api().errors)
        .expect_err("a problem the API does not declare is refused");

    assert_eq!(
        refused,
        ConformanceError::UnknownProblem {
            problem: "NoSuchProblemIsEverAnswered".to_owned(),
        },
        "the refusal does not name the problem the corpus invented"
    );
}

/// The guard that makes the corpus keep up with the API: a problem the API grows and nobody
/// classifies is a decision left unmade, and every target would otherwise make it on its own.
#[test]
fn a_problem_the_api_declares_and_the_corpus_leaves_out_is_refused_by_name() {
    let mut documents = committed();
    let dropped = list(&mut documents, "problems").remove(0);
    let dropped = dropped["problem"]
        .as_str()
        .expect("every classified problem is named")
        .to_owned();

    let corpus = read(&laid_out(&documents)).expect("the altered corpus is still readable");
    let refused = corpus
        .agrees_with(&api().errors)
        .expect_err("a problem the corpus does not classify is refused");

    assert_eq!(
        refused,
        ConformanceError::UnclassifiedProblem { problem: dropped },
        "the refusal does not name the problem nobody classified"
    );
}

/// A status nothing answers, and a status answered that nothing rules on, are both refused: the
/// first is a rule that never fires, the second a failure every client decides about alone.
#[test]
fn the_statuses_ruled_on_are_the_ones_the_api_answers() {
    let mut invented = committed();
    let ruled = list(&mut invented, "statuses");
    let mut extra = ruled[0].clone();
    extra["status"] = json!(418);
    ruled.push(extra);

    let refused = read(&laid_out(&invented))
        .expect("the altered corpus is still readable")
        .agrees_with(&api().errors)
        .expect_err("a status no operation answers is refused");
    assert_eq!(refused, ConformanceError::UnansweredStatus { status: 418 });

    let mut dropped = committed();
    let removed = list(&mut dropped, "statuses").remove(0);
    let removed = u16::try_from(removed["status"].as_u64().expect("a status is a number"))
        .expect("a status fits");

    let refused = read(&laid_out(&dropped))
        .expect("the altered corpus is still readable")
        .agrees_with(&api().errors)
        .expect_err("a status the API answers and the corpus does not rule on is refused");
    assert_eq!(refused, ConformanceError::UnruledStatus { status: removed });
}

/// A problem classified under a status the API never answers describes a failure that cannot
/// happen, which is a corpus somebody edited without looking at the API.
#[test]
fn a_problem_classified_under_a_status_the_api_does_not_answer_is_refused() {
    let mut documents = committed();
    list(&mut documents, "problems")[0]["status"] = json!(418);

    let refused = read(&laid_out(&documents))
        .expect("the altered corpus is still readable")
        .agrees_with(&api().errors)
        .expect_err("a problem answering a status the API does not is refused");

    assert_eq!(refused, ConformanceError::UnansweredStatus { status: 418 });
}

/// Every document says what it is and what changing it changes, since it is read by suites in
/// languages that share nothing else.
#[test]
fn a_document_carrying_no_header_comment_is_refused() {
    let mut documents = committed();
    let name = documents
        .keys()
        .next()
        .expect("the corpus carries a document")
        .clone();
    documents
        .get_mut(&name)
        .and_then(Value::as_object_mut)
        .expect("a document is an object")
        .remove("$comment");

    assert_eq!(
        read(&laid_out(&documents)).expect_err("an uncommented document is refused"),
        ConformanceError::Uncommented { document: name }
    );
}

/// A document nobody reads is refused rather than ignored: it would sit in the corpus looking like
/// contract while no target exercised a line of it.
#[test]
fn a_document_nothing_reads_stops_the_read() {
    let documents = committed();
    let directory = laid_out(&documents);
    fs::write(directory.path().join("extra.json"), b"{}").expect("an extra document is laid out");

    assert_eq!(
        read(&directory).expect_err("a document nothing reads is refused"),
        ConformanceError::UnreadDocument {
            document: "extra.json".to_owned(),
        }
    );
}

/// A corpus missing one of its documents is refused naming the one it lacks.
#[test]
fn a_missing_document_is_refused_by_name() {
    let mut documents = committed();
    let name = documents
        .keys()
        .next()
        .expect("the corpus carries a document")
        .clone();
    documents.remove(&name);

    assert_eq!(
        read(&laid_out(&documents)).expect_err("a corpus missing a document is refused"),
        ConformanceError::MissingDocument { document: name }
    );
}

/// A corpus above a ceiling is refused naming the ceiling it crossed, rather than read as far as it
/// fits.
#[test]
fn a_corpus_above_a_ceiling_is_refused_naming_it() {
    let documents = committed();
    let directory = laid_out(&documents);

    let limits = CorpusLimits {
        max_document_bytes: 16,
        ..CorpusLimits::default()
    };
    assert!(
        matches!(
            Corpus::read(directory.path(), &limits),
            Err(ConformanceError::DocumentTooLarge { limit: 16, .. })
        ),
        "a document above the byte ceiling was read anyway"
    );

    let limits = CorpusLimits {
        max_entries: 1,
        ..CorpusLimits::default()
    };
    assert!(
        matches!(
            Corpus::read(directory.path(), &limits),
            Err(ConformanceError::TooManyEntries { limit: 1, .. })
        ),
        "a list above the entry ceiling was read anyway"
    );

    let limits = CorpusLimits {
        max_text_bytes: 4,
        ..CorpusLimits::default()
    };
    assert!(
        matches!(
            Corpus::read(directory.path(), &limits),
            Err(ConformanceError::TextTooLong { limit: 4, .. })
        ),
        "a string above the text ceiling was read anyway"
    );
}

/// Bounds that contradict one another describe a send no client could make.
#[test]
fn bounds_that_contradict_one_another_are_refused() {
    for (subject, value) in [
        ("max_attempts", json!(0)),
        ("max_attempts_cap", json!(1)),
        ("initial_backoff_ms", json!(u32::MAX)),
        ("max_backoff_ms", json!(u32::MAX)),
        ("request_timeout_ms", json!(0)),
        ("max_payload_bytes", json!(0)),
        ("max_response_bytes", json!(0)),
        ("max_response_headers", json!(0)),
        ("max_header_bytes", json!(0)),
    ] {
        let mut documents = committed();
        let bounds = documents
            .values_mut()
            .find_map(|document| document.get_mut("bounds"))
            .expect("the corpus declares bounds");
        bounds[subject] = value.clone();

        assert!(
            matches!(
                read(&laid_out(&documents)),
                Err(ConformanceError::UnusableBounds { .. })
            ),
            "`{subject}` set to {value} was accepted"
        );
    }
}

/// A vector refused under a name the corpus declares no refusal for says nothing a target could
/// map onto one of its own failures.
#[test]
fn a_vector_refused_under_an_undeclared_name_is_refused() {
    let mut documents = committed();
    let vectors = list(&mut documents, "vectors");
    let refused = vectors
        .iter_mut()
        .find(|vector| vector["verdict"] == json!("refused"))
        .expect("the corpus refuses at least one delivery");
    let name = refused["name"]
        .as_str()
        .expect("every vector is named")
        .to_owned();
    refused["refusal"] = json!("something_no_target_maps");

    assert_eq!(
        read(&laid_out(&documents)).expect_err("an undeclared refusal is refused"),
        ConformanceError::UnknownRefusal {
            vector: name,
            refusal: "something_no_target_maps".to_owned(),
        }
    );
}

/// A corpus where nothing verifies pins every way a delivery is refused and no way one is accepted,
/// which a client refusing everything would satisfy.
#[test]
fn a_corpus_where_nothing_verifies_is_refused() {
    let mut documents = committed();
    for vector in list(&mut documents, "vectors") {
        vector["verdict"] = json!("refused");
        vector["refusal"] = json!("code_mismatch");
    }

    assert_eq!(
        read(&laid_out(&documents)).expect_err("a corpus accepting nothing is refused"),
        ConformanceError::NothingVerifies
    );
}

/// A refusal the corpus declares and no vector exercises is a name every target would have to map
/// and none would ever meet.
#[test]
fn a_refusal_no_vector_exercises_is_refused() {
    let mut documents = committed();
    list(&mut documents, "refusals").push(json!("a_refusal_nothing_produces"));

    assert_eq!(
        read(&laid_out(&documents)).expect_err("a refusal nothing exercises is refused"),
        ConformanceError::UnexercisedRefusal {
            refusal: "a_refusal_nothing_produces".to_owned(),
        }
    );
}

/// The same thing named twice leaves one of the two unread, whichever list it happens in.
#[test]
fn a_list_naming_the_same_thing_twice_is_refused() {
    for subject in [
        "problems",
        "statuses",
        "vectors",
        "cases",
        "refusals",
        "causes",
        "occasions",
    ] {
        let mut documents = committed();
        let entries = list(&mut documents, subject);
        let repeated = entries[0].clone();
        entries.push(repeated);

        assert!(
            matches!(
                read(&laid_out(&documents)),
                Err(ConformanceError::Duplicated { .. })
            ),
            "`{subject}` naming the same entry twice was accepted"
        );
    }
}

/// Reading the corpus twice yields the same contract, so a target that read it before a change and
/// one that reads it after are told apart by the change alone.
#[test]
fn reading_the_corpus_twice_yields_the_same_contract() {
    assert_eq!(corpus(), corpus());
}

/// The corpus sits where every target's suite looks for it, and holds nothing else.
#[test]
fn the_corpus_is_where_the_targets_read_it() {
    let directory = PathBuf::from(CORPUS);

    assert!(
        directory.is_dir(),
        "{} is not a directory the targets could read",
        directory.display()
    );
    for entry in fs::read_dir(&directory).expect("the corpus directory is readable") {
        let path = entry.expect("every entry of the corpus is readable").path();
        assert!(
            path.is_file(),
            "{} is not a document a target could read",
            path.display()
        );
    }
}
