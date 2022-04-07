use hook0_client::Hook0Client;
use log::{info, warn};
use reqwest::Url;
use uuid::Uuid;

pub fn initialize(
    api_url: Option<Url>,
    application_id: Option<Uuid>,
    application_secret: Option<Uuid>,
) -> Option<Hook0Client> {
    match (api_url, application_id, application_secret) {
        (Some(url), Some(id), Some(secret)) => match Hook0Client::new(url, id, &secret) {
            Ok(client) => {
                info!(
                    "Events from this Hook0 instance will be sent to {} [application ID = {}]",
                    client.api_url(),
                    client.application_id()
                );
                Some(client)
            }
            Err(_e) => {
                warn!("Could not initialize a Hook0 client that will receive events from this Hook0 instance");
                None
            }
        },
        _ => {
            info!("No Hook0 client was configured to receive events from this Hook0 instance");
            None
        }
    }
}
