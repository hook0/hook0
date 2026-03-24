//! In-process rate limiter using sliding window counters

use dashmap::DashMap;
use std::time::{Duration, Instant};

/// A sliding window rate limiter
#[derive(Debug)]
pub struct RateLimiter {
    /// Map of key -> (request timestamps within the window)
    windows: DashMap<String, Vec<Instant>>,
    /// Window duration
    window: Duration,
    /// Max requests per window
    max_requests: u32,
}

impl RateLimiter {
    pub fn new(window: Duration, max_requests: u32) -> Self {
        Self {
            windows: DashMap::new(),
            window,
            max_requests,
        }
    }

    /// Check if a request is allowed and record it if so.
    /// Returns Ok(()) if allowed, Err(remaining_seconds) if rate limited.
    pub fn check(&self, key: &str) -> Result<(), u64> {
        let now = Instant::now();
        let mut entry = self.windows.entry(key.to_string()).or_default();
        let timestamps = entry.value_mut();

        // Remove expired timestamps
        timestamps.retain(|t| now.duration_since(*t) < self.window);

        if timestamps.len() >= self.max_requests as usize {
            // Rate limited - calculate retry-after
            let oldest = timestamps.first().expect("timestamps is non-empty");
            let retry_after = self
                .window
                .checked_sub(now.duration_since(*oldest))
                .unwrap_or(Duration::ZERO);
            return Err(retry_after.as_secs().max(1));
        }

        timestamps.push(now);
        Ok(())
    }

    /// Cleanup old entries to prevent memory leaks
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.windows.retain(|_, timestamps| {
            timestamps.retain(|t| now.duration_since(*t) < self.window);
            !timestamps.is_empty()
        });
    }
}

/// Tracks invalid token attempts per IP for anti-enumeration
#[derive(Debug)]
pub struct InvalidTokenTracker {
    /// IP -> (count, window_start)
    attempts: DashMap<String, (u32, Instant)>,
    /// Window duration (reset after this)
    window: Duration,
    /// Max invalid attempts per window before blocking
    max_attempts: u32,
    /// Block duration after exceeding limit
    block_duration: Duration,
}

impl InvalidTokenTracker {
    pub fn new(window: Duration, max_attempts: u32, block_duration: Duration) -> Self {
        Self {
            attempts: DashMap::new(),
            window,
            max_attempts,
            block_duration,
        }
    }

    /// Check if an IP is allowed to make another attempt.
    /// Returns true if allowed, false if blocked.
    pub fn check_allowed(&self, ip: &str) -> bool {
        let now = Instant::now();
        let mut entry = self.attempts.entry(ip.to_string()).or_insert((0, now));
        let (count, window_start) = entry.value_mut();

        // If window expired, reset
        if now.duration_since(*window_start) > self.window + self.block_duration {
            *count = 0;
            *window_start = now;
            return true;
        }

        // If already over limit and still within block period
        if *count >= self.max_attempts {
            let elapsed = now.duration_since(*window_start);
            if elapsed < self.window + self.block_duration {
                return false;
            }
            // Block period expired, reset
            *count = 0;
            *window_start = now;
            return true;
        }

        true
    }

    /// Record an invalid token attempt
    pub fn record_invalid(&self, ip: &str) {
        let now = Instant::now();
        let mut entry = self.attempts.entry(ip.to_string()).or_insert((0, now));
        let (count, window_start) = entry.value_mut();

        // Reset if window expired
        if now.duration_since(*window_start) > self.window + self.block_duration {
            *count = 1;
            *window_start = now;
        } else {
            *count += 1;
        }
    }

    /// Cleanup old entries
    pub fn cleanup(&self) {
        let now = Instant::now();
        let expiry = self.window + self.block_duration;
        self.attempts
            .retain(|_, (_, start)| now.duration_since(*start) < expiry);
    }
}
