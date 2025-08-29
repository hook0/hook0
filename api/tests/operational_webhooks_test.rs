#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_webhook_signature_generation_and_verification() {
        let secret = Uuid::new_v4();
        let timestamp = Utc::now();
        let payload = b"test payload";

        // Generate signature
        let signature = OperationalWebhookDelivery::generate_signature(&secret, &timestamp, payload);

        // The signature should start with "v1,"
        assert!(signature.starts_with("v1,"));

        // Verify the signature
        let timestamp_header = timestamp.timestamp().to_string();
        let is_valid = OperationalWebhookDelivery::verify_signature(
            &secret,
            &signature,
            &timestamp_header,
            payload,
        );

        assert!(is_valid, "Signature verification should succeed");
    }

    #[test]
    fn test_webhook_signature_with_invalid_secret() {
        let secret = Uuid::new_v4();
        let wrong_secret = Uuid::new_v4();
        let timestamp = Utc::now();
        let payload = b"test payload";

        // Generate signature with one secret
        let signature = OperationalWebhookDelivery::generate_signature(&secret, &timestamp, payload);

        // Try to verify with a different secret
        let timestamp_header = timestamp.timestamp().to_string();
        let is_valid = OperationalWebhookDelivery::verify_signature(
            &wrong_secret,
            &signature,
            &timestamp_header,
            payload,
        );

        assert!(!is_valid, "Signature verification should fail with wrong secret");
    }

    #[test]
    fn test_webhook_signature_with_expired_timestamp() {
        let secret = Uuid::new_v4();
        let old_timestamp = Utc::now() - chrono::Duration::minutes(10);
        let payload = b"test payload";

        // Generate signature with old timestamp
        let signature = OperationalWebhookDelivery::generate_signature(&secret, &old_timestamp, payload);

        // Try to verify with expired timestamp
        let timestamp_header = old_timestamp.timestamp().to_string();
        let is_valid = OperationalWebhookDelivery::verify_signature(
            &secret,
            &signature,
            &timestamp_header,
            payload,
        );

        assert!(!is_valid, "Signature verification should fail with expired timestamp");
    }

    #[test]
    fn test_webhook_signature_with_tampered_payload() {
        let secret = Uuid::new_v4();
        let timestamp = Utc::now();
        let original_payload = b"original payload";
        let tampered_payload = b"tampered payload";

        // Generate signature with original payload
        let signature = OperationalWebhookDelivery::generate_signature(&secret, &timestamp, original_payload);

        // Try to verify with tampered payload
        let timestamp_header = timestamp.timestamp().to_string();
        let is_valid = OperationalWebhookDelivery::verify_signature(
            &secret,
            &signature,
            &timestamp_header,
            tampered_payload,
        );

        assert!(!is_valid, "Signature verification should fail with tampered payload");
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let payload = WebhookPayload {
            event_type: "endpoint.created".to_string(),
            timestamp: Utc::now(),
            data: serde_json::json!({
                "endpoint_id": Uuid::new_v4(),
                "url": "https://example.com/webhook"
            }),
        };

        let serialized = serde_json::to_string(&payload).expect("Should serialize webhook payload");
        assert!(serialized.contains("endpoint.created"));
        assert!(serialized.contains("timestamp"));
        assert!(serialized.contains("data"));

        let deserialized: WebhookPayload = serde_json::from_str(&serialized)
            .expect("Should deserialize webhook payload");
        assert_eq!(deserialized.event_type, payload.event_type);
    }
}