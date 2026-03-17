use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::{Duration, Instant};
use tokio_util::task::TaskTracker;
use tracing::info;

pub struct ThroughputStats {
    processed: AtomicU64,
    succeeded: AtomicU64,
    failed: AtomicU64,
    first_attempts: AtomicU64,
    retries: AtomicU64,
    latency_sum_ms: AtomicU64,
    latency_max_ms: AtomicU64,
    busy_ms_total: AtomicU64,
    db_fetch_sum_ms: AtomicU64,
    db_fetch_count: AtomicU64,
    lag_max_ms: AtomicU64,
    not_ready: AtomicU64,
    total_slots: u16,
}

impl ThroughputStats {
    pub fn new(total_slots: u16) -> Self {
        Self {
            processed: AtomicU64::new(0),
            succeeded: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            first_attempts: AtomicU64::new(0),
            retries: AtomicU64::new(0),
            latency_sum_ms: AtomicU64::new(0),
            latency_max_ms: AtomicU64::new(0),
            busy_ms_total: AtomicU64::new(0),
            db_fetch_sum_ms: AtomicU64::new(0),
            db_fetch_count: AtomicU64::new(0),
            lag_max_ms: AtomicU64::new(0),
            not_ready: AtomicU64::new(0),
            total_slots,
        }
    }

    pub fn record_attempt(&self, succeeded: bool, retry_count: i16, latency: Duration) {
        self.processed.fetch_add(1, Relaxed);
        if succeeded {
            self.succeeded.fetch_add(1, Relaxed);
        } else {
            self.failed.fetch_add(1, Relaxed);
        }
        if retry_count == 0 {
            self.first_attempts.fetch_add(1, Relaxed);
        } else {
            self.retries.fetch_add(1, Relaxed);
        }
        let ms = latency.as_millis() as u64;
        self.latency_sum_ms.fetch_add(ms, Relaxed);
        self.latency_max_ms.fetch_max(ms, Relaxed);
    }

    pub fn record_db_fetch(&self, duration: Duration) {
        self.db_fetch_sum_ms
            .fetch_add(duration.as_millis() as u64, Relaxed);
        self.db_fetch_count.fetch_add(1, Relaxed);
    }

    pub fn record_lag(&self, lag: Duration) {
        self.lag_max_ms.fetch_max(lag.as_millis() as u64, Relaxed);
    }

    pub fn record_not_ready(&self) {
        self.not_ready.fetch_add(1, Relaxed);
    }

    pub fn slot_enter(&self) -> SlotGuard<'_> {
        SlotGuard {
            stats: self,
            entered_at: Instant::now(),
        }
    }

    fn snapshot_and_reset(&self) -> Snapshot {
        Snapshot {
            processed: self.processed.swap(0, Relaxed),
            succeeded: self.succeeded.swap(0, Relaxed),
            failed: self.failed.swap(0, Relaxed),
            first_attempts: self.first_attempts.swap(0, Relaxed),
            retries: self.retries.swap(0, Relaxed),
            latency_sum_ms: self.latency_sum_ms.swap(0, Relaxed),
            latency_max_ms: self.latency_max_ms.swap(0, Relaxed),
            busy_ms_total: self.busy_ms_total.swap(0, Relaxed),
            db_fetch_sum_ms: self.db_fetch_sum_ms.swap(0, Relaxed),
            db_fetch_count: self.db_fetch_count.swap(0, Relaxed),
            lag_max_ms: self.lag_max_ms.swap(0, Relaxed),
            not_ready: self.not_ready.swap(0, Relaxed),
            total_slots: self.total_slots,
        }
    }
}

pub struct SlotGuard<'a> {
    stats: &'a ThroughputStats,
    entered_at: Instant,
}

impl Drop for SlotGuard<'_> {
    fn drop(&mut self) {
        let busy_ms = self.entered_at.elapsed().as_millis() as u64;
        self.stats.busy_ms_total.fetch_add(busy_ms, Relaxed);
    }
}

struct Snapshot {
    processed: u64,
    succeeded: u64,
    failed: u64,
    first_attempts: u64,
    retries: u64,
    latency_sum_ms: u64,
    latency_max_ms: u64,
    busy_ms_total: u64,
    db_fetch_sum_ms: u64,
    db_fetch_count: u64,
    lag_max_ms: u64,
    not_ready: u64,
    total_slots: u16,
}

impl Snapshot {
    fn rate(&self, interval: Duration) -> f64 {
        let secs = interval.as_secs_f64();
        if secs == 0.0 {
            0.0
        } else {
            self.processed as f64 / secs
        }
    }

    fn avg_latency_ms(&self) -> f64 {
        if self.processed == 0 {
            0.0
        } else {
            self.latency_sum_ms as f64 / self.processed as f64
        }
    }

    fn avg_db_fetch_ms(&self) -> f64 {
        if self.db_fetch_count == 0 {
            0.0
        } else {
            self.db_fetch_sum_ms as f64 / self.db_fetch_count as f64
        }
    }

    fn max_lag_s(&self) -> f64 {
        self.lag_max_ms as f64 / 1000.0
    }

    fn avg_busy(&self, interval: Duration) -> f64 {
        let interval_ms = interval.as_millis() as f64;
        if interval_ms == 0.0 {
            0.0
        } else {
            self.busy_ms_total as f64 / interval_ms
        }
    }
}

pub async fn run_throughput_log(
    stats: &ThroughputStats,
    interval: Duration,
    task_tracker: &TaskTracker,
) {
    let mut ticker = tokio::time::interval(interval);
    ticker.tick().await; // skip the first immediate tick
    let mut last_emit = Instant::now();

    loop {
        tokio::select! {
            biased;
            _ = task_tracker.wait() => break,
            _ = ticker.tick() => {
                let window = last_emit.elapsed();
                last_emit = Instant::now();
                emit_snapshot(stats, window);
            }
        }
    }

    // Emit one final snapshot before exiting
    emit_snapshot(stats, last_emit.elapsed());
}

fn emit_snapshot(stats: &ThroughputStats, window: Duration) {
    let snap = stats.snapshot_and_reset();
    info!(
        "throughput: not_ready={} processed={} succeeded={} failed={} first_attempts={} retries={} rate={:.2}/s avg_db_fetch_ms={:.1} max_lag_s={:.1} avg_latency_ms={:.1} max_latency_ms={} avg_busy={:.1} total_slots={}",
        snap.not_ready,
        snap.processed,
        snap.succeeded,
        snap.failed,
        snap.first_attempts,
        snap.retries,
        snap.rate(window),
        snap.avg_db_fetch_ms(),
        snap.max_lag_s(),
        snap.avg_latency_ms(),
        snap.latency_max_ms,
        snap.avg_busy(window),
        snap.total_slots,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_record_attempt_increments() {
        let stats = ThroughputStats::new(5);

        // Record a successful first attempt
        stats.record_attempt(true, 0, Duration::from_millis(100));
        assert_eq!(stats.processed.load(Relaxed), 1);
        assert_eq!(stats.succeeded.load(Relaxed), 1);
        assert_eq!(stats.failed.load(Relaxed), 0);
        assert_eq!(stats.first_attempts.load(Relaxed), 1);
        assert_eq!(stats.retries.load(Relaxed), 0);

        // Record a failed retry
        stats.record_attempt(false, 2, Duration::from_millis(200));
        assert_eq!(stats.processed.load(Relaxed), 2);
        assert_eq!(stats.succeeded.load(Relaxed), 1);
        assert_eq!(stats.failed.load(Relaxed), 1);
        assert_eq!(stats.first_attempts.load(Relaxed), 1);
        assert_eq!(stats.retries.load(Relaxed), 1);

        // Verify latency accumulation
        assert_eq!(stats.latency_sum_ms.load(Relaxed), 300);
        assert_eq!(stats.latency_max_ms.load(Relaxed), 200);
    }

    #[test]
    fn test_snapshot_and_reset() {
        let stats = ThroughputStats::new(5);

        stats.record_attempt(true, 0, Duration::from_millis(100));
        stats.record_attempt(false, 1, Duration::from_millis(200));
        stats.record_db_fetch(Duration::from_millis(10));
        stats.record_db_fetch(Duration::from_millis(30));
        stats.record_lag(Duration::from_millis(500));
        stats.record_lag(Duration::from_millis(800));
        stats.record_not_ready();

        let snap = stats.snapshot_and_reset();

        // Snapshot should contain accumulated values
        assert_eq!(snap.processed, 2);
        assert_eq!(snap.succeeded, 1);
        assert_eq!(snap.failed, 1);
        assert_eq!(snap.first_attempts, 1);
        assert_eq!(snap.retries, 1);
        assert_eq!(snap.latency_sum_ms, 300);
        assert_eq!(snap.latency_max_ms, 200);
        assert_eq!(snap.db_fetch_sum_ms, 40);
        assert_eq!(snap.db_fetch_count, 2);
        assert!(
            (snap.max_lag_s() - 0.8).abs() < 0.001,
            "max_lag_s={}",
            snap.max_lag_s()
        );
        assert_eq!(snap.not_ready, 1);
        assert_eq!(snap.total_slots, 5);

        // After reset, counters should be zero
        assert_eq!(stats.processed.load(Relaxed), 0);
        assert_eq!(stats.succeeded.load(Relaxed), 0);
        assert_eq!(stats.failed.load(Relaxed), 0);
        assert_eq!(stats.first_attempts.load(Relaxed), 0);
        assert_eq!(stats.retries.load(Relaxed), 0);
        assert_eq!(stats.latency_sum_ms.load(Relaxed), 0);
        assert_eq!(stats.latency_max_ms.load(Relaxed), 0);
        assert_eq!(stats.busy_ms_total.load(Relaxed), 0);
        assert_eq!(stats.db_fetch_sum_ms.load(Relaxed), 0);
        assert_eq!(stats.db_fetch_count.load(Relaxed), 0);
        assert_eq!(stats.lag_max_ms.load(Relaxed), 0);
        assert_eq!(stats.not_ready.load(Relaxed), 0);
    }

    #[tokio::test]
    async fn test_slot_guard_accumulates_busy_ms() {
        let stats = ThroughputStats::new(5);

        {
            let _guard = stats.slot_enter();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let busy = stats.busy_ms_total.load(Relaxed);
        assert!(busy >= 40, "Expected busy_ms >= 40, got {busy}");
    }

    #[test]
    fn test_snapshot_derived_values() {
        let stats = ThroughputStats::new(5);

        stats.record_attempt(true, 0, Duration::from_millis(100));
        stats.record_attempt(true, 0, Duration::from_millis(200));
        stats.record_attempt(false, 1, Duration::from_millis(300));

        // Simulate busy time: 3 slots busy for 1000ms each = 3000ms total
        stats.busy_ms_total.store(3000, Relaxed);

        let snap = stats.snapshot_and_reset();
        let interval = Duration::from_secs(60);

        // rate = 3 / 60 = 0.05
        let rate = snap.rate(interval);
        assert!((rate - 0.05).abs() < 0.001, "rate={rate}");

        // avg_latency = 600 / 3 = 200.0
        let avg = snap.avg_latency_ms();
        assert!((avg - 200.0).abs() < 0.001, "avg_latency={avg}");

        // avg_busy = 3000 / 60000 = 0.05
        let busy = snap.avg_busy(interval);
        assert!((busy - 0.05).abs() < 0.001, "avg_busy={busy}");

        // max latency
        assert_eq!(snap.latency_max_ms, 300);
    }

    #[test]
    fn test_snapshot_derived_values_zero_processed() {
        let stats = ThroughputStats::new(5);
        let snap = stats.snapshot_and_reset();
        let interval = Duration::from_secs(60);

        assert_eq!(snap.rate(interval), 0.0);
        assert_eq!(snap.avg_latency_ms(), 0.0);
        assert_eq!(snap.avg_busy(interval), 0.0);
        assert_eq!(snap.avg_db_fetch_ms(), 0.0);
    }

    #[test]
    fn test_record_db_fetch() {
        let stats = ThroughputStats::new(5);

        stats.record_db_fetch(Duration::from_millis(10));
        stats.record_db_fetch(Duration::from_millis(30));
        stats.record_db_fetch(Duration::from_millis(20));

        assert_eq!(stats.db_fetch_sum_ms.load(Relaxed), 60);
        assert_eq!(stats.db_fetch_count.load(Relaxed), 3);

        let snap = stats.snapshot_and_reset();
        let avg = snap.avg_db_fetch_ms();
        assert!((avg - 20.0).abs() < 0.001, "avg_db_fetch_ms={avg}");
    }

    #[test]
    fn test_record_lag_tracks_max() {
        let stats = ThroughputStats::new(5);

        stats.record_lag(Duration::from_millis(100));
        stats.record_lag(Duration::from_millis(500));
        stats.record_lag(Duration::from_millis(200));

        let snap = stats.snapshot_and_reset();
        assert!(
            (snap.max_lag_s() - 0.5).abs() < 0.001,
            "max_lag_s={}",
            snap.max_lag_s()
        );
    }

    #[test]
    fn test_record_not_ready() {
        let stats = ThroughputStats::new(5);

        stats.record_not_ready();
        stats.record_not_ready();
        stats.record_not_ready();

        assert_eq!(stats.not_ready.load(Relaxed), 3);
    }
}
