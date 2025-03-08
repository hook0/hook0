use log::debug;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::net::IpAddr;

use crate::problems::Hook0Problem;

const VERIFICATION_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

pub async fn verify(
    cloudflare_turnstile_secret_key: &str,
    turnstile_token: Option<&str>,
    user_ip: &IpAddr,
) -> Result<(), Hook0Problem> {
    if let Some(token) = turnstile_token {
        #[derive(Debug, Clone, Deserialize)]
        struct SiteVerifyResponse {
            success: bool,
        }

        let site_verify_response = Client::new()
            .post(VERIFICATION_URL)
            .json(&json!({
                "secret": cloudflare_turnstile_secret_key,
                "response": token,
                "remoteip": user_ip.to_string(),
            }))
            .send()
            .await
            .map_err(|_| Hook0Problem::Forbidden)?
            .error_for_status()
            .map_err(|_| Hook0Problem::Forbidden)?
            .json::<SiteVerifyResponse>()
            .await
            .map_err(|_| Hook0Problem::Forbidden)?;

        if site_verify_response.success {
            debug!("Request was successfully verified using Cloudflare Turnstile");
            Ok(())
        } else {
            Err(Hook0Problem::Forbidden)
        }
    } else {
        Err(Hook0Problem::Forbidden)
    }
}
