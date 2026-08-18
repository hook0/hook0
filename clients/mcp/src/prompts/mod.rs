//! Prompts module for Hook0 MCP server
//!
//! Provides guided interaction templates for common workflows.
//!
//! Every tool a prompt sends an assistant to is named through one of the constants below rather
//! than written into the prose, so that the suite at the bottom of this file has something to hold
//! against [`GENERATED_TOOLS`](crate::server::GENERATED_TOOLS). A prompt is text an assistant
//! follows literally: a name that no longer exists sends it to call a tool this server does not
//! have, the call comes back as an unknown tool, and nothing else says a word. The prompt still
//! renders, the server still starts, and every test still passes.

use crate::error::McpError;
use rmcp::model::*;
use std::collections::HashMap;

/// The tools the prompts of this module send an assistant to, by the names this server exposes.
///
/// One constant per tool, cited by interpolation, so the prose carries no name of its own and a
/// rename has one place to reach.
const APPLICATIONS_CREATE: &str = "applications.create";
const APPLICATIONS_LIST: &str = "applications.list";
const EVENT_TYPES_CREATE: &str = "eventTypes.create";
const EVENT_TYPES_LIST: &str = "eventTypes.list";
const EVENTS_GET: &str = "events.get";
const EVENTS_INGEST: &str = "events.ingest";
const EVENTS_LIST: &str = "events.list";
const EVENTS_REPLAY: &str = "events.replay";
const ORGANIZATIONS_LIST: &str = "organizations.list";
const REQUEST_ATTEMPTS_GET: &str = "requestAttempts.get";
const REQUEST_ATTEMPTS_LIST: &str = "requestAttempts.list";
const SUBSCRIPTIONS_CREATE: &str = "subscriptions.create";

/// Every tool the prompts of this module name, indexed for the suite below.
///
/// The prose cites the constants above, so this is what the suite reads instead of the prose. It
/// exists only under test: a constant added above and left out here is caught all the same, since
/// `every_tool_named_reaches_a_prompt` holds the rendered prompts against `GENERATED_TOOLS` and
/// fails on any tool it names that this list does not cover.
#[cfg(test)]
const CITED_TOOLS: &[&str] = &[
    APPLICATIONS_CREATE,
    APPLICATIONS_LIST,
    EVENT_TYPES_CREATE,
    EVENT_TYPES_LIST,
    EVENTS_GET,
    EVENTS_INGEST,
    EVENTS_LIST,
    EVENTS_REPLAY,
    ORGANIZATIONS_LIST,
    REQUEST_ATTEMPTS_GET,
    REQUEST_ATTEMPTS_LIST,
    SUBSCRIPTIONS_CREATE,
];

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
                PromptArgument::new("application_id")
                    .with_description("The application to create the subscription in")
                    .with_required(false),
                PromptArgument::new("target_url")
                    .with_description("The URL where webhooks will be delivered")
                    .with_required(false),
            ]),
        ),
        Prompt::new(
            "debug_event_delivery",
            Some(
                "Help troubleshoot webhook delivery issues by examining events, \
                 request attempts, and subscription configuration.",
            ),
            Some(vec![
                PromptArgument::new("event_id")
                    .with_description("The event ID to debug")
                    .with_required(false),
                PromptArgument::new("subscription_id")
                    .with_description("The subscription to examine")
                    .with_required(false),
            ]),
        ),
        Prompt::new(
            "setup_application",
            Some(
                "Guide through initial application setup including event type \
                 registration and first subscription creation.",
            ),
            Some(vec![
                PromptArgument::new("organization_id")
                    .with_description("The organization for the new application")
                    .with_required(false),
                PromptArgument::new("application_name")
                    .with_description("Name for the new application")
                    .with_required(false),
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
                content.push_str(&format!(
                    "First, let's identify which application should receive the subscription.\n\
                     Please use the `{APPLICATIONS_LIST}` tool to see available applications, \
                     or provide an application_id if you already know it.\n\n",
                ));
            }

            if let Some(target_url) = target_url {
                content.push_str(&format!("Target URL: {}\n\n", target_url));
            } else {
                content.push_str(
                    "Next, we need a target URL where webhooks will be sent.\n\
                     This should be an HTTPS endpoint that can receive POST requests.\n\n",
                );
            }

            content.push_str(&format!(
                "Then we'll configure:\n\
                 1. Which event types should trigger this webhook\n\
                 2. The subscription name for identification\n\
                 3. Any additional settings like retry policies\n\n\
                 Use `{EVENT_TYPES_LIST}` with the application_id to see available event types, \
                 then use `{SUBSCRIPTIONS_CREATE}` to create the webhook.",
            ));

            Ok(
                GetPromptResult::new(vec![PromptMessage::new_text(Role::User, content)])
                    .with_description("Create a webhook subscription step by step"),
            )
        }

        "debug_event_delivery" => {
            let event_id = args.get("event_id");
            let subscription_id = args.get("subscription_id");

            let mut content = String::from("I'll help you debug webhook delivery issues.\n\n");

            if let Some(eid) = event_id {
                content.push_str(&format!(
                    "Debugging event: {eid}\n\n\
                     To investigate this event:\n\
                     1. Use `{EVENTS_GET}` to see the event details and payload\n\
                     2. Use `{REQUEST_ATTEMPTS_LIST}` to see all delivery attempts\n\
                     3. Use `{REQUEST_ATTEMPTS_GET}` on any attempt whose outcome you need in full\n\
                     4. Check the HTTP status codes and response times\n\n",
                ));
            } else {
                content.push_str(&format!(
                    "To start debugging, I'll need an event_id.\n\
                     Use `{EVENTS_LIST}` with an application_id to find events, \
                     or check your logs for the event ID.\n\n",
                ));
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

            content.push_str(&format!(
                "\nCommon issues to check:\n\
                 - **4xx errors**: Check authentication, headers, or payload format\n\
                 - **5xx errors**: The receiving server has issues\n\
                 - **Timeouts**: The endpoint is too slow (>30s)\n\
                 - **Connection errors**: DNS or network issues\n\n\
                 Use `{EVENTS_REPLAY}` with the event_id and its application_id to send the event \
                 to its subscriptions again. Retrying one delivery attempt on its own is not \
                 something this server can do: `{REQUEST_ATTEMPTS_LIST}` and \
                 `{REQUEST_ATTEMPTS_GET}` read attempts, they do not repeat them.",
            ));

            Ok(
                GetPromptResult::new(vec![PromptMessage::new_text(Role::User, content)])
                    .with_description("Debug webhook delivery issues"),
            )
        }

        "setup_application" => {
            let org_id = args.get("organization_id");
            let app_name = args.get("application_name");

            let mut content =
                String::from("I'll guide you through setting up a new Hook0 application.\n\n");

            content.push_str("## Step 1: Create the Application\n\n");

            if org_id.is_none() {
                content.push_str(&format!(
                    "First, we need to choose an organization.\n\
                     Use `{ORGANIZATIONS_LIST}` to see your available organizations.\n\n",
                ));
            }

            if app_name.is_none() {
                content.push_str(
                    "Choose a descriptive name for your application \
                     (e.g., 'Order Notifications', 'User Events').\n\n",
                );
            }

            if let (Some(org_id), Some(app_name)) = (org_id, app_name) {
                content.push_str(&format!(
                    "Ready to create application '{app_name}' in organization '{org_id}'.\n\
                     Use `{APPLICATIONS_CREATE}` with these values.\n\n",
                ));
            }

            content.push_str(&format!(
                "## Step 2: Register Event Types\n\n\
                 Event types define what kinds of events your service will emit.\n\
                 Use naming convention: `service.resource.action`\n\
                 Examples:\n\
                 - `order.payment.completed`\n\
                 - `user.account.created`\n\
                 - `inventory.item.updated`\n\n\
                 Use `{EVENT_TYPES_CREATE}` for each event type you need.\n\n\
                 ## Step 3: Create Your First Subscription\n\n\
                 A subscription connects events to a webhook endpoint.\n\
                 You'll need:\n\
                 - A target URL (your webhook receiver)\n\
                 - Event types to subscribe to\n\n\
                 Use `{SUBSCRIPTIONS_CREATE}` to set up webhook delivery.\n\n\
                 ## Step 4: Test the Integration\n\n\
                 Use `{EVENTS_INGEST}` to send a test event and verify delivery.",
            ));

            Ok(
                GetPromptResult::new(vec![PromptMessage::new_text(Role::User, content)])
                    .with_description("Set up a new Hook0 application"),
            )
        }

        _ => Err(McpError::invalid_params(
            format!("Unknown prompt: {}", name),
            None,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{GENERATED_TOOLS, get_tool_info};

    /// What a prompt argument is filled with when a case wants the branch that reads it. The value
    /// itself is never checked; only the branch it opens is.
    const A_VALUE: &str = "00000000-0000-0000-0000-000000000000";

    /// Every prompt of this module, rendered once with no argument and once with every argument it
    /// declares, so a branch that only appears when an argument is present is rendered too.
    ///
    /// The prompts and their arguments are read from `list_prompts` rather than written down, so a
    /// prompt or an argument added there is covered by the cases below without being added twice.
    fn every_rendering() -> Vec<String> {
        let mut rendered = Vec::new();

        for prompt in list_prompts() {
            let filled: HashMap<String, String> = prompt
                .arguments
                .unwrap_or_default()
                .iter()
                .map(|argument| (argument.name.clone(), A_VALUE.to_owned()))
                .collect();

            for arguments in [None, Some(filled)] {
                let result = get_prompt(&prompt.name, arguments)
                    .unwrap_or_else(|e| panic!("`{}` renders: {e}", prompt.name));
                for message in result.messages {
                    let text = message
                        .content
                        .as_text()
                        .unwrap_or_else(|| panic!("`{}` renders text", prompt.name));
                    rendered.push(text.text.clone());
                }
            }
        }

        assert!(!rendered.is_empty(), "no prompt rendered anything");
        rendered
    }

    /// Every tool the prompts name is one this server exposes.
    ///
    /// This is the guard that was missing while the prompts named eleven tools that had been
    /// renamed out from under them. The failure it catches is functional and silent: an assistant
    /// follows the prompt, calls a tool the server does not have, and gets an unknown-tool answer
    /// that nothing in the build ever sees. The names are held against `GENERATED_TOOLS`, which the
    /// generator writes from the API document, so a renamed operation fails here rather than in a
    /// conversation.
    #[test]
    fn every_tool_a_prompt_names_is_one_this_server_exposes() {
        assert!(
            !GENERATED_TOOLS.is_empty(),
            "the generated tool table is empty, so nothing below would be checked"
        );

        for named in CITED_TOOLS {
            assert!(
                get_tool_info(named).is_some(),
                "the prompts send an assistant to `{named}`, which this server does not expose; \
                 the tools it does are in `src/server/generated.rs`",
            );
        }
    }

    /// Every tool the table names reaches the prose, and the prose names no other.
    ///
    /// Without the first half, a tool could be dropped from a prompt and left in the table, and the
    /// case above would keep passing on a name nothing cites. Without the second half, the prose
    /// could grow a name of its own that no constant covers, which is exactly how the eleven dead
    /// names survived: they were written into the strings.
    #[test]
    fn every_tool_named_reaches_a_prompt() {
        let rendered = every_rendering().join("\n");

        for named in CITED_TOOLS {
            assert!(
                rendered.contains(named),
                "`{named}` is in the table of cited tools but no prompt names it",
            );
        }

        for tool in GENERATED_TOOLS {
            if CITED_TOOLS.contains(&tool.name) {
                continue;
            }
            assert!(
                !rendered.contains(tool.name),
                "a prompt names `{}` without going through the table of cited tools, so nothing \
                 would catch it being renamed",
                tool.name,
            );
        }
    }
}
