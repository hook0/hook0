//! Bucket-lifecycle responsibility: population, closing, and retention cleanup.
//!
//! The bucket aggregation logic itself lives in
//! [`crate::health_monitor::queries`] (`upsert_buckets`, `close_full_buckets`).
//! This sub-module currently hosts no production code; its sibling test file
//! [`super::bucket_tests`] contains the black-box integration tests that
//! exercise that logic through the top-level
//! [`super::fetch_subscription_health_stats`] orchestrator.
