//! The one driver every target is written and checked through.
//!
//! Generated trees are committed rather than built: the crates that carry them are published
//! without the OpenAPI snapshot beside them, so what a target emits has to travel as source.
//! Nothing keeps the committed bytes and the snapshot in step on its own, which is what the drift
//! guard below is for.
//!
//! ```text
//! cargo test -p hook0-sdkgen sdk_targets                 # check every target
//! UPDATE_SDK=1 cargo test -p hook0-sdkgen sdk_targets    # rewrite every target
//! UPDATE_SDK=mcp cargo test -p hook0-sdkgen sdk_targets  # rewrite one, still check the rest
//! ```

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use hook0_sdkgen::targets::{EVERY_TARGET, Target, UPDATE_VARIABLE, targets, update_command};
use hook0_sdkgen::{ApiModel, FileTree, Limits, RelativePath, Snapshot, write_target};
use proptest::prelude::*;

mod common;

/// Where the targets land, from the crate this suite runs out of.
const REPOSITORY_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// Largest committed file read back, in bytes.
const MAX_COMMITTED_BYTES: u64 = 4 * 1024 * 1024;

/// How many differing files a report describes before it stops counting them.
const MAX_REPORTED_FILES: usize = 10;

/// How many differing lines of one file a report lists before it stops.
const MAX_REPORTED_LINES: usize = 20;

/// How much of a differing line a report prints.
const MAX_RENDERED_LINE_CHARS: usize = 120;

/// Which targets a run rewrites rather than checks.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Selection {
    /// Nothing is rewritten: every target is held against what is committed.
    Nothing,
    Every,
    /// One target is rewritten; every other one is still held against what is committed, so
    /// adopting a change to one cannot bury a drift in another.
    One(String),
}

impl Selection {
    fn rewrites(&self, target: &Target) -> bool {
        match self {
            Self::Nothing => false,
            Self::Every => true,
            Self::One(name) => name == target.name,
        }
    }
}

/// What a value of the update variable selects, or why it selects nothing.
///
/// The values accepted are the registry itself, so a target added to it is addressable without
/// anything else being told about it.
fn selection(value: Option<&str>) -> Result<Selection, String> {
    let Some(value) = value else {
        return Ok(Selection::Nothing);
    };
    if value == EVERY_TARGET {
        return Ok(Selection::Every);
    }
    if let Some(target) = targets().iter().find(|target| target.name == value) {
        return Ok(Selection::One(target.name.to_owned()));
    }

    Err(format!(
        "`{UPDATE_VARIABLE}={value}` names no target. Accepted: `{EVERY_TARGET}` for every target, \
         or one of {}.",
        named(", ")
    ))
}

/// The names the registry answers to, as a message may list them.
fn named(separator: &str) -> String {
    targets()
        .iter()
        .map(|target| format!("`{}`", target.name))
        .collect::<Vec<String>>()
        .join(separator)
}

/// Every target is emitted from the snapshot and either written out or held against what is
/// committed. A surface that never reached a generated tree stops here rather than at a release.
#[test]
fn sdk_targets_match_the_openapi_snapshot() {
    let selected =
        std::env::var_os(UPDATE_VARIABLE).map(|value| value.to_string_lossy().into_owned());
    let selection = selection(selected.as_deref()).unwrap_or_else(|reason| panic!("{reason}"));

    let limits = Limits::default();
    let mut drifted = Vec::new();

    for target in targets() {
        let tree = emitted(target, &limits);
        let root = root_of(target);

        if selection.rewrites(target) {
            let report =
                write_target(&root, target.ownership, &tree, &limits).unwrap_or_else(|err| {
                    panic!("target `{}` could not be written: {err}", target.name)
                });
            println!(
                "{}: {} written, {} removed, {} already up to date",
                target.name,
                report.written.len(),
                report.removed.len(),
                report.unchanged
            );
            continue;
        }

        if let Some(report) = drift(target, &root, &tree) {
            drifted.push(report);
        }
    }

    // Every target is reported at once: a drift adopted one target at a time would otherwise hide
    // behind the first one that failed.
    assert!(
        drifted.is_empty(),
        "{} of the {} targets are not what the OpenAPI snapshot describes.\n\n{}",
        drifted.len(),
        targets().len(),
        drifted.join("\n\n")
    );
}

/// Every target answers to a name of its own and lands somewhere a write can reach.
#[test]
fn every_target_is_addressable_and_lands_under_the_repository() {
    let limits = Limits::default();
    let mut seen: Vec<&str> = Vec::new();
    let mut roots: Vec<&str> = Vec::new();

    assert!(!targets().is_empty(), "the registry carries no target");

    for target in targets() {
        assert!(!target.name.is_empty(), "a target answers to no name");
        assert_ne!(
            target.name, EVERY_TARGET,
            "target `{}` answers to the value that names every target at once",
            target.name
        );
        assert!(
            !seen.contains(&target.name),
            "`{}` names two targets, so one of them could never be selected",
            target.name
        );
        seen.push(target.name);

        assert!(
            !target.tag.is_empty(),
            "target `{}` selects no tag out of the snapshot",
            target.name
        );

        RelativePath::build(target.root, &limits).unwrap_or_else(|err| {
            panic!(
                "target `{}` lands nowhere a write may reach: {err}",
                target.name
            )
        });
        assert!(
            !roots.contains(&target.root),
            "`{}` is the root of two targets, so one would write over the other",
            target.root
        );
        roots.push(target.root);
    }
}

/// Emitting a target twice writes the same tree, so a regeneration that changed nothing leaves no
/// diff behind.
#[test]
fn emitting_twice_yields_identical_trees() {
    let limits = Limits::default();

    for target in targets() {
        assert_eq!(
            emitted(target, &limits),
            emitted(target, &limits),
            "two emissions of target `{}` differ",
            target.name
        );
    }
}

/// A value naming no target says so, and says what it could have named.
#[test]
fn an_unknown_update_value_names_what_is_accepted() {
    let refused =
        selection(Some("no-such-target")).expect_err("a value naming no target is refused");

    assert!(
        refused.contains("no-such-target"),
        "the refusal does not say what was asked for: {refused}"
    );
    assert!(
        refused.contains(EVERY_TARGET),
        "the refusal does not name the value that selects every target: {refused}"
    );
    for target in targets() {
        assert!(
            refused.contains(target.name),
            "the refusal does not name target `{}`: {refused}",
            target.name
        );
    }
}

/// Every target is selectable by its own name, and by the value that names them all.
#[test]
fn the_registry_is_what_the_update_variable_accepts() {
    assert_eq!(selection(None), Ok(Selection::Nothing));
    assert_eq!(selection(Some(EVERY_TARGET)), Ok(Selection::Every));

    for target in targets() {
        assert_eq!(
            selection(Some(target.name)),
            Ok(Selection::One(target.name.to_owned())),
            "target `{}` cannot be selected by its own name",
            target.name
        );
    }
}

proptest! {
    /// Whatever it is handed, the update variable selects a target of the registry or refuses the
    /// value while naming everything it could have named.
    #[test]
    fn only_the_registry_is_selectable(value in ".{0,32}") {
        match selection(Some(&value)) {
            Ok(Selection::Every) => prop_assert_eq!(value.as_str(), EVERY_TARGET),
            Ok(Selection::One(name)) => prop_assert!(
                targets().iter().any(|target| target.name == name),
                "`{}` selected a target the registry does not carry",
                value
            ),
            Ok(Selection::Nothing) => prop_assert!(false, "`{}` selected nothing at all", value),
            Err(refused) => {
                prop_assert!(refused.contains(EVERY_TARGET));
                for target in targets() {
                    prop_assert!(
                        refused.contains(target.name),
                        "the refusal of `{}` does not name target `{}`",
                        value,
                        target.name
                    );
                }
            }
        }
    }
}

/// What a target emits from the committed snapshot, under the tag it selects.
fn emitted(target: &Target, limits: &Limits) -> FileTree {
    let snapshot = Snapshot::from_bytes(&common::fixture_bytes(), target.tag, limits)
        .unwrap_or_else(|err| {
            panic!(
                "the committed snapshot does not parse under the `{}` tag: {err}",
                target.tag
            )
        });
    let model = ApiModel::from_snapshot(&snapshot, limits).unwrap_or_else(|err| {
        panic!(
            "the committed snapshot yields no model under the `{}` tag: {err}",
            target.tag
        )
    });

    (target.emit)(&target.language, &model)
        .unwrap_or_else(|err| panic!("target `{}` emits nothing: {err}", target.name))
}

fn root_of(target: &Target) -> PathBuf {
    Path::new(REPOSITORY_ROOT).join(target.root)
}

/// How the committed target differs from what the snapshot dictates, or nothing when it does not.
fn drift(target: &Target, root: &Path, tree: &FileTree) -> Option<String> {
    let mut differing = Vec::new();
    let mut further = 0usize;

    for file in tree.files() {
        let committed = committed(&file.path.under(root));
        if committed.as_deref() == Some(file.contents.as_str()) {
            continue;
        }

        if differing.len() == MAX_REPORTED_FILES {
            further += 1;
            continue;
        }

        let rendered = match committed {
            None => "    no file is committed at this path".to_owned(),
            Some(committed) => difference(&committed, &file.contents),
        };
        differing.push(format!("  {}/{}\n{rendered}", target.root, file.path));
    }

    if differing.is_empty() {
        return None;
    }

    let mut report = format!(
        "target `{}` is not what the OpenAPI snapshot describes.\n\
         Adopt the change with:\n    {}\n\
         and commit what it rewrites.\n\
         (`-` committed, `+` what the snapshot dictates)\n{}",
        target.name,
        update_command(target.name),
        differing.join("\n")
    );
    if further > 0 {
        report.push_str(&format!("\n  … and {further} more files"));
    }

    Some(report)
}

/// What is committed at that path, or nothing when no file is.
fn committed(path: &Path) -> Option<String> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return None,
        Err(err) => panic!("{} cannot be looked at: {err}", path.display()),
    };
    assert!(
        metadata.len() <= MAX_COMMITTED_BYTES,
        "{} is {} bytes long, above the {MAX_COMMITTED_BYTES} read back",
        path.display(),
        metadata.len()
    );

    let bytes =
        fs::read(path).unwrap_or_else(|err| panic!("{} cannot be read: {err}", path.display()));
    Some(
        String::from_utf8(bytes)
            .unwrap_or_else(|err| panic!("{} is not UTF-8: {err}", path.display())),
    )
}

/// The lines that differ between what is committed and what the snapshot dictates, bounded so a
/// wholesale regeneration does not bury the report it is meant to explain.
fn difference(committed: &str, emitted: &str) -> String {
    let committed: Vec<&str> = committed.lines().collect();
    let emitted: Vec<&str> = emitted.lines().collect();

    let mut lines = Vec::new();
    let mut reported = 0;

    for index in 0..committed.len().max(emitted.len()) {
        let left = committed.get(index).copied();
        let right = emitted.get(index).copied();
        if left == right {
            continue;
        }

        if reported >= MAX_REPORTED_LINES {
            lines.push("    ...".to_owned());
            break;
        }
        reported += 1;

        let number = index + 1;
        if let Some(left) = left {
            lines.push(format!("    {number}- {}", render(left)));
        }
        if let Some(right) = right {
            lines.push(format!("    {number}+ {}", render(right)));
        }
    }

    lines.join("\n")
}

fn render(line: &str) -> String {
    if line.chars().count() <= MAX_RENDERED_LINE_CHARS {
        return line.to_owned();
    }

    let head: String = line.chars().take(MAX_RENDERED_LINE_CHARS).collect();
    format!("{head}…")
}
