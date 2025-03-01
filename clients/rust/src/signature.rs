use hmac::{Hmac, Mac};
use http::HeaderName;
use log::trace;
use sha2::Sha256;
use std::collections::HashMap;
use std::str::FromStr;

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

    pub fn verify(&self, payload: &[u8], ordered_header_values: &[String], secret: &str) -> bool {
        let timestamp_str = self.timestamp.to_string();
        let timestamp_str_bytes = timestamp_str.as_bytes();

        type HmacSha256 = Hmac<Sha256>;

        if let Some(v1) = self.v1.as_ref() {
            trace!("Verifying v1 signature...");
            let header_values = ordered_header_values.join(Self::PAYLOAD_SEPARATOR);

            let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap(); // MAC can take key of any size; this should never fail
            mac.update(timestamp_str_bytes);
            mac.update(Self::PAYLOAD_SEPARATOR_BYTES);
            mac.update(
                self.h
                    .join(Self::SIGNATURE_PART_HEADER_NAMES_SEPARATOR)
                    .as_bytes(),
            );
            mac.update(Self::PAYLOAD_SEPARATOR_BYTES);
            mac.update(header_values.as_bytes());
            mac.update(Self::PAYLOAD_SEPARATOR_BYTES);
            mac.update(payload);
            mac.verify_slice(v1).is_ok()
        } else if let Some(v0) = self.v0.as_ref() {
            trace!("Verifying v0 signature...");
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap(); // MAC can take key of any size; this should never fail
            mac.update(timestamp_str_bytes);
            mac.update(Self::PAYLOAD_SEPARATOR_BYTES);
            mac.update(payload);
            mac.verify_slice(v0).is_ok()
        } else {
            // This cannot happen because this error would be raised while parsing the signature
            trace!("Failed to decode signature: no v0 nor v1 field");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_signature_v0() {
        let signature = Signature::parse("t=123,v0=abcd").unwrap();
        assert_eq!(signature.timestamp, 123);
        assert_eq!(signature.v0, Some(hex::decode("abcd").unwrap()));
        assert_eq!(signature.h, Vec::<HeaderName>::new());
        assert_eq!(signature.v1, None);
    }

    #[test]
    fn parse_signature_v0_invalid_timestamp() {
        let signature = Signature::parse("t=error,v0=def");
        assert!(signature.is_err());
    }

    #[test]
    fn parse_signature_missing_signature_field() {
        let signature = Signature::parse("t=error,h=x-test,foo=bar");
        assert!(signature.is_err());
    }

    #[test]
    fn parse_signature_v1() {
        let signature = Signature::parse("t=123,h=x-test x-test2,v1=1234").unwrap();
        assert_eq!(signature.timestamp, 123);
        assert_eq!(signature.v0, None);
        assert_eq!(
            signature.h,
            vec![
                HeaderName::from_static("x-test"),
                HeaderName::from_static("x-test2")
            ]
        );
        assert_eq!(signature.v1, Some(hex::decode("1234").unwrap()));
    }

    #[test]
    fn parse_signature_v0_v1() {
        let signature = Signature::parse("t=123,v0=abcd,h=x-test x-test2,v1=1234").unwrap();
        assert_eq!(signature.timestamp, 123);
        assert_eq!(signature.v0, Some(hex::decode("abcd").unwrap()));
        assert_eq!(
            signature.h,
            vec![
                HeaderName::from_static("x-test"),
                HeaderName::from_static("x-test2")
            ]
        );
        assert_eq!(signature.v1, Some(hex::decode("1234").unwrap()));
    }

    #[test]
    fn verify_signature_v0_valid() {
        let signature = Signature {
            timestamp: 1636936200,
            v0: Some(
                hex::decode("1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98")
                    .unwrap(),
            ),
            h: Vec::new(),
            v1: None,
        };
        let payload = "hello !".as_bytes();
        let secret = "secret";
        assert!(signature.verify(payload, &[], secret));
    }

    #[test]
    fn verify_signature_v0_invalid() {
        let signature = Signature {
            timestamp: 1636936200,
            v0: Some(
                hex::decode("1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98")
                    .unwrap(),
            ),
            h: Vec::new(),
            v1: None,
        };
        let payload = "hello !".as_bytes();
        let secret = "another secret";
        assert!(!signature.verify(payload, &[], secret));
    }

    #[test]
    fn parse_and_verify_signature_v0() {
        let signature = Signature::parse(
            "t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98",
        )
        .unwrap();
        let payload = "hello !".as_bytes();
        let secret = "secret";
        assert!(signature.verify(payload, &[], secret));
    }

    #[test]
    fn verify_signature_v1_valid() {
        let signature = Signature {
            timestamp: 1636936200,
            v0: None,
            h: vec![
                HeaderName::from_static("x-test"),
                HeaderName::from_static("x-test2"),
            ],
            v1: Some(
                hex::decode("493c35f05443fdb74cb99fd4f00e0e7653c2ab6b24fbc97f4a7bd4d56b31758a")
                    .unwrap(),
            ),
        };
        let payload = "hello !".as_bytes();
        let header_values = vec!["val1".to_owned(), "val2".to_owned()];
        let secret = "secret";
        assert!(signature.verify(payload, &header_values, secret));
    }
}
