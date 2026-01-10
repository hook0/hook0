//! Configuration for Hook0 MCP server

use crate::error::Hook0McpError;
use std::env;
use url::Url;

/// Default Hook0 API URL
const DEFAULT_API_URL: &str = "https://app.hook0.com";

/// Default SSE server port
const DEFAULT_SSE_PORT: u16 = 3000;

/// Transport type for the MCP server
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Transport {
    /// Standard I/O transport (default, for local IDE integration)
    #[default]
    Stdio,
    /// Server-Sent Events transport (for remote/containerized deployment)
    Sse { port: u16 },
}

/// Configuration for the Hook0 MCP server
#[derive(Debug, Clone)]
pub struct Config {
    /// Hook0 API base URL
    pub api_url: Url,
    /// Hook0 API token (Bearer token)
    pub api_token: String,
    /// Transport type
    pub transport: Transport,
    /// Read-only mode (only expose GET endpoints)
    pub read_only: bool,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// Required environment variables:
    /// - `HOOK0_API_TOKEN`: The API token for authentication
    ///
    /// Optional environment variables:
    /// - `HOOK0_API_URL`: API base URL (default: https://app.hook0.com)
    /// - `HOOK0_READ_ONLY`: Set to "true" or "1" to enable read-only mode (default: false)
    /// - `MCP_TRANSPORT`: Transport type: "stdio" or "sse" (default: stdio)
    /// - `MCP_SSE_PORT`: Port for SSE server (default: 3000)
    pub fn from_env() -> Result<Self, Hook0McpError> {
        // Required: API token
        let api_token = env::var("HOOK0_API_TOKEN").map_err(|_| {
            Hook0McpError::Config(
                "HOOK0_API_TOKEN environment variable is required. \
                 Get your API token from the Hook0 dashboard."
                    .to_string(),
            )
        })?;

        if api_token.is_empty() {
            return Err(Hook0McpError::Config(
                "HOOK0_API_TOKEN cannot be empty".to_string(),
            ));
        }

        // Optional: API URL
        let api_url_str = env::var("HOOK0_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        let api_url = Url::parse(&api_url_str).map_err(|e| {
            Hook0McpError::Config(format!("Invalid HOOK0_API_URL '{}': {}", api_url_str, e))
        })?;

        // Optional: Transport type
        let transport = match env::var("MCP_TRANSPORT")
            .unwrap_or_else(|_| "stdio".to_string())
            .to_lowercase()
            .as_str()
        {
            "stdio" => Transport::Stdio,
            "sse" => {
                let port = env::var("MCP_SSE_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(DEFAULT_SSE_PORT);
                Transport::Sse { port }
            }
            other => {
                return Err(Hook0McpError::Config(format!(
                    "Invalid MCP_TRANSPORT '{}'. Must be 'stdio' or 'sse'.",
                    other
                )));
            }
        };

        // Optional: Read-only mode
        let read_only = env::var("HOOK0_READ_ONLY")
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes"))
            .unwrap_or(false);

        Ok(Config {
            api_url,
            api_token,
            transport,
            read_only,
        })
    }

    /// Get the API URL with a path appended
    pub fn api_endpoint(&self, path: &str) -> Url {
        let mut url = self.api_url.clone();
        let base_path = url.path().trim_end_matches('/');
        let clean_path = path.trim_start_matches('/');
        url.set_path(&format!("{}/{}", base_path, clean_path));
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_endpoint() {
        let config = Config {
            api_url: Url::parse("https://app.hook0.com").unwrap(),
            api_token: "test-token".to_string(),
            transport: Transport::Stdio,
            read_only: false,
        };

        let endpoint = config.api_endpoint("/api/v1/applications");
        assert_eq!(
            endpoint.as_str(),
            "https://app.hook0.com/api/v1/applications"
        );
    }

    #[test]
    fn test_api_endpoint_with_trailing_slash() {
        let config = Config {
            api_url: Url::parse("https://app.hook0.com/").unwrap(),
            api_token: "test-token".to_string(),
            transport: Transport::Stdio,
            read_only: false,
        };

        let endpoint = config.api_endpoint("api/v1/applications");
        assert_eq!(
            endpoint.as_str(),
            "https://app.hook0.com/api/v1/applications"
        );
    }
}
