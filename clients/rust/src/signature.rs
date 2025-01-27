use hmac::{Hmac, Mac};
use lazy_regex::regex_captures;
use sha2::Sha256;
use std::str::FromStr;

pub struct Signature {
    pub timestamp: i64,
    pub v0: String,
}

impl Signature {
    const PAYLOAD_SEPARATOR: &'static [u8] = b".";

    pub fn parse(signature: &str) -> Result<Self, ()> {
        let captures = regex_captures!("^t=([0-9]+),v0=([a-f0-9]+)$"i, signature);
        if let Some((_, timestamp, v0)) = captures {
            Ok(Self {
                timestamp: i64::from_str(timestamp).map_err(|_| ())?,
                v0: v0.to_owned(),
            })
        } else {
            Err(())
        }
    }

    pub fn compare(&self, payload: &[u8], secret: &str) -> bool {
        let timestamp_str = self.timestamp.to_string();
        let timestamp_str_bytes = timestamp_str.as_bytes();

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap(); // MAC can take key of any size; this should never fail
        mac.update(timestamp_str_bytes);
        mac.update(Self::PAYLOAD_SEPARATOR);
        mac.update(payload);

        match hex::decode(&self.v0) {
            Ok(decoded_signature) => mac.verify_slice(&decoded_signature).is_ok(),
            Err(_) => false, // If decoding fails, the signature is invalid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_successful_signature() {
        let signature = Signature::parse("t=123,v0=abc");
        assert!(signature.is_ok());
        let signature = signature.unwrap();
        assert_eq!(signature.timestamp, 123);
        assert_eq!(signature.v0, "abc");
    }

    #[test]
    fn parsing_failed_signature() {
        let signature = Signature::parse("t=error,v0=def");
        assert!(signature.is_err());
    }

    #[test]
    fn comparison_successful() {
        let signature = Signature {
            timestamp: 1636936200,
            v0: "1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98".to_owned(),
        };
        let payload = "hello !".as_bytes();
        let secret = "secret";
        assert!(signature.compare(payload, secret));
    }

    #[test]
    fn comparison_failed() {
        let signature = Signature {
            timestamp: 1636936200,
            v0: "1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98".to_owned(),
        };
        let payload = "hello !".as_bytes();
        let secret = "another secret";
        assert!(!signature.compare(payload, secret));
    }

    #[test]
    fn parsing_and_comparison_successful() {
        let signature = Signature::parse("t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98").unwrap();
        let payload = "hello !".as_bytes();
        let secret = "secret";
        assert!(signature.compare(payload, secret));
    }

    #[test]
    fn parsing_and_comparison_failed() {
        let signature = Signature::parse("t=1636936200,v0=1b3d69df55f1e52f05224ba94a5162abeb17ef52cd7f4948c390f810d6a87e98").unwrap();
        let payload = "hello !".as_bytes();
        let secret = "another secret";
        assert!(!signature.compare(payload, secret));
    }
}
