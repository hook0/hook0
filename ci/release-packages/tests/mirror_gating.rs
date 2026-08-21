//! Nothing is mirrored before the client being mirrored has passed its own suite.
//!
//! For Go, Packagist and Zig, pushing the tag to the mirror is the publication rather than a step
//! towards one, and the same push carries the other eight clients' source. So `sdk-release.mirrors`
//! waits for every client's check job, one `needs:` entry each.
//!
//! Eleven entries written by hand is eleven chances to forget one, and forgetting one is silent:
//! the release still runs, and the client whose suite was skipped is published anyway. What that
//! list has to match is not a list at all. The mirrored set comes from `mirrors()`, the same call
//! the release makes, and the job that checks a client is read from that client's own pipeline
//! file. Neither is written down here.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use release_packages::{Mirror, TargetRoot, discover, mirrors};
use yaml_rust2::{Yaml, YamlLoader};

/// The job whose `needs:` this is about.
const MIRROR_JOB: &str = "sdk-release.mirrors";

/// Where that job is declared.
const SDK_RELEASE: &str = "ci/release-sdk.gitlab-ci.yml";

/// The most bytes read out of one pipeline file.
const MAX_BYTES: u64 = 256 * 1024;

/// Top-level keys that configure a pipeline rather than declare a job.
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

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("two directories above this crate")
        .to_path_buf()
}

/// What the generator says it writes, which is the only thing that knows a client exists.
fn registry() -> Vec<TargetRoot> {
    hook0_sdkgen::targets::targets()
        .iter()
        .map(|target| TargetRoot {
            name: target.name.to_owned(),
            root: target.root.to_owned(),
        })
        .collect()
}

/// Every mirror one SDK release pushes, taken from the call the release itself makes.
fn pushed(root: &Path) -> Vec<Mirror> {
    let packages = discover(&registry(), root).expect("the tree yields its packages");
    mirrors(&packages).expect("the packages yield their mirrors")
}

fn document(path: &Path) -> Yaml {
    let size = fs::metadata(path)
        .unwrap_or_else(|err| panic!("{} cannot be looked at: {err}", path.display()))
        .len();
    assert!(
        size <= MAX_BYTES,
        "{} is larger than the {MAX_BYTES} bytes read at most",
        path.display()
    );
    let body = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("{} cannot be read: {err}", path.display()));
    YamlLoader::load_from_str(&body)
        .unwrap_or_else(|err| panic!("{} is unreadable: {err}", path.display()))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{} holds no document", path.display()))
}

/// The jobs a pipeline file declares, templates left out.
fn jobs_declared_in(path: &Path) -> BTreeSet<String> {
    let Yaml::Hash(top) = document(path) else {
        return BTreeSet::new();
    };
    top.into_iter()
        .filter_map(|(key, value)| {
            let name = key.as_str()?.to_owned();
            let is_job = !name.starts_with('.')
                && !NOT_A_JOB.contains(&name.as_str())
                && matches!(value, Yaml::Hash(_));
            is_job.then_some(name)
        })
        .collect()
}

/// What the mirror push waits for.
fn awaited(root: &Path) -> BTreeSet<String> {
    let document = document(&root.join(SDK_RELEASE));
    document[MIRROR_JOB]["needs"]
        .as_vec()
        .unwrap_or_else(|| panic!("`{MIRROR_JOB}` declares no `needs:` in {SDK_RELEASE}"))
        .iter()
        .filter_map(|entry| match entry {
            Yaml::String(name) => Some(name.clone()),
            other => Some(other["job"].as_str()?.to_owned()),
        })
        .collect()
}

#[test]
fn the_mirror_push_waits_for_every_client_it_mirrors() {
    let root = repository();
    let awaited = awaited(&root);
    let pushed = pushed(&root);

    assert!(
        !pushed.is_empty(),
        "no mirror was found, so this guard checked nothing"
    );

    let ungated: Vec<String> = pushed
        .iter()
        .filter_map(|mirror| {
            let pipeline = root.join(&mirror.directory).join(".gitlab-ci.yml");
            if !pipeline.is_file() {
                return Some(format!(
                    "  {} is mirrored and declares no pipeline of its own",
                    mirror.directory
                ));
            }
            let declared = jobs_declared_in(&pipeline);
            match declared.intersection(&awaited).next() {
                Some(_) => None,
                None => Some(format!(
                    "  {} is mirrored, and none of its jobs ({}) is waited for",
                    mirror.directory,
                    declared.into_iter().collect::<Vec<_>>().join(", ")
                )),
            }
        })
        .collect();

    assert!(
        ungated.is_empty(),
        "`{MIRROR_JOB}` pushes these clients to a repository their own suite never had to pass. \
         For Go, Packagist and Zig that push is the publication, and for the rest it is public \
         source under a tag:\n{}\n\
         Add a `needs:` on the client's check job in `{SDK_RELEASE}`.",
        ungated.join("\n")
    );
}
