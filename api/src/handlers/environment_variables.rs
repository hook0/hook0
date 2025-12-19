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
    operation_id = "environment_variables.get",
    produces = "application/json",
    tags("Hook0")
)]
pub async fn get() -> Result<Json<Vec<EnvVarMetadata>>, Hook0Problem> {
    let cmd = crate::Config::command();
    let arg_to_group = build_arg_to_group_map(&cmd);

    let metadata: Vec<EnvVarMetadata> = cmd
        .get_arguments()
        .filter_map(|arg| {
            arg.get_env().map(|env_os| {
                let arg_id = arg.get_id().as_str();
                let sensitive = arg.is_hide_env_values_set();

                EnvVarMetadata {
                    name: arg_id.to_string(),
                    env_var: env_os.to_string_lossy().to_string(),
                    description: arg.get_help().map(|h| h.to_string()),
                    default: if sensitive {
                        None
                    } else {
                        arg.get_default_values()
                            .first()
                            .map(|v| v.to_string_lossy().to_string())
                    },
                    sensitive,
                    required: arg.is_required_set(),
                    group: arg_to_group.get(arg_id).cloned(),
                }
            })
        })
        .collect();

    Ok(Json(metadata))
}

fn build_arg_to_group_map(cmd: &clap::Command) -> HashMap<String, String> {
    cmd.get_groups()
        .flat_map(|g| {
            let group_name = g.get_id().to_string();
            g.get_args()
                .map(move |a| (a.to_string(), group_name.clone()))
        })
        .collect()
}
