//! Invariants a target emission holds whatever it was asked to write.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path};

use hook0_sdkgen::emit::{EmittedFile, FileTree, Ownership, RelativePath, write_target};
use hook0_sdkgen::{Error, Limits};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use tempfile::TempDir;

/// Seeds of past failures, replayed before anything random is drawn.
const REGRESSIONS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/proptest-regressions/emission_properties.txt"
);

/// Deepest and widest a generated target is ever read back. A walk that ran away would hang the
/// suite rather than fail it.
const MAX_READ_DEPTH: usize = 8;
const MAX_READ_ENTRIES: usize = 1024;

/// Largest number of files a generated tree carries.
const FILE_SLOTS: usize = 8;

/// Deepest a generated file sits, counted in segments.
const DEPTH_SLOTS: usize = 3;

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 128,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(REGRESSIONS))),
        ..ProptestConfig::default()
    }
}

/// Directory and file names are drawn from `a`–`f` behind a `d` or an `f`, which keeps two things
/// true without a filter: no generated name can spell `test`, `tests`, `spec` or `specs`, and no
/// directory can ever carry the name of a file, so a tree is never asked to hold both at one path.
fn generated_path() -> impl Strategy<Value = String> {
    (
        prop::collection::vec("d[a-f]{1,3}", 0..DEPTH_SLOTS),
        "f[a-f]{1,3}",
    )
        .prop_map(|(directories, file)| {
            let mut segments = directories;
            segments.push(file);
            segments.join("/")
        })
}

fn generated_contents() -> impl Strategy<Value = String> {
    prop_oneof![Just(String::new()), "[a-z \n]{0,48}"]
}

/// A run of files, which may name one path more than once — the later reading wins, the way a
/// caller collecting an emission would have to resolve it before building a tree.
fn generated_files() -> impl Strategy<Value = BTreeMap<String, String>> {
    prop::collection::vec((generated_path(), generated_contents()), 0..FILE_SLOTS)
        .prop_map(|files| files.into_iter().collect())
}

fn tree_of(files: &BTreeMap<String, String>, limits: &Limits) -> Result<FileTree, Error> {
    let emitted = files
        .iter()
        .map(|(path, contents)| {
            RelativePath::build(path, limits).map(|path| EmittedFile {
                path,
                contents: contents.clone(),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    FileTree::build(emitted, limits)
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

/// Fragments a path is assembled from, spanning what a target may emit at and what it may not.
fn hostile_fragment() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("..".to_owned()),
        Just(".".to_owned()),
        Just("/".to_owned()),
        Just("\\".to_owned()),
        Just(":".to_owned()),
        Just("C:".to_owned()),
        Just("~".to_owned()),
        Just("\u{0}".to_owned()),
        Just("é".to_owned()),
        Just("日本".to_owned()),
        "[a-z]{1,4}",
    ]
}

fn hostile_path() -> impl Strategy<Value = String> {
    prop::collection::vec(hostile_fragment(), 0..6).prop_map(|fragments| fragments.concat())
}

proptest! {
    #![proptest_config(config())]

    /// Writing a tree lays it down exactly, and writing it again changes nothing at all — which is
    /// the report a regeneration check reads to know the target is already in step.
    #[test]
    fn writing_a_tree_a_second_time_writes_and_removes_nothing(files in generated_files()) {
        let limits = Limits::DEFAULT;
        let target = TempDir::new().map_err(|error| TestCaseError::fail(error.to_string()))?;
        let root = target.path().join("client");
        let tree = tree_of(&files, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        let first = write_target(&root, Ownership::Directory, &tree, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert_eq!(on_disk(&root), files.clone());
        prop_assert_eq!(first.written.len(), files.len());
        prop_assert!(first.removed.is_empty());

        let again = write_target(&root, Ownership::Directory, &tree, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        prop_assert!(again.written.is_empty(), "a second pass rewrote {:?}", again.written);
        prop_assert!(again.removed.is_empty(), "a second pass removed {:?}", again.removed);
        prop_assert_eq!(again.unchanged, files.len());
        prop_assert_eq!(on_disk(&root), files);
    }

    /// A second emission leaves no residue of the first: what the new tree does not carry is gone,
    /// which is what keeps a renamed entity from leaving an orphan file behind forever.
    #[test]
    fn a_second_emission_leaves_no_residue_of_the_first(
        before in generated_files(),
        after in generated_files(),
    ) {
        let limits = Limits::DEFAULT;
        let target = TempDir::new().map_err(|error| TestCaseError::fail(error.to_string()))?;
        let root = target.path().join("client");

        let before_tree = tree_of(&before, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let after_tree = tree_of(&after, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        write_target(&root, Ownership::Directory, &before_tree, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        write_target(&root, Ownership::Directory, &after_tree, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(on_disk(&root), after);
    }

    /// Under file ownership the target is shared with hand-written code, so a file the tree does
    /// not carry is left exactly where it is.
    #[test]
    fn file_ownership_removes_nothing(
        files in generated_files(),
        stranger in generated_path(),
        held in generated_contents(),
    ) {
        prop_assume!(!files.contains_key(&stranger));

        let limits = Limits::DEFAULT;
        let target = TempDir::new().map_err(|error| TestCaseError::fail(error.to_string()))?;
        let root = target.path().join("client");
        let tree = tree_of(&files, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        let beside = root.join(&stranger);
        if let Some(parent) = beside.parent() {
            fs::create_dir_all(parent).map_err(|error| TestCaseError::fail(error.to_string()))?;
        }
        fs::write(&beside, &held).map_err(|error| TestCaseError::fail(error.to_string()))?;

        let report = write_target(&root, Ownership::Files, &tree, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert!(report.removed.is_empty(), "file ownership removed {:?}", report.removed);
        prop_assert_eq!(
            fs::read_to_string(&beside).ok(),
            Some(held),
            "a file the emission does not carry was changed"
        );
    }

    /// Whatever it is handed, an accepted path stays under the root it is joined to, and a path
    /// that is absolute or climbs is never accepted in the first place.
    #[test]
    fn no_accepted_path_reaches_out_of_the_root(text in hostile_path()) {
        let limits = Limits::DEFAULT;
        let root = Path::new("/target/root");

        let climbs = text.starts_with('/')
            || text.split('/').any(|segment| segment == "..");

        match RelativePath::build(&text, &limits) {
            Err(_) => prop_assert!(true),
            Ok(path) => {
                prop_assert!(
                    !climbs,
                    "`{}` is absolute or climbs out of the root, and was accepted as `{}`",
                    text,
                    path
                );

                let joined = path.under(root);
                prop_assert!(
                    joined.starts_with(root),
                    "`{}` joined to the root reaches `{}`",
                    text,
                    joined.display()
                );

                let inside = joined
                    .strip_prefix(root)
                    .map_err(|_| TestCaseError::fail("the joined path left the root"))?;
                prop_assert!(
                    inside.components().all(|c| matches!(c, Component::Normal(_))),
                    "`{}` joined to the root carries a component that is not a plain name",
                    text
                );
                prop_assert!(inside.components().next().is_some(), "`{}` named nothing", text);
            }
        }
    }

    /// A tree above a ceiling is rejected, naming the ceiling it crossed, and one below it never is.
    #[test]
    fn a_tree_past_a_ceiling_is_rejected_naming_it(
        files in generated_files(),
        ceiling in 0usize..FILE_SLOTS,
    ) {
        let limits = Limits { max_emitted_files: ceiling, ..Limits::DEFAULT };
        let result = tree_of(&files, &limits);

        if files.len() > ceiling {
            prop_assert_eq!(
                result,
                Err(Error::TooManyEmittedFiles { count: files.len(), limit: ceiling })
            );
        } else {
            let refused_for_count = matches!(result, Err(Error::TooManyEmittedFiles { .. }));
            prop_assert!(!refused_for_count, "a tree below the ceiling was refused");
        }
    }

    /// The depth ceiling is enforced on every emitted path, and names the depth it reached.
    #[test]
    fn a_path_past_the_depth_ceiling_is_rejected_naming_it(
        path in generated_path(),
        ceiling in 0usize..DEPTH_SLOTS + 1,
    ) {
        let limits = Limits { max_path_depth: ceiling, ..Limits::DEFAULT };
        let depth = path.split('/').count();
        let result = RelativePath::build(&path, &limits);

        if depth > ceiling {
            prop_assert_eq!(
                result,
                Err(Error::PathTooDeep { path: path.clone(), depth, limit: ceiling })
            );
        } else {
            prop_assert!(result.is_ok(), "a path below the ceiling was refused: {:?}", result);
        }
    }

    /// The byte ceiling counts the whole emission, so a tree crossing it is refused whole rather
    /// than written down to the ceiling.
    #[test]
    fn an_emission_past_the_byte_ceiling_is_rejected_naming_it(
        files in generated_files(),
        ceiling in 0usize..64,
    ) {
        let limits = Limits { max_emitted_bytes: ceiling, ..Limits::DEFAULT };
        let weight: usize = files.values().map(String::len).sum();
        let result = tree_of(&files, &limits);

        if weight > ceiling {
            prop_assert_eq!(
                result,
                Err(Error::EmissionTooLarge { size: weight, limit: ceiling })
            );
        } else {
            let refused_for_weight = matches!(result, Err(Error::EmissionTooLarge { .. }));
            prop_assert!(!refused_for_weight, "an emission below the ceiling was refused");
        }
    }

    /// One model always writes one set of bytes: the tree is ordered by path whatever order the
    /// files were collected in, so two emissions compare directly.
    #[test]
    fn a_tree_is_ordered_by_path_whatever_order_it_was_collected_in(
        files in generated_files(),
    ) {
        let limits = Limits::DEFAULT;
        let forwards = tree_of(&files, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        let reversed = files
            .iter()
            .rev()
            .map(|(path, contents)| {
                RelativePath::build(path, &limits).map(|path| EmittedFile {
                    path,
                    contents: contents.clone(),
                })
            })
            .collect::<Result<Vec<_>, Error>>()
            .map_err(|error| TestCaseError::fail(error.to_string()))?;
        let backwards = FileTree::build(reversed, &limits)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(&forwards, &backwards);

        let ordered: Vec<&str> = forwards.files().iter().map(|f| f.path.as_str()).collect();
        let mut sorted = ordered.clone();
        sorted.sort_unstable();
        prop_assert_eq!(&ordered, &sorted);

        let distinct: BTreeSet<&str> = ordered.iter().copied().collect();
        prop_assert_eq!(distinct.len(), ordered.len());
    }
}
