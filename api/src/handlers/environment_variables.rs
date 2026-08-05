use clap::CommandFactory;
use paperclip::actix::web::Json;
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

use crate::problems::Hook0Problem;

#[derive(Debug, Clone, Serialize, Apiv2Schema)]
pub struct EnvVarMetadata {
    pub name: String,
    pub env_var: String,
    pub description: Option<String>,
    pub default: Option<String>,
    pub sensitive: bool,
    pub required: bool,
    pub group: Option<String>,
}

static ENV_VAR_METADATA: LazyLock<Vec<EnvVarMetadata>> = LazyLock::new(|| {
    let group_regex = Regex::new(r"^\[(.+?)\] (.+)$").unwrap();
    let cmd = crate::Config::command();

    cmd.get_arguments()
        .filter_map(|arg| {
            arg.get_env().map(|env_os| {
                let arg_id = arg.get_id().as_str();
                let full_description = arg.get_help().map(|h| h.to_string());

                let group_captures =
                    group_regex.captures(full_description.as_deref().unwrap_or_default());

                let group = group_captures
                    .as_ref()
                    .and_then(|c| c.get(1))
                    .map(|c| c.as_str().to_owned());
                let description = group_captures
                    .as_ref()
                    .and_then(|c| c.get(2))
                    .map(|c| c.as_str().to_owned())
                    .or(full_description);

                EnvVarMetadata {
                    name: arg_id.to_string(),
                    env_var: env_os.to_string_lossy().to_string(),
                    description,
                    default: arg
                        .get_default_values()
                        .first()
                        .map(|v| v.to_string_lossy().to_string()),
                    sensitive: arg.is_hide_env_values_set(),
                    required: arg.is_required_set(),
                    group,
                }
            })
        })
        .collect()
});

#[api_v2_operation(
    summary = "List environment variables metadata",
    description = "Returns metadata for all environment variables read by the API",
    operation_id = "environment_variables.list",
    produces = "application/json",
    tags("Hook0")
)]
pub async fn get() -> Result<Json<Vec<EnvVarMetadata>>, Hook0Problem> {
    Ok(Json(ENV_VAR_METADATA.clone()))
}

#[cfg(test)]
mod tests {
    use super::ENV_VAR_METADATA;

    #[test]
    fn bracketed_help_groups_on_first_bracket() {
        let entry = ENV_VAR_METADATA
            .iter()
            .find(|m| m.env_var == "SIGNUP_ATTRIBUTION_RETENTION_IN_DAYS")
            .expect("SIGNUP_ATTRIBUTION_RETENTION_IN_DAYS must be present in env var metadata");

        // This var's help text is "[Google Ads] … Bounded to [1, 3650]": a greedy first capture
        // group swallows up to the last `]`, yielding a malformed group and a description
        // truncated to "days". The group must be just the leading bracketed category.
        assert_eq!(
            entry.group.as_deref(),
            Some("Google Ads"),
            "group should be the leading bracketed category, not the greedy-captured help text",
        );

        // The full description must be preserved (the upper bound "3650" lives near the end),
        // proving the split kept everything after the first `]` rather than the last one.
        let description = entry
            .description
            .as_deref()
            .expect("description should be present");
        assert!(
            description.contains("3650"),
            "description should retain the full help text including the upper bound, got: {description}",
        );
    }
}
