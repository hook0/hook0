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

/// What a job says under one key, following `extends:` for a job that says nothing of its own.
fn inherited<'a>(
    declared: &'a BTreeMap<String, Declared>,
    name: &str,
    key: &str,
    left: usize,
) -> Option<&'a Yaml> {
    if left == 0 {
        return None;
    }
    let body = &declared.get(name)?.body;
    let own = &body[key];
    if !own.is_badvalue() {
        return Some(own);
    }

    let extends = &body["extends"];
    let parents: Box<dyn Iterator<Item = &Yaml>> = match extends {
        Yaml::Array(all) => Box::new(all.iter()),
        other => Box::new(iter::once(other)),
    };
    parents
        .filter_map(|parent| parent.as_str())
        .find_map(|parent| inherited(declared, parent, key, left - 1))
}

/// The rules a job runs under, following `extends:` for a job that declares none of its own.
fn rules<'a>(
    declared: &'a BTreeMap<String, Declared>,
    name: &str,
    left: usize,
) -> Option<&'a Vec<Yaml>> {
    inherited(declared, name, "rules", left)?.as_vec()
}

/// Whether one job waits for another, however many jobs stand between the two.
fn reaches(declared: &BTreeMap<String, Declared>, from: &str, to: &str) -> bool {
    let mut seen = BTreeSet::new();
    let mut pending = vec![from.to_owned()];
    // Every job at most once, so a cycle somebody wrote is answered rather than followed.
    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(job) = declared.get(&name) else {
            continue;
        };
        for (target, _) in needed(&job.body) {
            if target == to {
                return true;
            }
            pending.push(target);
        }
    }
    false
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

/// The pipeline file the SDK release is declared in. Every job it puts in the release stage
/// publishes something, to a registry or by writing a tag a fetch resolves.
const SDK_RELEASE: &str = "ci/release-sdk.gitlab-ci.yml";

/// The job that drives every client against a Hook0 that is really running.
const LIVE_SMOKE: &str = "clients.live-smoke";

/// Fewest publish jobs `SDK_RELEASE` is expected to hold. One per registry the SDKs go out on,
/// and the number only falls when a registry is dropped, which is a change somebody makes on
/// purpose and can adjust here.
const MIN_PUBLISH_JOBS: usize = 9;

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

#[test]
fn nothing_is_published_before_every_client_has_talked_to_a_real_hook0() {
    let declared = jobs(&repository());

    let publishing: Vec<&String> = declared
        .iter()
        .filter(|(name, job)| {
            !name.starts_with('.')
                && job.file == SDK_RELEASE
                && inherited(&declared, name, "stage", MAX_EXTENDS).and_then(Yaml::as_str)
                    == Some("release")
        })
        .map(|(name, _)| name)
        .collect();

    // The filter above is three conditions, and every one of them can stop matching without the
    // publishing stopping: the file renamed, a job moved out of it, a stage spelled differently.
    // Any of those leaves nothing to check and this test passing while nothing waits for the live
    // smoke at all, which is the failure it exists to prevent rather than one it may report.
    assert!(
        publishing.len() >= MIN_PUBLISH_JOBS,
        "only {} release-stage jobs were found in `{SDK_RELEASE}`, fewer than the \
         {MIN_PUBLISH_JOBS} this repository publishes, so this guard just held nothing. Either \
         the publishing moved and this has to follow it, or a registry lost its job.",
        publishing.len()
    );

    let ungated: Vec<String> = publishing
        .iter()
        .filter(|name| !reaches(&declared, name, LIVE_SMOKE))
        .map(|name| format!("  {name}"))
        .collect();

    assert!(
        !declared.contains_key(LIVE_SMOKE) || ungated.is_empty(),
        "`{SDK_RELEASE}` says nothing is published without `{LIVE_SMOKE}` having passed for every \
         client at once. These jobs publish without waiting for it, directly or through anything \
         they wait for, so what they put on a registry is whatever the tag happened to hold:\n{}",
        ungated.join("\n")
    );
    assert!(
        declared.contains_key(LIVE_SMOKE),
        "`{LIVE_SMOKE}` is gone, and with it the only thing every publish job was made to wait \
         for. Whatever replaced it is what this guard should name."
    );
}

/// What runs the guard that compares every generated tree to what the generator writes.
///
/// Named by what it runs rather than by the job it sits in, so moving the guard to another job
/// keeps this test pointed at it.
const DRIFT_GUARD_COMMAND: &str = "cargo test -p hook0-sdkgen";

/// Whether GitLab would consider `path` changed by a job watching `glob`.
///
/// GitLab asks Ruby's `File.fnmatch?` under `FNM_PATHNAME`, and one rule of that is worth
/// spelling out because reading the pattern does not give it away. A `**` walks down only when
/// it is followed by a `/`. Written at the end of a pattern it stands for a single segment,
/// exactly as `*` does. So `clients/*/src/**` matches `clients/rust/src/lib.rs` and nothing
/// deeper, while `clients/*/src/**/*` matches all the way down.
fn watched_by_gitlab(glob: &str, path: &str) -> bool {
    fn segments(pattern: &[&str], path: &[&str]) -> bool {
        match pattern.split_first() {
            None => path.is_empty(),
            Some((&"**", [])) => path.len() == 1,
            Some((&"**", rest)) => (0..=path.len()).any(|taken| segments(rest, &path[taken..])),
            Some((head, rest)) => path
                .split_first()
                .is_some_and(|(first, tail)| one_segment(head, first) && segments(rest, tail)),
        }
    }

    /// `*` within a segment, which never reaches across a `/`.
    fn one_segment(pattern: &str, name: &str) -> bool {
        match pattern.split_once('*') {
            None => pattern == name,
            Some((before, after)) => {
                name.len() >= before.len()
                    && name.starts_with(before)
                    && (before.len()..=name.len()).any(|at| one_segment(after, &name[at..]))
            }
        }
    }

    segments(
        &glob.split('/').collect::<Vec<_>>(),
        &path.split('/').collect::<Vec<_>>(),
    )
}

/// The end of a pattern decides how far down it reaches, and reading it does not say so.
#[test]
fn a_trailing_double_star_reaches_one_level_and_a_middle_one_reaches_all_of_them() {
    // What the pipeline used to watch, and what it missed.
    assert!(watched_by_gitlab(
        "clients/*/src/**",
        "clients/rust/src/lib.rs"
    ));
    assert!(!watched_by_gitlab(
        "clients/*/src/**",
        "clients/rust/src/generated/models.rs"
    ));

    // What it watches now.
    assert!(watched_by_gitlab(
        "clients/*/src/**/*",
        "clients/rust/src/generated/models.rs"
    ));
    assert!(watched_by_gitlab(
        "clients/*/src/**/*",
        "clients/java/src/main/java/com/hook0/client/generated/Models.java"
    ));

    // A wildcard segment stops at a separator either way.
    assert!(!watched_by_gitlab(
        "clients/*/src/**/*",
        "clients/go/generated/api.go"
    ));
    assert!(!watched_by_gitlab("clients/*", "clients/rust/src/lib.rs"));
    assert!(watched_by_gitlab("Cargo.*", "Cargo.lock"));
}

#[test]
fn a_hand_edit_of_any_generated_tree_faces_the_guard_that_wrote_it() {
    let declared = jobs(&repository());

    let running: Vec<(&String, BTreeSet<String>)> = declared
        .iter()
        .filter(|(name, job)| {
            !name.starts_with('.')
                && job.body["script"]
                    .as_vec()
                    .into_iter()
                    .flatten()
                    .filter_map(Yaml::as_str)
                    .any(|line| line.contains(DRIFT_GUARD_COMMAND))
        })
        .map(|(name, _)| {
            let watched = rules(&declared, name, MAX_EXTENDS)
                .map(|rules| triggering_paths(rules))
                .unwrap_or_default();
            (name, watched)
        })
        .collect();

    // Were the command spelled differently, nothing below would be checked and this would pass
    // having looked at no job at all.
    assert!(
        !running.is_empty(),
        "no job runs `{DRIFT_GUARD_COMMAND}`, so nothing compares a generated tree to what the \
         generator writes. If the guard moved, point this test at what runs it now."
    );

    let unwatched: Vec<String> = hook0_sdkgen::targets::targets()
        .iter()
        .filter(|target| {
            let edited = format!("{}/a-file-somebody-edited", target.root);
            !running
                .iter()
                .any(|(_, watched)| watched.iter().any(|glob| watched_by_gitlab(glob, &edited)))
        })
        .map(|target| format!("  {} writes {}", target.name, target.root))
        .collect();

    assert!(
        unwatched.is_empty(),
        "a hand edit under these generated trees triggers no job that runs \
         `{DRIFT_GUARD_COMMAND}`, so nothing would compare the edit to what the generator writes \
         and it would merge green:\n{}\n\nWatch them from the job that runs the guard. Mind that \
         a `**` at the end of a pattern reaches one level only.",
        unwatched.join("\n")
    );
}

/// The file the whole pipeline is assembled in.
///
/// It pins the version of every toolchain the jobs run on — the compilers, the runtimes, the
/// package managers, the scanners — declares the stages, and pulls in the other thirty-odd files.
/// There is no change to it that leaves the jobs it configures alone.
const ROOT_PIPELINE: &str = ".gitlab-ci.yml";

/// A job gated on paths runs on a change to the file that configures it.
///
/// A `changes:` rule is a claim about what can affect the job, and two files can always affect it:
/// the one that pins the toolchain it runs on, and the one it is written in. Neither was named
/// anywhere, so raising a compiler version, or editing the rules of a job, was a change that ran
/// none of what it changed — the pipeline came back green over a job that never started.
#[test]
fn a_job_gated_on_paths_runs_on_a_change_to_what_configures_it() {
    let root = repository();
    let declared = jobs(&root);
    let mut blind = Vec::new();
    let mut gated = 0usize;

    for (name, job) in &declared {
        let Some(rules) = rules(&declared, name, MAX_EXTENDS) else {
            continue;
        };
        let watched = triggering_paths(rules);
        // A job no path gates is one every pipeline considers, so there is nothing to miss.
        if watched.is_empty() {
            continue;
        }
        gated += 1;

        for configuring in [ROOT_PIPELINE, job.file.as_str()] {
            if !watched
                .iter()
                .any(|glob| watched_by_gitlab(glob, configuring))
            {
                blind.push(format!("{name} does not watch {configuring}"));
            }
        }
    }

    assert!(
        gated > 0,
        "no job in the pipeline is gated on paths, so this proves nothing"
    );
    assert!(
        blind.is_empty(),
        "these jobs would sit out a change to what configures them:\n{}",
        blind.join("\n")
    );
}

/// Both directions of the one list that decides which pipeline files exist.
///
/// GitLab resolves `include:` before anything else, so a `local:` naming a file that is not there
/// is not a job that fails: it is a pipeline that cannot be created, landing on whoever pushes next.
/// The other direction is quieter still — a pipeline file nobody includes declares jobs that never
/// run, and looks from the inside exactly like one that does.
#[test]
fn every_included_pipeline_file_is_there_and_every_file_there_is_included() {
    let root = repository();
    let body = fs::read_to_string(root.join(ROOT_PIPELINE)).expect("the root pipeline");
    let document = YamlLoader::load_from_str(&body).expect("the root pipeline parses");
    let includes = document
        .first()
        .map(|it| &it["include"])
        .and_then(Yaml::as_vec)
        .expect("the includes");

    let included: BTreeSet<String> = includes
        .iter()
        .filter_map(|include| include["local"].as_str())
        .map(|path| path.trim_start_matches('/').to_owned())
        .collect();
    assert!(!included.is_empty(), "the root pipeline includes no file");

    let missing: Vec<&String> = included
        .iter()
        .filter(|path| !root.join(path).is_file())
        .collect();
    assert!(
        missing.is_empty(),
        "the root pipeline includes files that are not there: {missing:?}"
    );

    let orphans: Vec<String> = pipeline_files(&root)
        .iter()
        .filter_map(|path| path.strip_prefix(&root).ok())
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .filter(|path| path != ROOT_PIPELINE && !included.contains(path))
        .collect();
    assert!(
        orphans.is_empty(),
        "these pipeline files declare jobs nothing includes: {orphans:?}"
    );
}
