use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::{Certificate, Identity, Request};
use std::sync::Arc;
use uuid::Uuid;

use crate::authentication::{
    config::{AuthenticationType, CertificateConfig},
    encryption::SecretEncryption,
};

use super::AuthenticationProvider;

/// Certificate authentication provider
#[allow(dead_code)]
pub struct CertificateProvider {
    config: CertificateConfig,
    application_id: Uuid,
    encryption: Arc<SecretEncryption>,
    identity: Option<Identity>,
    ca_cert: Option<Certificate>,
}

#[allow(dead_code)]
impl CertificateProvider {
    /// Create a new certificate provider
    pub async fn new(
        config: CertificateConfig,
        application_id: Uuid,
        encryption: Arc<SecretEncryption>,
    ) -> Result<Self> {
        // Load client certificate and key
        let client_cert = encryption
            .resolve_secret(&config.client_cert, &application_id)
            .await?;

        let client_key = encryption
            .resolve_secret(&config.client_key, &application_id)
            .await?;

        // Create identity from certificate and key
        let identity = if config.mtls {
            // Combine cert and key for mTLS
            let pem = format!("{}\n{}", client_cert, client_key);
            Some(
                Identity::from_pem(pem.as_bytes())
                    .map_err(|e| anyhow!("Failed to create identity from PEM: {}", e))?,
            )
        } else {
            None
        };

        // Load CA certificate if provided
        let ca_cert = if let Some(ca_cert_ref) = &config.ca_cert {
            let ca_cert_pem = encryption
                .resolve_secret(ca_cert_ref, &application_id)
                .await?;

            Some(
                Certificate::from_pem(ca_cert_pem.as_bytes())
                    .map_err(|e| anyhow!("Failed to load CA certificate: {}", e))?,
            )
        } else {
            None
        };

        Ok(Self {
            config,
            application_id,
            encryption,
            identity,
            ca_cert,
        })
    }

    /// Get the client identity for mTLS
    pub fn get_identity(&self) -> Option<&Identity> {
        self.identity.as_ref()
    }

    /// Get the CA certificate for server verification
    pub fn get_ca_cert(&self) -> Option<&Certificate> {
        self.ca_cert.as_ref()
    }

    /// Check if hostname verification is enabled
    pub fn verify_hostname(&self) -> bool {
        self.config.verify_hostname
    }
}

#[async_trait]
impl AuthenticationProvider for CertificateProvider {
    async fn authenticate(&self, _request: &mut Request) -> Result<()> {
        // Certificate authentication is handled at the HTTP client level
        // through the Identity and Certificate objects, not through headers
        // This method is a no-op for certificate auth

        // Note: The actual TLS configuration needs to be done when creating
        // the HTTP client, not per-request. The client should be configured
        // with the identity and CA certificate from this provider.

        Ok(())
    }

    async fn refresh_if_needed(&self) -> Result<()> {
        // Certificates don't need refresh in the same way as tokens
        // Certificate rotation would require recreating the provider
        Ok(())
    }

    fn get_type(&self) -> AuthenticationType {
        AuthenticationType::Certificate
    }
}

/// Helper to create an HTTP client with certificate authentication
#[allow(dead_code)]
pub fn create_client_with_certificates(provider: &CertificateProvider) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .danger_accept_invalid_hostnames(!provider.verify_hostname());

    // Add client identity for mTLS
    if let Some(identity) = provider.get_identity() {
        builder = builder.identity(identity.clone());
    }

    // Add CA certificate for server verification
    if let Some(ca_cert) = provider.get_ca_cert() {
        builder = builder.add_root_certificate(ca_cert.clone());
    }

    builder
        .build()
        .map_err(|e| anyhow!("Failed to create HTTP client with certificates: {}", e))
}
