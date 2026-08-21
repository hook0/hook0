//! What discovery answers, asked of real directories.
//!
//! Every tree here is built by the test that reads it, and every target put in front of the tool is
//! one it was never written for — which is the property under test: an example is proven because
//! its page is in the tree and its language is in the generator's registry, never because somebody
//! remembered to add it to a list. The suite would keep passing if every client this repository
//! ships today were renamed.

use std::fs;
use std::path::Path;

use hook0_documentation_examples::harness::{self, Region};
use hook0_documentation_examples::manifest::{self, Proof};
use hook0_documentation_examples::{Error, TargetRoot, discover, registry};

/// A tree with nothing in it but what a test puts there.
struct Tree(tempfile::TempDir);

impl Tree {
    fn new() -> Tree {
        Tree(tempfile::tempdir().expect("a temporary directory"))
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    fn write(&self, relative: &str, body: &str) -> &Tree {
        let path = self.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the directories above the file");
        }
        fs::write(&path, body).expect("the file");
        self
    }

    /// A page claiming a target, showing one example of it.
    fn page(&self, name: &str, target: &str, language: &str) -> &Tree {
        self.write(
            &format!("{name}.md"),
            &format!(
                "---\ntitle: \"a page\"\nsdkTarget: {target}\n---\n\n# a page\n\n\
                 ```{language} example=only\nthe snippet\n```\n"
            ),
        )
    }

    /// Everything a language needs beside its pages for discovery to be satisfied.
    fn harness(&self, target: &str) -> &Tree {
        self.write(
            &format!("examples/{target}/examples.toml"),
            "proof = \"parsed\"\nproves = \"read\"\npath = \"{{name}}.txt\"\n\
             timeout_seconds = 10\neach = [[\"true\", \"{{file}}\"]]\n",
        )
        .write(
            &format!("examples/{target}/harness.txt"),
            "# HARNESS only\nEXAMPLE\n# END HARNESS\n",
        )
    }
}

fn target(name: &str) -> TargetRoot {
    TargetRoot {
        name: name.to_owned(),
        client: format!("clients/{name}"),
    }
}

fn refusal(tree: &Tree, targets: &[TargetRoot]) -> Error {
    match discover(targets, tree.path()) {
        Ok(found) => panic!("expected a refusal, got {found:?}"),
        Err(error) => error,
    }
}

// --- Nothing is written down beside the tree ----------------------------------------------------

#[test]
fn a_language_the_checker_was_never_written_for_is_discovered() {
    let tree = Tree::new();
    tree.page("nobody", "nobody", "nobody").harness("nobody");

    let documentation =
        discover(&[target("nobody")], tree.path()).expect("what the tree describes");

    let [language] = &documentation.languages[..] else {
        panic!("one language, got {:?}", documentation.languages);
    };
    assert_eq!(language.name, "nobody");
    assert_eq!(language.examples.len(), 1);
    assert_eq!(language.examples[0].region, "only");
}

#[test]
fn the_registry_is_where_the_languages_come_from() {
    let found = registry();
    assert!(
        found.len() > 1,
        "the registry answered {found:?}, which is not a set of targets"
    );
    for target in &found {
        assert!(
            target.client.starts_with("clients/"),
            "{target:?} does not land inside a client"
        );
    }
}

// --- A page and a target that do not line up ----------------------------------------------------

#[test]
fn a_page_for_a_target_with_no_example_is_refused() {
    let tree = Tree::new();
    tree.write(
        "silent.md",
        "---\nsdkTarget: nobody\n---\n\n# a page\n\nno example at all.\n",
    )
    .harness("nobody");

    let Error::PageWithoutExample { page, target } = refusal(&tree, &[target("nobody")]) else {
        panic!("expected the page to be refused for showing nothing");
    };
    assert_eq!(page, "silent.md");
    assert_eq!(target, "nobody");
}

#[test]
fn a_page_whose_examples_are_all_deleted_is_refused_rather_than_passing() {
    let tree = Tree::new();
    tree.page("nobody", "nobody", "nobody").harness("nobody");
    discover(&[target("nobody")], tree.path()).expect("the tree before the deletion");

    tree.write(
        "nobody.md",
        "---\nsdkTarget: nobody\n---\n\n# a page\n\nthe examples were taken out.\n",
    );

    assert!(matches!(
        refusal(&tree, &[target("nobody")]),
        Error::PageWithoutExample { .. }
    ));
}

#[test]
fn a_page_claiming_a_target_the_registry_does_not_produce_is_refused() {
    let tree = Tree::new();
    tree.page("elsewhere", "elsewhere", "nobody")
        .harness("nobody");

    let Error::PageClaimsUnknownTarget {
        page,
        target: claimed,
        ..
    } = refusal(&tree, &[target("nobody")])
    else {
        panic!("expected the claim to be refused");
    };
    assert_eq!(page, "elsewhere.md");
    assert_eq!(claimed, "elsewhere");
}

#[test]
fn a_page_claiming_nothing_at_all_is_refused() {
    let tree = Tree::new();
    tree.write("mute.md", "# a page with no front matter\n")
        .harness("nobody");

    assert!(matches!(
        refusal(&tree, &[target("nobody")]),
        Error::PageClaimsNothing { .. }
    ));
}

#[test]
fn two_pages_claiming_one_target_are_refused() {
    let tree = Tree::new();
    tree.page("first", "nobody", "nobody")
        .page("second", "nobody", "nobody")
        .harness("nobody");

    let Error::TargetClaimedTwice {
        target: claimed, ..
    } = refusal(&tree, &[target("nobody")])
    else {
        panic!("expected the second claim to be refused");
    };
    assert_eq!(claimed, "nobody");
}

#[test]
fn an_overview_claiming_no_target_still_has_its_examples_proven() {
    let tree = Tree::new();
    tree.write(
        "index.md",
        "---\nsdkTarget: none\n---\n\n# the overview\n\n\
         ```nobody example=only\nthe first snippet a reader meets\n```\n",
    )
    .harness("nobody");

    let documentation =
        discover(&[target("nobody")], tree.path()).expect("what the tree describes");
    let [language] = &documentation.languages[..] else {
        panic!("one language, got {:?}", documentation.languages);
    };
    assert_eq!(language.examples.len(), 1);
    assert_eq!(language.page, None);
}

// --- A fence and a registry that do not line up -------------------------------------------------

#[test]
fn an_example_in_a_language_no_target_claims_is_refused() {
    let tree = Tree::new();
    tree.write(
        "nobody.md",
        "---\nsdkTarget: nobody\n---\n\n```nobody example=only\nfine\n```\n\
         \n```elsewhere example=only\nnot fine\n```\n",
    )
    .harness("nobody");

    let Error::ExampleInUnclaimedLanguage { page, language, .. } =
        refusal(&tree, &[target("nobody")])
    else {
        panic!("expected the unclaimed language to be refused");
    };
    assert_eq!(page, "nobody.md");
    assert_eq!(language, "elsewhere");
}

#[test]
fn an_example_that_says_nothing_about_how_it_is_assembled_is_refused() {
    let tree = Tree::new();
    tree.write(
        "nobody.md",
        "---\nsdkTarget: nobody\n---\n\n```nobody\nunmarked\n```\n",
    )
    .harness("nobody");

    let Error::ExampleWithoutRegion { page, line, .. } = refusal(&tree, &[target("nobody")]) else {
        panic!("expected the unmarked example to be refused");
    };
    assert_eq!(page, "nobody.md");
    assert_eq!(line, 5);
}

#[test]
fn a_block_in_a_language_no_target_claims_is_left_alone_when_it_claims_nothing() {
    let tree = Tree::new();
    tree.write(
        "nobody.md",
        "---\nsdkTarget: nobody\n---\n\n```bash\ninstall it\n```\n\
         \n```nobody example=only\nthe snippet\n```\n",
    )
    .harness("nobody");

    let documentation =
        discover(&[target("nobody")], tree.path()).expect("prose is not an example");
    assert_eq!(documentation.languages[0].examples.len(), 1);
}

#[test]
fn an_example_asking_for_a_region_the_harness_does_not_declare_is_refused() {
    let tree = Tree::new();
    tree.write(
        "nobody.md",
        "---\nsdkTarget: nobody\n---\n\n```nobody example=absent\nthe snippet\n```\n",
    )
    .harness("nobody");

    let Error::UnknownRegion { region, .. } = refusal(&tree, &[target("nobody")]) else {
        panic!("expected the missing region to be refused");
    };
    assert_eq!(region, "absent");
}

// --- A language and its harness that do not line up ---------------------------------------------

#[test]
fn a_documented_target_with_no_harness_is_refused() {
    let tree = Tree::new();
    tree.page("nobody", "nobody", "nobody");
    fs::create_dir_all(tree.path().join("examples")).expect("the examples directory");

    let Error::TargetWithoutHarness { target: named, .. } = refusal(&tree, &[target("nobody")])
    else {
        panic!("expected the missing harness to be refused");
    };
    assert_eq!(named, "nobody");
}

#[test]
fn a_harness_for_a_target_no_page_documents_is_refused() {
    let tree = Tree::new();
    tree.page("nobody", "nobody", "nobody")
        .harness("nobody")
        .harness("elsewhere");

    let Error::HarnessWithoutTarget { directory, .. } =
        refusal(&tree, &[target("nobody"), target("elsewhere")])
    else {
        panic!("expected the orphaned harness to be refused");
    };
    assert_eq!(directory, "elsewhere");
}

// --- Ceilings are refusals, never truncations ---------------------------------------------------

#[test]
fn an_example_above_the_ceiling_is_refused_rather_than_cut() {
    let tree = Tree::new();
    let enormous = "x".repeat(hook0_documentation_examples::limits::MAX_EXAMPLE_BYTES + 1);
    tree.write(
        "nobody.md",
        &format!("---\nsdkTarget: nobody\n---\n\n```nobody example=only\n{enormous}\n```\n"),
    )
    .harness("nobody");

    assert!(matches!(
        refusal(&tree, &[target("nobody")]),
        Error::ExampleTooLarge { .. }
    ));
}

#[test]
fn a_fence_that_is_never_closed_is_refused() {
    let tree = Tree::new();
    tree.write(
        "nobody.md",
        "---\nsdkTarget: nobody\n---\n\n```nobody example=only\nthe snippet\n",
    )
    .harness("nobody");

    assert!(matches!(
        refusal(&tree, &[target("nobody")]),
        Error::UnclosedFence { .. }
    ));
}

// --- What a harness and a manifest mean ---------------------------------------------------------

#[test]
fn a_region_puts_the_snippet_where_its_hole_was_at_the_indentation_it_had() {
    let regions = harness::parse(
        "// HARNESS body\nfn wrapper() {\n    EXAMPLE\n}\n// END HARNESS\n",
        "harness.rs",
    )
    .expect("the region the text declares");

    let Some(Region { .. }) = regions.get("body") else {
        panic!("no region named body in {regions:?}");
    };
    assert_eq!(
        regions["body"].fill("let a = 1;\n\nlet b = 2;"),
        "fn wrapper() {\n    let a = 1;\n\n    let b = 2;\n}\n",
    );
}

#[test]
fn a_region_with_no_hole_is_refused() {
    assert!(matches!(
        harness::parse(
            "// HARNESS body\nnothing to fill\n// END HARNESS\n",
            "harness.rs"
        ),
        Err(Error::RegionWithoutHole { .. })
    ));
}

#[test]
fn a_manifest_naming_a_level_that_does_not_exist_is_refused() {
    assert!(matches!(
        manifest::parse(
            "proof = \"looked-at\"\nproves = \"x\"\npath = \"{{name}}\"\n\
             timeout_seconds = 1\nrun = [[\"true\"]]\n",
            "examples.toml",
        ),
        Err(Error::Manifest { .. })
    ));
}

#[test]
fn a_manifest_whose_examples_would_overwrite_each_other_is_refused() {
    assert!(matches!(
        manifest::parse(
            "proof = \"parsed\"\nproves = \"x\"\npath = \"one.txt\"\n\
             timeout_seconds = 1\nrun = [[\"true\"]]\n",
            "examples.toml",
        ),
        Err(Error::Manifest { .. })
    ));
}

#[test]
fn a_manifest_proving_nothing_is_refused() {
    assert!(matches!(
        manifest::parse(
            "proof = \"parsed\"\nproves = \"x\"\npath = \"{{name}}\"\ntimeout_seconds = 1\n",
            "examples.toml",
        ),
        Err(Error::Manifest { .. })
    ));
}

#[test]
fn a_level_is_said_as_a_verb_an_example_can_fail_to_do() {
    assert_eq!(Proof::Compiled.verb(), "compile");
    assert_eq!(Proof::TypeChecked.verb(), "type-check");
    assert_eq!(Proof::Parsed.verb(), "parse");
}

// --- A command that never finishes is killed rather than waited on ------------------------------

#[test]
fn a_command_past_the_budget_its_language_declared_is_killed_and_reported() {
    let tree = Tree::new();
    tree.page("nobody", "nobody", "nobody")
        .write(
            "examples/nobody/examples.toml",
            "proof = \"parsed\"\nproves = \"read\"\npath = \"{{name}}.txt\"\n\
             timeout_seconds = 1\nrun = [[\"sleep\", \"30\"]]\n",
        )
        .write(
            "examples/nobody/harness.txt",
            "# HARNESS only\nEXAMPLE\n# END HARNESS\n",
        );

    let documentation =
        discover(&[target("nobody")], tree.path()).expect("what the tree describes");
    let assembled = Tree::new();

    let started = std::time::Instant::now();
    let outcome = hook0_documentation_examples::prove(
        &documentation.languages[0],
        assembled.path(),
        tree.path(),
        tree.path(),
    );

    let Err(Error::Timeout { seconds, .. }) = outcome else {
        panic!("expected the command to be killed, got {outcome:?}");
    };
    assert_eq!(seconds, 1);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "the command was waited on rather than killed",
    );
}
