# FIFO Subscription Monitoring Queries

## Overview

This document provides SQL queries for monitoring and troubleshooting FIFO-enabled subscriptions in Hook0.

## Key Metrics Queries

### 1. List All FIFO Subscriptions

```sql
SELECT
    s.subscription__id,
    s.application__id,
    s.is_enabled,
    s.created_at,
    COUNT(DISTINCT et.event_type__name) as event_types_count
FROM webhook.subscription s
LEFT JOIN webhook.subscription__event_type set ON set.subscription__id = s.subscription__id
LEFT JOIN event.event_type et ON et.event_type__id = set.event_type__id
WHERE s.fifo_mode = true
    AND s.deleted_at IS NULL
GROUP BY s.subscription__id, s.application__id, s.is_enabled, s.created_at
ORDER BY s.created_at DESC;
```

### 2. FIFO Subscriptions Currently Blocked

```sql
SELECT
    s.subscription__id,
    s.application__id,
    fss.current_request_attempt__id,
    ra.retry_count,
    ra.created_at as attempt_created_at,
    ra.picked_at as attempt_picked_at,
    EXTRACT(EPOCH FROM (NOW() - ra.created_at)) as age_seconds,
    EXTRACT(EPOCH FROM (NOW() - fss.updated_at)) as blocked_duration_seconds
FROM webhook.subscription s
INNER JOIN webhook.fifo_subscription_state fss ON fss.subscription__id = s.subscription__id
LEFT JOIN webhook.request_attempt ra ON ra.request_attempt__id = fss.current_request_attempt__id
WHERE s.fifo_mode = true
    AND s.deleted_at IS NULL
    AND fss.current_request_attempt__id IS NOT NULL
ORDER BY blocked_duration_seconds DESC;
```

### 3. Stuck FIFO Subscriptions (Blocked >5 Minutes)

```sql
SELECT
    s.subscription__id,
    s.application__id,
    o.name as organization_name,
    a.name as application_name,
    fss.current_request_attempt__id,
    ra.retry_count,
    ra.created_at as attempt_created_at,
    ra.picked_at as attempt_picked_at,
    ra.delay_until,
    EXTRACT(EPOCH FROM (NOW() - ra.created_at)) as age_seconds,
    EXTRACT(EPOCH FROM (NOW() - fss.updated_at)) as blocked_duration_seconds,
    COUNT(*) FILTER (WHERE ra2.succeeded_at IS NULL AND ra2.failed_at IS NULL) as pending_requests_count
FROM webhook.subscription s
INNER JOIN webhook.fifo_subscription_state fss ON fss.subscription__id = s.subscription__id
INNER JOIN event.application a ON a.application__id = s.application__id
INNER JOIN iam.organization o ON o.organization__id = a.organization__id
LEFT JOIN webhook.request_attempt ra ON ra.request_attempt__id = fss.current_request_attempt__id
LEFT JOIN webhook.request_attempt ra2 ON ra2.subscription__id = s.subscription__id
WHERE s.fifo_mode = true
    AND s.deleted_at IS NULL
    AND fss.current_request_attempt__id IS NOT NULL
    AND EXTRACT(EPOCH FROM (NOW() - fss.updated_at)) > 300  -- blocked for >5 minutes
GROUP BY s.subscription__id, s.application__id, o.name, a.name, fss.current_request_attempt__id,
         ra.retry_count, ra.created_at, ra.picked_at, ra.delay_until, fss.updated_at
ORDER BY blocked_duration_seconds DESC;
```

### 4. FIFO Queue Depth Per Subscription

```sql
SELECT
    s.subscription__id,
    s.application__id,
    COUNT(*) FILTER (WHERE ra.succeeded_at IS NULL AND ra.failed_at IS NULL) as pending_count,
    COUNT(*) FILTER (WHERE ra.succeeded_at IS NULL AND ra.failed_at IS NULL AND ra.delay_until IS NOT NULL) as delayed_count,
    MIN(ra.created_at) FILTER (WHERE ra.succeeded_at IS NULL AND ra.failed_at IS NULL) as oldest_pending,
    MAX(ra.created_at) FILTER (WHERE ra.succeeded_at IS NULL AND ra.failed_at IS NULL) as newest_pending,
    fss.current_request_attempt__id IS NOT NULL as is_blocked
FROM webhook.subscription s
LEFT JOIN webhook.fifo_subscription_state fss ON fss.subscription__id = s.subscription__id
LEFT JOIN webhook.request_attempt ra ON ra.subscription__id = s.subscription__id
WHERE s.fifo_mode = true
    AND s.deleted_at IS NULL
GROUP BY s.subscription__id, s.application__id, fss.current_request_attempt__id
HAVING COUNT(*) FILTER (WHERE ra.succeeded_at IS NULL AND ra.failed_at IS NULL) > 0
ORDER BY pending_count DESC;
```

### 5. FIFO Completion Statistics (Last 24 Hours)

```sql
SELECT
    s.subscription__id,
    COUNT(*) FILTER (WHERE ra.succeeded_at IS NOT NULL) as successful_count,
    COUNT(*) FILTER (WHERE ra.failed_at IS NOT NULL AND ra.retry_count >= 30) as failed_count,
    AVG(EXTRACT(EPOCH FROM (ra.succeeded_at - ra.created_at))) FILTER (WHERE ra.succeeded_at IS NOT NULL) as avg_success_time_seconds,
    MAX(ra.retry_count) FILTER (WHERE ra.succeeded_at IS NOT NULL OR ra.failed_at IS NOT NULL) as max_retries_used,
    fss.last_completed_event_created_at as last_completed_event
FROM webhook.subscription s
LEFT JOIN webhook.fifo_subscription_state fss ON fss.subscription__id = s.subscription__id
LEFT JOIN webhook.request_attempt ra ON ra.subscription__id = s.subscription__id
WHERE s.fifo_mode = true
    AND s.deleted_at IS NULL
    AND ra.created_at >= NOW() - INTERVAL '24 hours'
    AND (ra.succeeded_at IS NOT NULL OR ra.failed_at IS NOT NULL)
GROUP BY s.subscription__id, fss.last_completed_event_created_at
ORDER BY successful_count + failed_count DESC;
```

## Troubleshooting Queries

### 6. Identify Orphaned FIFO States

Detects FIFO states pointing to non-existent or completed request attempts:

```sql
SELECT
    fss.subscription__id,
    fss.current_request_attempt__id,
    fss.updated_at,
    EXTRACT(EPOCH FROM (NOW() - fss.updated_at)) as age_seconds,
    ra.succeeded_at,
    ra.failed_at,
    ra.picked_at
FROM webhook.fifo_subscription_state fss
LEFT JOIN webhook.request_attempt ra ON ra.request_attempt__id = fss.current_request_attempt__id
WHERE fss.current_request_attempt__id IS NOT NULL
    AND (
        ra.request_attempt__id IS NULL  -- Request attempt doesn't exist
        OR ra.succeeded_at IS NOT NULL  -- Already succeeded
        OR ra.failed_at IS NOT NULL     -- Already failed
        OR (ra.picked_at IS NOT NULL AND ra.picked_at < NOW() - INTERVAL '10 minutes')  -- Picked but not completed for >10 minutes
    );
```

### 7. Cleanup Orphaned FIFO States

**USE WITH CAUTION** - This clears orphaned FIFO states:

```sql
UPDATE webhook.fifo_subscription_state fss
SET current_request_attempt__id = NULL, updated_at = statement_timestamp()
FROM webhook.request_attempt ra
WHERE fss.current_request_attempt__id = ra.request_attempt__id
    AND (
        ra.succeeded_at IS NOT NULL
        OR ra.failed_at IS NOT NULL
        OR (ra.picked_at IS NOT NULL AND ra.picked_at < NOW() - INTERVAL '10 minutes')
    );

-- Also clear states with non-existent request attempts
UPDATE webhook.fifo_subscription_state fss
SET current_request_attempt__id = NULL, updated_at = statement_timestamp()
WHERE fss.current_request_attempt__id IS NOT NULL
    AND NOT EXISTS (
        SELECT 1 FROM webhook.request_attempt ra
        WHERE ra.request_attempt__id = fss.current_request_attempt__id
    );
```

### 8. Manual FIFO Unblock

**USE WITH EXTREME CAUTION** - Manually unblock a specific subscription:

```sql
-- First, check current state
SELECT
    fss.*,
    ra.request_attempt__id,
    ra.retry_count,
    ra.succeeded_at,
    ra.failed_at,
    ra.picked_at
FROM webhook.fifo_subscription_state fss
LEFT JOIN webhook.request_attempt ra ON ra.request_attempt__id = fss.current_request_attempt__id
WHERE fss.subscription__id = 'SUBSCRIPTION_ID_HERE';

-- Then, unblock if necessary
UPDATE webhook.fifo_subscription_state
SET
    current_request_attempt__id = NULL,
    updated_at = statement_timestamp()
WHERE subscription__id = 'SUBSCRIPTION_ID_HERE';
```

## Performance Monitoring

### 9. FIFO vs Non-FIFO Throughput Comparison

```sql
WITH stats AS (
    SELECT
        s.fifo_mode,
        COUNT(*) as total_attempts,
        COUNT(*) FILTER (WHERE ra.succeeded_at IS NOT NULL) as successful_attempts,
        AVG(EXTRACT(EPOCH FROM (COALESCE(ra.succeeded_at, ra.failed_at) - ra.created_at))) as avg_completion_time,
        PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (COALESCE(ra.succeeded_at, ra.failed_at) - ra.created_at))) as p50_completion_time,
        PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (COALESCE(ra.succeeded_at, ra.failed_at) - ra.created_at))) as p95_completion_time
    FROM webhook.subscription s
    INNER JOIN webhook.request_attempt ra ON ra.subscription__id = s.subscription__id
    WHERE s.deleted_at IS NULL
        AND ra.created_at >= NOW() - INTERVAL '24 hours'
        AND (ra.succeeded_at IS NOT NULL OR ra.failed_at IS NOT NULL)
    GROUP BY s.fifo_mode
)
SELECT
    CASE WHEN fifo_mode THEN 'FIFO' ELSE 'Non-FIFO' END as mode,
    total_attempts,
    successful_attempts,
    ROUND(successful_attempts::numeric / NULLIF(total_attempts, 0) * 100, 2) as success_rate_pct,
    ROUND(avg_completion_time::numeric, 2) as avg_completion_seconds,
    ROUND(p50_completion_time::numeric, 2) as p50_completion_seconds,
    ROUND(p95_completion_time::numeric, 2) as p95_completion_seconds
FROM stats
ORDER BY fifo_mode DESC;
```

### 10. FIFO Retry Pattern Analysis

```sql
SELECT
    s.subscription__id,
    COUNT(*) as total_attempts,
    AVG(ra.retry_count) as avg_retries,
    MAX(ra.retry_count) as max_retries,
    COUNT(*) FILTER (WHERE ra.retry_count = 0) as first_attempt_count,
    COUNT(*) FILTER (WHERE ra.retry_count > 0 AND ra.retry_count <= 5) as fast_retry_count,
    COUNT(*) FILTER (WHERE ra.retry_count > 5) as slow_retry_count,
    COUNT(*) FILTER (WHERE ra.succeeded_at IS NOT NULL) as successful_count,
    COUNT(*) FILTER (WHERE ra.failed_at IS NOT NULL) as failed_count
FROM webhook.subscription s
INNER JOIN webhook.request_attempt ra ON ra.subscription__id = s.subscription__id
WHERE s.fifo_mode = true
    AND s.deleted_at IS NULL
    AND ra.created_at >= NOW() - INTERVAL '7 days'
GROUP BY s.subscription__id
ORDER BY total_attempts DESC;
```

## Alerting Thresholds

### Recommended Alert Conditions

1. **Stuck Subscription Alert**
   - Condition: FIFO subscription blocked for >10 minutes
   - Severity: WARNING
   - Query: Use Query #3 with threshold of 600 seconds

2. **High Queue Depth Alert**
   - Condition: Pending request count >100 for a FIFO subscription
   - Severity: WARNING
   - Query: Use Query #4 with threshold of 100

3. **Orphaned State Alert**
   - Condition: Any orphaned FIFO states detected
   - Severity: ERROR
   - Query: Use Query #6

4. **Low Success Rate Alert**
   - Condition: FIFO subscription success rate <80% over 24 hours
   - Severity: WARNING
   - Query: Use Query #5 with success rate calculation

## Operational Procedures

### Daily Monitoring Checklist

1. Run Query #2 to check for blocked subscriptions
2. Run Query #6 to check for orphaned states
3. Run Query #9 to compare FIFO vs non-FIFO performance
4. Review any subscriptions blocked >5 minutes (Query #3)

### Weekly Review

1. Run Query #5 to analyze FIFO completion statistics
2. Run Query #10 to review retry patterns
3. Compare FIFO performance trends

### Emergency Response

1. **Subscription Stuck**: Use Query #8 to manually unblock after investigation
2. **Multiple Orphaned States**: Use Query #7 to cleanup (with approval)
3. **Performance Degradation**: Review Query #9 and consider disabling FIFO for problematic subscriptions

## Logging Patterns

### FIFO-Specific Log Messages

All FIFO operations are logged with the `[FIFO]` prefix:

- `[FIFO] Subscription {id} entering FIFO mode, blocking request attempt {id}`
- `[FIFO] Subscription {id} unblocked after successful request attempt {id}`
- `[FIFO] Subscription {id} remains blocked, retry {id} scheduled for {seconds}s`
- `[FIFO] Subscription {id} unblocked after exhausting retries for request attempt {id}`

### Example Log Queries (if using structured logging)

```
# Find all FIFO operations for a subscription
grep "[FIFO] Subscription SUBSCRIPTION_ID_HERE" /var/log/hook0/output-worker.log

# Find stuck subscriptions from logs
grep "\[FIFO\].*remains blocked" /var/log/hook0/output-worker.log | grep -E "scheduled for [5-9][0-9]{2,}s"
```
