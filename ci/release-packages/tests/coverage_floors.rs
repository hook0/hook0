//! Every client is measured, and the measurement is held to a number.
//!
//! Ten of the twelve clients carry a coverage floor. Two did not, and nothing said so. The MCP
//! crate ran `cargo test` over 2599 lines published to crates.io with no instrument at all, and
//! the Zig package measured nothing outside the exhaustiveness walk over its generated half. Both
//! were found by reading the pipeline files one at a time, which is not a thing anybody does twice.
//!
//! So this reads them instead. What a client is, and where its floor might be written, are both
//! found on disk. What a floor looks like is not findable, and is the one list written down here:
//! an ecosystem states a floor in its own words, and there is no place to derive `fail_under` from
//! `--fail-under-lines` from `<minimum>`. A client whose floor is stated in some thirteenth way
//! fails this until the way it states it is added below, which is the safe direction, since the
//! answer to a false alarm is one line and the answer to a silence is a client nobody measures.

use std::fs;
use std::path::{Path, PathBuf};

/// The most files read under one client.
const MAX_FILES: usize = 4096;

/// The most bytes read out of one file.
const MAX_BYTES: u64 = 1 << 20;

/// How far below a client the search goes.
const MAX_DEPTH: usize = 8;

/// What a build leaves behind. None of it is committed, and reading it would answer with whatever
/// the last local run happened to write.
const DERIVED: [&str; 10] = [
    ".git",
    ".venv",
    ".zig-cache",
    "__pycache__",
    "bin",
    "node_modules",
    "obj",
    "target",
    "vendor",
    "zig-out",
];

/// How an ecosystem says "and not below this".
///
/// `number` marks the tokens whose floor is written beside them, which is most of them. Requiring
/// the digit is what keeps prose about a floor from reading as a floor. `coverageThreshold` is the
/// exception, being a key whose numbers sit in the block under it.
struct Marker {
    token: &'static str,
    number: bool,
}

const MARKERS: [Marker; 6] = [
    // `cargo llvm-cov`, used by the Rust client and the MCP crate.
    Marker {
        token: "--fail-under-lines",
        number: true,
    },
    // The awk this repository writes when an ecosystem ships no threshold of its own: Go, C#, Lua
    // and PHP each hand their floor in as `-v <something>floor=`.
    Marker {
        token: "floor=",
        number: true,
    },
    // `coverage.py`, through `pyproject.toml`.
    Marker {
        token: "fail_under",
        number: true,
    },
    // SimpleCov, through the suite's own helper.
    Marker {
        token: "minimum_coverage",
        number: true,
    },
    // JaCoCo, through `pom.xml`, for Java and Kotlin.
    Marker {
        token: "<minimum>",
        number: true,
    },
    // Jest, whose thresholds are a block rather than a value.
    Marker {
        token: "coverageThreshold",
        number: false,
    },
];

/// The client that states no floor, and why it is allowed not to.
///
/// Zig has no coverage instrument this pipeline can run. kcov is the only one that reads its debug
/// information, and four things were measured in `debian:trixie-slim`, the image the Zig job uses:
/// the package exists in Debian, so installing it is possible; without `SYS_PTRACE` and a relaxed
/// seccomp profile it stops at `Can't set personality: Operation not permitted` and still writes a
/// report saying `covered_lines: 0`, which is a floor of zero dressed as a measurement; permitted,
/// it does read Zig's line table, mapping 214 files and 3228 lines; and the suite driven under it
/// reaches 46 of those 3228, because a Zig test binary run by kcov rather than by its own build
/// runner barely executes. The runner this pipeline uses is a Kubernetes executor, whose default
/// seccomp profile blocks the `personality` call kcov needs.
///
/// So the entry below is a decision rather than an oversight, and `clients/zig/.gitlab-ci.yml`
/// carries the same reasoning next to the exhaustiveness walk that stands in for a percentage over
/// the generated half.
///
/// It cleans itself up: a client named here that does state a floor fails this test too, so the
/// day Zig can be measured the list stops being right rather than stops being read.
const MEASURED_ANOTHER_WAY: [&str; 1] = ["zig"];

/// The repository this crate sits in.
fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("two directories above this crate")
        .to_path_buf()
}

/// A client is a directory under `clients/` that carries a pipeline of its own.
///
/// Two directories beside them are not clients and are told apart that way. The shared conformance
/// corpus is data, and the generator's suite runs inside the Rust client's job, measured with it.
fn clients(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let entries = fs::read_dir(root.join("clients")).expect("the repository carries clients");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join(".gitlab-ci.yml").is_file() {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// Whether a number follows the token closely enough to be its value.
///
/// On the same line, since every one of these is written on one, and within a short reach, so that
/// a token and an unrelated number further down the file are not read as a pair. A word may stand
/// between the two: SimpleCov writes `minimum_coverage line: 99.64`, where the counter is named
/// before the figure it is about.
fn followed_by_a_number(rest: &str) -> bool {
    rest.chars()
        .take(24)
        .take_while(|character| *character != '\n')
        .any(|character| character.is_ascii_digit())
}

/// Whether the text states a floor.
fn states_a_floor(text: &str) -> bool {
    MARKERS.iter().any(|marker| {
        text.match_indices(marker.token)
            .any(|(at, _)| !marker.number || followed_by_a_number(&text[at + marker.token.len()..]))
    })
}

/// Every file under one client, bounded, with what a build left behind left out.
fn files_under(client: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![(client.to_path_buf(), 0usize)];

    while let Some((directory, depth)) = pending.pop() {
        if depth > MAX_DEPTH || found.len() >= MAX_FILES {
            continue;
        }
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name().to_string_lossy().into_owned();
            if kind.is_dir() {
                if !DERIVED.contains(&name.as_str()) {
                    pending.push((entry.path(), depth + 1));
                }
            } else if kind.is_file() && found.len() < MAX_FILES {
                found.push(entry.path());
            }
        }
    }

    found
}

/// Whether any file under this client states a floor.
fn is_measured(client: &Path) -> bool {
    files_under(client).into_iter().any(|path| {
        fs::metadata(&path).is_ok_and(|it| it.len() <= MAX_BYTES)
            && fs::read_to_string(&path).is_ok_and(|text| states_a_floor(&text))
    })
}

#[test]
fn every_client_holds_its_suite_to_a_coverage_floor() {
    let root = repository();
    let clients = clients(&root);

    assert!(
        clients.len() >= 12,
        "only {} clients were found under `clients/`, so this guard is looking in the wrong place",
        clients.len()
    );

    let named = |client: &Path| {
        format!(
            "  {}",
            client.strip_prefix(&root).unwrap_or(client).display()
        )
    };
    let excused = |client: &Path| {
        client
            .file_name()
            .is_some_and(|name| MEASURED_ANOTHER_WAY.iter().any(|it| name == *it))
    };

    let unmeasured: Vec<String> = clients
        .iter()
        .filter(|client| !excused(client) && !is_measured(client))
        .map(|client| named(client))
        .collect();

    assert!(
        unmeasured.is_empty(),
        "these clients run a suite and hold it to nothing, so their coverage can fall to zero \
         without a pipeline noticing:\n{}\n\
         Give each one a floor its own ecosystem enforces, measured from what its suite reaches \
         today. If one already has a floor this did not recognise, add how it is written to \
         `MARKERS` in this file.",
        unmeasured.join("\n")
    );

    let no_longer_excused: Vec<String> = clients
        .iter()
        .filter(|client| excused(client) && is_measured(client))
        .map(|client| named(client))
        .collect();

    assert!(
        no_longer_excused.is_empty(),
        "these clients state a floor and are still written down as unable to:\n{}\n\
         Take them out of `MEASURED_ANOTHER_WAY`, so that what this file says about them is what \
         is true of them.",
        no_longer_excused.join("\n")
    );
}

#[test]
fn a_floor_is_told_apart_from_prose_about_one() {
    // The forms the twelve clients actually use.
    assert!(states_a_floor("cargo llvm-cov --fail-under-lines 98.92"));
    assert!(states_a_floor("awk -v floor=99.2 '"));
    assert!(states_a_floor(
        "awk -v lines_floor=4 -v branches_floor=15 '"
    ));
    assert!(states_a_floor("fail_under = 99.74"));
    assert!(states_a_floor(
        "minimum_coverage line: 99.64, branch: 97.09"
    ));
    assert!(states_a_floor("<minimum>1</minimum>"));
    assert!(states_a_floor("  coverageThreshold: {"));

    // Prose about a floor is not a floor, which is what the digit is for.
    assert!(!states_a_floor(
        "# at or above the floor this suite reaches, which is where to look"
    ));
    assert!(!states_a_floor("# a floor over that rewards not testing"));
    assert!(!states_a_floor("- cargo test --all-features"));
    assert!(!states_a_floor(""));
}
