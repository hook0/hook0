//! What the commits since a release demand of the next one, asked of real histories.
//!
//! Every history here is committed by the test that reads it, into a repository of its own. The
//! question is what `git log` answers about a tree and a tag, and answering it against a fabricated
//! log would only prove the fabrication — so the suite runs git, tags what it commits, and reads
//! back what the tool makes of it.

use std::cell::Cell;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use release_packages::{Bump, Error, Required, Since, check_bump, required_bump};

/// A git repository with nothing in it but what a test commits.
struct History {
    at: tempfile::TempDir,
    /// How many commits have been made, so that each one changes the file it touches — a commit
    /// leaving a path byte-identical is not a commit `git log -- <path>` lists.
    written: Cell<usize>,
}

impl History {
    fn new() -> History {
        let history = History {
            at: tempfile::tempdir().expect("a temporary directory"),
            written: Cell::new(0),
        };
        history.git(&["init", "--initial-branch=main"]);
        history.git(&["config", "user.email", "nobody@example.invalid"]);
        history.git(&["config", "user.name", "Nobody"]);
        history.git(&["config", "commit.gpgsign", "false"]);
        history
    }

    fn path(&self) -> &Path {
        self.at.path()
    }

    /// Run git here, holding it to succeeding.
    ///
    /// Whoever's configuration the suite happens to run under is kept out of it: a template
    /// directory, a hooks path or a signing key set globally would otherwise decide what these
    /// commits look like.
    fn git(&self, arguments: &[&str]) -> String {
        let ran = Command::new("git")
            .args(arguments)
            .current_dir(self.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .expect("git");
        assert!(
            ran.status.success(),
            "git {arguments:?}: {}",
            String::from_utf8_lossy(&ran.stderr)
        );
        String::from_utf8_lossy(&ran.stdout).into_owned()
    }

    /// One commit changing one file under `directory`, carrying `message` whole: the first line is
    /// its subject and whatever follows the blank line is its body, which is where a footer lives.
    fn commit(&self, directory: &str, message: &str) -> &History {
        let nth = self.written.get() + 1;
        self.written.set(nth);

        let file = self.path().join(directory).join("changed.txt");
        std::fs::create_dir_all(file.parent().expect("the directory")).expect("the directory");
        std::fs::write(&file, format!("{nth}\n")).expect("the file");

        self.git(&["add", "--all", "."]);
        self.git(&["commit", "--message", message]);
        self
    }

    fn tag(&self, name: &str) -> &History {
        self.git(&["tag", name]);
        self
    }

    /// A history of `count` further commits under `directory`, written in one pass rather than one
    /// process apiece — the ceiling is thousands of commits, and a suite that spawned that many git
    /// processes to prove one refusal would be a suite nobody runs.
    ///
    /// This writes commits, not a working tree: the index and the checkout are left where they were,
    /// so nothing may be committed the ordinary way after it.
    fn crowd(&self, directory: &str, count: usize) -> &History {
        let mut stream = String::new();
        for nth in 0..count {
            let message = format!("chore: one of many, number {nth}\n");
            let body = format!("{nth}\n");
            stream.push_str("commit refs/heads/main\n");
            stream.push_str("committer Nobody <nobody@example.invalid> 0 +0000\n");
            stream.push_str(&format!("data {}\n{message}", message.len()));
            if nth == 0 {
                stream.push_str("from refs/heads/main^0\n");
            }
            stream.push_str(&format!(
                "M 644 inline {directory}/crowded.txt\ndata {}\n{body}\n",
                body.len()
            ));
        }

        let mut importing = Command::new("git")
            .args(["fast-import", "--quiet"])
            .current_dir(self.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .stdin(Stdio::piped())
            .spawn()
            .expect("git fast-import");
        importing
            .stdin
            .take()
            .expect("a pipe to write the history down")
            .write_all(stream.as_bytes())
            .expect("the history");
        assert!(
            importing.wait().expect("git fast-import").success(),
            "the history was not imported"
        );
        self
    }
}

fn demanded(history: &History, glob: &str, paths: &[&str]) -> Required {
    required_bump(history.path(), glob, paths).expect("what the history demands")
}

/// One package, released once, so that what follows the tag is what is under test.
fn released() -> History {
    let history = History::new();
    history
        .commit("clients/sole", "fix: the shape this package was born with")
        .tag("sole/v1.0.0");
    history
}

// --- What a commit says it did ------------------------------------------------------------------

#[test]
fn a_bang_before_the_colon_makes_the_release_major() {
    let history = released();
    history.commit(
        "clients/sole",
        "fix(wire)!: rename the field every payload carries",
    );

    let required = demanded(&history, "sole/v*", &["clients/sole"]);

    assert_eq!(required.bump, Bump::Major);
    assert_eq!(required.since, Since::Tag("sole/v1.0.0".to_string()));
    assert_eq!(required.reasons.len(), 1, "{:?}", required.reasons);
    assert_eq!(
        required.reasons[0].subject,
        "fix(wire)!: rename the field every payload carries"
    );
}

/// The mark is read whether or not the type carries a scope, since both are the same declaration.
#[test]
fn a_bang_on_a_type_without_a_scope_makes_the_release_major() {
    let history = released();
    history.commit(
        "clients/sole",
        "feat!: answer on a protocol nothing else speaks",
    );

    assert_eq!(
        demanded(&history, "sole/v*", &["clients/sole"]).bump,
        Bump::Major
    );
}

/// The footer is the mechanism for a commit whose type says nothing about what it broke, which is
/// the case a `chore` that changes a protocol is.
#[test]
fn a_breaking_change_footer_makes_the_release_major_in_either_spelling() {
    for token in ["BREAKING CHANGE", "BREAKING-CHANGE"] {
        let history = released();
        history.commit(
            "clients/sole",
            &format!(
                "chore(mcp): update to rmcp 3\n\n{token}: the server speaks a different set of \
                 protocol revisions.\n"
            ),
        );

        let required = demanded(&history, "sole/v*", &["clients/sole"]);

        assert_eq!(required.bump, Bump::Major, "{token}");
        assert_eq!(required.reasons.len(), 1, "{token}: {:?}", required.reasons);
    }
}

/// The phrase is a declaration where a footer goes and a sentence anywhere else. Reading it
/// anywhere would refuse a release over a commit saying the opposite of what it was read as saying.
#[test]
fn the_phrase_written_as_prose_is_not_a_footer() {
    let history = released();
    history.commit(
        "clients/sole",
        "fix: widen a timeout\n\nThis is not a BREAKING CHANGE: nothing on the wire moved, and\n  \
         BREAKING CHANGE: an indented line is prose too.\n",
    );

    let required = demanded(&history, "sole/v*", &["clients/sole"]);

    assert_eq!(required.bump, Bump::Patch);
    assert!(required.reasons.is_empty(), "{:?}", required.reasons);
}

#[test]
fn a_feature_makes_the_release_minor_whatever_case_it_is_written_in() {
    for subject in ["feat: add a field", "FEAT(client): add a field"] {
        let history = released();
        history.commit("clients/sole", subject);

        assert_eq!(
            demanded(&history, "sole/v*", &["clients/sole"]).bump,
            Bump::Minor,
            "{subject}"
        );
    }
}

#[test]
fn fixes_and_chores_ask_for_nothing_more_than_a_patch() {
    let history = released();
    history
        .commit("clients/sole", "fix: stop reading past the end")
        .commit("clients/sole", "chore: update dependencies")
        .commit("clients/sole", "docs(client): say what a retry does");

    let required = demanded(&history, "sole/v*", &["clients/sole"]);

    assert_eq!(required.bump, Bump::Patch);
    assert_eq!(required.read, 3);
    assert!(required.reasons.is_empty(), "{:?}", required.reasons);
}

/// A merge commit and a sentence that happens to hold a colon are not conventional commits, and
/// reading a type off either would be reading one out of prose.
#[test]
fn a_subject_that_is_not_a_conventional_commit_demands_nothing() {
    let history = released();
    history
        .commit("clients/sole", "Merge branch 'feat/something!' into 'main'")
        .commit("clients/sole", "Revert the thing: it was wrong");

    let required = demanded(&history, "sole/v*", &["clients/sole"]);

    assert_eq!(required.bump, Bump::Patch, "{:?}", required.reasons);
}

// --- What the read covers -----------------------------------------------------------------------

#[test]
fn a_commit_touching_another_package_says_nothing_about_this_one() {
    let history = released();
    history.commit(
        "clients/elsewhere",
        "feat!: break something in a package this tag does not cover",
    );

    let required = demanded(&history, "sole/v*", &["clients/sole"]);

    assert_eq!(required.bump, Bump::Patch);
    assert_eq!(required.read, 0);
}

/// A release covering several packages at once weighs the commits of all of them, which is what
/// makes one `sdk-v` tag answerable to every client it carries.
#[test]
fn a_release_covering_several_packages_is_read_over_all_of_them() {
    let history = History::new();
    history
        .commit("clients/one", "fix: the shape these were born with")
        .tag("sdk-v1.0.0");
    history.commit("clients/two", "feat: add a field to the other client");

    let required = demanded(&history, "sdk-v*", &["clients/one", "clients/two"]);

    assert_eq!(required.bump, Bump::Minor);
}

/// Only this package's own tags bound the read. A tag of another shape sitting in between is
/// another package's release and says nothing about where this one last went out.
#[test]
fn the_read_starts_at_this_packages_own_last_tag() {
    let history = History::new();
    history
        .commit(
            "clients/sole",
            "feat!: the shape this package was born with",
        )
        .tag("sole/v1.0.0");
    history
        .commit("clients/sole", "feat: add a field")
        .tag("other-v9.9.9");
    history.commit("clients/sole", "fix: stop reading past the end");

    let required = demanded(&history, "sole/v*", &["clients/sole"]);

    assert_eq!(required.read, 2, "a commit already released was read again");
    assert_eq!(required.bump, Bump::Minor);
}

#[test]
fn a_package_that_has_never_been_released_is_read_from_its_first_commit() {
    let history = History::new();
    history.commit("clients/sole", "feat: everything this package is");

    let required = demanded(&history, "sole/v*", &["clients/sole"]);

    assert_eq!(
        required.since,
        Since::FirstCommit {
            pattern: "sole/v*".to_string()
        }
    );
    assert_eq!(required.bump, Bump::Minor);
}

/// The other way a last release can be missing: tags of this shape exist, and HEAD is on a history
/// that reaches none of them. Reading from the beginning there would weigh commits that already
/// went out, so it is a refusal naming what was looked for.
#[test]
fn a_last_release_head_cannot_see_is_refused_rather_than_read_from_the_beginning() {
    let history = History::new();
    history.commit("clients/sole", "feat: everything this package is");
    history.git(&["checkout", "--quiet", "-b", "aside"]);
    history
        .commit("clients/sole", "fix: something on a history left behind")
        .tag("sole/v1.0.0");
    history.git(&["checkout", "--quiet", "main"]);
    history.commit("clients/sole", "fix: something on the history that went on");

    let error = required_bump(history.path(), "sole/v*", &["clients/sole"])
        .expect_err("a last release HEAD cannot see");
    let said = error.to_string();

    assert!(said.contains("sole/v*"), "{said}");
    assert!(matches!(error, Error::NoReachableTag { .. }), "{error:?}");
}

// --- What is refused, and what is a decision ----------------------------------------------------

#[test]
fn a_bump_smaller_than_the_commits_demand_is_refused_naming_them() {
    let history = released();
    history
        .commit(
            "clients/sole",
            "chore(mcp): update to rmcp 3\n\nBREAKING CHANGE: the server speaks a different set of \
             protocol revisions.\n",
        )
        .commit("clients/sole", "feat: add a field");

    let required = demanded(&history, "sole/v*", &["clients/sole"]);
    let error =
        check_bump(&required, Bump::Minor).expect_err("a release smaller than its own commits");
    let said = error.to_string();

    assert!(said.contains("chore(mcp): update to rmcp 3"), "{said}");
    assert!(
        !said.contains("add a field"),
        "a commit demanding no more than was asked for is named: {said}"
    );
    assert!(said.contains("sole/v1.0.0"), "{said}");
    assert!(said.contains("major"), "{said}");
    assert!(matches!(error, Error::BumpTooSmall { .. }), "{error:?}");
}

/// The list is bounded, and a list in history order can lose the very commit that decided the
/// answer — leaving a refusal saying `major` with nothing breaking left in it. So the ones demanding
/// most come first, however long ago they landed.
#[test]
fn what_is_named_leads_with_the_commits_that_decided_the_answer() {
    let history = released();
    history
        .commit(
            "clients/sole",
            "chore(mcp): update to rmcp 3\n\nBREAKING CHANGE: the server speaks a different set of \
             protocol revisions.\n",
        )
        .commit("clients/sole", "feat: add a field")
        .commit("clients/sole", "feat: add another field");

    let required = demanded(&history, "sole/v*", &["clients/sole"]);
    let named = release_packages::naming(&required.reasons, Bump::Patch);
    let first = named.lines().next().expect("a commit named");

    assert!(first.contains("chore(mcp): update to rmcp 3"), "{named}");
    assert_eq!(named.lines().count(), 3, "{named}");
}

/// Deciding a release is bigger than its commits require is a statement about what it means to
/// whoever installs it, and nothing in a log can contradict it.
#[test]
fn a_bump_larger_than_the_commits_demand_is_a_decision_rather_than_a_mistake() {
    let history = released();
    history.commit("clients/sole", "fix: stop reading past the end");

    let required = demanded(&history, "sole/v*", &["clients/sole"]);

    check_bump(&required, Bump::Patch).expect("the patch the commits demand");
    check_bump(&required, Bump::Minor).expect("a minor nobody's commits demanded");
    check_bump(&required, Bump::Major).expect("a major nobody's commits demanded");
}

// --- Bounds -------------------------------------------------------------------------------------

/// Reading part of a history would miss a breaking commit sitting just past the last one read,
/// which is exactly the commit this exists to find.
#[test]
fn more_commits_than_the_ceiling_are_refused_rather_than_read_in_part() {
    let history = released();
    history.crowd("clients/sole", release_packages::MAX_COMMITS + 1);

    let error = required_bump(history.path(), "sole/v*", &["clients/sole"])
        .expect_err("a history past the ceiling");

    assert!(matches!(error, Error::TooManyCommits { .. }), "{error:?}");
}

#[test]
fn more_paths_than_the_ceiling_are_refused_before_git_is_asked_anything() {
    let history = History::new();
    let paths: Vec<String> = (0..=release_packages::MAX_PATHS)
        .map(|nth| format!("clients/c{nth}"))
        .collect();
    let over: Vec<&str> = paths.iter().map(String::as_str).collect();

    let error =
        required_bump(history.path(), "sole/v*", &over).expect_err("a read past the ceiling");

    assert!(matches!(error, Error::TooManyPaths { .. }), "{error:?}");
}
