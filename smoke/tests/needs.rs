//! What a job says it needs, against what the pipeline is going to contain.
//!
//! GitLab refuses to create a pipeline when a job needs another job that the rules of the moment
//! left out, and it names both jobs when it does. That refusal takes the whole repository's
//! pipeline down rather than skipping a job, and it lands on whoever pushes next rather than on
//! whoever caused it, since which files a push touches is what decides.
//!
//! The question asked here is the cheap one that catches the shape which actually occurred: a job
//! triggered by a path its dependency does not watch. `clients.live-smoke` watched
//! `docker-compose.yaml`, `smoke/**/*` and the generator's target registry; the build it needs its
//! two binaries from watched none of the three, so a change confined to any of them was a pipeline
//! that could not be created at all.
//!
//! This is a sufficient condition rather than the exact one. A pattern covers a trigger when the
//! two are written alike, or when the pattern is a directory wildcard standing above it, which is
//! the one form that appears here often enough to be worth reading properly. Every other pair of
//! patterns that happen to mean the same thing is reported as a difference it does not have to be.
//! That is the direction worth being wrong in, since the answer is to make the two read alike
//! while the cost of the other direction is a pipeline nobody can create.
//!
//! Tag pipelines are deliberately outside this. A rule carrying `changes:` and no `if:` matches
//! everything on a tag, because there is no push to compare against, and that is what lets every
//! release job here need a check job gated on paths. Those edges are not defects and are not
//! reported.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::{fs, iter};

use yaml_rust2::{Yaml, YamlLoader};

/// The most pipeline files read. The repository has one per component; anything past this is a
/// walk that has escaped rather than a repository that grew.
const MAX_FILES: usize = 128;

/// The most bytes read out of one pipeline file.
const MAX_BYTES: u64 = 256 * 1024;

/// How far below the repository root a pipeline file is looked for.
const MAX_DEPTH: usize = 6;

/// How far an `extends:` chain is followed before it is treated as a loop.
const MAX_EXTENDS: usize = 8;

/// Directories that hold no pipeline of this repository's, only copies of one.
const NOT_SOURCE: [&str; 3] = ["target", "node_modules", ".git"];

/// Top-level keys of a pipeline file that configure the pipeline rather than declare a job.
const NOT_A_JOB: [&str; 9] = [
    "include",
    "stages",
    "variables",
    "workflow",
    "default",
    "image",
    "before_script",
    "after_script",
    "cache",
];

/// Every pipeline file this repository commits, found rather than listed.
fn pipeline_files(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![(root.to_path_buf(), 0usize)];

    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_DEPTH || found.len() >= MAX_FILES {
            continue;
        }
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            // `file_type` rather than `metadata`, so a symlink is what it is rather than what it
            // points at and the walk cannot be sent somewhere else.
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name();
            let name = name.to_string_lossy().into_owned();
            if kind.is_dir() {
                if !NOT_SOURCE.contains(&name.as_str()) {
                    pending.push((path, depth + 1));
                }
            } else if kind.is_file() && name.ends_with(".gitlab-ci.yml") && found.len() < MAX_FILES
            {
                found.push(path);
            }
        }
    }

    found.sort();
    found
}

/// A job, as the pipeline file declares it, and where that was.
struct Declared {
    body: Yaml,
    file: String,
}

/// Every job the repository declares, hidden templates included, since those are what `extends:`
/// reaches.
fn jobs(root: &Path) -> BTreeMap<String, Declared> {
    let mut declared = BTreeMap::new();

    for path in pipeline_files(root) {
        let readable = fs::metadata(&path).is_ok_and(|it| it.len() <= MAX_BYTES);
        assert!(
            readable,
            "{} is larger than the {MAX_BYTES} bytes read at most, so it is not a pipeline file",
            path.display()
        );
        let body = fs::read_to_string(&path).expect("a readable pipeline file");
        let documents = YamlLoader::load_from_str(&body)
            .unwrap_or_else(|cause| panic!("{} is unreadable: {cause}", path.display()));
        let Some(Yaml::Hash(top)) = documents.first() else {
            continue;
        };

        let shown = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        for (key, value) in top {
            let Some(name) = key.as_str() else {
                continue;
            };
            if NOT_A_JOB.contains(&name) || !matches!(value, Yaml::Hash(_)) {
                continue;
            }
            if let Some(before) = declared.insert(
                name.to_owned(),
                Declared {
                    body: value.clone(),
                    file: shown.clone(),
                },
            ) {
                panic!(
                    "`{name}` is declared in both {} and {shown}; GitLab keeps one of the two and \
                     which one is not something to depend on",
                    before.file
                );
            }
        }
    }

    assert!(
        !declared.is_empty(),
        "no pipeline file was found under {}, so this guard checked nothing",
        root.display()
    );
    declared
}

/// The rules a job runs under, following `extends:` for a job that declares none of its own.
fn rules<'a>(
    declared: &'a BTreeMap<String, Declared>,
    name: &str,
    left: usize,
) -> Option<&'a Vec<Yaml>> {
    if left == 0 {
        return None;
    }
    let body = &declared.get(name)?.body;
    if let Some(own) = body["rules"].as_vec() {
        return Some(own);
    }

    let extends = &body["extends"];
    let parents: Box<dyn Iterator<Item = &Yaml>> = match extends {
        Yaml::Array(all) => Box::new(all.iter()),
        other => Box::new(iter::once(other)),
    };
    parents
        .filter_map(|parent| parent.as_str())
        .find_map(|parent| rules(declared, parent, left - 1))
}

/// The paths that make a job run. A rule that names none contributes none, because on a branch
/// its `if:` is what decides and on a tag it matches regardless.
fn triggering_paths(rules: &[Yaml]) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    for rule in rules {
        // A rule that refuses is not one that triggers.
        if rule["when"].as_str() == Some("never") {
            continue;
        }
        // `changes:` is either the list itself or a map declaring `paths:`.
        let changes = &rule["changes"];
        let listed = changes.as_vec().or_else(|| changes["paths"].as_vec());
        for path in listed.into_iter().flatten() {
            if let Some(path) = path.as_str() {
                paths.insert(path.to_owned());
            }
        }
    }
    paths
}

/// Whether a pattern a job watches stands over a path that triggers another job.
///
/// Written alike is the common case. Beyond it, one rule applies. A pattern ending in a
/// directory wildcard covers everything under that directory, which is what
/// `tests-api-integrations/**/*` does to `tests-api-integrations/.gitlab-ci.yml`. Anything subtler is left to report a
/// difference, since being told to make two lines read alike costs a line and being wrong the
/// other way costs the pipeline.
fn covers(watched: &str, trigger: &str) -> bool {
    if watched == trigger {
        return true;
    }
    let above = watched
        .strip_suffix("/**/*")
        .or_else(|| watched.strip_suffix("/**"));
    above.is_some_and(|directory| {
        trigger
            .strip_prefix(directory)
            .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// What a job needs: the name, and whether it said the job may be absent.
fn needed(body: &Yaml) -> Vec<(String, bool)> {
    let Some(all) = body["needs"].as_vec() else {
        return Vec::new();
    };
    all.iter()
        .filter_map(|one| match one {
            Yaml::String(name) => Some((name.clone(), false)),
            // A need reaching into another project is about that project's pipeline, not this one's.
            other if other["project"].is_badvalue() => Some((
                other["job"].as_str()?.to_owned(),
                other["optional"].as_bool().unwrap_or(false),
            )),
            _ => None,
        })
        .collect()
}

/// The repository this crate sits in.
fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("a directory above this crate")
        .to_path_buf()
}

#[test]
fn every_needed_job_is_declared() {
    let declared = jobs(&repository());

    let missing: Vec<String> = declared
        .iter()
        .filter(|(name, _)| !name.starts_with('.'))
        .flat_map(|(name, job)| {
            needed(&job.body)
                .into_iter()
                .filter(|(target, optional)| !optional && !declared.contains_key(target))
                .map(move |(target, _)| {
                    format!("  {name} needs `{target}`, which nothing declares")
                })
        })
        .collect();

    assert!(
        missing.is_empty(),
        "a job needs a job this repository does not declare, which is a pipeline GitLab refuses \
         to create:\n{}",
        missing.join("\n")
    );
}

#[test]
fn a_job_is_not_triggered_by_a_path_the_job_it_needs_ignores() {
    let declared = jobs(&repository());

    let mut divergences = Vec::new();
    for (name, job) in &declared {
        if name.starts_with('.') {
            continue;
        }
        let Some(own) = rules(&declared, name, MAX_EXTENDS) else {
            continue;
        };
        let triggers = triggering_paths(own);
        if triggers.is_empty() {
            continue;
        }

        for (target, optional) in needed(&job.body) {
            if optional {
                continue;
            }
            // A job with no rules of its own is in every pipeline, so it can never be the one
            // missing.
            let Some(theirs) = rules(&declared, &target, MAX_EXTENDS) else {
                continue;
            };
            let watched = triggering_paths(theirs);
            let unwatched: Vec<&String> = triggers
                .iter()
                .filter(|trigger| !watched.iter().any(|pattern| covers(pattern, trigger)))
                .collect();
            if !unwatched.is_empty() {
                let listed = unwatched
                    .iter()
                    .map(|path| format!("      {path}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                divergences.push(format!(
                    "  {name} ({}) needs {target} ({}), and runs on paths it does not watch:\n{listed}",
                    job.file, declared[&target].file
                ));
            }
        }
    }

    assert!(
        divergences.is_empty(),
        "a change confined to one of the paths below builds a pipeline where a job needs a job \
         that is not in it, which GitLab refuses to create at all. Add the path to the job that \
         is needed, or make the need optional and the job able to run without it:\n{}",
        divergences.join("\n")
    );
}

#[test]
fn a_directory_wildcard_covers_what_is_under_it_and_nothing_beside_it() {
    assert!(covers("smoke/**/*", "smoke/tests/needs.rs"));
    assert!(covers("smoke/**", "smoke/tests/needs.rs"));
    assert!(covers(
        "tests-api-integrations/**/*",
        "tests-api-integrations/.gitlab-ci.yml"
    ));
    assert!(covers("Cargo.*", "Cargo.*"));

    // The prefix is a directory, so a sibling whose name merely starts with it is not under it.
    assert!(!covers("smoke/**/*", "smoke-harness/tests/needs.rs"));
    assert!(!covers("smoke/**/*", "smoketest"));
    // The directory itself is not a file under itself, and an unrelated path is not covered.
    assert!(!covers("smoke/**/*", "smoke"));
    assert!(!covers("smoke/**/*", "api/openapi.snapshot.json"));
    // A wildcard is not read as one anywhere but at the end, so this is only itself.
    assert!(!covers("clients/*/src/**", "clients/rust/src/lib.rs"));
}
