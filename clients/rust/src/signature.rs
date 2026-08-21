use hmac::{Hmac, KeyInit, Mac};
use http::HeaderName;
use sha2::Sha256;
use std::collections::HashMap;
use std::str::FromStr;
use tracing::trace;

use crate::Hook0ClientError;

pub struct Signature {
    pub timestamp: i64,
    pub v0: Option<Vec<u8>>,
    pub h: Vec<HeaderName>,
    pub v1: Option<Vec<u8>>,
}

impl Signature {
    const PAYLOAD_SEPARATOR: &'static str = ".";
    const PAYLOAD_SEPARATOR_BYTES: &'static [u8] = Self::PAYLOAD_SEPARATOR.as_bytes();
    const SIGNATURE_PART_ASSIGNATOR: char = '=';
    const SIGNATURE_PART_SEPARATOR: char = ',';
    const SIGNATURE_PART_HEADER_NAMES_SEPARATOR: &'static str = " ";

    pub fn parse(signature: &str) -> Result<Self, Hook0ClientError> {
        let parts = signature
            .split(Self::SIGNATURE_PART_SEPARATOR)
            .flat_map(|part| {
                part.split_once(Self::SIGNATURE_PART_ASSIGNATOR)
                    .map(|(k, v)| vec![(k.trim(), v.trim())])
                    .unwrap_or_default()
            })
            .collect::<HashMap<_, _>>();

        if parts.len() >= 2 {
            let t = parts.get("t").copied().ok_or_else(|| {
                Hook0ClientError::SignatureHeaderParsing("Missing 't' field".to_owned())
            })?;
            let timestamp =
                i64::from_str(t).map_err(|error| Hook0ClientError::TimestampParsing {
                    timestamp: t.to_owned(),
                    error,
                })?;

            let v0 = match parts.get("v0").copied() {
                Some(v0_str) => Some(hex::decode(v0_str).map_err(|error| {
                    Hook0ClientError::V0SignatureParsing {
                        signature: v0_str.to_owned(),
                        error,
                    }
                })?),
                None => None,
            };

            let h = match parts.get("h").copied() {
                Some(h_str) => h_str
                    .split(' ')
                    .map(|h| {
                        HeaderName::from_str(h).map_err(|error| {
                            Hook0ClientError::HeaderNameParsing {
                                header: h.to_owned(),
                                error,
                            }
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                None => Vec::new(),
            };

            let v1 = match parts.get("v1").copied() {
                Some(v1_str) => Some(hex::decode(v1_str).map_err(|error| {
                    Hook0ClientError::V1SignatureParsing {
                        signature: v1_str.to_owned(),
                        error,
                    }
                })?),
                None => None,
            };

            if v0.is_none() && v1.is_none() {
                Err(Hook0ClientError::SignatureHeaderParsing(
                    "There must be at least one of 'v0' or 'v1' field".to_owned(),
                ))
            } else {
                Ok(Self {
                    timestamp,
                    v0,
                    h,
                    v1,
                })
            }
        } else {
            Err(Hook0ClientError::SignatureHeaderParsing(
                "Signature header format is invalid".to_owned(),
            ))
        }
    }

    /// Whether the signature was produced over that payload with that secret.
    ///
    /// Keying the MAC is fallible in the type system even though HMAC accepts a key of any size, and
    /// a verifier that panics on it takes down the webhook handler it was called from. The refusal
    /// is returned instead, as the same [`Hook0ClientError::InvalidSignature`] a signature that does
    /// not verify answers with: a secret this client cannot key an HMAC with is one it cannot
    /// establish anything about the webhook under, so it establishes nothing.
    pub fn verify(
        &self,
        payload: &[u8],
        ordered_header_values: &[String],
        secret: &str,
    ) -> Result<bool, Hook0ClientError> {
        let timestamp_str = self.timestamp.to_string();
        let timestamp_str_bytes = timestamp_str.as_bytes();

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| Hook0ClientError::InvalidSignature)?;
        mac.update(timestamp_str_bytes);
        mac.update(Self::PAYLOAD_SEPARATOR_BYTES);

        if let Some(v1) = self.v1.as_ref() {
            trace!("Verifying v1 signature...");

            mac.update(
                self.h
                    .join(Self::SIGNATURE_PART_HEADER_NAMES_SEPARATOR)
                    .as_bytes(),
            );
            mac.update(Self::PAYLOAD_SEPARATOR_BYTES);
            mac.update(
                ordered_header_values
                    .join(Self::PAYLOAD_SEPARATOR)
                    .as_bytes(),
            );
            mac.update(Self::PAYLOAD_SEPARATOR_BYTES);
            mac.update(payload);
            Ok(mac.verify_slice(v1).is_ok())
        } else if let Some(v0) = self.v0.as_ref() {
            trace!("Verifying v0 signature...");

            mac.update(payload);
            Ok(mac.verify_slice(v0).is_ok())
        } else {
            // This cannot happen because this error would be raised while parsing the signature
            trace!("Failed to decode signature: no v0 nor v1 field");
            Ok(false)
        }
    }
}
