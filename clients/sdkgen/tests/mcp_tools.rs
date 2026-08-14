//! Black-box suite over the tool definitions the Hook0 MCP server compiles.
//!
//! What the emitter writes is the shape of the tool surface: one tool per selected operation, the
//! same bytes for the same model, and a tag of its own. That those bytes are what is committed
//! under `clients/mcp` is a different question, and the one `tests/targets.rs` answers for every
//! target at once.

use hook0_sdkgen::{EntityModel, Limits, MCP_TAG, PUBLIC_TAG, Snapshot, mcp};

mod common;

fn model_of(tag: &str) -> EntityModel {
    let limits = Limits::default();
    let snapshot = Snapshot::from_bytes(&common::fixture_bytes(), tag, &limits)
        .expect("the committed snapshot parses");
    EntityModel::from_snapshot(&snapshot, &limits).expect("the committed snapshot yields a model")
}

fn emitted() -> String {
    mcp::tool_definitions(&model_of(MCP_TAG)).expect("the committed snapshot yields tools")
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
