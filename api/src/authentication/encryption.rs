use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use ring::rand::{SecureRandom, SystemRandom};
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

use super::config::EncryptedSecret;

/// Service for encrypting and decrypting secrets
#[allow(dead_code)]
pub struct SecretEncryption {
    master_key: Vec<u8>,
    db_pool: PgPool,
}

#[allow(dead_code)]
impl SecretEncryption {
    /// Create a new SecretEncryption service
    pub fn new(db_pool: PgPool) -> Result<Self> {
        // Get master key from environment
        let master_key_b64 = env::var("HOOK0_ENCRYPTION_KEY")
            .map_err(|_| anyhow!("HOOK0_ENCRYPTION_KEY environment variable not set"))?;

        let master_key = BASE64
            .decode(&master_key_b64)
            .map_err(|e| anyhow!("Failed to decode master key: {}", e))?;

        if master_key.len() != 32 {
            return Err(anyhow!("Master key must be 32 bytes (256 bits)"));
        }

        Ok(Self {
            master_key,
            db_pool,
        })
    }

    /// Generate a new encryption key for production use
    pub fn generate_master_key() -> String {
        let rng = SystemRandom::new();
        let mut key = vec![0u8; 32];
        rng.fill(&mut key).expect("Failed to generate random key");
        BASE64.encode(&key)
    }

    /// Encrypt a secret value
    pub fn encrypt(&self, plaintext: &str) -> Result<(String, String)> {
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);

        // Generate random nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt the plaintext
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Encode both nonce and ciphertext as base64
        let nonce_b64 = BASE64.encode(nonce);
        let ciphertext_b64 = BASE64.encode(&ciphertext);

        Ok((ciphertext_b64, nonce_b64))
    }

    /// Decrypt a secret value
    pub fn decrypt(&self, ciphertext_b64: &str, nonce_b64: &str) -> Result<String> {
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);

        // Decode from base64
        let nonce_bytes = BASE64
            .decode(nonce_b64)
            .map_err(|e| anyhow!("Failed to decode nonce: {}", e))?;

        let ciphertext = BASE64
            .decode(ciphertext_b64)
            .map_err(|e| anyhow!("Failed to decode ciphertext: {}", e))?;

        // Create nonce from bytes
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| anyhow!("Failed to convert decrypted bytes to string: {}", e))
    }

    /// Resolve a secret value (handles both env:// and encrypted values)
    pub async fn resolve_secret(&self, value: &str, application_id: &Uuid) -> Result<String> {
        if let Some(var_name) = value.strip_prefix("env://") {
            // Environment variable reference
            env::var(var_name)
                .map_err(|e| anyhow!("Failed to get environment variable {}: {}", var_name, e))
        } else if let Some(secret_name) = value.strip_prefix("encrypted://") {
            // Encrypted value stored in database
            self.get_encrypted_secret(application_id, secret_name).await
        } else {
            // Plain text value (for non-sensitive config)
            Ok(value.to_string())
        }
    }

    /// Store an encrypted secret in the database
    pub async fn store_encrypted_secret(
        &self,
        application_id: &Uuid,
        name: &str,
        value: &str,
        metadata: Option<serde_json::Value>,
    ) -> Result<Uuid> {
        let (encrypted_value, nonce) = self.encrypt(value)?;

        let result = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO auth.encrypted_secret (
                application__id,
                name,
                encrypted_value,
                nonce,
                metadata
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (application__id, name)
            DO UPDATE SET
                encrypted_value = EXCLUDED.encrypted_value,
                nonce = EXCLUDED.nonce,
                metadata = EXCLUDED.metadata,
                updated_at = NOW(),
                rotated_at = NOW()
            RETURNING encrypted_secret__id
            "#,
        )
        .bind(application_id)
        .bind(name)
        .bind(encrypted_value)
        .bind(nonce)
        .bind(metadata)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(result)
    }

    /// Get and decrypt a secret from the database
    pub async fn get_encrypted_secret(&self, application_id: &Uuid, name: &str) -> Result<String> {
        let secret = sqlx::query_as::<_, EncryptedSecret>(
            r#"
            SELECT 
                encrypted_secret__id,
                application__id,
                name,
                encrypted_value,
                nonce,
                metadata,
                created_at,
                updated_at,
                rotated_at
            FROM auth.encrypted_secret
            WHERE application__id = $1 AND name = $2
            "#,
        )
        .bind(application_id)
        .bind(name)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| anyhow!("Secret not found: {}", e))?;

        self.decrypt(&secret.encrypted_value, &secret.nonce)
    }

    /// Rotate a secret (update with new value)
    pub async fn rotate_secret(
        &self,
        application_id: &Uuid,
        name: &str,
        new_value: &str,
    ) -> Result<()> {
        let (encrypted_value, nonce) = self.encrypt(new_value)?;

        sqlx::query(
            r#"
            UPDATE auth.encrypted_secret
            SET 
                encrypted_value = $3,
                nonce = $4,
                updated_at = NOW(),
                rotated_at = NOW()
            WHERE application__id = $1 AND name = $2
            "#,
        )
        .bind(application_id)
        .bind(name)
        .bind(encrypted_value)
        .bind(nonce)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Delete a secret
    pub async fn delete_secret(&self, application_id: &Uuid, name: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM auth.encrypted_secret
            WHERE application__id = $1 AND name = $2
            "#,
        )
        .bind(application_id)
        .bind(name)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

// Tests are commented out as they require database connection
// TODO: Implement proper unit tests with mocked database
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_master_key() {
        let key1 = SecretEncryption::generate_master_key();
        let key2 = SecretEncryption::generate_master_key();

        // Keys should be different
        assert_ne!(key1, key2);

        // Keys should be valid base64
        let decoded = BASE64.decode(&key1).unwrap();
        assert_eq!(decoded.len(), 32);
    }
}
