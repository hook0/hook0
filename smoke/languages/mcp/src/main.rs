//! The MCP server against a Hook0 that is really running.
//!
//! This client is not a library an application links: it is a process an assistant speaks a
//! protocol to. So the smoke builds the server, starts it, and drives it over the stdio transport
//! it ships — an `initialize`, then `tools/call` twice on the tool that ingests an event. What is
//! being asked is the same as everywhere else: that the token is accepted, and that a duplicated
//! ingestion comes back naming the conflict rather than as some generic failure.
//!
//! One thing the other eleven smokes do is missing here, and its absence is the answer rather than
//! a gap: the server's tools are generated from the API's OpenAPI document, which declares no
//! operation for verifying a webhook signature. There is no consumer half to hold a
//! server-produced signature against.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::thread;
use std::time::Duration;

use serde_json::{Value, json};

/// The conflict the API answers a duplicated ingestion with.
const ALREADY_INGESTED: &str = "EventAlreadyIngested";

/// Where the server this smoke drives lives, relative to this crate.
const SERVER_MANIFEST: &str = "../../../clients/mcp/Cargo.toml";

/// What the server is called, so its executable can be picked out of what Cargo reports building.
const SERVER: &str = "hook0-mcp";

/// How long one answer from the server is waited for.
const ANSWER_WITHIN: Duration = Duration::from_secs(60);

/// The protocol release this smoke speaks.
const PROTOCOL: &str = "2024-11-05";

fn main() -> Result<(), String> {
    let api_url = setting("HOOK0_API_URL")?;
    let application_id = setting("HOOK0_APPLICATION_ID")?;
    let token = setting("HOOK0_TOKEN")?;
    let event_type = setting("HOOK0_EVENT_TYPE")?;

    let executable = build()?;
    let mut server = start(&executable, &base_of(&api_url), &token)?;
    let mut speaking = Speaking::to(&mut server)?;

    speaking.request(
        1,
        "initialize",
        json!({
            "protocolVersion": PROTOCOL,
            "capabilities": {},
            "clientInfo": { "name": "hook0-live-smoke", "version": "0.0.0" },
        }),
    )?;
    speaking.notify("notifications/initialized")?;

    let event_id = new_event_id();
    let ingested = speaking.request(
        2,
        "tools/call",
        json!({
            "name": "events.ingest",
            "arguments": ingest(&application_id, &event_type, &event_id),
        }),
    )?;
    if ingested.get("error").is_some() {
        stop(server);
        return Err(format!("the instance refused the first send: {ingested}"));
    }
    println!("ingested {event_id}");

    let again = speaking.request(
        3,
        "tools/call",
        json!({
            "name": "events.ingest",
            "arguments": ingest(&application_id, &event_type, &event_id),
        }),
    )?;
    stop(server);

    let said = format!("{again}");
    if again.get("error").is_none() {
        return Err("sending the same event twice was accepted twice".to_owned());
    }
    if !said.contains(ALREADY_INGESTED) {
        return Err(format!(
            "the second send failed without naming {ALREADY_INGESTED}: {said}"
        ));
    }
    println!("the second send reported {ALREADY_INGESTED}");
    println!("no signature is verified here: the generated tool set declares no operation for it");
    Ok(())
}

/// The arguments one ingestion is called with.
fn ingest(application_id: &str, event_type: &str, event_id: &str) -> Value {
    json!({
        "application_id": application_id,
        "event_id": event_id,
        "event_type": event_type,
        "labels": { "language": "mcp" },
        "occurred_at": "2026-01-01T00:00:00Z",
        "payload": "{\"from\":\"the mcp smoke\"}",
        "payload_content_type": "application/json",
    })
}

/// The base URL the server is configured with: its tools carry `/api/v1` in their own paths, so
/// handing it the one the SDKs are given would reach `/api/v1/api/v1`.
fn base_of(api_url: &str) -> String {
    api_url
        .trim_end_matches('/')
        .trim_end_matches("/api/v1")
        .to_owned()
}

/// An identifier this smoke mints, in the shape the API keys events on.
///
/// Version 4 rather than 7: nothing here orders events, and the shape is what the API validates.
fn new_event_id() -> String {
    let mut random = [0u8; 16];
    if let Ok(mut source) = std::fs::File::open("/dev/urandom") {
        use std::io::Read as _;
        let _ = source.read_exact(&mut random);
    }
    random[6] = (random[6] & 0x0f) | 0x40;
    random[8] = (random[8] & 0x3f) | 0x80;
    let hex: String = random.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Builds the server and answers where its executable landed, as Cargo reports it rather than as a
/// path written down here.
fn build() -> Result<PathBuf, String> {
    let built = Command::new("cargo")
        .args([
            "build",
            "--quiet",
            "--message-format=json-render-diagnostics",
            "--manifest-path",
            SERVER_MANIFEST,
        ])
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|cause| format!("building {SERVER}: {cause}"))?;
    if !built.status.success() {
        return Err(format!("building {SERVER} failed: {}", built.status));
    }

    let reported = String::from_utf8_lossy(&built.stdout);
    for line in reported.lines() {
        let message: Value = match serde_json::from_str(line) {
            Ok(message) => message,
            Err(_) => continue,
        };
        let names_server = message
            .get("target")
            .and_then(|target| target.get("name"))
            .and_then(Value::as_str)
            == Some(SERVER);
        if !names_server {
            continue;
        }
        if let Some(executable) = message.get("executable").and_then(Value::as_str) {
            return Ok(PathBuf::from(executable));
        }
    }
    Err(format!(
        "cargo built {SERVER} without reporting where its executable landed"
    ))
}

/// Starts the server on its stdio transport.
fn start(executable: &Path, api_url: &str, token: &str) -> Result<Child, String> {
    Command::new(executable)
        .env("HOOK0_API_URL", api_url)
        .env("HOOK0_API_TOKEN", token)
        .env("MCP_TRANSPORT", "stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|cause| format!("starting {}: {cause}", executable.display()))
}

/// Stops the server, whatever it was doing: the smoke is over either way.
fn stop(mut server: Child) {
    let _ = server.kill();
    let _ = server.wait();
}

/// One conversation with the server, newline-delimited JSON-RPC in both directions.
struct Speaking<'a> {
    to: &'a mut std::process::ChildStdin,
    from: Receiver<String>,
}

impl<'a> Speaking<'a> {
    fn to(server: &'a mut Child) -> Result<Speaking<'a>, String> {
        let out = server
            .stdout
            .take()
            .ok_or("the server was started without a readable stdout")?;
        let (said, hearing) = channel();
        thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if said.send(line).is_err() {
                    return;
                }
            }
        });

        let to = server
            .stdin
            .as_mut()
            .ok_or("the server was started without a writable stdin")?;
        Ok(Speaking { to, from: hearing })
    }

    /// Sends a request and waits for the answer carrying its identifier.
    fn request(&mut self, id: u32, method: &str, params: Value) -> Result<Value, String> {
        self.write(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        }))?;

        loop {
            let line = match self.from.recv_timeout(ANSWER_WITHIN) {
                Ok(line) => line,
                Err(RecvTimeoutError::Timeout) => {
                    return Err(format!(
                        "the server did not answer `{method}` within {}s",
                        ANSWER_WITHIN.as_secs()
                    ));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(format!("the server stopped before answering `{method}`"));
                }
            };
            let answer: Value = match serde_json::from_str(&line) {
                Ok(answer) => answer,
                Err(_) => continue,
            };
            if answer.get("id").and_then(Value::as_u64) == Some(u64::from(id)) {
                return Ok(answer);
            }
        }
    }

    /// Sends a notification, which by definition is not answered.
    fn notify(&mut self, method: &str) -> Result<(), String> {
        self.write(json!({ "jsonrpc": "2.0", "method": method, "params": {} }))
    }

    fn write(&mut self, message: Value) -> Result<(), String> {
        writeln!(self.to, "{message}")
            .map_err(|cause| format!("writing to the server: {cause}"))?;
        self.to
            .flush()
            .map_err(|cause| format!("writing to the server: {cause}"))
    }
}

/// A setting the harness passes, or a refusal naming it.
fn setting(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("{name} is not set"))
}
