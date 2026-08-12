//! Black-box suite over the tool definitions the Hook0 MCP server compiles.
//!
//! The server is published to crates.io without the OpenAPI snapshot beside it, so its tool table
//! is a committed source file this crate writes. Nothing keeps the two in step on its own, which
//! is what the drift test at the top of this file is for.

use std::fs;
use std::path::Path;

use hook0_sdkgen::{EntityModel, Limits, MCP_TAG, PUBLIC_TAG, Snapshot, mcp};

mod common;

/// The committed file this crate writes and the MCP server compiles.
const GENERATED_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../mcp/src/server/generated.rs"
);

/// Set to any value to rewrite the committed file instead of failing on a difference.
const UPDATE_VAR: &str = "UPDATE_MCP_TOOLS";

/// What to run to adopt a deliberate change of the tool surface.
const UPDATE_COMMAND: &str = "UPDATE_MCP_TOOLS=1 cargo test -p hook0-sdkgen mcp_tool_definitions";

/// Largest committed file read back, in bytes.
const MAX_GENERATED_BYTES: u64 = 4 * 1024 * 1024;

/// How many differing lines a failure report lists before it stops.
const MAX_REPORTED_LINES: usize = 20;

/// How much of a differing line a failure report prints.
const MAX_RENDERED_LINE_CHARS: usize = 120;

fn model_of(tag: &str) -> EntityModel {
    let limits = Limits::default();
    let snapshot = Snapshot::from_bytes(&common::fixture_bytes(), tag, &limits)
        .expect("the committed snapshot parses");
    EntityModel::from_snapshot(&snapshot, &limits).expect("the committed snapshot yields a model")
}

fn emitted() -> String {
    mcp::tool_definitions(&model_of(MCP_TAG)).expect("the committed snapshot yields tools")
}

fn committed() -> String {
    let path = Path::new(GENERATED_PATH);
    let metadata = fs::metadata(path)
        .unwrap_or_else(|err| panic!("{GENERATED_PATH} cannot be looked at: {err}"));
    assert!(
        metadata.len() <= MAX_GENERATED_BYTES,
        "{GENERATED_PATH} is {} bytes long, above the {MAX_GENERATED_BYTES} accepted",
        metadata.len()
    );

    let bytes =
        fs::read(path).unwrap_or_else(|err| panic!("{GENERATED_PATH} cannot be read: {err}"));
    String::from_utf8(bytes).unwrap_or_else(|err| panic!("{GENERATED_PATH} is not UTF-8: {err}"))
}

/// The tools the MCP server exposes are a committed source file, which is what lets the published
/// crate build without the snapshot. A change of the API surface that never reached that file
/// stops here rather than at a release.
#[test]
fn mcp_tool_definitions_match_the_openapi_snapshot() {
    let emitted = emitted();

    if std::env::var_os(UPDATE_VAR).is_some() {
        fs::write(GENERATED_PATH, &emitted)
            .unwrap_or_else(|err| panic!("{GENERATED_PATH} is not writable: {err}"));
        println!("Wrote {GENERATED_PATH}");
        return;
    }

    let committed = committed();
    if committed == emitted {
        return;
    }

    panic!(
        "the tools the MCP server exposes are not the ones the OpenAPI snapshot describes.\n\
         Adopt the change with:\n    {UPDATE_COMMAND}\n\
         and commit {GENERATED_PATH}.\n\
         (`-` committed, `+` what the snapshot dictates)\n{}",
        difference(&committed, &emitted)
    );
}

#[test]
fn emitting_twice_yields_the_same_bytes() {
    assert_eq!(emitted(), emitted(), "two emissions of the snapshot differ");
}

/// The MCP server selects its own tag, not the one the SDKs are built from: operations the API
/// exposes to the server alone would disappear from the tool list if the two were confused.
#[test]
fn the_mcp_tag_selects_operations_the_public_tag_leaves_out() {
    let mcp = common::tool_names(&emitted());
    let public: Vec<String> = ids(&model_of(PUBLIC_TAG));

    let mcp_only: Vec<&String> = mcp.iter().filter(|name| !public.contains(name)).collect();

    assert!(
        !mcp_only.is_empty(),
        "no operation is reserved for the MCP server, so this cannot tell the two tags apart"
    );
    for name in mcp_only {
        assert!(
            mcp.contains(name),
            "`{name}` is marked for the MCP server yet carries no tool"
        );
    }
}

#[test]
fn every_selected_operation_carries_exactly_one_tool() {
    let model = model_of(MCP_TAG);
    let selected = ids(&model);
    let mut emitted = common::tool_names(&emitted());

    assert!(!selected.is_empty(), "the snapshot selects no operation");
    assert_eq!(
        emitted.len(),
        selected.len(),
        "the tool list and the selection do not carry the same number of operations"
    );

    emitted.sort();
    let mut selected = selected;
    selected.sort();
    assert_eq!(emitted, selected, "a selected operation carries no tool");
}

/// The operation ids the model carries, whichever way the convention could place them.
fn ids(model: &EntityModel) -> Vec<String> {
    model
        .entities()
        .iter()
        .flat_map(|entity| entity.methods.iter())
        .map(|method| method.operation_id.clone())
        .chain(
            model
                .unconventional()
                .iter()
                .filter_map(|set_aside| set_aside.operation.operation_id.clone()),
        )
        .collect()
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
            lines.push("  ...".to_owned());
            break;
        }
        reported += 1;

        let number = index + 1;
        if let Some(left) = left {
            lines.push(format!("  {number}- {}", render(left)));
        }
        if let Some(right) = right {
            lines.push(format!("  {number}+ {}", render(right)));
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
