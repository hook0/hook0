//! Token generation for webhook URLs

use rand::prelude::IndexedRandom;

/// Base62 alphabet for URL-safe tokens
const BASE62_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

/// Token prefix
const TOKEN_PREFIX: &str = "c_";

/// Length of the random part of the token
const TOKEN_RANDOM_LENGTH: usize = 27;

/// Generate a new random token for webhook URLs.
///
/// Format: `c_<27-char-base62-string>`
///
/// # Example
/// ```
/// let token = hook0_cli::tunnel::generate_token();
/// assert!(token.starts_with("c_"));
/// assert_eq!(token.len(), 29); // "c_" + 27 chars
/// ```
pub fn generate_token() -> String {
    let mut rng = rand::rng();
    let random_part: String = (0..TOKEN_RANDOM_LENGTH)
        .map(|_| {
            // SAFETY: BASE62_ALPHABET is a non-empty constant slice, choose always returns Some
            let &byte = BASE62_ALPHABET
                .choose(&mut rng)
                .expect("BASE62_ALPHABET is non-empty");
            byte as char
        })
        .collect();

    format!("{TOKEN_PREFIX}{random_part}")
}
