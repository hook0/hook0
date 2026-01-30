//! Header and input sanitization

use std::collections::HashMap;
use thiserror::Error;

/// Maximum number of headers allowed
const MAX_HEADERS: usize = 100;
/// Maximum size of a single header (name + value) in bytes
const MAX_HEADER_SIZE: usize = 8192; // 8KB
/// Maximum total size of all headers in bytes
const MAX_TOTAL_HEADERS_SIZE: usize = 65536; // 64KB

#[derive(Debug, Error)]
pub enum SanitizeError {
    #[error("Too many headers: {count} (max: {max})")]
    TooManyHeaders { count: usize, max: usize },

    #[error("Header too large: {name} ({size} bytes, max: {max} bytes)")]
    HeaderTooLarge {
        name: String,
        size: usize,
        max: usize,
    },

    #[error("Total headers too large: {size} bytes (max: {max} bytes)")]
    TotalHeadersTooLarge { size: usize, max: usize },
}

/// Sanitize and validate HTTP headers
///
/// - Validates header names contain only valid HTTP header characters (RFC 7230)
/// - Limits the number of headers
/// - Limits individual header size
/// - Limits total headers size
pub fn sanitize_headers(
    headers: HashMap<String, String>,
) -> Result<HashMap<String, String>, SanitizeError> {
    if headers.len() > MAX_HEADERS {
        return Err(SanitizeError::TooManyHeaders {
            count: headers.len(),
            max: MAX_HEADERS,
        });
    }

    let mut sanitized = HashMap::with_capacity(headers.len());
    let mut total_size: usize = 0;

    for (name, value) in headers {
        // Validate header name (RFC 7230: token characters)
        if !is_valid_header_name(&name) {
            // Skip headers with invalid names instead of rejecting the whole request
            continue;
        }

        let size = name.len() + value.len();
        if size > MAX_HEADER_SIZE {
            return Err(SanitizeError::HeaderTooLarge {
                name,
                size,
                max: MAX_HEADER_SIZE,
            });
        }

        total_size += size;
        if total_size > MAX_TOTAL_HEADERS_SIZE {
            return Err(SanitizeError::TotalHeadersTooLarge {
                size: total_size,
                max: MAX_TOTAL_HEADERS_SIZE,
            });
        }

        sanitized.insert(name, value);
    }

    Ok(sanitized)
}

/// Validate that a header name contains only valid HTTP token characters (RFC 7230)
fn is_valid_header_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    name.bytes().all(is_tchar)
}

/// Check if a byte is a valid HTTP token character (RFC 7230 Section 3.2.6)
/// tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*" / "+" / "-" / "." /
///         "^" / "_" / "`" / "|" / "~" / DIGIT / ALPHA
fn is_tchar(b: u8) -> bool {
    matches!(b,
        b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'.' |
        b'^' | b'_' | b'`' | b'|' | b'~' |
        b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z'
    )
}

/// Validate an HTTP status code is within valid range
pub fn is_valid_status_code(status: u16) -> bool {
    (100..=599).contains(&status)
}
