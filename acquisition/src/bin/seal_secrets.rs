//! One-shot operator tool: produces the SealedSecret manifest for the
//! acquisition deployment.
//!
//! Reads the Google Ads OAuth credentials from local files, generates a
//! random `api-token`, builds the Kubernetes `Secret` in memory, and pipes
//! it through `kubeseal` to produce the encrypted SealedSecret manifest.
//!
//! Plain credential values are never written to disk and never printed.
//! Only the encrypted output is materialized.
//!
//! Usage:
//!   cargo run -p hook0-acquisition --bin seal-secrets -- \
//!     --output acquisition/charts/templates/sealed-secret.yaml
//!
//! Required on PATH: `kubeseal` (Bitnami SealedSecrets CLI).
//! Required kubeconfig context: `france-nuage-prod` (where the controller runs).

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

const NAMESPACE: &str = "hosted-hook0-acquisition-prod";
const SECRET_NAME: &str = "hook0-acquisition-secrets";
const KUBECONFIG_CONTEXT: &str = "france-nuage-prod";

// Hook0-specific values that are not sensitive enough to keep out of the
// repository long-term, but we still pipe them through kubeseal for a
// uniform mechanism.
const GOOGLE_ADS_DEVELOPER_TOKEN: &str = "z0pKqtvj9flCXcEle_pIrw";
const GOOGLE_ADS_CUSTOMER_ID: &str = "629-941-8464";
const GOOGLE_ADS_LOGIN_CUSTOMER_ID: &str = "343-494-6488";
const GOOGLE_ADS_CONVERSION_ACTION_ID: &str = "7576442588";

const CREDS_PATH: &str = "/Users/fgribreau/.mcp-google-ads/credentials.json";
const TOKEN_PATH: &str = "/Users/fgribreau/.mcp-google-ads/token.json";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let output_path = parse_output_arg(&args).unwrap_or_else(|| {
        eprintln!("usage: seal-secrets --output <path/to/sealed-secret.yaml>");
        std::process::exit(2);
    });

    let creds_raw =
        fs::read_to_string(CREDS_PATH).unwrap_or_else(|e| die(&format!("read {CREDS_PATH}: {e}")));
    let token_raw =
        fs::read_to_string(TOKEN_PATH).unwrap_or_else(|e| die(&format!("read {TOKEN_PATH}: {e}")));

    let creds: serde_json::Value = serde_json::from_str(&creds_raw)
        .unwrap_or_else(|e| die(&format!("parse credentials.json: {e}")));
    let token: serde_json::Value =
        serde_json::from_str(&token_raw).unwrap_or_else(|e| die(&format!("parse token.json: {e}")));

    let oauth_obj = creds
        .get("installed")
        .or_else(|| creds.get("web"))
        .and_then(|v| v.as_object())
        .unwrap_or_else(|| die("credentials.json missing 'installed' or 'web' object"));

    let client_id = oauth_obj
        .get("client_id")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| die("credentials.json missing client_id"));
    let client_secret = oauth_obj
        .get("client_secret")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| die("credentials.json missing client_secret"));
    let refresh_token = token
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| die("token.json missing refresh_token"));

    // Random 32-byte hex string for the shared bearer token.
    let api_token = generate_api_token();

    // Build the Secret YAML using stringData (kubeseal will base64-encode).
    let secret_yaml = build_secret_yaml(&api_token, client_id, client_secret, refresh_token);

    let sealed_yaml = run_kubeseal(&secret_yaml);

    // Verify the output is a SealedSecret and contains no plain credentials.
    assert_no_plaintext(
        &sealed_yaml,
        &[&api_token, client_id, client_secret, refresh_token],
    );

    fs::write(&output_path, sealed_yaml.as_bytes())
        .unwrap_or_else(|e| die(&format!("write {}: {e}", output_path.display())));

    println!("Wrote SealedSecret to {}", output_path.display());
    println!("API token (kept here once for sharing with Hook0 API/output-worker):");
    println!("    ACQUISITION_API_TOKEN={api_token}");
    println!("Add this value to the Hook0 API CI/CD variables. It will not be reprintable.");
}

fn parse_output_arg(args: &[String]) -> Option<PathBuf> {
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        if a == "--output" || a == "-o" {
            return iter.next().map(PathBuf::from);
        }
    }
    None
}

fn generate_api_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Pull 32 bytes of OS entropy via /dev/urandom (POSIX) for an opaque token.
    let mut buf = [0u8; 32];
    let mut f = std::fs::File::open("/dev/urandom")
        .unwrap_or_else(|e| die(&format!("open /dev/urandom: {e}")));
    use std::io::Read;
    f.read_exact(&mut buf)
        .unwrap_or_else(|e| die(&format!("read /dev/urandom: {e}")));
    // Mix with current time as defense-in-depth (not strictly required).
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    buf[0] ^= (nanos & 0xff) as u8;
    let mut out = String::with_capacity(64);
    for b in &buf {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn build_secret_yaml(
    api_token: &str,
    oauth_client_id: &str,
    oauth_client_secret: &str,
    oauth_refresh_token: &str,
) -> String {
    // stringData lets us pass plain UTF-8; kubeseal base64-encodes server side.
    format!(
        "apiVersion: v1\n\
         kind: Secret\n\
         metadata:\n  \
         name: {SECRET_NAME}\n  \
         namespace: {NAMESPACE}\n\
         type: Opaque\n\
         stringData:\n  \
         api-token: \"{api_token}\"\n  \
         google-ads-developer-token: \"{GOOGLE_ADS_DEVELOPER_TOKEN}\"\n  \
         google-ads-customer-id: \"{GOOGLE_ADS_CUSTOMER_ID}\"\n  \
         google-ads-login-customer-id: \"{GOOGLE_ADS_LOGIN_CUSTOMER_ID}\"\n  \
         google-ads-conversion-action-id: \"{GOOGLE_ADS_CONVERSION_ACTION_ID}\"\n  \
         google-ads-oauth-client-id: \"{oauth_client_id}\"\n  \
         google-ads-oauth-client-secret: \"{oauth_client_secret}\"\n  \
         google-ads-oauth-refresh-token: \"{oauth_refresh_token}\"\n",
    )
}

fn run_kubeseal(secret_yaml: &str) -> String {
    let mut child = Command::new("kubeseal")
        .args([
            "--context",
            KUBECONFIG_CONTEXT,
            "--namespace",
            NAMESPACE,
            "--name",
            SECRET_NAME,
            "--scope",
            "strict",
            "--format",
            "yaml",
            "--controller-name",
            "sealed-secrets",
            "--controller-namespace",
            "kube-system",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| die(&format!("spawn kubeseal: {e}")));

    {
        let stdin = child
            .stdin
            .as_mut()
            .unwrap_or_else(|| die("kubeseal stdin"));
        stdin
            .write_all(secret_yaml.as_bytes())
            .unwrap_or_else(|e| die(&format!("write to kubeseal stdin: {e}")));
    }

    let output = child
        .wait_with_output()
        .unwrap_or_else(|e| die(&format!("wait kubeseal: {e}")));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        die(&format!("kubeseal failed: {stderr}"));
    }

    String::from_utf8(output.stdout).unwrap_or_else(|e| die(&format!("kubeseal stdout: {e}")))
}

fn assert_no_plaintext(sealed_yaml: &str, plain_values: &[&str]) {
    for v in plain_values {
        if v.len() >= 8 && sealed_yaml.contains(v) {
            die("INTERNAL ERROR: kubeseal output contains plaintext fragment, refusing to write");
        }
    }
}

fn die(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(1);
}
