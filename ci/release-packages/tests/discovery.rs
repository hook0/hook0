//! What discovery answers, asked of real directories.
//!
//! Every tree here is built by the test that reads it, and every target put in front of the tool is
//! one it was never written for — which is the property under test: a client is released because it
//! is in the tree and in the generator's registry, never because somebody remembered to add it to a
//! list. The suite would keep passing if every target this repository ships today were renamed.

use std::fs;
use std::path::{Path, PathBuf};

use release_packages::{
    Change, Error, Package, TargetRoot, Version, check_version, current_version, discover, render,
    sdk_train, set_version,
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
        "// A module in a subdirectory.\nmodule github.com/hook0/hook0/clients/go\n\ngo 1.24\n",
    );

    assert_eq!(package.name, "github.com/hook0/hook0/clients/go");
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

#[test]
fn a_manifest_past_the_byte_ceiling_is_refused_rather_than_read() {
    let (tree, targets) = one_package(
        "package.json",
        &format!(
            "{{ \"name\": \"big\", \"version\": \"1.0.0\", \"pad\": \"{}\" }}",
            "p".repeat(release_packages::manifest::MAX_MANIFEST_BYTES as usize)
        ),
    );

    let error = refusal(&tree, &targets);

    assert!(matches!(error, Error::ManifestTooLarge { .. }), "{error:?}");
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
