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

/// Validate that a token has the correct format
pub fn is_valid_token(token: &str) -> bool {
    // Must start with prefix
    if !token.starts_with(TOKEN_PREFIX) {
        return false;
    }

    let random_part = &token[TOKEN_PREFIX.len()..];

    // Must have correct length
    if random_part.len() != TOKEN_RANDOM_LENGTH {
        return false;
    }

    // Must only contain base62 characters
    random_part.chars().all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_format() {
        let token = generate_token();
        assert!(token.starts_with(TOKEN_PREFIX));
        assert_eq!(token.len(), TOKEN_PREFIX.len() + TOKEN_RANDOM_LENGTH);
    }

    #[test]
    fn test_generate_token_uniqueness() {
        let tokens: Vec<String> = (0..100).map(|_| generate_token()).collect();
        let unique: std::collections::HashSet<_> = tokens.iter().collect();
        assert_eq!(tokens.len(), unique.len());
    }

    #[test]
    fn test_is_valid_token() {
        let token = generate_token();
        assert!(is_valid_token(&token));

        assert!(!is_valid_token(""));
        assert!(!is_valid_token("abc"));
        assert!(!is_valid_token("c_tooshort"));
        assert!(!is_valid_token("x_123456789012345678901234567"));
        assert!(!is_valid_token("c_123456789012345678901234567!")); // invalid char
    }
}
