//! What the development stack is configured with does not reach a cluster.
//!
//! `self-hosting/kubernetes/deployments.yaml` and the chart beside it were first produced by
//! running kompose over `docker-compose.yaml`, and they have been maintained by hand since. That
//! leaves one file describing two things: a stack a developer runs on a laptop, and a deployment
//! someone else runs in a cluster. A setting added for the first has no way of announcing that it
//! is not for the second, and the way it travels is quiet — kompose copies an environment entry
//! verbatim, and a value that is merely wrong in a cluster looks like every other value beside it.
//!
//! Two settings arrived that way. One raises a rate limit so a browser suite can drive the
//! endpoints that send mail; the other names the compose network's own bridge gateway as a trusted
//! reverse proxy, which is the address a caller on the developer's host arrives as. In a cluster
//! neither means anything, and the second means something worse than nothing: whatever range is
//! trusted there is a range whose occupants can name any client address they like and walk past
//! every per-IP limiter.
//!
//! So the compose file says which of its settings belong to the development stack, in a comment
//! above them, and this holds the manifests to it — in both directions, because only one of them
//! is a guard. A setting marked development-only must appear in no manifest. A setting *not*
//! marked must appear in every one, which is what stops the next arrival from being carried by
//! nobody noticing it was neither.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use yaml_rust2::{Yaml, YamlLoader};

/// The comment that says a setting exists for the development stack and for nothing else.
const DEVELOPMENT_ONLY: &str = "development stack only";

/// The compose services whose environment a manifest carries.
const API: &str = "api";
const WORKER: &str = "output-worker";

/// The most bytes read out of any one file. Every file here is a manifest; anything larger is not.
const MAX_BYTES: u64 = 1 << 20;

/// The most environment entries one service declares.
const MAX_SETTINGS: usize = 128;

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("two directories above this crate")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    let size = fs::metadata(path)
        .unwrap_or_else(|err| panic!("{} cannot be read: {err}", path.display()))
        .len();
    assert!(
        size <= MAX_BYTES,
        "{} is larger than {MAX_BYTES} bytes, so it is not one of the files this reads",
        path.display()
    );
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("{} cannot be read: {err}", path.display()))
}

/// One service's environment, split by whether the comment above each entry marks it as belonging
/// to the development stack.
///
/// Read as text rather than as YAML because the mark is a comment, and a comment is the one thing a
/// YAML reader throws away.
fn declared(compose: &str, service: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let (mut carried, mut development) = (BTreeSet::new(), BTreeSet::new());
    let mut here = false;
    let mut marked = false;

    for line in compose.lines() {
        if let Some(name) = line
            .strip_suffix(':')
            .and_then(|l| l.strip_prefix("  "))
            .filter(|name| !name.starts_with([' ', '-']))
        {
            here = name == service;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            marked |= trimmed.to_lowercase().contains(DEVELOPMENT_ONLY);
            continue;
        }
        if let Some(setting) = trimmed
            .strip_prefix("- ")
            .filter(|_| here && trimmed.contains('='))
            .and_then(|entry| entry.split('=').next())
            .filter(|setting| {
                !setting.is_empty()
                    && setting
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            })
        {
            if marked {
                development.insert(setting.to_owned());
            } else {
                carried.insert(setting.to_owned());
            }
        }
        marked = false;
    }

    assert!(
        carried.len() + development.len() <= MAX_SETTINGS,
        "the `{service}` service declares more than {MAX_SETTINGS} settings"
    );
    assert!(
        !carried.is_empty(),
        "no setting was read out of the `{service}` service, so this test is reading the compose \
         file wrong and would pass whatever the manifests said"
    );
    (carried, development)
}

/// Every environment name a manifest declares, read as text.
///
/// The chart's templates are not YAML — `name: {{ .Values.api.name }}` is a flow mapping to a YAML
/// reader and an error to most — so both kinds of file are read the same way, by the shape of an
/// environment entry. A container or a volume is named on the same kind of line, which is why only
/// a name spelled the way an environment variable is spelled counts.
fn settings_in(body: &str) -> BTreeSet<String> {
    body.lines()
        .filter_map(|line| line.trim_start().strip_prefix("- name: "))
        .map(str::trim)
        .filter(|name| {
            !name.is_empty()
                && name.starts_with(|c: char| c.is_ascii_uppercase())
                && name
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        })
        .map(str::to_owned)
        .collect()
}

/// Every file under `self-hosting`, so that a manifest added later is read without being listed.
fn manifests(root: &Path) -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    let mut pending = vec![root.join("self-hosting")];
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory)
            .unwrap_or_else(|err| panic!("{} cannot be listed: {err}", directory.display()));
        for entry in entries {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
                let body = read(&path);
                found.push((path, body));
            }
        }
    }
    assert!(
        !found.is_empty(),
        "no manifest was found under `self-hosting`, so this test would pass whatever it held"
    );
    found
}

/// One Deployment's environment out of the multi-document manifest, which is plain YAML.
fn deployment_settings(body: &str, name: &str) -> BTreeSet<String> {
    let documents = YamlLoader::load_from_str(body).expect("the manifest is readable YAML");
    let container = documents
        .iter()
        .find(|document| {
            document["kind"].as_str() == Some("Deployment")
                && document["metadata"]["name"].as_str() == Some(name)
        })
        .unwrap_or_else(|| panic!("the manifest declares a Deployment named `{name}`"));

    let environment = &container["spec"]["template"]["spec"]["containers"][0]["env"];
    environment
        .as_vec()
        .map(|entries| {
            entries
                .iter()
                .filter_map(|entry| entry["name"].as_str())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_else(|| {
            assert!(
                matches!(environment, Yaml::BadValue),
                "the `{name}` Deployment declares an environment that is not a list"
            );
            BTreeSet::new()
        })
}

#[test]
fn no_manifest_carries_a_setting_the_compose_file_keeps_for_the_development_stack() {
    let root = repository();
    let compose = read(&root.join("docker-compose.yaml"));

    let development: BTreeSet<String> = [API, WORKER]
        .iter()
        .flat_map(|service| declared(&compose, service).1)
        .collect();
    assert!(
        !development.is_empty(),
        "the compose file marks no setting `{DEVELOPMENT_ONLY}`, so either the mark was renamed \
         and this test now guards nothing, or a development-only setting has lost its comment."
    );

    for (path, body) in manifests(&root) {
        let carried: Vec<&String> = development
            .iter()
            .filter(|setting| settings_in(&body).contains(*setting))
            .collect();
        assert!(
            carried.is_empty(),
            "{} declares {carried:?}, which the compose file marks `{DEVELOPMENT_ONLY}`. These \
             files are applied to real clusters: a rate limit raised for a browser suite, or a \
             trusted-proxy range naming a Docker bridge, is a development value there and the \
             second one lets whoever is inside it claim any client address and walk past every \
             per-IP limiter. Regenerating from the compose file is what carries them across, \
             which is why `self-hosting/generate-from-docker-compose.sh` writes nothing.",
            path.display()
        );
    }
}

#[test]
fn every_setting_the_compose_file_does_not_keep_for_itself_reaches_both_manifests() {
    let root = repository();
    let compose = read(&root.join("docker-compose.yaml"));
    let self_hosting = root.join("self-hosting");
    let deployments = read(&self_hosting.join("kubernetes/deployments.yaml"));

    for (service, template) in [
        (API, "helm/templates/api-deployment.yaml"),
        (WORKER, "helm/templates/output-worker-deployment.yaml"),
    ] {
        let (carried, _) = declared(&compose, service);
        let in_manifest = deployment_settings(&deployments, service);
        let in_chart = settings_in(&read(&self_hosting.join(template)));

        for (where_, held) in [
            ("kubernetes/deployments.yaml", &in_manifest),
            (template, &in_chart),
        ] {
            let missing: Vec<&String> = carried.iter().filter(|s| !held.contains(*s)).collect();
            assert!(
                missing.is_empty(),
                "the `{service}` service declares {missing:?}, which `self-hosting/{where_}` does \
                 not. Either the setting is one a cluster needs, and it belongs there too, or it \
                 exists for the development stack, and the comment above it in the compose file \
                 has to say `{DEVELOPMENT_ONLY}` — which is what keeps it out of these files and \
                 out of the next person's cluster."
            );
        }
    }
}
