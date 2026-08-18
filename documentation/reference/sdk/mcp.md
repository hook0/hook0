---
title: "Hook0 MCP server: webhook tools for AI assistants"
description: "The hook0-mcp server exposes 23 Hook0 API operations as MCP tools over stdio. Generated from the same OpenAPI snapshot as the SDKs, with a read-only mode and no retrying of its own."
keywords: [Hook0 MCP server, MCP webhook tools, Model Context Protocol Hook0, hook0-mcp crate, AI assistant webhooks, read-only MCP server]
sdkTarget: mcp
---

# MCP server

`hook0-mcp` is the twelfth thing the SDK generator writes, and the only one that is not a library. It is a [Model Context Protocol](https://modelcontextprotocol.io/) server, a process an assistant starts, talks to over stdin and stdout, and calls tools on. Nothing imports it to send an event. An assistant calls `events.ingest` on it.

It is generated from the same `api/openapi.snapshot.json` the eleven clients are generated from, so it moves when the API moves. What it generates is different. The clients get one method per operation, typed; this one gets one *tool* per operation, described to the assistant by a JSON schema, with the answer handed on as the JSON that arrived.

For pointing Claude Desktop, Cursor, Windsurf or Cline at a running server, see [MCP Server for AI Assistants](../mcp.md). This page is about what the server is.

## Installation

```bash
cargo install hook0-mcp
```

The binary is `hook0-mcp`. It reads its whole configuration from the environment and speaks MCP on stdio, which is what an assistant's `mcpServers` entry starts:

```json
{
  "mcpServers": {
    "hook0": {
      "command": "hook0-mcp",
      "env": {
        "HOOK0_API_TOKEN": "your-service-token"
      }
    }
  }
}
```

| Variable | Default | What it does |
|----------|---------|--------------|
| `HOOK0_API_TOKEN` | *required* | The service token every request is authenticated with. Empty is refused at startup |
| `HOOK0_API_URL` | `https://app.hook0.com` | The instance, without the `/api/v1`, which the server adds |
| `HOOK0_READ_ONLY` | `false` | `true`, `1` or `yes` hides every write tool |
| `MCP_TRANSPORT` | `stdio` | Only `stdio` is implemented |
| `MCP_SSE_PORT` | `3000` | Reserved for a transport that does not exist |

`HOOK0_API_URL` is the origin rather than the API root. The tools carry `/api/v1/...` in their own paths, and the server prepends that prefix to anything not already starting with it, so pointing this at `https://hook0.example.com/api/v1` reaches `/api/v1/api/v1/...`.

`MCP_TRANSPORT=sse` parses and then stops the process with an error rather than starting on stdio instead. A server that quietly ran on a transport nobody asked for is worse than one that refuses.

## The tools

Twenty-three, one per operation the API document tags `mcp`, named `<entity>.<operation>` after the operation id. Those are the same names the SDKs group their methods under, so `applications.list` here is `ApplicationsApi.list` there.

The two tags are not the same set. The SDKs are generated from `public`, which is thirty-six operations; this server is generated from `mcp`, which is twenty-three. Twenty-one are in both. Fifteen reach the SDKs and not an assistant, and two go the other way. `organizations.get` and `organizations.list` are tools here, and no client has a group for them.

| Tool | Method | Path |
|------|--------|------|
| `applications.create` | POST | `/api/v1/applications/` |
| `applications.delete` | DELETE | `/api/v1/applications/{application_id}` |
| `applications.get` | GET | `/api/v1/applications/{application_id}` |
| `applications.list` | GET | `/api/v1/applications/` |
| `applications.update` | PUT | `/api/v1/applications/{application_id}` |
| `eventTypes.create` | POST | `/api/v1/event_types/` |
| `eventTypes.delete` | DELETE | `/api/v1/event_types/{event_type_name}` |
| `eventTypes.get` | GET | `/api/v1/event_types/{event_type_name}` |
| `eventTypes.list` | GET | `/api/v1/event_types/` |
| `events.get` | GET | `/api/v1/events/{event_id}` |
| `events.ingest` | POST | `/api/v1/event/` |
| `events.list` | GET | `/api/v1/events/` |
| `events.replay` | POST | `/api/v1/events/{event_id}/replay` |
| `organizations.get` | GET | `/api/v1/organizations/{organization_id}/` |
| `organizations.list` | GET | `/api/v1/organizations/` |
| `payload_content_types.list` | GET | `/api/v1/payload_content_types/` |
| `requestAttempts.get` | GET | `/api/v1/request_attempts/{request_attempt_id}` |
| `requestAttempts.list` | GET | `/api/v1/request_attempts/` |
| `subscriptions.create` | POST | `/api/v1/subscriptions/` |
| `subscriptions.delete` | DELETE | `/api/v1/subscriptions/{subscription_id}` |
| `subscriptions.get` | GET | `/api/v1/subscriptions/{subscription_id}` |
| `subscriptions.list` | GET | `/api/v1/subscriptions/` |
| `subscriptions.update` | PUT | `/api/v1/subscriptions/{subscription_id}` |

That table is not maintained by hand and is not read off the running server either. It is what `src/server/generated.rs` carries, and `UPDATE_SDK=mcp cargo test -p hook0-sdkgen sdk_targets` is what rewrites that file when the snapshot describes something else. Touching a handler tagged `mcp` in the API fails the generator's drift test until the rewritten table is committed beside the change.

### Where an argument goes

An assistant fills in one flat JSON object per call, and the server sorts it. A name the path template holds a placeholder for is interpolated into the path. A name the tool declares as a query parameter goes in the request line. Everything left over becomes the request body.

Which names are query parameters is stated on each tool rather than worked out from its schema, because a schema says what a caller fills in and not where any of it goes. `requestAttempts.list` is where that shows. Seven of its arguments are filters travelling in the query string, namely `application_id`, `event.event_type_names`, `event_id`, `max_created_at`, `min_created_at`, `pagination_cursor` and `subscription_id`. An argument left out is simply not asked for, which is what an optional filter has to mean.

An object, an array or `null` is a value the server cannot write as text, so it fills neither a path nor a query parameter. It reaches the API in the body or not at all.

## Read-only mode

`HOOK0_READ_ONLY=true` splits the twenty-three by method. The thirteen `GET` tools stay; the ten that write disappear from `tools/list` and are refused if called anyway, with the error naming the variable that would allow them.

That restraint is the server's own, and the API knows nothing of it. The token it holds still has every permission it was issued with, so a second process started without the variable has the whole set back. Where the restriction has to hold, [attenuate the service token](../../how-to-guides/manage-service-tokens.md#token-attenuation) as well, since the API enforces that one.

## What it does not do

The eleven clients share four contracts, covering what they put on the wire, what they repeat, what they verify and what they bound. This server is held to the first and to none of the others, which is a statement about the code rather than an oversight to be fixed later.

**It sends what every client sends.** `Authorization`, `Accept`, `Content-Type`, and the two headers an instance reads a client by:

```
User-Agent: hook0-client-mcp/2.0.0 (rust; linux x86_64)
Hook0-Client-Options: attempts=1,backoff=0,ceiling=0,budget=0
```

The second one is the interesting half. The SDKs state a retry policy a caller configured; this server states the one attempt it makes and the nothing it waits between attempts it does not make. Saying nothing at all would leave an instance unable to tell this server apart from an SDK whose header went missing.

**It does not retry.** A failed call fails, and the assistant sees why. That is deliberate. A retry loop under an assistant is a loop the assistant cannot see, and `events.ingest` under an assistant has no client-minted event ID keeping a repeat idempotent the way [an SDK send does](index.md#it-mints-the-event-id-so-a-retry-cannot-duplicate-an-event). One attempt gets 30 seconds, and 10 of those to open the connection.

**It verifies no signature.** Nothing delivers a webhook to it. Verification is a consumer's job, and every SDK page has it.

**It applies none of the payload or response bounds.** No `max_payload_bytes` refuses an oversized ingestion before it goes out, and no ceiling bounds what an answer costs to read. The bounds exist in the SDKs because an emitter runs unattended in production; this process runs beside an assistant on a developer's machine, reaching an instance that already refuses what it will not accept.

## Failures

An answer that is not a success becomes an MCP error carrying the problem the API named, ahead of the prose beside it:

```
EventAlreadyIngested: an event with this id has already been ingested
```

The stable identifier comes first because it is the part an assistant can act on. It is what tells an ingestion that already happened, and must not be repeated, from any other conflict. A `404` becomes a resource-not-found error, a `401` or `403` an authentication failure, everything else an internal error carrying the status. A body that is not one of Hook0's problem documents is reported as it arrived, cut at 2 048 characters with the count of what was left out, because a proxy between the assistant and Hook0 writes what it likes and what it wrote is the only clue there is.

Unlike the SDKs, nothing here reads an answer into a type. The server holds no model of what the API returns, so a success travels on to the assistant as the JSON that arrived.

## Protocol revisions

MCP is versioned by dated revisions, settled on during `initialize`. This server implements `2024-11-05`, `2025-03-26`, `2025-06-18` and `2025-11-25`, advertises the last, and answers with it when a client asks for one that is not on the list, so a client pinned elsewhere negotiates down rather than being turned away. A request whose inline `_meta` names an unlisted revision is refused outright with `-32022`.

`2026-07-28` is deliberately absent: it asks for a stateless lifecycle, `subscriptions/listen` and input-required tool handling this server does not provide, and claiming a revision it has not implemented would be worse for a client than not claiming it.

## Resources and prompts

Beside the tools, the server answers eight `hook0://` resource URIs: `organizations`, `applications`, `applications/{id}` and that application's `/events`, `/subscriptions` and `/event_types`, plus `events/{id}` and its `/attempts`. Only the first two are advertised by `resources/list`. The six that carry an identifier are answered when asked for and listed nowhere, since there is nothing to enumerate them from without first fetching the applications.

Three guided prompts walk an assistant through the workflows those tools compose into, namely `create_webhook_subscription`, `debug_event_delivery` and `setup_application`.

## Embedding it

The crate is a library as well as a binary, so a process that already exists can serve the same tools rather than spawning `hook0-mcp` beside itself:

```mcp example=serve
use hook0_mcp::{Config, Hook0Client, Hook0McpServer};
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let client = Hook0Client::new(&config)?;
    let server = Hook0McpServer::new(client, config.read_only);

    server.serve(stdio()).await?.waiting().await?;
    Ok(())
}
```

`Config::from_env` reads the same variables the binary does, and its fields are public, so an embedding whose configuration comes from somewhere else builds one directly instead. `Hook0McpServer::new` takes the read-only flag separately from the configuration it came out of, which is what lets one process serve a restricted server off an unrestricted configuration.

The tool table is public, and is what a build step would read to check its own assumptions against the API:

```mcp example=tools
use hook0_mcp::server::GENERATED_TOOLS;

for tool in GENERATED_TOOLS {
    println!(
        "{} {} {} ({})",
        if tool.is_write_operation() { "write" } else { "read " },
        tool.method,
        tool.path_template,
        tool.name,
    );
}
```

`get_tool_info` looks one up by name, `is_write_tool` answers the question read-only mode asks, and `interpolate_path` fills a template the way the dispatcher does:

```mcp example=dispatch
use hook0_mcp::server::{get_tool_info, interpolate_path, is_write_tool};

let Some(tool) = get_tool_info(name) else {
    return; // no tool answers to that name
};

if is_write_tool(name) {
    // refused when the server was started read-only
}

let path = interpolate_path(tool.path_template, arguments);
println!("{} {path}", tool.method);
```

## Links

- **Crate**: [hook0-mcp on crates.io](https://crates.io/crates/hook0-mcp)
- **Source**: [clients/mcp](https://gitlab.com/hook0/hook0/-/tree/master/clients/mcp)
- **Setting it up in an assistant**: [MCP Server for AI Assistants](../mcp.md)
- **API reference**: [Hook0 API](../../openapi/intro)
- **Other SDKs**: [SDKs & client libraries](index.md)
