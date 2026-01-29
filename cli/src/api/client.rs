use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use thiserror::Error;
use uuid::Uuid;

use super::models::*;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication failed: invalid or expired token")]
    Unauthorized,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Hook0 API client
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    secret: String,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: &str, secret: &str) -> Self {
        let client = Client::builder()
            .user_agent(format!("hook0-cli/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            secret: secret.to_string(),
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a URL with path
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Handle API response
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T, ApiError> {
        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED => {
                let body = response.text().await?;
                serde_json::from_str(&body).map_err(|e| {
                    ApiError::UnexpectedResponse(format!("Failed to parse response: {e}, body: {body}"))
                })
            }
            StatusCode::NO_CONTENT => {
                // Return empty for no content responses
                serde_json::from_str("null").map_err(|e| {
                    ApiError::UnexpectedResponse(format!("Failed to handle no content: {e}"))
                })
            }
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(ApiError::Unauthorized)
            }
            StatusCode::NOT_FOUND => {
                let body = response.text().await.unwrap_or_default();
                Err(ApiError::NotFound(body))
            }
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                let body = response.text().await.unwrap_or_default();
                Err(ApiError::ValidationError(body))
            }
            _ => {
                let body = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(format!("Status {status}: {body}")))
            }
        }
    }

    /// Handle API response that might return empty
    async fn handle_response_optional<T: DeserializeOwned>(&self, response: Response) -> Result<Option<T>, ApiError> {
        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED => {
                let body = response.text().await?;
                if body.is_empty() || body == "null" {
                    Ok(None)
                } else {
                    serde_json::from_str(&body)
                        .map(Some)
                        .map_err(|e| ApiError::UnexpectedResponse(format!("Failed to parse: {e}")))
                }
            }
            StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => Ok(None),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(ApiError::Unauthorized),
            _ => {
                let body = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(format!("Status {status}: {body}")))
            }
        }
    }

    // =========================================================================
    // Application endpoints
    // =========================================================================

    /// Get current application (verify authentication)
    pub async fn get_current_application(&self, application_id: &Uuid) -> Result<Application, ApiError> {
        let response = self
            .client
            .get(self.url(&format!("/applications/{}", application_id)))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// List applications for an organization
    pub async fn list_applications(&self, organization_id: &Uuid) -> Result<Vec<Application>, ApiError> {
        let response = self
            .client
            .get(self.url("/applications"))
            .bearer_auth(&self.secret)
            .query(&[("organization_id", organization_id.to_string())])
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get an application by ID
    pub async fn get_application(&self, application_id: &Uuid) -> Result<Application, ApiError> {
        let response = self
            .client
            .get(self.url(&format!("/applications/{}", application_id)))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create a new application
    pub async fn create_application(&self, app: &ApplicationPost) -> Result<Application, ApiError> {
        let response = self
            .client
            .post(self.url("/applications"))
            .bearer_auth(&self.secret)
            .json(app)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // =========================================================================
    // Organization endpoints
    // =========================================================================

    /// List organizations
    pub async fn list_organizations(&self) -> Result<Vec<Organization>, ApiError> {
        let response = self
            .client
            .get(self.url("/organizations"))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get an organization by ID
    pub async fn get_organization(&self, organization_id: &Uuid) -> Result<Organization, ApiError> {
        let response = self
            .client
            .get(self.url(&format!("/organizations/{}", organization_id)))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // =========================================================================
    // Event Type endpoints
    // =========================================================================

    /// List event types for an application
    pub async fn list_event_types(&self, application_id: &Uuid) -> Result<Vec<EventType>, ApiError> {
        let response = self
            .client
            .get(self.url("/event_types"))
            .bearer_auth(&self.secret)
            .query(&[("application_id", application_id.to_string())])
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create an event type
    pub async fn create_event_type(&self, event_type: &EventTypePost) -> Result<EventType, ApiError> {
        let response = self
            .client
            .post(self.url("/event_types"))
            .bearer_auth(&self.secret)
            .json(event_type)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Delete an event type
    pub async fn delete_event_type(
        &self,
        application_id: &Uuid,
        service: &str,
        resource_type: &str,
        verb: &str,
    ) -> Result<(), ApiError> {
        let response = self
            .client
            .delete(self.url("/event_types"))
            .query(&[
                ("application_id", application_id.to_string()),
                ("service", service.to_string()),
                ("resource_type", resource_type.to_string()),
                ("verb", verb.to_string()),
            ])
            .bearer_auth(&self.secret)
            .send()
            .await?;

        if response.status() == StatusCode::NO_CONTENT || response.status() == StatusCode::OK {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::ServerError(format!("Status {status}: {body}")))
        }
    }

    // =========================================================================
    // Event endpoints
    // =========================================================================

    /// List events for an application
    pub async fn list_events(
        &self,
        application_id: &Uuid,
        filters: &EventFilters,
        pagination: &PaginationParams,
    ) -> Result<Vec<Event>, ApiError> {
        let mut query: Vec<(&str, String)> = vec![("application_id", application_id.to_string())];
        query.extend(filters.to_query_params());
        query.extend(pagination.to_query_params());

        let response = self
            .client
            .get(self.url("/events"))
            .bearer_auth(&self.secret)
            .query(&query)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get an event by ID
    pub async fn get_event(&self, event_id: &Uuid) -> Result<Event, ApiError> {
        let response = self
            .client
            .get(self.url(&format!("/events/{}", event_id)))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Send an event
    pub async fn send_event(&self, event: &EventPost) -> Result<Event, ApiError> {
        let response = self
            .client
            .post(self.url("/events"))
            .bearer_auth(&self.secret)
            .json(event)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // =========================================================================
    // Subscription endpoints
    // =========================================================================

    /// List subscriptions for an application
    pub async fn list_subscriptions(
        &self,
        application_id: &Uuid,
        labels: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<Subscription>, ApiError> {
        let mut query: Vec<(&str, String)> = vec![("application_id", application_id.to_string())];
        for (key, value) in labels {
            query.push(("label", format!("{}={}", key, value)));
        }

        let response = self
            .client
            .get(self.url("/subscriptions"))
            .bearer_auth(&self.secret)
            .query(&query)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get a subscription by ID
    pub async fn get_subscription(&self, subscription_id: &Uuid) -> Result<Subscription, ApiError> {
        let response = self
            .client
            .get(self.url(&format!("/subscriptions/{}", subscription_id)))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create a subscription
    pub async fn create_subscription(&self, subscription: &SubscriptionPost) -> Result<Subscription, ApiError> {
        let response = self
            .client
            .post(self.url("/subscriptions"))
            .bearer_auth(&self.secret)
            .json(subscription)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Update a subscription
    pub async fn update_subscription(
        &self,
        subscription_id: &Uuid,
        subscription: &SubscriptionPut,
    ) -> Result<Subscription, ApiError> {
        let response = self
            .client
            .put(self.url(&format!("/subscriptions/{}", subscription_id)))
            .bearer_auth(&self.secret)
            .json(subscription)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Delete a subscription
    pub async fn delete_subscription(&self, subscription_id: &Uuid) -> Result<(), ApiError> {
        let response = self
            .client
            .delete(self.url(&format!("/subscriptions/{}", subscription_id)))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        if response.status() == StatusCode::NO_CONTENT || response.status() == StatusCode::OK {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::ServerError(format!("Status {status}: {body}")))
        }
    }

    /// Enable a subscription
    pub async fn enable_subscription(&self, subscription_id: &Uuid) -> Result<Subscription, ApiError> {
        let sub = self.get_subscription(subscription_id).await?;
        let update = SubscriptionPut {
            event_types: sub.event_types,
            is_enabled: true,
            description: sub.description,
            labels: Some(sub.labels),
            metadata: Some(sub.metadata),
            target: sub.target,
            dedicated_workers: Some(sub.dedicated_workers),
        };
        self.update_subscription(subscription_id, &update).await
    }

    /// Disable a subscription
    pub async fn disable_subscription(&self, subscription_id: &Uuid) -> Result<Subscription, ApiError> {
        let sub = self.get_subscription(subscription_id).await?;
        let update = SubscriptionPut {
            event_types: sub.event_types,
            is_enabled: false,
            description: sub.description,
            labels: Some(sub.labels),
            metadata: Some(sub.metadata),
            target: sub.target,
            dedicated_workers: Some(sub.dedicated_workers),
        };
        self.update_subscription(subscription_id, &update).await
    }

    // =========================================================================
    // Request Attempt endpoints
    // =========================================================================

    /// List request attempts for an event
    pub async fn list_request_attempts(
        &self,
        application_id: &Uuid,
        event_id: Option<&Uuid>,
    ) -> Result<Vec<RequestAttempt>, ApiError> {
        let mut query: Vec<(&str, String)> = vec![("application_id", application_id.to_string())];
        if let Some(eid) = event_id {
            query.push(("event_id", eid.to_string()));
        }

        let response = self
            .client
            .get(self.url("/request_attempts"))
            .bearer_auth(&self.secret)
            .query(&query)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get a request attempt by ID
    pub async fn get_request_attempt(&self, request_attempt_id: &Uuid) -> Result<RequestAttempt, ApiError> {
        let response = self
            .client
            .get(self.url(&format!("/request_attempts/{}", request_attempt_id)))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Replay an event (create new request attempts)
    pub async fn replay_event(&self, event_id: &Uuid) -> Result<Vec<RequestAttempt>, ApiError> {
        let response = self
            .client
            .post(self.url(&format!("/events/{}/replay", event_id)))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // =========================================================================
    // Application Secret endpoints
    // =========================================================================

    /// List application secrets
    pub async fn list_application_secrets(&self, application_id: &Uuid) -> Result<Vec<ApplicationSecret>, ApiError> {
        let response = self
            .client
            .get(self.url("/application_secrets"))
            .bearer_auth(&self.secret)
            .query(&[("application_id", application_id.to_string())])
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Create an application secret
    pub async fn create_application_secret(&self, secret: &ApplicationSecretPost) -> Result<ApplicationSecret, ApiError> {
        let response = self
            .client
            .post(self.url("/application_secrets"))
            .bearer_auth(&self.secret)
            .json(secret)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Delete an application secret
    pub async fn delete_application_secret(&self, application_id: &Uuid, token: &Uuid) -> Result<(), ApiError> {
        let response = self
            .client
            .delete(self.url("/application_secrets"))
            .query(&[
                ("application_id", application_id.to_string()),
                ("token", token.to_string()),
            ])
            .bearer_auth(&self.secret)
            .send()
            .await?;

        if response.status() == StatusCode::NO_CONTENT || response.status() == StatusCode::OK {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(ApiError::ServerError(format!("Status {status}: {body}")))
        }
    }

    // =========================================================================
    // Response endpoints
    // =========================================================================

    /// Get a response by ID
    pub async fn get_response(&self, response_id: &Uuid) -> Result<Option<super::models::Response>, ApiError> {
        let response = self
            .client
            .get(self.url(&format!("/responses/{}", response_id)))
            .bearer_auth(&self.secret)
            .send()
            .await?;

        self.handle_response_optional(response).await
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_url_building() {
        let client = ApiClient::new("https://api.hook0.com/v1/", "secret123");
        assert_eq!(client.base_url(), "https://api.hook0.com/v1");
        assert_eq!(client.url("/events"), "https://api.hook0.com/v1/events");
    }

}
