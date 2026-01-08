//! Prompts module for Hook0 MCP server
//!
//! Provides guided interaction templates for common workflows.

use crate::error::McpError;
use rmcp::model::*;
use std::collections::HashMap;

/// List all available prompts
pub fn list_prompts() -> Vec<Prompt> {
    vec![
        Prompt::new(
            "create_webhook_subscription",
            Some(
                "Interactive guide to create a new webhook subscription. \
                 Walks through application selection, event type filtering, and target configuration.",
            ),
            Some(vec![
                PromptArgument {
                    name: "application_id".into(),
                    title: None,
                    description: Some("The application to create the subscription in".into()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "target_url".into(),
                    title: None,
                    description: Some("The URL where webhooks will be delivered".into()),
                    required: Some(false),
                },
            ]),
        ),
        Prompt::new(
            "debug_event_delivery",
            Some(
                "Help troubleshoot webhook delivery issues by examining events, \
                 request attempts, and subscription configuration.",
            ),
            Some(vec![
                PromptArgument {
                    name: "event_id".into(),
                    title: None,
                    description: Some("The event ID to debug".into()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "subscription_id".into(),
                    title: None,
                    description: Some("The subscription to examine".into()),
                    required: Some(false),
                },
            ]),
        ),
        Prompt::new(
            "setup_application",
            Some(
                "Guide through initial application setup including event type \
                 registration and first subscription creation.",
            ),
            Some(vec![
                PromptArgument {
                    name: "organization_id".into(),
                    title: None,
                    description: Some("The organization for the new application".into()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "application_name".into(),
                    title: None,
                    description: Some("Name for the new application".into()),
                    required: Some(false),
                },
            ]),
        ),
    ]
}

/// Get a specific prompt by name
pub fn get_prompt(
    name: &str,
    arguments: Option<HashMap<String, String>>,
) -> Result<GetPromptResult, McpError> {
    let args = arguments.unwrap_or_default();

    match name {
        "create_webhook_subscription" => {
            let app_id = args.get("application_id");
            let target_url = args.get("target_url");

            let mut content =
                String::from("I'll help you create a webhook subscription step by step.\n\n");

            if let Some(app_id) = app_id {
                content.push_str(&format!("Using application: {}\n\n", app_id));
            } else {
                content.push_str(
                    "First, let's identify which application should receive the subscription.\n\
                     Please use the `list_applications` tool to see available applications, \
                     or provide an application_id if you already know it.\n\n",
                );
            }

            if let Some(target_url) = target_url {
                content.push_str(&format!("Target URL: {}\n\n", target_url));
            } else {
                content.push_str(
                    "Next, we need a target URL where webhooks will be sent.\n\
                     This should be an HTTPS endpoint that can receive POST requests.\n\n",
                );
            }

            content.push_str(
                "Then we'll configure:\n\
                 1. Which event types should trigger this webhook\n\
                 2. The subscription name for identification\n\
                 3. Any additional settings like retry policies\n\n\
                 Use `list_event_types` with the application_id to see available event types, \
                 then use `create_subscription` to create the webhook.",
            );

            Ok(GetPromptResult {
                description: Some("Create a webhook subscription step by step".into()),
                messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
            })
        }

        "debug_event_delivery" => {
            let event_id = args.get("event_id");
            let subscription_id = args.get("subscription_id");

            let mut content = String::from("I'll help you debug webhook delivery issues.\n\n");

            if let Some(eid) = event_id {
                content.push_str(&format!(
                    "Debugging event: {}\n\n\
                     To investigate this event:\n\
                     1. Use `get_event` to see the event details and payload\n\
                     2. Use `list_request_attempts` to see all delivery attempts\n\
                     3. Check the HTTP status codes and response times\n\n",
                    eid
                ));
            } else {
                content.push_str(
                    "To start debugging, I'll need an event_id.\n\
                     Use `list_events` with an application_id to find events, \
                     or check your logs for the event ID.\n\n",
                );
            }

            if let Some(sid) = subscription_id {
                content.push_str(&format!(
                    "Examining subscription: {}\n\n\
                     I'll check the subscription configuration to ensure:\n\
                     - The target URL is correct and reachable\n\
                     - The event type filters match your events\n\
                     - The subscription is enabled\n",
                    sid
                ));
            }

            content.push_str(
                "\nCommon issues to check:\n\
                 - **4xx errors**: Check authentication, headers, or payload format\n\
                 - **5xx errors**: The receiving server has issues\n\
                 - **Timeouts**: The endpoint is too slow (>30s)\n\
                 - **Connection errors**: DNS or network issues\n\n\
                 Use `retry_delivery` with a request_attempt_id to retry a failed delivery.",
            );

            Ok(GetPromptResult {
                description: Some("Debug webhook delivery issues".into()),
                messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
            })
        }

        "setup_application" => {
            let org_id = args.get("organization_id");
            let app_name = args.get("application_name");

            let mut content =
                String::from("I'll guide you through setting up a new Hook0 application.\n\n");

            content.push_str("## Step 1: Create the Application\n\n");

            if org_id.is_none() {
                content.push_str(
                    "First, we need to choose an organization.\n\
                     Use `list_organizations` to see your available organizations.\n\n",
                );
            }

            if app_name.is_none() {
                content.push_str(
                    "Choose a descriptive name for your application \
                     (e.g., 'Order Notifications', 'User Events').\n\n",
                );
            }

            if let (Some(org_id), Some(app_name)) = (org_id, app_name) {
                content.push_str(&format!(
                    "Ready to create application '{}' in organization '{}'.\n\
                     Use `create_application` with these values.\n\n",
                    app_name, org_id
                ));
            }

            content.push_str(
                "## Step 2: Register Event Types\n\n\
                 Event types define what kinds of events your service will emit.\n\
                 Use naming convention: `service.resource.action`\n\
                 Examples:\n\
                 - `order.payment.completed`\n\
                 - `user.account.created`\n\
                 - `inventory.item.updated`\n\n\
                 Use `create_event_type` for each event type you need.\n\n\
                 ## Step 3: Create Your First Subscription\n\n\
                 A subscription connects events to a webhook endpoint.\n\
                 You'll need:\n\
                 - A target URL (your webhook receiver)\n\
                 - Event types to subscribe to\n\n\
                 Use `create_subscription` to set up webhook delivery.\n\n\
                 ## Step 4: Test the Integration\n\n\
                 Use `ingest_event` to send a test event and verify delivery.",
            );

            Ok(GetPromptResult {
                description: Some("Set up a new Hook0 application".into()),
                messages: vec![PromptMessage::new_text(PromptMessageRole::User, content)],
            })
        }

        _ => Err(McpError::invalid_params(
            format!("Unknown prompt: {}", name),
            None,
        )),
    }
}
