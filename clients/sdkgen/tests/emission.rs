//! What writing a target out does to the directory it writes into.
//!
//! Every check here runs against a real directory rather than a stand-in: the behaviour worth
//! pinning down is what the filesystem ends up holding, which nothing but a filesystem can report.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use hook0_sdkgen::emit::{
    CommentStyle, EmittedFile, FileTree, Ownership, RelativePath, banner, write_target,
};
use hook0_sdkgen::{Error, Limits};
use tempfile::TempDir;

/// Deepest and widest a target is ever read back in these checks. A test that walked away with the
/// filesystem would hang the suite rather than fail it.
const MAX_READ_DEPTH: usize = 8;
const MAX_READ_ENTRIES: usize = 1024;

/// The command a banner names, chosen without a digit in it so that finding one in the rendered
/// banner means the banner put it there.
const COMMAND: &str = "just generate";

fn limits() -> Limits {
    Limits::DEFAULT
}

fn emitted(path: &str, contents: &str) -> EmittedFile {
    EmittedFile {
        path: RelativePath::build(path, &limits()).expect("the path stays under the target root"),
        contents: contents.to_owned(),
    }
}

fn tree(files: &[(&str, &str)]) -> FileTree {
    let files = files
        .iter()
        .map(|(path, contents)| emitted(path, contents))
        .collect();

    FileTree::build(files, &limits()).expect("the files sit at distinct paths under the root")
}

/// Everything the target holds, read back off the disk rather than out of the tree that wrote it.
fn on_disk(root: &Path) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    if !root.is_dir() {
        return found;
    }

    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut visited = 0usize;

    while let Some((directory, depth)) = stack.pop() {
        assert!(depth <= MAX_READ_DEPTH, "{} nests too deep", root.display());

        for entry in fs::read_dir(&directory).expect("the target is readable") {
            let entry = entry.expect("the target entry is readable");

            visited += 1;
            assert!(
                visited <= MAX_READ_ENTRIES,
                "{} is too wide",
                root.display()
            );

            let path = entry.path();
            if entry.file_type().expect("the entry has a type").is_dir() {
                stack.push((path, depth + 1));
                continue;
            }

            let relative = path
                .strip_prefix(root)
                .expect("the entry sits under the target root")
                .to_str()
                .expect("the entry name is UTF-8")
                .replace(std::path::MAIN_SEPARATOR, "/");
            found.insert(
                relative,
                fs::read_to_string(&path).expect("the file is readable"),
            );
        }
    }

    found
}

/// The tree as the disk would hold it, so the two can be compared directly.
fn expected(tree: &FileTree) -> BTreeMap<String, String> {
    tree.files()
        .iter()
        .map(|file| (file.path.to_string(), file.contents.clone()))
        .collect()
}

fn names(paths: &[RelativePath]) -> Vec<&str> {
    paths.iter().map(RelativePath::as_str).collect()
}

#[test]
fn a_first_pass_lays_the_whole_tree_down() {
    let target = TempDir::new().expect("a temporary directory is available");
    let root = target.path().join("client");
    let tree = tree(&[
        ("client.rs", "the client"),
        ("model/event.rs", "an event"),
        ("model/application.rs", "an application"),
    ]);

    let report = write_target(&root, Ownership::Directory, &tree, &limits())
        .expect("a first pass writes the tree out");

    assert_eq!(on_disk(&root), expected(&tree));
    assert_eq!(
        names(&report.written),
        ["client.rs", "model/application.rs", "model/event.rs"]
    );
    assert_eq!(report.removed, []);
    assert_eq!(report.unchanged, 0);
}

#[test]
fn writing_a_tree_that_is_already_there_reports_nothing_and_touches_nothing() {
    let target = TempDir::new().expect("a temporary directory is available");
    let root = target.path().join("client");
    let tree = tree(&[("client.rs", "the client"), ("model/event.rs", "an event")]);

    write_target(&root, Ownership::Directory, &tree, &limits()).expect("a first pass writes");
    let written_at = fs::metadata(root.join("client.rs"))
        .expect("the file was written")
        .modified()
        .expect("the platform reports modification times");

    let again = write_target(&root, Ownership::Directory, &tree, &limits())
        .expect("a second pass writes the same tree");

    assert_eq!(again.written, []);
    assert_eq!(again.removed, []);
    assert_eq!(again.unchanged, tree.files().len());
    assert_eq!(on_disk(&root), expected(&tree));
    assert_eq!(
        fs::metadata(root.join("client.rs"))
            .expect("the file is still there")
            .modified()
            .expect("the platform reports modification times"),
        written_at,
        "a file already holding the emitted bytes was rewritten, which forces a rebuild downstream"
    );
}

#[test]
fn a_renamed_entity_leaves_no_orphan_behind_under_directory_ownership() {
    let target = TempDir::new().expect("a temporary directory is available");
    let root = target.path().join("client");
    let before = tree(&[
        ("client.rs", "the client"),
        ("model/event.rs", "an event"),
        ("model/nested/deep.rs", "nested"),
    ]);
    let after = tree(&[
        ("client.rs", "the client"),
        ("model/event_type.rs", "an event type"),
    ]);

    write_target(&root, Ownership::Directory, &before, &limits()).expect("a first pass writes");
    let report = write_target(&root, Ownership::Directory, &after, &limits())
        .expect("a second pass writes the new tree");

    assert_eq!(on_disk(&root), expected(&after));
    assert_eq!(
        names(&report.removed),
        ["model/event.rs", "model/nested/deep.rs"]
    );
    assert!(
        !root.join("model/nested").exists(),
        "a directory the emission emptied was left behind"
    );
    assert!(
        root.join("model").is_dir(),
        "a directory the emission still fills was removed"
    );
}

#[test]
fn hand_written_code_beside_the_emission_survives_under_file_ownership() {
    let target = TempDir::new().expect("a temporary directory is available");
    let root = target.path().join("client");
    fs::create_dir_all(root.join("by_hand")).expect("the directory is created");
    fs::write(root.join("by_hand/kept.rs"), "written by hand").expect("the file is written");

    let tree = tree(&[("generated.rs", "generated")]);
    let report = write_target(&root, Ownership::Files, &tree, &limits())
        .expect("a pass over shared ground writes");

    assert_eq!(report.removed, []);
    assert_eq!(
        fs::read_to_string(root.join("by_hand/kept.rs")).expect("the hand-written file is there"),
        "written by hand"
    );
    assert_eq!(
        on_disk(&root),
        BTreeMap::from([
            ("by_hand/kept.rs".to_owned(), "written by hand".to_owned()),
            ("generated.rs".to_owned(), "generated".to_owned()),
        ])
    );
}

#[test]
fn a_path_that_could_reach_out_of_the_root_is_refused() {
    let limits = limits();
    let refused = [
        "/etc/passwd",
        "../outside.rs",
        "model/../../outside.rs",
        "model//event.rs",
        "model/",
        "",
        "model\\event.rs",
        "C:/windows/system32",
        "model/\u{0}.rs",
    ];

    for path in refused {
        let error = RelativePath::build(path, &limits)
            .expect_err(&format!("`{path}` reaches out of the target root"));
        let refused_as_unsafe = matches!(error, Error::UnsafePath { .. });
        assert!(
            refused_as_unsafe,
            "`{path}` gave an unexpected error: {error}"
        );
    }
}

#[test]
fn a_path_a_target_may_emit_at_is_accepted() {
    let limits = limits();
    let accepted = ["client.rs", "model/event.rs", "a/b/c/d.rs", "événement.rs"];

    for path in accepted {
        let built = RelativePath::build(path, &limits)
            .unwrap_or_else(|error| panic!("`{path}` is a path a target may emit: {error}"));
        assert_eq!(built.as_str(), path);
    }
}

#[test]
fn two_files_claiming_one_path_stop_the_emission() {
    let files = vec![
        emitted("model/event.rs", "one reading"),
        emitted("model/event.rs", "another reading"),
    ];

    let error = FileTree::build(files, &limits()).expect_err("one path cannot hold two files");
    let refused_as_duplicate = matches!(error, Error::DuplicateEmittedPath { .. });
    assert!(refused_as_duplicate, "unexpected error: {error}");
}

/// A name cannot be a file for one emitter and the directory another one writes into. Left to the
/// filesystem, that stops the write partway through, on a tree no emission describes.
#[test]
fn a_file_and_the_directory_another_file_needs_cannot_share_a_name() {
    let files = vec![
        emitted("model", "a file"),
        emitted("model/event.rs", "a file under it"),
    ];

    assert_eq!(
        FileTree::build(files, &limits()),
        Err(Error::EmittedPathCollision {
            path: "model".to_owned(),
            nested: "model/event.rs".to_owned(),
        })
    );
}

#[test]
fn a_tree_above_a_ceiling_is_rejected_naming_the_ceiling_it_crossed() {
    let limits = Limits {
        max_emitted_files: 1,
        ..Limits::DEFAULT
    };
    let files = vec![emitted("a.rs", "a"), emitted("b.rs", "b")];

    assert_eq!(
        FileTree::build(files, &limits),
        Err(Error::TooManyEmittedFiles { count: 2, limit: 1 })
    );

    let deep = Limits {
        max_path_depth: 2,
        ..Limits::DEFAULT
    };
    assert_eq!(
        RelativePath::build("a/b/c.rs", &deep),
        Err(Error::PathTooDeep {
            path: "a/b/c.rs".to_owned(),
            depth: 3,
            limit: 2,
        })
    );

    let long = Limits {
        max_path_bytes: 4,
        ..Limits::DEFAULT
    };
    assert_eq!(
        RelativePath::build("client.rs", &long),
        Err(Error::PathTooLong {
            path: "client.rs".to_owned(),
            size: 9,
            limit: 4,
        })
    );
}

#[test]
fn a_test_file_that_drifted_into_the_target_stops_the_write_rather_than_being_deleted() {
    let target = TempDir::new().expect("a temporary directory is available");
    let root = target.path().join("client");
    fs::create_dir_all(&root).expect("the target root is created");
    fs::write(root.join("client.test.ts"), "coverage").expect("the test file is written");

    let before = on_disk(&root);
    let error = write_target(
        &root,
        Ownership::Directory,
        &tree(&[("client.ts", "generated")]),
        &limits(),
    )
    .expect_err("a test file in a generated tree stops the write");

    let refused_as_test = matches!(error, Error::TestFileUnderTarget { .. });
    assert!(refused_as_test, "unexpected error: {error}");
    assert_eq!(
        on_disk(&root),
        before,
        "the write was refused but the target was changed anyway"
    );
}

/// The guard above pins one naming convention; a generated tree is written in every language the
/// targets cover, and a test that drifted into any of them is coverage all the same.
#[test]
fn every_naming_convention_for_a_test_file_stops_the_write() {
    for name in [
        "client.test.ts",
        "client_test.go",
        "test_client.py",
        "ClientTests.cs",
        "client_spec.rb",
        "__tests__/client.js",
        "specs/client.js",
    ] {
        let target = TempDir::new().expect("a temporary directory is available");
        let root = target.path().join("client");
        let path = root.join(name);
        fs::create_dir_all(path.parent().expect("the file sits somewhere"))
            .expect("the target root is created");
        fs::write(&path, "coverage").expect("the test file is written");

        let outcome = write_target(
            &root,
            Ownership::Files,
            &tree(&[("client.ts", "generated")]),
            &limits(),
        );

        let refused_as_test = matches!(outcome, Err(Error::TestFileUnderTarget { .. }));
        assert!(
            refused_as_test,
            "`{name}` did not stop the write: {outcome:?}"
        );
    }
}

#[test]
fn the_banner_carries_no_date_and_no_version() {
    let rendered = banner(CommentStyle::DoubleSlash, COMMAND, &limits())
        .expect("the command is one a banner may name");

    // Anything that dated or versioned the banner would have to write a digit to do it. The two
    // places a digit legitimately appears — the name of this crate and the command it was handed —
    // are taken back out first, so what is left carrying one can only be a date or a version.
    let stamped = rendered
        .replace(env!("CARGO_PKG_NAME"), "")
        .replace(COMMAND, "");
    assert!(
        !stamped.chars().any(|character| character.is_ascii_digit()),
        "the banner carries a digit, so it carries a date or a version and every emitted file \
         would be rewritten on each release: {rendered}"
    );
    assert!(
        !rendered.contains(env!("CARGO_PKG_VERSION")),
        "the banner carries the version of the generator: {rendered}"
    );
    assert!(
        rendered.contains(COMMAND),
        "the banner names no way to rewrite the file"
    );
    assert_eq!(
        banner(CommentStyle::DoubleSlash, COMMAND, &limits()),
        Ok(rendered),
        "the banner is not the same twice, so no emission compares byte for byte"
    );
}

#[test]
fn every_line_of_the_banner_stays_inside_a_comment() {
    for style in [
        CommentStyle::DoubleSlash,
        CommentStyle::Hash,
        CommentStyle::DoubleDash,
    ] {
        let rendered =
            banner(style, COMMAND, &limits()).expect("the command is one a banner may name");

        for line in rendered.lines() {
            assert!(
                line.starts_with(style.marker()),
                "`{line}` is not a comment, so it would land in the source as code"
            );
        }
    }
}

/// A command reaching the banner has to survive being read back out of a comment and pasted into a
/// shell. One carrying a line break would leave everything after the first line outside the
/// comment, and inside the emitted source.
#[test]
fn a_command_a_banner_could_not_carry_is_refused() {
    for command in ["", "just generate\nrm -rf /", "just\tgenerate"] {
        let outcome = banner(CommentStyle::DoubleSlash, command, &limits());

        let refused = matches!(outcome, Err(Error::UnusableCommand { .. }));
        assert!(refused, "`{command}` reached a banner: {outcome:?}");
    }
}

#[cfg(unix)]
#[test]
fn an_emitted_path_standing_on_a_symbolic_link_is_not_written_through() {
    use std::os::unix::fs::symlink;

    let outside = TempDir::new().expect("a temporary directory is available");
    let elsewhere = outside.path().join("hand_written.rs");
    fs::write(&elsewhere, "written by hand, outside the target").expect("the file is written");

    let target = TempDir::new().expect("a temporary directory is available");
    let root = target.path().join("client");
    fs::create_dir_all(&root).expect("the target root is created");
    symlink(&elsewhere, root.join("client.rs")).expect("the link is created");

    let outcome = write_target(
        &root,
        Ownership::Files,
        &tree(&[("client.rs", "generated")]),
        &limits(),
    );

    assert_eq!(
        fs::read_to_string(&elsewhere).expect("the file outside the target is still there"),
        "written by hand, outside the target",
        "the emission followed a symbolic link and wrote outside the target root: {outcome:?}"
    );
}

/// A link on a directory in the middle of the path is the same escape one component earlier, and it
/// is the one that survives a guard placed on the file itself: `symlink_metadata` declines to follow
/// only the last component, so every component before it is still traversed by the write.
///
/// Both ownerships are driven here on purpose. Under [`Ownership::Directory`] a linked directory the
/// tree does not carry is swept up by the stale-file pass, which runs before the write and closes
/// the escape as a side effect of that ordering. Under [`Ownership::Files`] nothing is ever removed,
/// so the link stands and the write goes straight through it. A check written for the first
/// ownership alone passes while the second stays open.
#[cfg(unix)]
#[test]
fn an_emitted_path_reached_through_a_linked_directory_is_not_written_through() {
    use std::os::unix::fs::symlink;

    for ownership in [Ownership::Files, Ownership::Directory] {
        let outside = TempDir::new().expect("a temporary directory is available");
        let elsewhere = outside.path().join("by_hand");
        fs::create_dir_all(&elsewhere).expect("the directory outside the target is created");

        let target = TempDir::new().expect("a temporary directory is available");
        let root = target.path().join("client");
        fs::create_dir_all(&root).expect("the target root is created");
        symlink(&elsewhere, root.join("model")).expect("the link is created");

        let outcome = write_target(
            &root,
            ownership,
            &tree(&[("model/event.rs", "generated")]),
            &limits(),
        );

        assert!(
            !elsewhere.join("event.rs").exists(),
            "under {ownership:?} the emission wrote through a linked directory and landed at {}: \
             {outcome:?}",
            elsewhere.join("event.rs").display()
        );
    }
}
