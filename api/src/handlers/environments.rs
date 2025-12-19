use std::collections::HashMap;

use clap::CommandFactory;
use paperclip::actix::web::Json;
use paperclip::actix::{Apiv2Schema, api_v2_operation};
use serde::Serialize;

use crate::problems::Hook0Problem;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct EnvVarMetadata {
    pub name: String,
    pub env_var: String,
    pub description: Option<String>,
    pub default: Option<String>,
    pub sensitive: bool,
    pub required: bool,
    pub group: Option<String>,
}

/// Get environment variables metadata
#[api_v2_operation(
    summary = "Get environment variables metadata",
    description = "Returns metadata for all environment variables read by the API",
    operation_id = "environments.get",
    consumes = "application/json",
    produces = "application/json",
    tags("Hook0")
)]
pub async fn get() -> Result<Json<Vec<EnvVarMetadata>>, Hook0Problem> {
    let cmd = crate::Config::command();
    let groups = get_arg_groups(&cmd);

    let metadata: Vec<EnvVarMetadata> = cmd
        .get_arguments()
        .filter_map(|arg| {
            arg.get_env().map(|env_os| {
                let arg_id = arg.get_id().as_str();

                EnvVarMetadata {
                    name: arg_id.to_string(),
                    env_var: env_os.to_string_lossy().to_string(),
                    description: arg.get_help().map(|h| h.to_string()),
                    default: arg
                        .get_default_values()
                        .first()
                        .map(|v| v.to_string_lossy().to_string()),
                    sensitive: arg.is_hide_env_values_set(),
                    required: arg.is_required_set(),
                    group: find_group_for_arg(&groups, arg_id),
                }
            })
        })
        .collect();

    Ok(Json(metadata))
}

fn get_arg_groups(cmd: &clap::Command) -> HashMap<String, Vec<String>> {
    cmd.get_groups()
        .map(|g| {
            (
                g.get_id().to_string(),
                g.get_args().map(|a| a.to_string()).collect(),
            )
        })
        .collect()
}

fn find_group_for_arg(groups: &HashMap<String, Vec<String>>, arg_id: &str) -> Option<String> {
    groups
        .iter()
        .find(|(_, args)| args.contains(&arg_id.to_string()))
        .map(|(name, _)| name.clone())
}
