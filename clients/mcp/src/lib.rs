//! Hook0 MCP Server
//!
//! A Model Context Protocol (MCP) server for Hook0 Webhooks-as-a-Service.
//!
//! This crate provides an MCP server that allows AI agents to interact with
//! the Hook0 API for managing webhooks, subscriptions, and events.
//!
//! # Features
//!
//! - **Tools**: Actions like creating applications, subscriptions, and ingesting events
//! - **Resources**: Read-only access to organizations, applications, and event history
//! - **Prompts**: Guided workflows for common tasks
//!
//! # Usage
//!
//! Set the required environment variables and run the server:
//!
//! ```bash
//! export HOOK0_API_TOKEN="your-api-token"
//! export HOOK0_API_URL="https://app.hook0.com"  # optional
//! hook0-mcp
//! ```
//!
//! # Environment Variables
//!
//! - `HOOK0_API_TOKEN` (required): Your Hook0 API token
//! - `HOOK0_API_URL` (optional): API base URL (default: https://app.hook0.com)
//! - `MCP_TRANSPORT` (optional): Transport type: "stdio" or "sse" (default: stdio)
//! - `MCP_SSE_PORT` (optional): Port for SSE server (default: 3000)

pub mod client;
pub mod config;
pub mod error;
pub mod prompts;
pub mod server;

pub use client::Hook0Client;
pub use config::{Config, Transport};
pub use error::{Hook0McpError, McpError};
pub use server::Hook0McpServer;
