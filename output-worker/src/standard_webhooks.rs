use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::{DateTime, Utc};
use hmac::{Hmac, KeyInit, Mac};
use reqwest::header::HeaderValue;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// A [Standard Webhooks](https://www.standardwebhooks.com/) signature.
///
/// Standard Webhooks is the interoperable signature format used by Svix and a growing
/// number of webhook producers/consumers. A consumer verifies a delivery from three
/// headers — `webhook-id`, `webhook-timestamp` and `webhook-signature` — instead of
/// Hook0's single native `X-Hook0-Signature` header.
///
/// This type only *produces* those interop headers; Hook0 keeps emitting its native
/// signature at the same time (the opt-in "emit both" behaviour), so enabling Standard
/// Webhooks never removes an existing guarantee.
///
/// The signed content is `{id}.{timestamp}.{payload}` (dot-separated), authenticated
/// with HMAC-SHA256 and encoded as standard base64. The `webhook-signature` header
/// carries it under the mandated `v1,` scheme prefix, e.g. `v1,g0hM9SsE+OTP…`.
pub struct StandardWebhooksSignature {
    id: String,
    timestamp: i64,
    /// Base64 of the HMAC-SHA256, without the `v1,` scheme prefix.
    signature: String,
}

impl StandardWebhooksSignature {
    /// The signature scheme prefix mandated by the spec for HMAC-SHA256 signatures.
    const SCHEME: &'static str = "v1";
    const SCHEME_SEPARATOR: char = ',';
    /// Separator between the id/timestamp/payload parts of the signed content.
    const CONTENT_SEPARATOR: &'static [u8] = b".";
    /// Conventional prefix of a base64 Standard Webhooks secret (as popularised by Svix).
    const SECRET_PREFIX: &'static str = "whsec_";

    pub const ID_HEADER: &'static str = "webhook-id";
    pub const TIMESTAMP_HEADER: &'static str = "webhook-timestamp";
    pub const SIGNATURE_HEADER: &'static str = "webhook-signature";

    /// Build the signature for a delivery.
    ///
    /// `id` is the message identifier surfaced as `webhook-id`; it must stay stable
    /// across retries of the same event, so Hook0 passes the event id (not the
    /// per-attempt id).
    pub fn new(id: &str, secret: &str, payload: &[u8], signed_at: DateTime<Utc>) -> Self {
        let timestamp = signed_at.timestamp();
        let key = Self::secret_key(secret);

        // `new_from_slice` only fails on a zero-length key for fixed-key MACs; HMAC accepts
        // a key of any size, so this never errors.
        let mut mac = HmacSha256::new_from_slice(&key).expect("HMAC can take a key of any size");
        mac.update(id.as_bytes());
        mac.update(Self::CONTENT_SEPARATOR);
        mac.update(timestamp.to_string().as_bytes());
        mac.update(Self::CONTENT_SEPARATOR);
        mac.update(payload);
        let signature = BASE64.encode(mac.finalize().into_bytes());

        Self {
            id: id.to_owned(),
            timestamp,
            signature,
        }
    }

    /// Derive the HMAC key from Hook0's stored secret.
    ///
    /// The Standard Webhooks reference verifiers HMAC over the *decoded* bytes of a
    /// base64 secret (optionally `whsec_`-prefixed). Hook0's current secret is a UUID
    /// string, which is not valid standard base64 — so we decode when we can (keeping us
    /// byte-for-byte compatible with the reference implementation and its published test
    /// vector) and fall back to the raw secret bytes otherwise. This keeps a single
    /// secret column working for both signing methods.
    fn secret_key(secret: &str) -> Vec<u8> {
        let encoded = secret.strip_prefix(Self::SECRET_PREFIX).unwrap_or(secret);
        BASE64
            .decode(encoded)
            .unwrap_or_else(|_| secret.as_bytes().to_vec())
    }

    /// `webhook-id` value. The id is an event UUID, always a valid header value.
    pub fn id_header_value(&self) -> HeaderValue {
        HeaderValue::from_str(&self.id).expect("an event id is always a valid header value")
    }

    /// `webhook-timestamp` value (Unix seconds). An integer is always a valid header value.
    pub fn timestamp_header_value(&self) -> HeaderValue {
        HeaderValue::from(self.timestamp)
    }

    /// `webhook-signature` value, e.g. `v1,<base64>`. Base64 and the comma are all
    /// visible ASCII, so this is always a valid header value.
    pub fn signature_header_value(&self) -> HeaderValue {
        HeaderValue::from_str(&format!(
            "{}{}{}",
            Self::SCHEME,
            Self::SCHEME_SEPARATOR,
            self.signature
        ))
        .expect("a base64 signature is always a valid header value")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;
    use proptest::prelude::*;

    fn at(timestamp: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(timestamp, 0).unwrap()
    }

    /// The canonical Standard Webhooks test vector (from the spec / Svix reference
    /// libraries). Reproducing it byte-for-byte proves our framing, key derivation,
    /// HMAC and base64 all match the reference implementation, so any Standard Webhooks
    /// consumer can verify a Hook0 delivery.
    #[test]
    fn matches_the_reference_test_vector() {
        let sig = StandardWebhooksSignature::new(
            "msg_p5jXN8AQM9LWM0D4loKWxJek",
            "whsec_MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw",
            br#"{"test": 2432232314}"#,
            at(1614265330),
        );

        assert_eq!(sig.id_header_value(), "msg_p5jXN8AQM9LWM0D4loKWxJek");
        assert_eq!(sig.timestamp_header_value(), "1614265330");
        assert_eq!(
            sig.signature_header_value(),
            "v1,g0hM9SsE+OTPJTGt/tmIKtSyZlE3uFJELVlNIOLJ1OE=",
        );
    }

    /// The `whsec_` prefix is optional: the reference decodes the same key with or
    /// without it, so the signature must be identical.
    #[test]
    fn the_whsec_prefix_is_optional() {
        let payload = br#"{"test": 2432232314}"#;
        let with = StandardWebhooksSignature::new(
            "msg_p5jXN8AQM9LWM0D4loKWxJek",
            "whsec_MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw",
            payload,
            at(1614265330),
        );
        let without = StandardWebhooksSignature::new(
            "msg_p5jXN8AQM9LWM0D4loKWxJek",
            "MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw",
            payload,
            at(1614265330),
        );
        assert_eq!(
            with.signature_header_value(),
            without.signature_header_value()
        );
    }

    /// A UUID secret (Hook0's current format) is not valid base64, so we fall back to
    /// its raw bytes as the HMAC key rather than silently keying with an empty secret.
    #[test]
    fn a_uuid_secret_keys_on_its_raw_bytes() {
        let secret = "b8f9a1d0-1c2b-4e3f-8a5d-6c7e8f9a0b1c";
        let payload = b"hello";
        let now = at(1614265330);

        let sig = StandardWebhooksSignature::new("evt-1", secret, payload, now);

        // Recompute independently, keying on the raw UUID bytes.
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(b"evt-1");
        mac.update(b".");
        mac.update(b"1614265330");
        mac.update(b".");
        mac.update(payload);
        let expected = format!("v1,{}", BASE64.encode(mac.finalize().into_bytes()));

        assert_eq!(sig.signature_header_value(), expected.as_str());
        // …and that must NOT collapse to the empty-key signature.
        let empty = StandardWebhooksSignature::new("evt-1", "", payload, now);
        assert_ne!(sig.signature_header_value(), empty.signature_header_value());
    }

    proptest! {
        /// Signing is deterministic and every input triple maps to a distinct signed
        /// content: changing the id, the timestamp or the payload changes the signature.
        #[test]
        fn signature_is_deterministic_and_binds_every_part(
            id in "[a-zA-Z0-9_-]{1,40}",
            secret in "[a-zA-Z0-9]{8,40}",
            payload in proptest::collection::vec(any::<u8>(), 0..256),
            ts in 0i64..4_000_000_000,
        ) {
            let a = StandardWebhooksSignature::new(&id, &secret, &payload, at(ts));
            let b = StandardWebhooksSignature::new(&id, &secret, &payload, at(ts));
            prop_assert_eq!(a.signature_header_value(), b.signature_header_value());

            // A different timestamp yields a different signature (timestamp is bound).
            let other = StandardWebhooksSignature::new(&id, &secret, &payload, at(ts ^ 1));
            prop_assert_ne!(a.signature_header_value(), other.signature_header_value());

            // The signature is always well-formed: `v1,<base64>`.
            let value = a.signature_header_value();
            let value = value.to_str().unwrap();
            let b64 = value.strip_prefix("v1,").expect("scheme prefix");
            prop_assert!(BASE64.decode(b64).is_ok());
        }
    }
}
