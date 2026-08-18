//! What pairing a target with a smoke answers, asked of real directories.
//!
//! Every target put in front of discovery here is one it was never written for, and every tree is
//! built by the test that reads it. That is the property: a client is smoked because the generator
//! declares it and a directory answers to its name, never because somebody remembered to add it to
//! a list. These cases would keep passing if every client this repository ships today were renamed.

use std::fs;
use std::path::Path;

use hook0_smoke::discovery::{Requirement, Smoke, discover};
use hook0_smoke::error::Error;

/// A languages directory with nothing in it but what a test puts there.
struct Tree(tempfile::TempDir);

impl Tree {
    fn new() -> Tree {
        Tree(tempfile::tempdir().expect("a temporary directory"))
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    /// A smoke directory declaring how it is run.
    fn smoke(&self, name: &str, run: &str) -> &Tree {
        let directory = self.path().join(name);
        fs::create_dir_all(&directory).expect("the smoke directory");
        fs::write(directory.join("smoke.toml"), run).expect("the manifest");
        self
    }
}

fn names(targets: &[&str]) -> Vec<String> {
    targets.iter().map(|name| (*name).to_owned()).collect()
}

fn refusal(targets: &[&str], tree: &Tree) -> Error {
    discover(&names(targets), tree.path()).expect_err("a refusal")
}

#[test]
fn a_target_is_paired_with_the_directory_answering_to_its_name() {
    let tree = Tree::new();
    tree.smoke("brainfuck", "run = [\"bf\", \"smoke.bf\"]");

    let found = discover(&names(&["brainfuck"]), tree.path()).expect("the pairing");

    assert_eq!(
        found,
        vec![Smoke {
            target: "brainfuck".to_owned(),
            directory: tree.path().join("brainfuck"),
            command: vec!["bf".to_owned(), "smoke.bf".to_owned()],
            requires: vec![],
            // Absent from the manifest, so the smoke is held to every operation the API document
            // declares. A client added tomorrow can be in no other state.
            drives_surface: true,
        }]
    );
}

#[test]
fn a_smoke_declares_what_it_needs_before_it_runs_and_what_that_sets() {
    let tree = Tree::new();
    tree.smoke(
        "brainfuck",
        "run = [\"bf\", \"smoke.bf\"]\n\
         [[requires]]\n\
         run = [\"bfrocks\", \"path\"]\n\
         sets = \"BF_PATH\"\n\
         suffix = \";;\"\n\
         remedy = \"bfrocks install bfsocket\"\n",
    );

    let found = discover(&names(&["brainfuck"]), tree.path()).expect("the pairing");

    assert_eq!(
        found[0].requires,
        vec![Requirement {
            run: vec!["bfrocks".to_owned(), "path".to_owned()],
            sets: Some("BF_PATH".to_owned()),
            suffix: ";;".to_owned(),
            remedy: "bfrocks install bfsocket".to_owned(),
        }]
    );
}

#[test]
fn a_requirement_that_cannot_be_asked_is_refused_with_the_line_that_fixes_it() {
    // The point of the remedy: a machine without the package manager is told how to get one,
    // rather than being handed whatever the missing program's absence looks like.
    let tree = Tree::new();
    tree.smoke(
        "brainfuck",
        "run = [\"bf\", \"smoke.bf\"]\n\
         [[requires]]\n\
         run = [\"a-package-manager-no-machine-has\", \"path\"]\n\
         sets = \"BF_PATH\"\n\
         remedy = \"apt-get install bfrocks\"\n",
    );
    let found = discover(&names(&["brainfuck"]), tree.path()).expect("the pairing");

    let refused = found[0].satisfied().expect_err("a refusal");

    let said = format!("{refused}");
    assert!(said.contains("apt-get install bfrocks"), "{said}");
    assert!(said.contains("brainfuck"), "the smoke is named: {said}");
}

#[test]
fn a_requirement_that_holds_answers_the_variable_it_sets() {
    let tree = Tree::new();
    tree.smoke(
        "brainfuck",
        "run = [\"bf\", \"smoke.bf\"]\n\
         [[requires]]\n\
         run = [\"echo\", \"/where/rocks/went\"]\n\
         sets = \"BF_PATH\"\n\
         suffix = \";;\"\n\
         remedy = \"unreachable\"\n\
         [[requires]]\n\
         run = [\"true\"]\n\
         remedy = \"unreachable\"\n",
    );
    let found = discover(&names(&["brainfuck"]), tree.path()).expect("the pairing");

    let derived = found[0].satisfied().expect("the requirements hold");

    // The second sets nothing, so it contributes nothing but its verdict.
    assert_eq!(
        derived,
        vec![("BF_PATH".to_owned(), "/where/rocks/went;;".to_owned())]
    );
}

#[test]
fn a_requirement_that_fails_is_refused_even_though_the_program_is_there() {
    let tree = Tree::new();
    tree.smoke(
        "brainfuck",
        "run = [\"bf\", \"smoke.bf\"]\n\
         [[requires]]\n\
         run = [\"false\"]\n\
         remedy = \"bfrocks install bfsocket 1.2.3\"\n",
    );
    let found = discover(&names(&["brainfuck"]), tree.path()).expect("the pairing");

    let refused = found[0].satisfied().expect_err("a refusal");

    assert!(
        format!("{refused}").contains("bfrocks install bfsocket 1.2.3"),
        "{refused}"
    );
}

#[test]
fn a_requirement_naming_no_remedy_is_refused_because_it_would_teach_nobody() {
    let tree = Tree::new();
    tree.smoke(
        "brainfuck",
        "run = [\"bf\", \"smoke.bf\"]\n\
         [[requires]]\n\
         run = [\"bfrocks\", \"path\"]\n",
    );

    let refused = refusal(&["brainfuck"], &tree);

    assert!(format!("{refused}").contains("remedy"), "{refused}");
}

#[test]
fn the_registry_decides_the_order_and_every_target_is_paired() {
    let tree = Tree::new();
    tree.smoke("prolog", "run = [\"swipl\"]");
    tree.smoke("forth", "run = [\"gforth\"]");

    let found = discover(&names(&["forth", "prolog"]), tree.path()).expect("the pairing");

    let paired: Vec<&str> = found.iter().map(|smoke| smoke.target.as_str()).collect();
    assert_eq!(paired, vec!["forth", "prolog"]);
}

#[test]
fn a_target_the_tree_does_not_answer_to_is_refused_rather_than_skipped() {
    let tree = Tree::new();
    tree.smoke("prolog", "run = [\"swipl\"]");

    let refused = refusal(&["prolog", "forth"], &tree);

    let said = format!("{refused}");
    assert!(said.contains("forth"), "the missing one is named: {said}");
    assert!(
        !said.contains("prolog"),
        "the one that is there is not accused: {said}"
    );
}

#[test]
fn a_directory_naming_no_target_is_refused_because_nothing_would_run_it() {
    let tree = Tree::new();
    tree.smoke("prolog", "run = [\"swipl\"]");
    tree.smoke("cobol", "run = [\"cobc\"]");

    let refused = refusal(&["prolog"], &tree);

    let said = format!("{refused}");
    assert!(said.contains("cobol"), "the stray one is named: {said}");
}

#[test]
fn a_smoke_that_does_not_say_how_it_is_run_is_refused() {
    let tree = Tree::new();
    tree.smoke("prolog", "started = \"somehow\"");

    let refused = refusal(&["prolog"], &tree);

    let said = format!("{refused}");
    assert!(said.contains("run"), "what is missing is named: {said}");
}

#[test]
fn a_smoke_naming_no_program_is_refused() {
    let tree = Tree::new();
    tree.smoke("prolog", "run = []");

    let refused = refusal(&["prolog"], &tree);

    assert!(format!("{refused}").contains("no program"), "{refused}");
}

#[test]
fn what_this_repository_declares_today_is_paired_with_what_is_beside_it() {
    let targets: Vec<String> = hook0_sdkgen::targets::targets()
        .iter()
        .map(|target| target.name.to_owned())
        .collect();
    let languages = Path::new(env!("CARGO_MANIFEST_DIR")).join("languages");

    let found = discover(&targets, &languages).expect("every target paired with a smoke");

    assert_eq!(
        found.len(),
        targets.len(),
        "one smoke per target the generator declares"
    );
}

#[test]
fn a_smoke_that_does_not_drive_the_surface_yet_says_so_in_its_own_manifest() {
    // The only thing that tells a language still to be written from one that has stopped
    // reporting. It lives beside the smoke rather than in a list of ported languages the harness
    // keeps, so porting a language and saying it has been ported are the same edit.
    let tree = Tree::new();
    tree.smoke(
        "brainfuck",
        "run = [\"bf\", \"smoke.bf\"]\ndrives_surface = false\n",
    );

    let found = discover(&names(&["brainfuck"]), tree.path()).expect("the pairing");

    assert!(!found[0].drives_surface);
}

#[test]
fn a_manifest_saying_something_other_than_true_or_false_about_the_surface_is_refused() {
    let tree = Tree::new();
    tree.smoke(
        "brainfuck",
        "run = [\"bf\", \"smoke.bf\"]\ndrives_surface = \"later\"\n",
    );

    let said = format!("{}", refusal(&["brainfuck"], &tree));

    assert!(said.contains("drives_surface"), "{said}");
    assert!(said.contains("true"), "{said}");
}
