//! The guard: what the registry says the dashboard shows, and what the tree actually holds.
//!
//! Strict in both directions and carrying no list of exceptions. A target held to the whole
//! conformance corpus without its examples fails, and an artefact naming an SDK the registry does
//! not fails with it — one of the two is wrong either way, and neither is knowable from the other
//! alone.

use std::path::PathBuf;

use hook0_dashboard_examples::{
    ARTEFACT, REGENERATE, emit,
    error::Error,
    manifest::{self, Proof},
    sdks, shown,
};

mod common;
mod declaration;

/// The artefact as it is committed, or nothing when it has never been written.
fn committed() -> Option<String> {
    std::fs::read_to_string(common::tree().join(ARTEFACT)).ok()
}

/// Every SDK the dashboard shows owes it three files, and owes them under the names its own
/// language spells them with.
#[test]
fn every_shown_sdk_carries_its_examples() {
    let tree = common::tree();
    let mut missing = Vec::new();

    for target in shown().expect("the registry carries more targets than the ceiling allows") {
        for path in [target.send(), target.verify(), target.manifest()] {
            if !tree.join(&path).is_file() {
                missing.push(format!("  {} owes {path}", target.target));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "every target held to the whole conformance corpus shows the dashboard how it sends, how \
         it verifies and how its language spells things, and these do not:\n{}",
        missing.join("\n")
    );
}

/// A directory under `clients/` no target of the registry claims, spelled so that no language the
/// repository grows can ever land on it.
const UNCLAIMED: &str = "clients/no-such-language";

/// Where a client keeps what the dashboard shows.
const EXAMPLES: &str = "examples";

/// A tree holding nothing but that one directory.
///
/// Examples nothing claims are refused before anything else in `sdks` is read, so a tree this thin
/// reaches that refusal and nothing past it. The repository itself cannot stand in: it is claimed
/// all the way through, which is what the guard is for and what leaves this half of it unreachable
/// from there.
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    /// Named after the running process and the case, since the suites of this crate share the
    /// temporary directory and one path written by two of them is one file written by two of them.
    fn new(named: &str) -> Scratch {
        let root = std::env::temp_dir().join(format!(
            "hook0-dashboard-examples-tree-{}-{named}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(UNCLAIMED).join(EXAMPLES))
            .expect("the temporary directory is not writable");
        Scratch { root }
    }

    /// The path the refusal names, whether or not anything was written to it.
    fn path_of(&self, name: &str) -> String {
        format!("{UNCLAIMED}/{EXAMPLES}/{name}")
    }

    fn write(&self, name: &str, body: &str) {
        std::fs::write(self.root.join(self.path_of(name)), body)
            .expect("the temporary directory is not writable");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// The mirror image: examples under a directory no shown target claims stop the run, naming both
/// the file and the directory holding it.
///
/// Nothing would compile them and nothing would show them, which is what the day a target's
/// contract narrows and its examples stay behind looks like from here.
#[test]
fn examples_under_a_directory_no_target_claims_are_refused() {
    let scratch = Scratch::new("unclaimed");
    scratch.write(manifest::FILE, "display_name = \"Unclaimed\"\n");

    let refused = sdks(&scratch.root).expect_err("examples no target claims were read as an SDK's");

    match refused {
        Error::ExamplesWithoutTarget { path, directory } => {
            assert_eq!(
                path,
                scratch.path_of(manifest::FILE),
                "the refusal names a file other than the one left behind"
            );
            assert_eq!(
                directory, UNCLAIMED,
                "the refusal names a directory other than the one holding it"
            );
        }
        other => panic!("examples no target claims were refused for something else: {other}"),
    }
}

/// A snippet left behind without the manifest beside it is refused just the same, and named.
///
/// The manifest is the file a person edits, so it is the one likeliest to be taken away on purpose
/// — leaving two snippets nothing compiles and nothing shows, which is the whole of what this
/// direction is about. The extension is one no language here spells, so what recognises the file is
/// its stem rather than a list of the extensions the repository happens to hold today.
#[test]
fn a_snippet_left_without_its_manifest_is_refused() {
    let orphan = "dashboard_send.nosuchlanguage";
    let scratch = Scratch::new("orphan");
    scratch.write(orphan, "send();\n");

    let refused = sdks(&scratch.root).expect_err("a snippet no target claims was read as an SDK's");

    match refused {
        Error::ExamplesWithoutTarget { path, directory } => {
            assert_eq!(
                path,
                scratch.path_of(orphan),
                "the refusal names a file other than the one left behind"
            );
            assert_eq!(
                directory, UNCLAIMED,
                "the refusal names a directory other than the one holding it"
            );
        }
        other => panic!("a snippet no target claims was refused for something else: {other}"),
    }
}

/// A file in that directory which is none of the three a target owes is left where it is.
///
/// The direction is about examples nothing compiles, not about a directory being tidy. Refusing
/// whatever else somebody put beside them would make it a guard about the wrong thing, and the
/// next person would move the note somewhere the guard cannot see rather than argue with it.
#[test]
fn a_file_no_target_owes_is_left_where_it_is() {
    let scratch = Scratch::new("bystander");
    scratch.write("README.md", "why this directory is still here\n");

    let refused = sdks(&scratch.root).expect_err("a tree carrying no package at all read as one");

    assert!(
        !matches!(refused, Error::ExamplesWithoutTarget { .. }),
        "a file no target owes was swept up with the ones they do: {refused}"
    );
}

/// The same directory with nothing in it is refused for something else.
///
/// A tree this thin fails several ways over — nothing in it says what installs anything — so a test
/// reading only that something failed would read every one of those as this one.
#[test]
fn a_directory_no_target_claims_is_not_refused_for_examples_it_does_not_hold() {
    let scratch = Scratch::new("bare");

    let refused = sdks(&scratch.root).expect_err("a tree carrying no package at all read as one");

    assert!(
        !matches!(refused, Error::ExamplesWithoutTarget { .. }),
        "`{UNCLAIMED}` holds no `{}` and was refused as though it did: {refused}",
        manifest::FILE
    );
}

/// A `clients/` that cannot be read stops the run rather than reporting a clean sweep.
///
/// This direction is a sweep, and a sweep that answers "nothing here" because it could not look has
/// stopped being one — every example left behind would pass, quietly, for as long as the directory
/// stayed unreadable. A file where the directory should be is what makes that unreadable on any
/// machine and for any user, which a permission bit does not.
#[test]
fn a_clients_directory_that_cannot_be_read_is_refused() {
    let root = std::env::temp_dir().join(format!(
        "hook0-dashboard-examples-tree-{}-unreadable",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the temporary directory is not writable");
    std::fs::write(root.join("clients"), "not a directory\n").expect("the file is not writable");

    let refused = sdks(&root).expect_err("a tree whose `clients` cannot be read was swept clean");
    let _ = std::fs::remove_dir_all(&root);

    assert!(
        matches!(refused, Error::ReadFile { .. }),
        "a `clients` nothing can read was passed over rather than refused: {refused}"
    );
}

/// The whole of it, read: every file parses, every region comes out, every marker is one the
/// dashboard substitutes, and every package resolves to something that installs it.
#[test]
fn every_shown_sdk_reads() {
    let found = sdks(&common::tree()).expect("the dashboard examples do not read");
    assert_eq!(
        found.len(),
        shown().expect("the registry is unreadable").len(),
        "the dashboard reads a different number of SDKs than the registry shows"
    );
}

/// The artefact names the SDKs the registry shows, and no others.
///
/// The other direction of the same question. An entry left behind by a target whose contract
/// narrowed would go on being offered on screen, pointing at examples nothing compiles any more.
#[test]
fn the_artefact_names_what_the_registry_shows() {
    let artefact = committed()
        .unwrap_or_else(|| panic!("{ARTEFACT} has never been written; `{REGENERATE}` writes it"));

    let named: Vec<String> = artefact
        .lines()
        .filter_map(|line| line.trim().strip_prefix("target: "))
        .map(|value| value.trim_matches([' ', ',', '\'']).to_owned())
        .collect();

    let mut expected: Vec<String> = shown()
        .expect("the registry is unreadable")
        .into_iter()
        .map(|target| target.target)
        .collect();
    expected.sort();

    let mut carried = named.clone();
    carried.sort();

    assert_eq!(
        carried, expected,
        "{ARTEFACT} offers {named:?} and the registry shows {expected:?}; `{REGENERATE}` settles it"
    );
}

/// What is committed is what the examples say.
///
/// The artefact is committed so that a change to a snippet is visible in review and so that
/// building the frontend needs no Rust toolchain. Both of those are worth having only while the two
/// agree, which is what this asks.
#[test]
fn the_artefact_is_fresh() {
    let tree = common::tree();
    let regenerated = emit::artefact(&sdks(&tree).expect("the dashboard examples do not read"));
    let committed = committed()
        .unwrap_or_else(|| panic!("{ARTEFACT} has never been written; `{REGENERATE}` writes it"));

    if committed == regenerated {
        return;
    }

    panic!(
        "{ARTEFACT} is not what the examples say any more.\n\n{}\n\n`{REGENERATE}` rewrites \
         it. If nobody touched it, `npm run lint:fix` in `frontend/` did, on somebody's own \
         machine: it runs `eslint --fix` over `src/`, `src/generated` is not ignored, and a \
         generated file Prettier would write differently is rewritten in place. No pipeline job \
         runs that script, so a rewrite came from a checkout rather than from CI.",
        difference(&committed, &regenerated)
    );
}

/// How many bytes of a differing line are shown before a report stops quoting it.
const MAX_QUOTED_CHARS: usize = 120;

/// How many differing lines a report lists before it stops.
const MAX_QUOTED_LINES: usize = 20;

/// Where the two disagree, as far as a report goes before it stops counting.
fn difference(committed: &str, regenerated: &str) -> String {
    let mut report = Vec::new();
    let committed: Vec<&str> = committed.lines().collect();
    let regenerated: Vec<&str> = regenerated.lines().collect();

    for at in 0..committed.len().max(regenerated.len()) {
        if report.len() == MAX_QUOTED_LINES {
            report.push("  … and more".to_owned());
            break;
        }
        let was = committed.get(at).copied().unwrap_or("");
        let is = regenerated.get(at).copied().unwrap_or("");
        if was != is {
            report.push(format!(
                "  line {}:\n    committed: {}\n    examples:  {}",
                at + 1,
                quoted(was),
                quoted(is)
            ));
        }
    }
    report.join("\n")
}

fn quoted(line: &str) -> String {
    match line.chars().count() > MAX_QUOTED_CHARS {
        true => format!(
            "{}…",
            line.chars().take(MAX_QUOTED_CHARS).collect::<String>()
        ),
        false => line.to_owned(),
    }
}

/// Every language declares how far the job carrying it goes, and it declares one of the three
/// levels the repository has a word for.
///
/// Not one level, three: `compiled` is out of reach for Ruby and for Lua, where nothing standard
/// resolves a symbol at all. That is a property of those ecosystems rather than work left undone,
/// and an honest `parsed` is worth more than a `compiled` that is not true — which is the kind of
/// claim this whole arrangement exists to remove.
#[test]
fn every_language_says_how_far_its_proof_goes() {
    let tree = common::tree();
    let mut declared = Vec::new();

    for target in shown().expect("the registry is unreadable") {
        let read = manifest::read(&tree.join(target.manifest()))
            .unwrap_or_else(|cause| panic!("`{}` declares nothing usable: {cause}", target.target));
        assert!(
            !read.proves.trim().is_empty(),
            "`{}` claims `{}` and says nothing about what it rests on",
            target.target,
            read.proof.declared()
        );
        declared.push((target.target, read.proof));
    }

    // One is all this asks for, and all it can ask for: the level a language declares is its own
    // and several of them are honestly below this one. What holds each language to the level it
    // does declare is `every_language_names_what_puts_its_examples_under_its_job` below.
    assert!(
        declared.iter().any(|(_, level)| *level == Proof::Compiled),
        "no language at all is built against its client, which leaves the highest level standing \
         for nothing"
    );
}

/// What each language says puts its examples under its job is still in the file it names.
///
/// Every level rests on a line somebody wrote — a compiled source root, a `paths` entry, a target
/// declaration. Delete it and the examples stay exactly where they are while nothing reads them any
/// more: the manifest goes on claiming `compiled`, every file the guard above looks for is present,
/// and the job it names passes over a directory it no longer opens. A missing example is caught;
/// this is the one that is merely no longer read.
///
/// Some languages name nothing, because their command reads a tree rather than a directory. There
/// is no line of theirs to delete, so there is none to hold, and their manifests say which command
/// it is instead of pretending otherwise.
#[test]
fn every_language_names_what_puts_its_examples_under_its_job() {
    let tree = common::tree();
    let mut lost = Vec::new();

    for target in shown().expect("the registry is unreadable") {
        let read = manifest::read(&tree.join(target.manifest()))
            .unwrap_or_else(|cause| panic!("`{}` declares nothing usable: {cause}", target.target));

        let manifest::Reach::Named { file, lines } = read.reach else {
            continue;
        };
        let Ok(configured) = std::fs::read_to_string(tree.join(&file)) else {
            lost.push(format!(
                "  {}: `{file}` is not there to be read",
                target.target
            ));
            continue;
        };
        for line in lines {
            if !configured.contains(&line) {
                lost.push(format!(
                    "  {}: `{file}` no longer carries `{line}`",
                    target.target
                ));
            }
        }
    }

    assert!(
        lost.is_empty(),
        "these declare how far their job proves their examples, and the configuration that was \
         putting the examples under that job no longer says so:\n{}",
        lost.join("\n")
    );
}

/// A manifest saying nothing about what puts its examples under its job is refused.
#[test]
fn a_manifest_that_says_nothing_about_what_reaches_its_examples_is_refused() {
    let mut declared = declaration::Declaration::new("unreached");
    declared.reach = String::new();

    let refused = manifest::read(&declared.written())
        .expect_err("a level resting on nothing anybody named was accepted");
    assert!(
        matches!(refused, Error::ExamplesReachUnsaid { .. }),
        "a manifest naming neither was refused for something else: {refused}"
    );
}

/// One saying it both ways is refused too.
///
/// A language declaring a line to hold and a command that names no directory has said nothing about
/// which of the two the level rests on, and reading the first would settle that on its behalf.
#[test]
fn a_manifest_saying_both_ways_its_examples_are_reached_is_refused() {
    let mut declared = declaration::Declaration::new("reached-twice");
    declared
        .reach
        .push_str("\nexamples_swept_by = \"a command reading the whole client\"");

    let refused = manifest::read(&declared.written())
        .expect_err("a manifest declaring a line to hold and a sweep at once was accepted");
    assert!(
        matches!(refused, Error::ExamplesReachSaidTwice { .. }),
        "a manifest naming both was refused for something else: {refused}"
    );
}

/// A manifest naming nothing extra is the ordinary case, and reads as nothing extra.
///
/// The key is the one a manifest may leave out, and nine of the eleven do. Held here so that
/// making it compulsory shows up as this rather than as nine languages failing at once.
#[test]
fn a_manifest_naming_nothing_else_its_snippet_needs_is_read() {
    let declared = declaration::Declaration::new("also-needs-omitted");

    let read = manifest::read(&declared.written())
        .expect("a manifest naming nothing beyond its package was refused");
    assert_eq!(
        read.snippet_also_needs, None,
        "a manifest that named nothing came back naming something"
    );
}

/// What a manifest says a reader needs beyond the package is read, and is what comes back.
#[test]
fn what_else_a_snippet_needs_is_read() {
    let wired = "cargo add tokio --features macros,rt-multi-thread";
    let mut declared = declaration::Declaration::new("also-needs-read");
    declared.snippet_also_needs = format!("snippet_also_needs = \"{wired}\"");

    let read = manifest::read(&declared.written())
        .expect("a manifest naming what else its snippet needs was refused");
    assert_eq!(
        read.snippet_also_needs.as_deref(),
        Some(wired),
        "what the manifest says a reader needs is not what came back"
    );
}

/// A substitution marker in it is refused.
///
/// The screen renders the install block through the same table as the two snippets, so a marker
/// the dashboard knows puts whatever the reader typed into the form inside a command they were
/// told to run, and one it does not know is copied out as it stands. Neither is anything an
/// install step says. The test above is the positive control: without it this would pass just as
/// well over a key nothing ever reads.
#[test]
fn a_marker_in_what_else_a_snippet_needs_is_refused() {
    let mut declared = declaration::Declaration::new("also-needs-marker");
    declared.snippet_also_needs =
        "snippet_also_needs = \"cargo add tokio --features __HOOK0_PAYLOAD__\"".to_owned();

    let refused = manifest::read(&declared.written())
        .expect_err("a marker in what a reader runs was accepted");
    assert!(
        matches!(refused, Error::MarkerInAlsoNeeds { .. }),
        "a marker in what a reader runs was refused for something else: {refused}"
    );
}

/// A level nobody has defined is refused, naming the three that are.
#[test]
fn a_proof_nobody_defined_is_refused() {
    let mut declared = declaration::Declaration::new("proof");
    declared.proof = "verified".to_owned();

    let refused =
        manifest::read(&declared.written()).expect_err("a level nobody has defined was accepted");
    let said = refused.to_string();
    for level in ["compiled", "type-checked", "parsed"] {
        assert!(
            said.contains(level),
            "the refusal does not offer `{level}`: {said}"
        );
    }
}

/// A share written as a whole number is refused with the correction in the message.
///
/// TOML reads `66` as an integer and `66.0` as a float, and a round figure among them is the one
/// most likely to be typed the first way.
#[test]
fn a_share_written_as_a_whole_number_is_refused() {
    let mut declared = declaration::Declaration::new("share");
    declared.usage_share = "66".to_owned();

    let refused = manifest::read(&declared.written()).expect_err("an integer share was accepted");
    assert!(
        refused.to_string().contains("66.0"),
        "the refusal does not say how to write it: {refused}"
    );
}

/// A share with nothing above it saying where it came from is refused.
#[test]
fn a_share_without_its_survey_is_refused() {
    let mut declared = declaration::Declaration::new("source");
    declared.usage_source = String::new();

    assert!(matches!(
        manifest::read(&declared.written()),
        Err(Error::UsageShareWithoutSource { .. })
    ));
}

/// Every share is read off the same survey, compared as it is written.
///
/// This is what makes declaring the figure once per language safe. A survey replaces the last in
/// one go, so some manifests moved and the rest forgotten is not a difference of opinion about a
/// number: it is an order that is part one year and part another, and nothing else would say so.
#[test]
fn every_share_is_read_off_the_same_survey() {
    let tree = common::tree();
    let mut surveys: Vec<(String, String)> = Vec::new();

    for target in shown().expect("the registry is unreadable") {
        let read = manifest::read(&tree.join(target.manifest()))
            .unwrap_or_else(|cause| panic!("`{}` declares nothing usable: {cause}", target.target));
        surveys.push((target.target, read.usage_source));
    }

    let Some((named, survey)) = surveys.first().cloned() else {
        panic!("the registry shows no SDK at all");
    };
    for (target, read) in &surveys {
        assert_eq!(
            read, &survey,
            "`{target}` reads its share off a different survey than `{named}`, so the order the \
             languages are offered in is part one and part the other"
        );
    }
}

/// The artefact is emitted in the layout Prettier would have written, not merely one it accepts.
///
/// This is not tidiness. `frontend.check` runs `npm run lint`, whose ESLint config extends
/// `plugin:prettier-vue/recommended` — Prettier as a lint rule, at `error` — over `src/`, and
/// `src/generated` is not ignored. So a generated file written in a layout Prettier disagrees with
/// is not tidied away: it fails that job, on a file no frontend developer touched, over a
/// difference this crate introduced. Emitting what Prettier emits is what keeps that job green, so
/// the rules that make it so are held here rather than remembered.
#[test]
fn the_artefact_is_written_the_way_prettier_writes() {
    // The quote is whichever leaves fewer escapes, and single when it is a tie.
    assert_eq!(emit::literal("plain"), "'plain'");
    assert_eq!(emit::literal("a \"quoted\" word"), "'a \"quoted\" word'");
    // A tie keeps the preferred quote, which is why this one stays single; one more apostrophe
    // than there are quotes is what tips it over. Both read off Prettier rather than reasoned out.
    assert_eq!(emit::literal("it's it's \"x\""), "'it\\'s it\\'s \"x\"'");
    assert_eq!(
        emit::literal("it's it's it's \"x\""),
        "\"it's it's it's \\\"x\\\"\""
    );
    assert_eq!(emit::literal("a\\b\nc"), "'a\\\\b\\nc'");

    // A value past the margin moves to a line of its own, one indent further in.
    let long = "x".repeat(120);
    let broken = emit::field(4, "install", &long);
    assert_eq!(
        broken,
        format!("    install:\n      '{long}',\n"),
        "a value past the margin was left beside its key"
    );

    // Except under a key Prettier never breaks after, which is why `body` runs past the margin and
    // `install` does not.
    let kept = emit::field(6, "body", &long);
    assert_eq!(
        kept,
        format!("      body: '{long}',\n"),
        "a short key was broken after"
    );
    assert_eq!(
        emit::field(4, "version", "2.0.2"),
        "    version: '2.0.2',\n"
    );

    // An array of pairs breaks onto one pair per line whatever its width; a single pair does not.
    let pair = |from: &str, to: &str| (from.to_owned(), to.to_owned());
    assert_eq!(
        emit::escapes(&[pair("\\", "\\\\"), pair("\"", "\\\"")]),
        "      escape: [\n        ['\\\\', '\\\\\\\\'],\n        ['\"', '\\\\\"'],\n      ],\n"
    );
    assert_eq!(
        emit::escapes(&[pair("\\", "\\\\")]),
        "      escape: [['\\\\', '\\\\\\\\']],\n"
    );
}
