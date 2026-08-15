//! What pairing a target with a smoke answers, asked of real directories.
//!
//! Every target put in front of discovery here is one it was never written for, and every tree is
//! built by the test that reads it. That is the property: a client is smoked because the generator
//! declares it and a directory answers to its name, never because somebody remembered to add it to
//! a list. These cases would keep passing if every client this repository ships today were renamed.

use std::fs;
use std::path::Path;

use hook0_smoke::discovery::{Smoke, discover};
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
        }]
    );
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
