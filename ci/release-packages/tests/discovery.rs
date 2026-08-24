//! What discovery answers, asked of real directories.
//!
//! Every tree here is built by the test that reads it, and every target put in front of the tool is
//! one it was never written for — which is the property under test: a client is released because it
//! is in the tree and in the generator's registry, never because somebody remembered to add it to a
//! list. The suite would keep passing if every target this repository ships today were renamed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use release_packages::{
    Change, Error, Mirror, Package, TargetRoot, Version, check_mirrors, check_publishers,
    check_version, current_version, discover, mirrors, render, sdk_train, set_version,
};

/// A tree with nothing in it but what a test puts there.
struct Tree(tempfile::TempDir);

impl Tree {
    fn new() -> Tree {
        Tree(tempfile::tempdir().expect("a temporary directory"))
    }

    fn path(&self) -> &Path {
        self.0.path()
    }

    /// Write a file, making the directories above it.
    fn write(&self, relative: &str, body: &str) -> &Tree {
        let path = self.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the directories above the file");
        }
        fs::write(&path, body).expect("the file");
        self
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.path().join(relative)).expect("the file")
    }

    fn directory(&self, relative: &str) -> &Tree {
        fs::create_dir_all(self.path().join(relative)).expect("the directory");
        self
    }
}

fn target(name: &str, root: &str) -> TargetRoot {
    TargetRoot {
        name: name.to_string(),
        root: root.to_string(),
    }
}

fn only(packages: Vec<Package>) -> Package {
    assert_eq!(packages.len(), 1, "one package, got {packages:?}");
    packages.into_iter().next().expect("the one package")
}

fn refusal(tree: &Tree, targets: &[TargetRoot]) -> Error {
    match discover(targets, tree.path()) {
        Ok(found) => panic!("expected a refusal, got {found:?}"),
        Err(error) => error,
    }
}

// --- Nothing is written down beside the tree ----------------------------------------------------

#[test]
fn a_target_the_tool_was_never_written_for_is_discovered() {
    let tree = Tree::new();
    tree.directory("clients/nobody/generated").write(
        "clients/nobody/package.json",
        r#"{ "name": "@hook0/nobody", "version": "4.5.6" }"#,
    );

    let package = only(
        discover(&[target("nobody", "clients/nobody/generated")], tree.path())
            .expect("the package the tree describes"),
    );

    assert_eq!(package.name, "@hook0/nobody");
    assert_eq!(package.directory, "clients/nobody");
    assert_eq!(package.manifest, "clients/nobody/package.json");
    assert_eq!(package.registry.id(), "npm");
    assert_eq!(package.version.to_string(), "4.5.6");
}

#[test]
fn a_target_whose_manifest_is_gone_stops_the_release_naming_itself() {
    let tree = Tree::new();
    tree.directory("clients/nobody/generated");

    let error = refusal(&tree, &[target("nobody", "clients/nobody/generated")]);
    let said = error.to_string();

    assert!(said.contains("nobody"), "{said}");
    assert!(said.contains("clients/nobody/generated"), "{said}");
    assert!(said.contains("clients/nobody"), "{said}");
    assert!(matches!(error, Error::NoManifest { .. }), "{error:?}");
}

#[test]
fn a_manifest_shape_the_tool_does_not_know_is_a_refusal_rather_than_a_skip() {
    let tree = Tree::new();
    tree.directory("clients/swifty/generated").write(
        "clients/swifty/Package.swift",
        "let package = Package(name: \"Hook0\")",
    );

    let error = refusal(&tree, &[target("swifty", "clients/swifty/generated")]);
    let said = error.to_string();

    assert!(said.contains("swifty"), "{said}");
    assert!(said.contains("clients/swifty"), "{said}");
}

#[test]
fn the_ascent_stops_at_the_package_and_never_reaches_the_repository() {
    let tree = Tree::new();
    tree.write(
        "Cargo.toml",
        "[package]\nname = \"the-repo\"\nversion = \"9.9.9\"\n",
    )
    .directory("clients/nobody/generated");

    let error = refusal(&tree, &[target("nobody", "clients/nobody/generated")]);
    let said = error.to_string();

    assert!(!said.contains("the-repo"), "{said}");
    assert!(matches!(error, Error::NoManifest { .. }), "{error:?}");
}

#[test]
fn a_target_landing_outside_the_clients_directory_is_refused() {
    let tree = Tree::new();
    tree.write(
        "elsewhere/Cargo.toml",
        "[package]\nname = \"x\"\nversion = \"1.0.0\"\n",
    );

    let error = refusal(&tree, &[target("elsewhere", "elsewhere/generated")]);

    assert!(
        matches!(error, Error::RootOutsideClients { .. }),
        "{error:?}"
    );
}

#[test]
fn two_manifests_in_one_directory_are_two_rather_than_a_choice() {
    let tree = Tree::new();
    tree.directory("clients/both/generated")
        .write(
            "clients/both/package.json",
            r#"{ "name": "a", "version": "1.0.0" }"#,
        )
        .write(
            "clients/both/Cargo.toml",
            "[package]\nname = \"a\"\nversion = \"1.0.0\"\n",
        );

    let error = refusal(&tree, &[target("both", "clients/both/generated")]);
    let said = error.to_string();

    assert!(
        said.contains("Cargo.toml") && said.contains("package.json"),
        "{said}"
    );
    assert!(
        matches!(error, Error::AmbiguousManifest { .. }),
        "{error:?}"
    );
}

#[test]
fn two_packages_published_as_the_same_thing_are_refused() {
    let tree = Tree::new();
    tree.directory("clients/one/generated")
        .write(
            "clients/one/package.json",
            r#"{ "name": "same", "version": "1.0.0" }"#,
        )
        .directory("clients/two/generated")
        .write(
            "clients/two/package.json",
            r#"{ "name": "same", "version": "2.0.0" }"#,
        );

    let error = refusal(
        &tree,
        &[
            target("one", "clients/one/generated"),
            target("two", "clients/two/generated"),
        ],
    );

    assert!(matches!(error, Error::DuplicatePackage { .. }), "{error:?}");
}

#[test]
fn the_same_name_on_two_registries_is_two_packages() {
    let tree = Tree::new();
    tree.directory("clients/one/generated")
        .write(
            "clients/one/package.json",
            r#"{ "name": "hook0-client", "version": "1.0.0" }"#,
        )
        .directory("clients/two/generated")
        .write(
            "clients/two/Cargo.toml",
            "[package]\nname = \"hook0-client\"\nversion = \"1.0.0\"\n",
        );

    let packages = discover(
        &[
            target("one", "clients/one/generated"),
            target("two", "clients/two/generated"),
        ],
        tree.path(),
    )
    .expect("two packages on two registries");

    assert_eq!(packages.len(), 2);
}

// --- The omission in the other direction --------------------------------------------------------

#[test]
fn a_package_no_target_claims_is_refused_rather_than_passed_over() {
    let tree = Tree::new();
    tree.directory("clients/claimed/generated")
        .write(
            "clients/claimed/package.json",
            r#"{ "name": "claimed", "version": "1.0.0" }"#,
        )
        .write(
            "clients/forgotten/composer.json",
            r#"{ "name": "hook0/forgotten" }"#,
        );

    let error = refusal(&tree, &[target("claimed", "clients/claimed/generated")]);
    let said = error.to_string();

    assert!(said.contains("clients/forgotten"), "{said}");
    assert!(matches!(error, Error::UnclaimedPackage { .. }), "{error:?}");
}

#[test]
fn a_manifest_that_says_it_is_not_for_a_registry_is_not_a_package() {
    let tree = Tree::new();
    tree.directory("clients/claimed/generated")
        .write(
            "clients/claimed/package.json",
            r#"{ "name": "claimed", "version": "1.0.0" }"#,
        )
        .write(
            "clients/toolchain/Cargo.toml",
            "[package]\nname = \"toolchain\"\nversion = \"0.1.0\"\npublish = false\n",
        )
        .write(
            "clients/scripts/package.json",
            r#"{ "name": "scripts", "version": "0.1.0", "private": true }"#,
        );

    let packages = discover(
        &[target("claimed", "clients/claimed/generated")],
        tree.path(),
    )
    .expect("only the claimed package");

    assert_eq!(packages.len(), 1);
}

#[test]
fn a_directory_carrying_nothing_is_not_a_package() {
    let tree = Tree::new();
    tree.directory("clients/claimed/generated")
        .write(
            "clients/claimed/package.json",
            r#"{ "name": "claimed", "version": "1.0.0" }"#,
        )
        .write("clients/conformance/answers.json", "[]");

    let packages = discover(
        &[target("claimed", "clients/claimed/generated")],
        tree.path(),
    )
    .expect("only the claimed package");

    assert_eq!(packages.len(), 1);
}

// --- Whether anything publishes it ----------------------------------------------------------

/// A tree holding one npm package and one release flow, for the question of whether anything
/// publishes it. What the flow says is what each test says it says, since a publisher is read off
/// what a job declares rather than off a list of jobs kept somewhere.
fn one_publishable(flow: &str) -> (Tree, Vec<TargetRoot>) {
    let tree = Tree::new();
    tree.directory("clients/sole/generated")
        .write(
            "clients/sole/package.json",
            r#"{ "name": "hook0-client", "version": "1.0.0" }"#,
        )
        .write("ci/release-sdk.gitlab-ci.yml", flow);
    (tree, vec![target("sole", "clients/sole/generated")])
}

fn publishers(tree: &Tree, targets: &[TargetRoot]) -> Result<(), Error> {
    let packages = discover(targets, tree.path()).expect("the packages");
    check_publishers(&packages, tree.path())
}

fn unpublished(tree: &Tree, targets: &[TargetRoot]) -> Error {
    match publishers(tree, targets) {
        Ok(()) => panic!("expected a refusal, got a release nothing publishes"),
        Err(error) => error,
    }
}

/// A job that publishes nothing, which is every job in the pipeline that is not a publisher.
const NO_PUBLISHER: &str = "sdk.packages:\n  script:\n    - cargo run -p release-packages\n";

/// A job that publishes the one package the tree holds, written the way the flows write it.
const PUBLISHER: &str = "sdk-release.npm:\n  variables:\n    PUBLISHES: clients/sole\n  \
                         script:\n    - cd \"$PUBLISHES\"\n    - npm publish --access public\n";

#[test]
fn a_package_a_job_declares_it_publishes_reaches_a_registry() {
    let (tree, targets) = one_publishable(PUBLISHER);

    publishers(&tree, &targets).expect("the package its job publishes");
}

#[test]
fn a_package_no_job_publishes_stops_the_release_naming_it() {
    let (tree, targets) = one_publishable(NO_PUBLISHER);

    let error = unpublished(&tree, &targets);
    let said = error.to_string();

    assert!(said.contains("hook0-client"), "{said}");
    assert!(said.contains("clients/sole"), "{said}");
    assert!(said.contains("npm"), "{said}");
    assert!(said.contains(release_packages::NO_PUBLISH_JOB), "{said}");
    assert!(matches!(error, Error::NoPublisher { .. }), "{error:?}");
}

/// The declaration is read as a declaration rather than as text that happens to be in the file, so
/// a job somebody commented out publishes nothing.
#[test]
fn a_declaration_behind_a_comment_publishes_nothing() {
    let (tree, targets) = one_publishable("# sdk-release.npm:\n#   PUBLISHES: clients/sole\n");

    let error = unpublished(&tree, &targets);

    assert!(matches!(error, Error::NoPublisher { .. }), "{error:?}");
}

#[test]
fn a_recorded_reason_stands_in_for_the_job_that_is_missing() {
    let (tree, targets) = one_publishable(NO_PUBLISHER);
    tree.write(
        release_packages::NO_PUBLISH_JOB,
        "[[package]]\ndirectory = \"clients/sole\"\npublished_as = \"hook0-client\"\n\
         registry = \"npm\"\n\
         reason = \"The registry is waiting on something a pipeline cannot supply.\"\n",
    );

    publishers(&tree, &targets).expect("the package whose absence is recorded");
}

#[test]
fn a_reason_recorded_for_a_package_that_is_not_there_is_refused() {
    let (tree, targets) = one_publishable(PUBLISHER);
    tree.write(
        release_packages::NO_PUBLISH_JOB,
        "[[package]]\ndirectory = \"clients/nowhere\"\npublished_as = \"hook0-client\"\n\
         registry = \"npm\"\n\
         reason = \"A reason about a package this repository does not have.\"\n",
    );

    let error = unpublished(&tree, &targets);
    let said = error.to_string();

    assert!(said.contains("clients/nowhere"), "{said}");
    assert!(said.contains(release_packages::NO_PUBLISH_JOB), "{said}");
    assert!(
        matches!(error, Error::UnknownPackageNamed { .. }),
        "{error:?}"
    );
}

#[test]
fn a_job_publishing_something_the_inventory_does_not_know_is_refused() {
    let (tree, targets) = one_publishable(
        "sdk-release.npm:\n  variables:\n    PUBLISHES: clients/ghost\n  script:\n    - cd .\n",
    );

    let error = unpublished(&tree, &targets);
    let said = error.to_string();

    assert!(said.contains("clients/ghost"), "{said}");
    assert!(said.contains("ci/release-sdk.gitlab-ci.yml"), "{said}");
    assert!(
        matches!(error, Error::UnknownPackageNamed { .. }),
        "{error:?}"
    );
}

#[test]
fn a_reason_for_a_package_a_job_does_publish_is_refused() {
    let (tree, targets) = one_publishable(PUBLISHER);
    tree.write(
        release_packages::NO_PUBLISH_JOB,
        "[[package]]\ndirectory = \"clients/sole\"\npublished_as = \"hook0-client\"\n\
         registry = \"npm\"\n\
         reason = \"Written when there was no job, and left behind when there was.\"\n",
    );

    let error = unpublished(&tree, &targets);
    let said = error.to_string();

    assert!(said.contains("ci/release-sdk.gitlab-ci.yml"), "{said}");
    assert!(
        matches!(error, Error::RecordedYetPublished { .. }),
        "{error:?}"
    );
}

#[test]
fn a_reason_about_a_registry_the_package_no_longer_has_is_refused() {
    let (tree, targets) = one_publishable(NO_PUBLISHER);
    tree.write(
        release_packages::NO_PUBLISH_JOB,
        "[[package]]\ndirectory = \"clients/sole\"\npublished_as = \"hook0-client\"\n\
         registry = \"Packagist\"\n\
         reason = \"True of the ecosystem this package used to be in.\"\n",
    );

    let error = unpublished(&tree, &targets);
    let said = error.to_string();

    assert!(said.contains("Packagist") && said.contains("npm"), "{said}");
    assert!(
        matches!(error, Error::RecordedDisagrees { .. }),
        "{error:?}"
    );
}

/// The name a record states is the name the manifest gives, so prose about a package that has since
/// been renamed goes red rather than on describing something that is no longer there.
#[test]
fn a_reason_about_a_name_the_package_no_longer_publishes_under_is_refused() {
    let (tree, targets) = one_publishable(NO_PUBLISHER);
    tree.write(
        release_packages::NO_PUBLISH_JOB,
        "[[package]]\ndirectory = \"clients/sole\"\npublished_as = \"hook0-sole\"\n\
         registry = \"npm\"\n\
         reason = \"True of the name this package used to go out under.\"\n",
    );

    let error = unpublished(&tree, &targets);
    let said = error.to_string();

    assert!(
        said.contains("hook0-sole") && said.contains("hook0-client"),
        "{said}"
    );
    assert!(
        matches!(error, Error::RecordedDisagrees { .. }),
        "{error:?}"
    );
}

#[test]
fn a_record_naming_a_package_without_saying_why_is_not_a_reason() {
    let (tree, targets) = one_publishable(NO_PUBLISHER);
    tree.write(
        release_packages::NO_PUBLISH_JOB,
        "[[package]]\ndirectory = \"clients/sole\"\npublished_as = \"hook0-client\"\n\
         registry = \"npm\"\nreason = \"\"\n",
    );

    let error = unpublished(&tree, &targets);
    let said = error.to_string();

    assert!(said.contains("reason"), "{said}");
    assert!(matches!(error, Error::MissingField { .. }), "{error:?}");
}

// --- The repository each client is mirrored to ----------------------------------------------

/// Where a client is mirrored is derived from where it sits, so a language nobody wrote this for
/// reaches a repository of its own the day its target lands.
#[test]
fn every_client_on_the_release_is_mirrored_to_a_repository_named_after_it() {
    let tree = Tree::new();
    tree.directory("clients/nobody/generated")
        .write(
            "clients/nobody/package.json",
            r#"{ "name": "@hook0/nobody", "version": "1.0.0" }"#,
        )
        .directory("clients/somebody/generated")
        .write(
            "clients/somebody/Cargo.toml",
            "[package]\nname = \"hook0-somebody\"\nversion = \"1.0.0\"\n",
        );

    let packages = discover(
        &[
            target("nobody", "clients/nobody/generated"),
            target("somebody", "clients/somebody/generated"),
        ],
        tree.path(),
    )
    .expect("both packages");

    let pushed = mirrors(&packages).expect("a mirror each");

    assert_eq!(
        pushed,
        vec![
            Mirror {
                directory: "clients/nobody".to_string(),
                repository: "hook0/hook0-nobody".to_string(),
            },
            Mirror {
                directory: "clients/somebody".to_string(),
                repository: "hook0/hook0-somebody".to_string(),
            },
        ]
    );
    assert_eq!(pushed[0].url_path(), "github.com/hook0/hook0-nobody");
}

/// A package with a flow of its own goes out under a tag this release never pushes, so a `vX.Y.Z`
/// cut from the SDK tag would put a version on its mirror that nothing ever released.
#[test]
fn a_package_on_its_own_release_is_not_mirrored_by_the_sdk_release() {
    let (tree, targets) = two_trains();
    let packages = discover(&targets, tree.path()).expect("both packages");

    let pushed = mirrors(&packages).expect("the one mirror");

    assert_eq!(pushed.len(), 1);
    assert_eq!(pushed[0].directory, "clients/rider");
}

/// A tree holding one Go module, whose path is whatever the test says it is.
fn one_module(module: &str) -> Vec<Package> {
    let tree = Tree::new();
    tree.directory("clients/goey/generated").write(
        "clients/goey/go.mod",
        &format!("module {module}\n\ngo 1.24\n"),
    );
    discover(&[target("goey", "clients/goey/generated")], tree.path()).expect("the module")
}

#[test]
fn a_module_named_after_the_mirror_serving_it_is_what_a_user_can_install() {
    check_mirrors(&one_module("github.com/hook0/hook0-goey")).expect("a path that resolves");
}

/// The failure this catches is the one a user reports rather than a pipeline: a module path and the
/// repository serving it are one fact in two places, and `go get` of a path nothing answers at
/// resolves nothing while every job stays green.
#[test]
fn a_module_path_that_is_not_the_url_of_its_mirror_is_refused() {
    let error = check_mirrors(&one_module("github.com/hook0/hook0/clients/goey"))
        .expect_err("a path pointing somewhere the mirror is not");
    let said = error.to_string();

    assert!(
        said.contains("github.com/hook0/hook0/clients/goey"),
        "{said}"
    );
    assert!(said.contains("github.com/hook0/hook0-goey"), "{said}");
    assert!(said.contains("clients/goey"), "{said}");
    assert!(
        matches!(error, Error::ModulePathNotTheMirror { .. }),
        "{error:?}"
    );
}

#[test]
fn more_mirrors_than_one_run_pushes_are_refused_rather_than_pushed() {
    let tree = Tree::new();
    let names: Vec<String> = (0..=release_packages::MAX_MIRRORS)
        .map(|n| format!("c{n}"))
        .collect();
    for name in &names {
        tree.directory(&format!("clients/{name}/generated")).write(
            &format!("clients/{name}/package.json"),
            &format!(r#"{{ "name": "{name}", "version": "1.0.0" }}"#),
        );
    }
    let targets: Vec<TargetRoot> = names
        .iter()
        .map(|name| target(name, &format!("clients/{name}/generated")))
        .collect();
    let packages = discover(&targets, tree.path()).expect("every package");

    let error = mirrors(&packages).expect_err("a run past the ceiling");

    assert!(matches!(error, Error::TooManyMirrors { .. }), "{error:?}");
}

// --- One case per manifest shape ----------------------------------------------------------------

/// A tree holding one package of one shape, with the target landing two levels inside it.
fn one_package(manifest: &str, body: &str) -> (Tree, Vec<TargetRoot>) {
    let tree = Tree::new();
    tree.directory("clients/sole/deep/generated")
        .write(&format!("clients/sole/{manifest}"), body);
    (tree, vec![target("sole", "clients/sole/deep/generated")])
}

fn read_one(manifest: &str, body: &str) -> Package {
    let (tree, targets) = one_package(manifest, body);
    only(discover(&targets, tree.path()).expect("the package"))
}

#[test]
fn a_cargo_manifest_is_read_off_its_package_table_and_not_off_a_dependency() {
    let package = read_one(
        "Cargo.toml",
        "[package]\nname = \"hook0-client\"\nversion = \"1.2.3\"\n\n\
         [dependencies]\nchrono = { version = \"0.4.45\" }\n",
    );

    assert_eq!(package.name, "hook0-client");
    assert_eq!(package.version.to_string(), "1.2.3");
    assert_eq!(package.registry.id(), "crates.io");
}

#[test]
fn a_pyproject_is_read_off_its_project_table() {
    let package = read_one(
        "pyproject.toml",
        "[build-system]\nrequires = [\"setuptools>=77\"]\n\n\
         [project]\nname = \"hook0-client\"\nversion = \"3.2.1\"\n",
    );

    assert_eq!(package.name, "hook0-client");
    assert_eq!(package.version.to_string(), "3.2.1");
    assert_eq!(package.registry.id(), "PyPI");
}

#[test]
fn a_go_module_is_named_by_its_url_and_versioned_by_nothing_in_the_tree() {
    let package = read_one(
        "go.mod",
        "// A module reached at the address it names.\nmodule github.com/hook0/hook0-sole\n\n\
         go 1.24\n",
    );

    assert_eq!(package.name, "github.com/hook0/hook0-sole");
    assert_eq!(package.version, Version::FromTag);
    assert_eq!(package.registry.id(), "go-proxy");
}

#[test]
fn a_composer_package_is_versioned_by_nothing_in_the_tree() {
    let package = read_one(
        "composer.json",
        r#"{ "name": "hook0/client", "type": "library" }"#,
    );

    assert_eq!(package.name, "hook0/client");
    assert_eq!(package.version, Version::FromTag);
    assert_eq!(package.registry.id(), "Packagist");
}

#[test]
fn a_gemspec_is_followed_to_the_constant_it_names() {
    let tree = Tree::new();
    tree.directory("clients/sole/lib/hook0/generated")
        .write(
            "clients/sole/hook0-client.gemspec",
            "require_relative \"lib/hook0/version\"\n\n\
             Gem::Specification.new do |spec|\n\
             \x20 spec.name = \"hook0-client\"\n\
             \x20 spec.version = Hook0::VERSION\n\
             \x20 spec.required_ruby_version = \">= 3.1\"\n\
             end\n",
        )
        .write(
            "clients/sole/lib/hook0/version.rb",
            "module Hook0\n  VERSION = \"7.8.9\"\nend\n",
        );

    let package = only(
        discover(
            &[target("sole", "clients/sole/lib/hook0/generated")],
            tree.path(),
        )
        .expect("the gem"),
    );

    assert_eq!(package.name, "hook0-client");
    assert_eq!(package.version.to_string(), "7.8.9");
    assert_eq!(package.registry.id(), "RubyGems");
}

#[test]
fn a_csproj_is_read_off_its_properties() {
    let package = read_one(
        "Hook0.Client.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <PropertyGroup>\n\
         \x20   <TargetFramework>net8.0</TargetFramework>\n\
         \x20   <PackageId>Hook0.Client</PackageId>\n\
         \x20   <Version>2.3.4</Version>\n  </PropertyGroup>\n\
         \x20 <ItemGroup>\n    <None Include=\"../README.md\" Pack=\"true\" PackagePath=\"\\\" />\n\
         \x20 </ItemGroup>\n</Project>\n",
    );

    assert_eq!(package.name, "Hook0.Client");
    assert_eq!(package.version.to_string(), "2.3.4");
    assert_eq!(package.registry.id(), "NuGet");
}

#[test]
fn a_pom_is_read_off_the_artefact_it_is_and_not_off_the_ones_it_depends_on() {
    let package = read_one(
        "pom.xml",
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project>\n\
         \x20 <!-- <version>0.0.0</version> is a comment, not a version -->\n\
         \x20 <groupId>com.hook0</groupId>\n  <artifactId>hook0-client</artifactId>\n\
         \x20 <version>5.6.7</version>\n  <dependencies>\n    <dependency>\n\
         \x20     <groupId>org.junit</groupId>\n      <artifactId>junit</artifactId>\n\
         \x20     <version>1.1.1</version>\n    </dependency>\n  </dependencies>\n</project>\n",
    );

    assert_eq!(package.name, "com.hook0:hook0-client");
    assert_eq!(package.version.to_string(), "5.6.7");
    assert_eq!(package.registry.id(), "Maven-Central");
}

#[test]
fn a_gradle_build_takes_the_artefact_name_from_the_settings_beside_it() {
    let tree = Tree::new();
    tree.directory("clients/sole/src/generated")
        .write(
            "clients/sole/build.gradle.kts",
            "group = \"com.hook0\"\nversion = \"1.4.0\"\n",
        )
        .write(
            "clients/sole/settings.gradle.kts",
            "rootProject.name = \"hook0-client\"\n",
        );

    let package = only(
        discover(&[target("sole", "clients/sole/src/generated")], tree.path())
            .expect("the artefact"),
    );

    assert_eq!(package.name, "com.hook0:hook0-client");
    assert_eq!(package.version.to_string(), "1.4.0");
}

#[test]
fn a_gradle_build_without_settings_falls_back_to_the_name_gradle_itself_would_use() {
    let package = read_one(
        "build.gradle.kts",
        "group = \"com.hook0\"\nversion = \"1.4.0\"\n",
    );

    assert_eq!(package.name, "com.hook0:sole");
}

/// LuaRocks writes `<version>-<revision>`, and the revision counts repackagings of the same
/// upstream release rather than saying anything about what the package is a version of.
#[test]
fn a_rockspec_is_read_without_the_packaging_revision_beside_its_version() {
    let package = read_one(
        "hook0-client-1.0.0-1.rockspec",
        "package = \"hook0-client\"\nversion = \"1.0.0-1\"\nsource = { url = \"...\" }\n",
    );

    assert_eq!(package.name, "hook0-client");
    assert_eq!(package.version.to_string(), "1.0.0");
    assert_eq!(package.registry.id(), "LuaRocks");
}

#[test]
fn versioning_a_rockspec_leaves_its_packaging_revision_where_it_was() {
    let (tree, targets) = one_package(
        "hook0-client-1.0.0-1.rockspec",
        "package = \"hook0-client\"\nversion = \"1.0.0-1\"\nsource = { url = \"...\" }\n",
    );
    let packages = discover(&targets, tree.path()).expect("the rock");
    let asked = semver::Version::parse("1.2.0").expect("a version");

    set_version(tree.path(), &packages, &asked).expect("the write");

    let written = tree.read("clients/sole/hook0-client-1.0.0-1.rockspec");
    assert!(written.contains("version = \"1.2.0-1\""), "{written}");
}

#[test]
fn a_zon_manifest_is_read_whether_its_name_is_quoted_or_bare() {
    let bare = read_one(
        "build.zig.zon",
        ".{\n    .name = .hook0_client,\n    .version = \"0.3.1\",\n}\n",
    );
    assert_eq!(bare.name, "hook0_client");
    assert_eq!(bare.version.to_string(), "0.3.1");

    let quoted = read_one(
        "build.zig.zon",
        ".{\n    .name = \"hook0-client\",\n    .version = \"0.3.1\",\n}\n",
    );
    assert_eq!(quoted.name, "hook0-client");
}

#[test]
fn a_version_that_is_not_one_is_refused_naming_what_was_read() {
    let (tree, targets) = one_package(
        "Cargo.toml",
        "[package]\nname = \"hook0-client\"\nversion = \"the latest\"\n",
    );

    let error = refusal(&tree, &targets);

    assert!(error.to_string().contains("the latest"), "{error}");
    assert!(matches!(error, Error::NotAVersion { .. }), "{error:?}");
}

// --- Setting a version --------------------------------------------------------------------------

#[test]
fn every_declared_version_is_written_and_nothing_else_is() {
    let tree = Tree::new();
    tree.directory("clients/rusty/src/generated")
        .write(
            "clients/rusty/Cargo.toml",
            "[package]\nname = \"hook0-client\"\nversion = \"1.1.0\"\n\n\
             [dependencies]\nchrono = { version = \"0.4.45\" }\n",
        )
        .directory("clients/goey/generated")
        .write(
            "clients/goey/go.mod",
            "module example.com/goey\n\ngo 1.24\n",
        );

    let targets = [
        target("rusty", "clients/rusty/src/generated"),
        target("goey", "clients/goey/generated"),
    ];
    let packages = discover(&targets, tree.path()).expect("both packages");
    let asked = semver::Version::parse("2.0.0").expect("a version");

    let changes = set_version(tree.path(), &packages, &asked).expect("the writes");

    assert_eq!(
        changes,
        vec![
            (
                "hook0-client".to_string(),
                Change::Written {
                    file: "clients/rusty/Cargo.toml".to_string(),
                    from: "1.1.0".to_string(),
                },
            ),
            ("example.com/goey".to_string(), Change::VersionedByTag),
        ]
    );
    let written = tree.read("clients/rusty/Cargo.toml");
    assert!(written.contains("version = \"2.0.0\""), "{written}");
    assert!(
        written.contains("chrono = { version = \"0.4.45\" }"),
        "the dependency was rewritten too: {written}"
    );
}

#[test]
fn a_gem_is_versioned_where_its_version_actually_lives() {
    let tree = Tree::new();
    tree.directory("clients/gemmy/lib/hook0/generated")
        .write(
            "clients/gemmy/hook0-client.gemspec",
            "require_relative \"lib/hook0/version\"\n\n\
             Gem::Specification.new do |spec|\n\
             \x20 spec.name = \"hook0-client\"\n  spec.version = Hook0::VERSION\nend\n",
        )
        .write(
            "clients/gemmy/lib/hook0/version.rb",
            "module Hook0\n  VERSION = \"1.1.0\"\nend\n",
        );

    let packages = discover(
        &[target("gemmy", "clients/gemmy/lib/hook0/generated")],
        tree.path(),
    )
    .expect("the gem");
    let asked = semver::Version::parse("1.2.0").expect("a version");

    let changes = set_version(tree.path(), &packages, &asked).expect("the write");

    assert_eq!(
        changes,
        vec![(
            "hook0-client".to_string(),
            Change::Written {
                file: "clients/gemmy/lib/hook0/version.rb".to_string(),
                from: "1.1.0".to_string(),
            },
        )]
    );
    assert!(
        tree.read("clients/gemmy/lib/hook0/version.rb")
            .contains("\"1.2.0\"")
    );
    assert!(
        tree.read("clients/gemmy/hook0-client.gemspec")
            .contains("Hook0::VERSION")
    );
}

#[test]
fn setting_the_version_that_is_already_there_writes_nothing() {
    let (tree, targets) = one_package(
        "package.json",
        "{\n  \"name\": \"hook0-client\",\n  \"version\": \"1.1.0\"\n}\n",
    );
    let before = tree.read("clients/sole/package.json");
    let packages = discover(&targets, tree.path()).expect("the package");
    let asked = semver::Version::parse("1.1.0").expect("a version");

    let changes = set_version(tree.path(), &packages, &asked).expect("no write");

    assert!(
        matches!(changes[0].1, Change::Unchanged { .. }),
        "{changes:?}"
    );
    assert_eq!(tree.read("clients/sole/package.json"), before);
}

#[test]
fn setting_a_version_leaves_a_tree_discovery_reads_back_the_same_way() {
    let tree = Tree::new();
    for (directory, manifest, body) in [
        (
            "npmish",
            "package.json",
            "{\n  \"name\": \"a\",\n  \"version\": \"1.0.0\"\n}\n",
        ),
        (
            "cargoish",
            "Cargo.toml",
            "[package]\nname = \"b\"\nversion = \"1.0.0\"\n",
        ),
        (
            "pyish",
            "pyproject.toml",
            "[project]\nname = \"c\"\nversion = \"1.0.0\"\n",
        ),
        (
            "netish",
            "D.csproj",
            "<Project>\n  <PropertyGroup>\n    <PackageId>D</PackageId>\n\
             \x20   <Version>1.0.0</Version>\n  </PropertyGroup>\n</Project>\n",
        ),
        (
            "mavenish",
            "pom.xml",
            "<project>\n  <groupId>e</groupId>\n  <artifactId>f</artifactId>\n\
             \x20 <version>1.0.0</version>\n</project>\n",
        ),
    ] {
        tree.directory(&format!("clients/{directory}/generated"))
            .write(&format!("clients/{directory}/{manifest}"), body);
    }

    let targets: Vec<TargetRoot> = ["npmish", "cargoish", "pyish", "netish", "mavenish"]
        .iter()
        .map(|name| target(name, &format!("clients/{name}/generated")))
        .collect();
    let packages = discover(&targets, tree.path()).expect("five packages");
    let asked = semver::Version::parse("9.9.9").expect("a version");

    set_version(tree.path(), &packages, &asked).expect("five writes");

    let after = discover(&targets, tree.path()).expect("five packages again");
    for package in &after {
        assert_eq!(package.version.to_string(), "9.9.9", "{package:?}");
    }
}

// --- Which release a package rides --------------------------------------------------------------

/// Two packages: one with a release flow of its own beside the others, one without.
fn two_trains() -> (Tree, Vec<TargetRoot>) {
    let tree = Tree::new();
    tree.write(
        "ci/release-loner.gitlab-ci.yml",
        "loner.publish:\n  script: []\n",
    )
    .directory("clients/loner/generated")
    .write(
        "clients/loner/Cargo.toml",
        "[package]\nname = \"hook0-loner\"\nversion = \"1.0.2\"\n",
    )
    .directory("clients/rider/generated")
    .write(
        "clients/rider/Cargo.toml",
        "[package]\nname = \"hook0-rider\"\nversion = \"1.1.0\"\n",
    );
    (
        tree,
        vec![
            target("loner", "clients/loner/generated"),
            target("rider", "clients/rider/generated"),
        ],
    )
}

#[test]
fn a_package_with_a_flow_of_its_own_is_released_under_its_own_tag() {
    let (tree, targets) = two_trains();

    let packages = discover(&targets, tree.path()).expect("both packages");

    let loner = &packages[0];
    let rider = &packages[1];
    assert!(!loner.on_sdk_train(), "{loner:?}");
    assert_eq!(loner.train.tag_prefix(), "loner/v");
    assert!(rider.on_sdk_train(), "{rider:?}");
    assert_eq!(rider.train.tag_prefix(), "sdk-v");
    assert_eq!(sdk_train(&packages).len(), 1);
}

#[test]
fn a_package_on_its_own_release_is_not_bumped_by_the_sdk_release() {
    let (tree, targets) = two_trains();
    let packages = discover(&targets, tree.path()).expect("both packages");
    let asked = semver::Version::parse("1.2.0").expect("a version");

    set_version(tree.path(), &sdk_train(&packages), &asked).expect("the one write");

    assert!(tree.read("clients/rider/Cargo.toml").contains("\"1.2.0\""));
    assert!(
        tree.read("clients/loner/Cargo.toml").contains("\"1.0.2\""),
        "the package on its own release was bumped by the SDK's"
    );
}

#[test]
fn the_version_a_bump_starts_from_is_read_off_the_whole_release() {
    let (tree, targets) = two_trains();
    let packages = discover(&targets, tree.path()).expect("both packages");

    let current = current_version(&sdk_train(&packages)).expect("the one version");

    assert_eq!(current.to_string(), "1.1.0");
}

#[test]
fn two_sdks_at_different_versions_are_a_refusal_rather_than_a_pick() {
    let tree = Tree::new();
    tree.directory("clients/one/generated")
        .write(
            "clients/one/Cargo.toml",
            "[package]\nname = \"one\"\nversion = \"1.1.0\"\n",
        )
        .directory("clients/two/generated")
        .write(
            "clients/two/package.json",
            "{\n  \"name\": \"two\",\n  \"version\": \"1.0.0\"\n}\n",
        );
    let packages = discover(
        &[
            target("one", "clients/one/generated"),
            target("two", "clients/two/generated"),
        ],
        tree.path(),
    )
    .expect("both packages");

    let error = current_version(&packages).expect_err("a disagreement");
    let said = error.to_string();

    assert!(said.contains("1.1.0") && said.contains("1.0.0"), "{said}");
}

// --- Checking a version against a tag -----------------------------------------------------------

#[test]
fn a_package_left_behind_by_a_bump_is_named_rather_than_published() {
    let tree = Tree::new();
    tree.directory("clients/moved/generated")
        .write(
            "clients/moved/package.json",
            "{\n  \"name\": \"moved\",\n  \"version\": \"2.0.0\"\n}\n",
        )
        .directory("clients/behind/generated")
        .write(
            "clients/behind/Cargo.toml",
            "[package]\nname = \"behind\"\nversion = \"1.1.0\"\n",
        );

    let packages = discover(
        &[
            target("moved", "clients/moved/generated"),
            target("behind", "clients/behind/generated"),
        ],
        tree.path(),
    )
    .expect("both packages");
    let tagged = semver::Version::parse("2.0.0").expect("a version");

    let error = check_version(&packages, &tagged).expect_err("the one left behind");
    let said = error.to_string();

    assert!(said.contains("behind"), "{said}");
    assert!(said.contains("1.1.0") && said.contains("2.0.0"), "{said}");
}

#[test]
fn a_package_the_host_versions_by_its_tag_has_nothing_to_check() {
    let tree = Tree::new();
    tree.directory("clients/goey/generated").write(
        "clients/goey/go.mod",
        "module example.com/goey\n\ngo 1.24\n",
    );

    let packages =
        discover(&[target("goey", "clients/goey/generated")], tree.path()).expect("the module");
    let tagged = semver::Version::parse("2.0.0").expect("a version");

    check_version(&packages, &tagged).expect("nothing to disagree with");
}

// --- Bounds and legibility ----------------------------------------------------------------------

#[test]
fn more_targets_than_the_ceiling_are_refused_before_a_single_directory_is_read() {
    let tree = Tree::new();
    let targets: Vec<TargetRoot> = (0..=release_packages::MAX_TARGETS)
        .map(|n| target(&format!("t{n}"), &format!("clients/t{n}/generated")))
        .collect();

    let error = refusal(&tree, &targets);

    assert!(matches!(error, Error::TooManyTargets { .. }), "{error:?}");
}

/// Fill a directory past the ceiling, so that reading all of it is the thing that cannot be done.
fn crowd(tree: &Tree, relative: &str) {
    let directory = tree.path().join(relative);
    fs::create_dir_all(&directory).expect("the directory");
    for entry in 0..=release_packages::MAX_DIR_ENTRIES {
        fs::write(directory.join(format!("entry-{entry}")), "").expect("the entry");
    }
}

/// Reading part of a directory would leave whatever sat past the last entry out of the inventory
/// without a word, which is the omission the whole tool is against.
#[test]
fn a_package_directory_past_the_entry_ceiling_is_refused_rather_than_trimmed() {
    let tree = Tree::new();
    tree.directory("clients/sole/generated").write(
        "clients/sole/package.json",
        r#"{ "name": "hook0-client", "version": "1.0.0" }"#,
    );
    crowd(&tree, "clients/sole");

    let error = refusal(&tree, &[target("sole", "clients/sole/generated")]);
    let said = error.to_string();

    assert!(said.contains("clients/sole"), "{said}");
    assert!(matches!(error, Error::TooManyEntries { .. }), "{error:?}");
}

#[test]
fn the_clients_directory_past_the_entry_ceiling_is_refused_rather_than_trimmed() {
    let tree = Tree::new();
    tree.directory("clients/sole/generated").write(
        "clients/sole/package.json",
        r#"{ "name": "hook0-client", "version": "1.0.0" }"#,
    );
    crowd(&tree, "clients");

    let error = refusal(&tree, &[target("sole", "clients/sole/generated")]);
    let said = error.to_string();

    assert!(said.contains("clients"), "{said}");
    assert!(matches!(error, Error::TooManyEntries { .. }), "{error:?}");
}

#[test]
fn a_manifest_past_the_byte_ceiling_is_refused_rather_than_read() {
    let (tree, targets) = one_package(
        "package.json",
        &format!(
            "{{ \"name\": \"big\", \"version\": \"1.0.0\", \"pad\": \"{}\" }}",
            "p".repeat(release_packages::manifest::MAX_FILE_BYTES as usize)
        ),
    );

    let error = refusal(&tree, &targets);

    assert!(matches!(error, Error::FileTooLarge { .. }), "{error:?}");
}

#[test]
fn what_is_printed_names_every_package_and_where_it_goes() {
    let tree = Tree::new();
    tree.directory("clients/one/generated")
        .write(
            "clients/one/package.json",
            r#"{ "name": "hook0-client", "version": "1.1.0" }"#,
        )
        .directory("clients/two/generated")
        .write("clients/two/composer.json", r#"{ "name": "hook0/client" }"#);

    let packages = discover(
        &[
            target("one", "clients/one/generated"),
            target("two", "clients/two/generated"),
        ],
        tree.path(),
    )
    .expect("two packages");

    let printed = render(&packages);

    assert!(printed.contains("2 packages"), "{printed}");
    for expected in [
        "one",
        "clients/one",
        "package.json",
        "npm",
        "hook0-client",
        "1.1.0",
        "two",
        "Packagist",
        "hook0/client",
        "(from tag)",
    ] {
        assert!(
            printed.contains(expected),
            "{expected} missing from:\n{printed}"
        );
    }
}

/// The paths this reports are what a `.gitlab-ci.yml` and a `git-cliff --include-path` are handed,
/// so they are written the one way both read rather than the platform's way.
#[test]
fn paths_are_reported_with_forward_slashes() {
    let package = read_one(
        "Cargo.toml",
        "[package]\nname = \"a\"\nversion = \"1.0.0\"\n",
    );

    assert_eq!(package.manifest, "clients/sole/Cargo.toml");
    assert!(!package.manifest.contains(std::path::MAIN_SEPARATOR) || cfg!(unix));
    assert_eq!(PathBuf::from(&package.manifest).components().count(), 3);
}

// --- The release this repository actually ships -------------------------------------------------

/// The most manifests read out of the tree in one walk. This repository holds a few dozen; past
/// this the bound is raised deliberately rather than crossed quietly.
const MAX_MANIFESTS: usize = 512;

/// How deep the walk goes. Every manifest here sits within a handful of directories of the root.
const MAX_DEPTH: usize = 8;

/// Directories a manifest of ours is never in: build output and dependency caches, which hold
/// thousands of manifests belonging to somebody else.
const NOT_OURS: [&str; 4] = [".git", "target", "node_modules", "vendor"];

/// The tables a dependency can be declared in, including the ones a workspace declares once.
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// The tree this crate lives in.
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

/// A path under the tree, written the one way every file that quotes one writes it.
fn shown(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Every table a dependency can be declared in: the package's own, the ones a workspace declares
/// once for the crates under it, and the ones a platform narrows.
fn dependency_tables(document: &toml::Table) -> Vec<&toml::Table> {
    let mut holders = vec![document];
    holders.extend(document.get("workspace").and_then(toml::Value::as_table));
    holders.extend(
        document
            .get("target")
            .and_then(toml::Value::as_table)
            .into_iter()
            .flat_map(|targets| targets.values().filter_map(toml::Value::as_table)),
    );

    let mut tables = Vec::new();
    for holder in holders {
        for name in DEPENDENCY_TABLES {
            tables.extend(holder.get(name).and_then(toml::Value::as_table));
        }
    }
    tables
}

/// One dependency asking for a released client by a version number.
struct Pin {
    manifest: String,
    dependency: String,
    pinned: String,
}

/// Every Cargo manifest in the tree, symlinks not followed.
fn manifests(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![(root.to_path_buf(), 0usize)];

    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_DEPTH {
            continue;
        }
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                if !NOT_OURS.contains(&name) {
                    pending.push((entry.path(), depth + 1));
                }
            } else if kind.is_file() && name == "Cargo.toml" {
                found.push(entry.path());
            }
        }
    }

    assert!(
        found.len() < MAX_MANIFESTS,
        "{} manifests is at the ceiling of {MAX_MANIFESTS}, so the walk may have stopped short of \
         one that pins a client",
        found.len()
    );
    found.sort();
    found
}

/// Every dependency in the tree that names a released client by path and asks for it by version.
///
/// The release rewrites those pins by substituting the version it is bumping from, so a pin that
/// has drifted off the release is not rewritten and nothing says so: the workspace goes on asking
/// for a version that was never published. The walk finds them rather than a list naming them,
/// since a pin added to a new crate tomorrow has the same problem.
fn pins_on(root: &Path, train: &[Package]) -> Vec<Pin> {
    let homes: Vec<PathBuf> = train
        .iter()
        .filter_map(|package| root.join(&package.directory).canonicalize().ok())
        .collect();
    let mut pins = Vec::new();

    for manifest in manifests(root) {
        let Ok(body) = fs::read_to_string(&manifest) else {
            continue;
        };
        let Ok(document) = body.parse::<toml::Table>() else {
            continue;
        };
        let Some(directory) = manifest.parent() else {
            continue;
        };

        for table in dependency_tables(&document) {
            for (name, declared) in table {
                let Some(declared) = declared.as_table() else {
                    continue;
                };
                let (Some(path), Some(version)) = (
                    declared.get("path").and_then(toml::Value::as_str),
                    declared.get("version").and_then(toml::Value::as_str),
                ) else {
                    continue;
                };
                let Ok(asked) = directory.join(path).canonicalize() else {
                    continue;
                };
                if homes.contains(&asked) {
                    pins.push(Pin {
                        manifest: shown(root, &manifest),
                        dependency: name.clone(),
                        pinned: version.to_owned(),
                    });
                }
            }
        }
    }

    pins
}

/// What `ci/pre-release-sdk.sh` takes for granted before it writes a single file: one version
/// across the whole release, and every pin on a released client sitting at it.
///
/// Both only fail today at release time, on master, after the script has already refused or — worse
/// for the pin — after it has quietly rewritten nothing.
#[test]
fn the_release_this_tree_ships_agrees_on_one_version_and_every_pin_sits_at_it() {
    let root = repository();
    let packages = discover(&registry(), &root).expect("the packages the registry resolves to");
    let train = sdk_train(&packages);

    let version = current_version(&train).unwrap_or_else(|refusal| panic!("{refusal}"));

    let pins = pins_on(&root, &train);
    assert!(
        !pins.is_empty(),
        "nothing in the tree pins a released client by version, so this proves nothing"
    );
    for pin in &pins {
        assert_eq!(
            pin.pinned,
            version.to_string(),
            "{} asks for {} at {} while the release is at {version}; the release rewrites pins by \
             substituting the version it bumps from, so this one is left behind in silence",
            pin.manifest,
            pin.dependency,
            pin.pinned
        );
    }
}

/// Whether cargo would let this manifest be published, which is `[package] publish` and nothing
/// else. The key of the same name under `[package.metadata.release]` is read by cargo-release
/// alone, and the two spellings sitting in one tree is how a crate comes to look settled either way
/// while only one of them decides.
fn cargo_would_publish(document: &toml::Table) -> bool {
    let Some(package) = document.get("package").and_then(toml::Value::as_table) else {
        return false;
    };
    match package.get("publish") {
        Some(toml::Value::Boolean(allowed)) => *allowed,
        Some(toml::Value::Array(registries)) => !registries.is_empty(),
        _ => true,
    }
}

/// A crate left publishable has to be one cargo can actually package, and a dependency asked for by
/// path alone is one it cannot.
///
/// Cargo refuses the whole crate over it, since the path is dropped on upload and what remains asks
/// for nothing in particular. It refuses at `cargo publish` and nowhere earlier, so a crate can sit
/// unpublishable for as long as nobody tries — which is what three of them had done here.
#[test]
fn a_crate_left_publishable_asks_for_every_path_dependency_by_version() {
    let root = repository();
    let mut refused = Vec::new();
    let mut publishable = 0usize;

    for manifest in manifests(&root) {
        let Ok(body) = fs::read_to_string(&manifest) else {
            continue;
        };
        let Ok(document) = body.parse::<toml::Table>() else {
            continue;
        };
        if !cargo_would_publish(&document) {
            continue;
        }
        publishable += 1;

        for table in dependency_tables(&document) {
            for (name, declared) in table {
                let Some(declared) = declared.as_table() else {
                    continue;
                };
                if declared.contains_key("path") && !declared.contains_key("version") {
                    refused.push(format!(
                        "{} asks for {name} by path alone",
                        shown(&root, &manifest)
                    ));
                }
            }
        }
    }

    assert!(
        publishable > 0,
        "no manifest in the tree is publishable, so this proves nothing"
    );
    assert!(
        refused.is_empty(),
        "cargo will not package these until the dependency is asked for by version too:\n{}",
        refused.join("\n")
    );
}

/// The version a semver-looking run of characters spells, if it spells one.
fn semver_at(text: &str, from: usize) -> Option<String> {
    let rest = &text[from..];
    let end = rest
        .find(|character: char| !character.is_ascii_digit() && character != '.')
        .unwrap_or(rest.len());
    // A version at the end of a sentence, or before a file extension, is followed by the dot that
    // ends it: `/v1.1.0.tar.gz` reads one character too far without this, and answers nothing.
    let candidate = rest[..end].trim_end_matches('.');
    let parts: Vec<&str> = candidate.split('.').collect();
    let spelled = parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
    spelled.then(|| candidate.to_owned())
}

/// Every version this text names right after one of these markers.
fn versions_after(text: &str, markers: &[String]) -> Vec<String> {
    let mut found = Vec::new();
    for marker in markers {
        let mut at = 0;
        while let Some(offset) = text[at..].find(marker.as_str()) {
            let after = at + offset + marker.len();
            if let Some(version) = semver_at(text, after) {
                found.push(version);
            }
            at = after;
        }
    }
    found
}

/// A README naming a version of its own package names the one the release is at.
///
/// The four SDKs no registry resolves tell a reader to build from a checkout or to fetch a tag, so
/// they spell a version out in prose. `set-version` writes manifests and a README is not one, which
/// is how those four came to advertise a release that was already two behind. `pre-release-sdk.sh`
/// rewrites them now, in the shapes below, and this is what says so.
#[test]
fn a_readme_naming_a_version_of_its_own_package_names_the_one_the_release_is_at() {
    let root = repository();
    let packages = discover(&registry(), &root).expect("the packages the registry resolves to");
    let train = sdk_train(&packages);
    let version = current_version(&train).unwrap_or_else(|refusal| panic!("{refusal}"));

    let mut stale = Vec::new();
    let mut named = 0usize;

    for package in &train {
        let readme = root.join(&package.directory).join("README.md");
        let Ok(body) = fs::read_to_string(&readme) else {
            continue;
        };
        // How a version of this package is written where a reader is told to fetch or build it:
        // an XML element, after the coordinates, after the name, or as the tag of an archive.
        let markers = [
            "<version>".to_owned(),
            format!("{}:", package.name),
            format!("{}-", package.name),
            "/v".to_owned(),
        ];
        for found in versions_after(&body, &markers) {
            named += 1;
            if found != version.to_string() {
                stale.push(format!(
                    "{}/README.md tells a reader to use {found} while the release is at {version}",
                    package.directory
                ));
            }
        }
    }

    assert!(
        named > 0,
        "no README names a version of its own package, so this proves nothing"
    );
    assert!(stale.is_empty(), "{}", stale.join("\n"));
}

/// The most bytes read out of one source file while looking for a version constant.
const MAX_SOURCE_BYTES: u64 = 512 * 1024;

/// The most bytes `git ls-files` may answer with for one client. Far above what any of them holds;
/// a client above it is one this guard reports rather than one it half-reads, since a truncated
/// answer here would read as a clean bill of health.
const MAX_GIT_BYTES: usize = 8 * 1024 * 1024;

/// Every file the repository tracks under a client, which is the set a release actually carries.
///
/// Asked of git rather than walked. A walk has to name what to skip, and the names that would need
/// naming are exactly the ones an ignore rule already carries: `clients/java` holds 1778 files and
/// tracks 167 of them, so a walk bounded at any sane ceiling spends it on Gradle's cache and
/// answers "nothing declares a version" for the file that does.
///
/// Markdown is left out because the README guard beside this one reads it, and a lock file because
/// the versions in one are other people's.
fn tracked_under(root: &Path, directory: &str) -> Vec<PathBuf> {
    let answer = Command::new("git")
        .args(["ls-files", "-z", "--", directory])
        .current_dir(root)
        .output()
        .unwrap_or_else(|failure| panic!("could not run `git ls-files` in {directory}: {failure}"));

    assert!(
        answer.status.success(),
        "`git ls-files` failed under {directory}: {}",
        String::from_utf8_lossy(&answer.stderr)
    );
    assert!(
        answer.stdout.len() <= MAX_GIT_BYTES,
        "`git ls-files` answered more than {MAX_GIT_BYTES} bytes for {directory}"
    );

    String::from_utf8_lossy(&answer.stdout)
        .split('\0')
        .filter(|path| !path.is_empty())
        .filter(|path| !path.ends_with(".md"))
        .filter(|path| {
            !path
                .rsplit('/')
                .next()
                .unwrap_or(path)
                .to_ascii_lowercase()
                .contains("lock")
        })
        .map(|path| root.join(path))
        .collect()
}

/// The version a line declares, when the line declares one at all.
///
/// A line qualifies when what sits immediately left of its first `=` is an identifier that reads as
/// `version` once case and underscores are set aside, which is what `VERSION`, `__version__` and
/// `Hook0.VERSION` all have in common. Anything else carrying a version-shaped literal, a pinned
/// dependency above all, is left alone because its version is somebody else's.
fn version_declared_on(line: &str) -> Option<String> {
    let (left, right) = line.split_once('=')?;
    // Whatever sits between the name and the `=` is spacing, and dropping it is what lets the
    // name be read by walking back from the `=`.
    let left = left.trim_end();
    // `==` is a comparison rather than a declaration, and `!=`, `<=`, `>=` likewise.
    if right.starts_with('=') || left.ends_with(['!', '<', '>', '=']) {
        return None;
    }

    let name: String = left
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    let name: String = name.chars().rev().filter(|c| *c != '_').collect();
    if !name.eq_ignore_ascii_case("version") {
        return None;
    }

    let opening = right.find(['"', '\''])?;
    let quote = right.as_bytes()[opening] as char;
    let rest = &right[opening + 1..];
    let literal = &rest[..rest.find(quote)?];

    let parts: Vec<&str> = literal.split('.').collect();
    let numeric = parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
    numeric.then(|| literal.to_owned())
}

/// A client that tells the API which version it is tells it the one the release is at.
///
/// `set-version` writes manifests. Five clients also carry the version as a constant in their own
/// source, because that is what they put in the `User-Agent`, and a manifest is not a source file.
/// Bumping the train therefore left them introducing themselves as the release before, which the
/// conformance corpus caught on `sdk-v2.0.0` only after the tag had been pushed: the package built
/// as `2.0.0` and said `hook0-client-typescript/1.1.0` on the wire.
///
/// Read from the tree rather than from a list of the five, so a sixth client written tomorrow is
/// held to the same thing without this test changing.
#[test]
fn a_client_naming_its_own_version_in_its_source_names_the_one_the_release_is_at() {
    let root = repository();
    let packages = discover(&registry(), &root).expect("the packages the registry resolves to");
    let train = sdk_train(&packages);
    let version = current_version(&train).unwrap_or_else(|refusal| panic!("{refusal}"));

    let mut stale = Vec::new();
    let mut declared = 0usize;

    for package in &train {
        for path in tracked_under(&root, &package.directory) {
            let readable = fs::metadata(&path).is_ok_and(|it| it.len() <= MAX_SOURCE_BYTES);
            if !readable {
                continue;
            }
            let Ok(body) = fs::read_to_string(&path) else {
                continue;
            };
            for (number, line) in body.lines().enumerate() {
                let Some(found) = version_declared_on(line) else {
                    continue;
                };
                declared += 1;
                if found != version.to_string() {
                    stale.push(format!(
                        "{}:{} declares version {found} while the release is at {version}, so this \
                         client introduces itself as a release it is not",
                        shown(&root, &path),
                        number + 1
                    ));
                }
            }
        }
    }

    assert!(
        declared > 0,
        "no client declares a version in its source, so this proves nothing"
    );
    assert!(stale.is_empty(), "{}", stale.join("\n"));
}
