//! Turns what a target decided to emit into files on disk, once, safely.
//!
//! An emitter builds a [`FileTree`] — paths that cannot climb out of the target root, sorted, with
//! no two files claiming the same place — and [`write_target`] is the only thing that touches the
//! filesystem. Two emissions of one model therefore compare byte for byte, and a file whose bytes
//! already match is left alone: an empty [`WriteReport`] is a checkable proof that regenerating
//! changed nothing, and the untouched mtimes keep a no-op regeneration from forcing a rebuild
//! downstream.
//!
//! What a target owns is stated rather than guessed. [`Ownership::Directory`] means the generator
//! owns everything under the root, so a file the tree no longer carries is stale and goes — which is
//! what stops a renamed entity from leaving an orphan behind forever. [`Ownership::Files`] means the
//! generator owns exactly the paths it emits and nothing else, which is what a generated file
//! dropped among hand-written code needs.
//!
//! Whatever the ownership, a file whose name marks it as holding tests stops the write outright: a
//! hand-written test that drifted into a generated tree is coverage that would otherwise be deleted
//! without a word.

mod banner;

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::{Error as IoError, ErrorKind};
use std::path::{Component, Path, PathBuf};

use crate::error::{Error, preview};
use crate::identifier;
use crate::limits::Limits;

pub use banner::{CommentStyle, banner};

/// The separator an emitted path is written with, whatever the platform spells its own paths like.
const SEPARATOR: char = '/';

/// A segment naming the directory above, which is the one way a relative path climbs out of a root.
const PARENT_SEGMENT: &str = "..";

/// A segment naming the directory a file already sits in, which names one place twice.
const CURRENT_SEGMENT: &str = ".";

/// The words that mark a file or directory as holding tests, across naming conventions:
/// `foo.test.ts`, `foo_test.go`, `test_foo.py`, `FooTests.cs`, `foo_spec.rb`, `__tests__/`.
const TEST_WORDS: [&str; 4] = ["test", "tests", "spec", "specs"];

/// Where one emitted file sits, relative to the root of its target.
///
/// The only way to obtain one is [`RelativePath::build`], so a value of this type is a path that has
/// already been shown to stay under whatever root it is joined to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativePath(String);

impl RelativePath {
    pub fn build(text: &str, limits: &Limits) -> Result<Self, Error> {
        if text.len() > limits.max_path_bytes {
            return Err(Error::PathTooLong {
                path: preview(text),
                size: text.len(),
                limit: limits.max_path_bytes,
            });
        }
        if text.is_empty() {
            return Err(unsafe_path(text, "it names nothing"));
        }
        if text.contains('\\') {
            return Err(unsafe_path(
                text,
                "it carries a backslash, which separates directories on some platforms and names a \
                 file on others",
            ));
        }
        if text.starts_with(SEPARATOR) {
            return Err(unsafe_path(
                text,
                "it is absolute, so the target root would have no say over where it lands",
            ));
        }

        let segments: Vec<&str> = text.split(SEPARATOR).collect();
        if segments.len() > limits.max_path_depth {
            return Err(Error::PathTooDeep {
                path: preview(text),
                depth: segments.len(),
                limit: limits.max_path_depth,
            });
        }

        for segment in segments {
            if segment.is_empty() {
                return Err(unsafe_path(text, "it carries a segment that names nothing"));
            }
            if segment == PARENT_SEGMENT {
                return Err(unsafe_path(text, "it climbs above the target root"));
            }
            if segment == CURRENT_SEGMENT {
                return Err(unsafe_path(
                    text,
                    "it carries a segment that names the directory it already sits in",
                ));
            }
            if segment.contains(':') {
                return Err(unsafe_path(
                    text,
                    "it carries a colon, which names a drive on some platforms",
                ));
            }
            if segment.chars().any(char::is_control) {
                return Err(unsafe_path(text, "it carries a control character"));
            }
        }

        Ok(Self(text.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Where this file sits once the target root is known.
    ///
    /// The path is joined one segment at a time, so nothing of what the platform reads into a
    /// separator applies: the segments were settled when the value was built.
    pub fn under(&self, root: &Path) -> PathBuf {
        self.0
            .split(SEPARATOR)
            .fold(root.to_path_buf(), |path, segment| path.join(segment))
    }

    /// The directories this file needs, shallowest first.
    fn directories(&self) -> Vec<&str> {
        let mut directories = Vec::new();
        let mut end = 0;

        while let Some(offset) = self.0[end..].find(SEPARATOR) {
            end += offset;
            directories.push(&self.0[..end]);
            end += SEPARATOR.len_utf8();
        }

        directories
    }
}

impl fmt::Display for RelativePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// One file a target decided to emit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedFile {
    pub path: RelativePath,
    pub contents: String,
}

/// Everything a target emits, in the one order it is ever written in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTree {
    files: Vec<EmittedFile>,
    total_bytes: usize,
}

impl FileTree {
    pub fn build(mut files: Vec<EmittedFile>, limits: &Limits) -> Result<Self, Error> {
        if files.len() > limits.max_emitted_files {
            return Err(Error::TooManyEmittedFiles {
                count: files.len(),
                limit: limits.max_emitted_files,
            });
        }

        files.sort_by(|left, right| left.path.cmp(&right.path));

        for pair in files.windows(2) {
            if let [previous, next] = pair
                && previous.path == next.path
            {
                return Err(Error::DuplicateEmittedPath {
                    path: previous.path.to_string(),
                });
            }
        }

        // A file and the directory another file needs cannot share a name, and finding that out
        // halfway through a write would leave the tree in a state no emission describes.
        let paths: BTreeSet<&str> = files.iter().map(|file| file.path.as_str()).collect();
        for file in &files {
            for directory in file.path.directories() {
                if paths.contains(directory) {
                    return Err(Error::EmittedPathCollision {
                        path: directory.to_owned(),
                        nested: file.path.to_string(),
                    });
                }
            }
        }

        let total_bytes = files.iter().fold(0usize, |total, file| {
            total.saturating_add(file.contents.len())
        });
        if total_bytes > limits.max_emitted_bytes {
            return Err(Error::EmissionTooLarge {
                size: total_bytes,
                limit: limits.max_emitted_bytes,
            });
        }

        Ok(Self { files, total_bytes })
    }

    pub fn files(&self) -> &[EmittedFile] {
        &self.files
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }
}

/// What a target is allowed to do to the directory it writes into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    /// Everything under the root belongs to the generator: what the tree no longer carries goes.
    Directory,
    /// Only the emitted paths belong to the generator: everything else is left where it is.
    Files,
}

/// What one write did, and — by being empty — what it did not do.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WriteReport {
    pub written: Vec<RelativePath>,
    pub removed: Vec<RelativePath>,
    pub unchanged: usize,
}

/// Writes a target out, leaving alone every file that already holds the bytes it should.
///
/// Nothing is touched until the whole tree has been placed against what is already on disk, so a
/// path that cannot be written stops the write instead of leaving a tree half rewritten.
pub fn write_target(
    root: &Path,
    ownership: Ownership,
    tree: &FileTree,
    limits: &Limits,
) -> Result<WriteReport, Error> {
    for file in tree.files() {
        refuse_a_path_reached_through_a_link(root, &file.path)?;
    }

    let walked = walk(root, limits)?;

    let mut unchanged = 0;
    let mut to_write = Vec::new();
    for file in tree.files() {
        let absolute = file.path.under(root);
        if already_holds(&absolute, file.contents.as_bytes())? {
            unchanged += 1;
        } else {
            to_write.push((file, absolute));
        }
    }

    let mut to_remove = Vec::new();
    if ownership == Ownership::Directory {
        let kept: BTreeSet<&str> = tree.files().iter().map(|file| file.path.as_str()).collect();
        for absolute in &walked.files {
            let relative = relative_of(root, absolute, limits)?;
            if !kept.contains(relative.as_str()) {
                to_remove.push((relative, absolute.clone()));
            }
        }
        to_remove.sort_by(|left, right| left.0.cmp(&right.0));
    }

    let mut report = WriteReport {
        unchanged,
        ..WriteReport::default()
    };

    for (relative, absolute) in to_remove {
        fs::remove_file(&absolute).map_err(|err| unwritable(&absolute, &err))?;
        report.removed.push(relative);
    }

    for (file, absolute) in to_write {
        if let Some(parent) = absolute.parent() {
            fs::create_dir_all(parent).map_err(|err| unwritable(parent, &err))?;
        }
        fs::write(&absolute, file.contents.as_bytes())
            .map_err(|err| unwritable(&absolute, &err))?;
        report.written.push(file.path.clone());
    }

    if ownership == Ownership::Directory {
        prune(walked.directories)?;
    }

    Ok(report)
}

/// What is already under the target root, walked to a bounded depth and breadth.
#[derive(Debug, Default)]
struct Walked {
    /// Everything that is not a directory, symbolic links included: a link is never descended, and
    /// under [`Ownership::Directory`] it is as stale as any other file the tree does not carry.
    files: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}

fn walk(root: &Path, limits: &Limits) -> Result<Walked, Error> {
    let mut walked = Walked::default();

    match fs::metadata(root) {
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(walked),
        Err(err) => return Err(unreadable(root, &err)),
        Ok(metadata) if !metadata.is_dir() => {
            return Err(Error::TargetUnreadable {
                path: preview(&root.display().to_string()),
                reason: "the target root is not a directory".to_owned(),
            });
        }
        Ok(_) => {}
    }

    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut visited = 0usize;

    while let Some((directory, depth)) = stack.pop() {
        if depth > limits.max_path_depth {
            return Err(Error::PathTooDeep {
                path: preview(&directory.display().to_string()),
                depth,
                limit: limits.max_path_depth,
            });
        }

        let entries = fs::read_dir(&directory).map_err(|err| unreadable(&directory, &err))?;
        for entry in entries {
            let entry = entry.map_err(|err| unreadable(&directory, &err))?;

            visited += 1;
            if visited > limits.max_target_entries {
                return Err(Error::TargetTooLarge {
                    root: preview(&root.display().to_string()),
                    limit: limits.max_target_entries,
                });
            }

            let absolute = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.len() > limits.max_path_bytes {
                return Err(Error::PathTooLong {
                    path: preview(&name),
                    size: name.len(),
                    limit: limits.max_path_bytes,
                });
            }
            if marks_tests(&name) {
                return Err(Error::TestFileUnderTarget {
                    path: preview(&absolute.display().to_string()),
                });
            }

            // `file_type` does not follow symbolic links, so a link back up the tree is never
            // descended and no part of the tree is ever walked twice.
            let file_type = entry
                .file_type()
                .map_err(|err| unreadable(&absolute, &err))?;
            if file_type.is_dir() {
                walked.directories.push(absolute.clone());
                stack.push((absolute, depth + 1));
            } else {
                walked.files.push(absolute);
            }
        }
    }

    Ok(walked)
}

fn marks_tests(name: &str) -> bool {
    identifier::words(name)
        .iter()
        .any(|word| TEST_WORDS.contains(&word.as_str()))
}

/// Refuses an emitted path that a write would reach through a symbolic link.
///
/// `symlink_metadata` declines to follow only the last component of a path, so a link left on a
/// directory in the middle of one is still traversed — by that check, by `create_dir_all` and by the
/// write itself. Every component under the root is therefore looked at in turn, which is the only
/// shape of the check that actually bounds where a write can land. Without it, the whole guarantee
/// [`RelativePath`] gives is lexical: it settles what the path says, never what the disk does with
/// it.
///
/// Neither ownership is treated differently. Under [`Ownership::Directory`] a linked directory the
/// tree does not carry happens to be swept away by the stale-file pass before the write reaches it,
/// but that is an accident of the order the passes run in and it never applies under
/// [`Ownership::Files`], where nothing is ever removed. A link inside a target is an anomaly for
/// someone to look at either way, so it stops the write rather than being quietly replaced.
fn refuse_a_path_reached_through_a_link(root: &Path, path: &RelativePath) -> Result<(), Error> {
    let mut reached = root.to_path_buf();

    for segment in path.as_str().split(SEPARATOR) {
        reached.push(segment);

        match fs::symlink_metadata(&reached) {
            // Nothing stands here, so nothing stands below it either.
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
            Err(err) => return Err(unreadable(&reached, &err)),
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(Error::TargetUnwritable {
                    path: preview(&reached.display().to_string()),
                    reason: "a symbolic link stands on the way to a file the target emits, and \
                             writing would follow it out of the target"
                        .to_owned(),
                });
            }
            Ok(_) => {}
        }
    }

    Ok(())
}

/// Whether the file already holds exactly these bytes, so writing it would change nothing.
fn already_holds(path: &Path, contents: &[u8]) -> Result<bool, Error> {
    // Read as the link itself rather than as what it points at, so a link is never mistaken for the
    // file or the directory it names. That a link cannot stand anywhere along this path at all is
    // settled before any of this, by `refuse_a_path_reached_through_a_link`.
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(unreadable(path, &err)),
    };

    if metadata.is_dir() {
        return Err(Error::TargetUnwritable {
            path: preview(&path.display().to_string()),
            reason: "a directory already sits where this file has to be written".to_owned(),
        });
    }

    // A file of another length differs whatever it holds, which is what keeps a stale artefact of
    // any size from being read back in full only to be overwritten.
    if metadata.len() != contents.len() as u64 {
        return Ok(false);
    }

    let existing = fs::read(path).map_err(|err| unreadable(path, &err))?;
    Ok(existing == contents)
}

/// Where a walked path sits relative to the root it was walked from.
fn relative_of(root: &Path, absolute: &Path, limits: &Limits) -> Result<RelativePath, Error> {
    let suffix = absolute
        .strip_prefix(root)
        .map_err(|_| unsafe_path_of(absolute, "it sits outside the target root"))?;

    let mut text = String::new();
    for component in suffix.components() {
        let Component::Normal(name) = component else {
            return Err(unsafe_path_of(
                absolute,
                "it is not a plain path under the target root",
            ));
        };
        let name = name
            .to_str()
            .ok_or_else(|| unsafe_path_of(absolute, "its name is not UTF-8"))?;

        if !text.is_empty() {
            text.push(SEPARATOR);
        }
        text.push_str(name);
    }

    RelativePath::build(&text, limits)
}

/// Removes the directories the removals left empty, deepest first so a directory holding nothing
/// but emptied directories goes with them.
///
/// Sorting the paths in reverse puts every directory ahead of the one it sits in, since a parent is
/// a prefix of its children and therefore sorts before them.
fn prune(mut directories: Vec<PathBuf>) -> Result<(), Error> {
    directories.sort();
    directories.reverse();

    for directory in directories {
        let mut entries = fs::read_dir(&directory).map_err(|err| unreadable(&directory, &err))?;
        if entries.next().is_none() {
            fs::remove_dir(&directory).map_err(|err| unwritable(&directory, &err))?;
        }
    }

    Ok(())
}

fn unsafe_path(text: &str, reason: &str) -> Error {
    Error::UnsafePath {
        path: preview(text),
        reason: reason.to_owned(),
    }
}

fn unsafe_path_of(path: &Path, reason: &str) -> Error {
    unsafe_path(&path.display().to_string(), reason)
}

fn unreadable(path: &Path, err: &IoError) -> Error {
    Error::TargetUnreadable {
        path: preview(&path.display().to_string()),
        reason: err.to_string(),
    }
}

fn unwritable(path: &Path, err: &IoError) -> Error {
    Error::TargetUnwritable {
        path: preview(&path.display().to_string()),
        reason: err.to_string(),
    }
}
