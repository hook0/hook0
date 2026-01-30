//! Token generation for webhook URLs

use rand::Rng;

/// Base62 alphabet for URL-safe tokens
const BASE62_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Token prefix
const TOKEN_PREFIX: &str = "c_";

/// Length of the random part of the token
const TOKEN_RANDOM_LENGTH: usize = 27;

/// Generate a new random token for webhook URLs
/// Format: c_<27-char-base62-string>
pub fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    let random_part: String = (0..TOKEN_RANDOM_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..BASE62_ALPHABET.len());
            BASE62_ALPHABET[idx] as char
        })
        .collect();

    format!("{}{}", TOKEN_PREFIX, random_part)
}
