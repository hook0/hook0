# FIFO Mode User Guide

## Overview

FIFO (First-In-First-Out) mode is an optional feature for webhook subscriptions that guarantees **strict event ordering**. When enabled, Hook0 ensures that webhooks for a subscription are delivered in the exact order events were received, even in the presence of retries and concurrent workers.

## When to Use FIFO Mode

### ✅ Use FIFO Mode When:

1. **State Machine Systems**: Your application processes sequential state transitions that must be applied in order
   - Example: Order status updates (pending → confirmed → shipped → delivered)

2. **Financial Operations**: Transaction sequences where order affects account balances or business logic
   - Example: Deposit $100, then withdraw $50 (order matters for balance calculation)

3. **Workflow Orchestration**: Multi-step processes where each step depends on the previous one completing
   - Example: Document approval workflow (submit → review → approve → publish)

4. **Audit Trails**: Systems requiring chronological event processing for compliance or legal reasons
   - Example: Healthcare record updates, financial audit logs

### ❌ Don't Use FIFO Mode When:

1. **Independent Events**: Each event can be processed independently regardless of order
   - Example: User signup notifications, analytics events

2. **High Throughput Requirements**: You need maximum webhook delivery speed
   - FIFO mode can reduce throughput by 50-90% depending on webhook latency

3. **Idempotent Operations**: Your system handles out-of-order events correctly
   - Example: Cache invalidation, metrics updates

## How FIFO Mode Works

### Normal Mode (Default)
```
Event 1 → Worker A → Webhook sent immediately
Event 2 → Worker B → Webhook sent immediately (parallel)
Event 3 → Worker C → Webhook sent immediately (parallel)
```
**Result**: Maximum throughput, events may arrive out of order

### FIFO Mode
```
Event 1 → Worker A → Webhook sent
Event 2 → [BLOCKED] Waiting for Event 1 to complete
Event 3 → [BLOCKED] Waiting for Event 2 to start

Event 1 succeeds
Event 2 → Worker B → Webhook sent
Event 3 → [BLOCKED] Waiting for Event 2 to complete

Event 2 succeeds
Event 3 → Worker C → Webhook sent
```
**Result**: Strict ordering, reduced throughput

### Retry Behavior

When a webhook fails in FIFO mode:

1. **Retry Scheduled**: The failed webhook enters retry phase (exponential backoff)
2. **Queue Blocked**: All subsequent webhooks for this subscription wait
3. **Retry Attempts**: System retries up to 60 times (configurable)
4. **Success or Exhaustion**: Once succeeded or retries exhausted, queue unblocks
5. **Next Webhook**: The next event in sequence can now be processed

**Example Timeline**:
```
00:00 - Event 1 sent, fails (500 error)
00:05 - Event 1 retry 1, fails
00:15 - Event 1 retry 2, fails
00:35 - Event 1 retry 3, succeeds
00:35 - Event 2 unblocked, sent immediately
```

## Enabling FIFO Mode

### For New Subscriptions

**API Request**:
```bash
curl -X POST https://api.hook0.com/api/v1/subscriptions \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "event_types": ["order.created", "order.updated"],
    "target": {
      "type": "http",
      "url": "https://your-app.com/webhooks",
      "method": "POST"
    },
    "fifo_mode": true
  }'
```

**Response**:
```json
{
  "subscription__id": "550e8400-e29b-41d4-a716-446655440000",
  "fifo_mode": true,
  ...
}
```

### For Existing Subscriptions

**API Request**:
```bash
curl -X PUT https://api.hook0.com/api/v1/subscriptions/{subscription_id} \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "event_types": ["order.created", "order.updated"],
    "target": {
      "type": "http",
      "url": "https://your-app.com/webhooks",
      "method": "POST"
    },
    "fifo_mode": true
  }'
```

### Verifying FIFO Mode

**Check Subscription Status**:
```bash
curl -X GET https://api.hook0.com/api/v1/subscriptions/{subscription_id} \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**Response**:
```json
{
  "subscription__id": "550e8400-e29b-41d4-a716-446655440000",
  "fifo_mode": true,
  "is_enabled": true,
  ...
}
```

## Performance Considerations

### Throughput Impact

FIFO mode significantly reduces webhook delivery throughput:

| Webhook Latency | Normal Mode Throughput | FIFO Mode Throughput | Impact |
|-----------------|------------------------|----------------------|--------|
| 50ms | 1000/sec (50 workers) | 20/sec | -98% |
| 100ms | 1000/sec (50 workers) | 10/sec | -99% |
| 500ms | 1000/sec (50 workers) | 2/sec | -99.8% |
| 1000ms | 1000/sec (50 workers) | 1/sec | -99.9% |

**Key Insight**: FIFO throughput = 1 / (webhook latency), regardless of worker count.

### Best Practices for Performance

1. **Optimize Webhook Handler**:
   - Process webhooks as quickly as possible
   - Return 2xx status immediately, process asynchronously if needed
   - Target <100ms response times

2. **Split Subscriptions**:
   - Use FIFO only for event types that require ordering
   - Keep non-critical events in separate normal subscriptions

   ```
   Subscription 1 (FIFO): order.created, order.updated
   Subscription 2 (Normal): analytics.tracked, email.sent
   ```

3. **Monitor Queue Depth**:
   - Set up alerts for growing FIFO queues
   - Indicates webhook handler performance issues

4. **Consider Alternatives**:
   - Can your system handle idempotent operations?
   - Could you use event timestamps instead of delivery order?
   - Is eventual consistency acceptable?

## Monitoring FIFO Subscriptions

### Key Metrics

1. **Queue Depth**: Number of pending webhooks for FIFO subscription
   - Alert threshold: >100 pending

2. **Blocked Duration**: How long subscription has been blocked
   - Alert threshold: >10 minutes

3. **Success Rate**: Percentage of successful deliveries
   - Alert threshold: <80%

4. **Webhook Latency**: Average response time from your endpoint
   - Target: <100ms for good FIFO performance

### Monitoring Queries

See [FIFO_MONITORING_QUERIES.md](./FIFO_MONITORING_QUERIES.md) for SQL queries to monitor:
- Currently blocked subscriptions
- Stuck subscriptions (blocked >5 minutes)
- Queue depth per subscription
- Completion statistics
- Orphaned states

### Log Filtering

All FIFO operations are logged with `[FIFO]` prefix:

```bash
# View FIFO operations
grep "\[FIFO\]" /var/log/hook0/output-worker.log

# Monitor blocking events
grep "\[FIFO\] .* entering FIFO mode" /var/log/hook0/output-worker.log

# Monitor completions
grep "\[FIFO\] .* unblocked" /var/log/hook0/output-worker.log
```

## Troubleshooting

### Problem: Webhooks Stuck/Not Delivering

**Symptoms**:
- New events not triggering webhooks
- Queue depth growing
- Subscription shows as blocked in monitoring

**Diagnosis**:
```sql
-- Check if subscription is blocked
SELECT fss.subscription__id,
       fss.current_request_attempt__id,
       ra.picked_at,
       ra.succeeded_at,
       ra.failed_at
FROM webhook.fifo_subscription_state fss
JOIN webhook.request_attempt ra ON ra.request_attempt__id = fss.current_request_attempt__id
WHERE fss.subscription__id = 'YOUR_SUBSCRIPTION_ID';
```

**Solutions**:

1. **Check Webhook Handler**:
   - Is your endpoint responding?
   - Are responses <30s? (timeout threshold)
   - Check for 5xx errors

2. **Review Retry Status**:
   - Current webhook may be in retry phase
   - Check last attempt time and next retry schedule
   - Consider if webhook will ever succeed

3. **Manual Unblock** (Emergency Only):
   ```sql
   -- Clear FIFO state to unblock
   UPDATE webhook.fifo_subscription_state
   SET current_request_attempt__id = NULL,
       updated_at = statement_timestamp()
   WHERE subscription__id = 'YOUR_SUBSCRIPTION_ID';
   ```
   ⚠️ **Warning**: This breaks ordering guarantee for one event

### Problem: Low Throughput

**Symptoms**:
- Webhooks delivering slowly
- Growing backlog of events

**Diagnosis**:
```sql
-- Check queue depth
SELECT COUNT(*) as pending_webhooks
FROM webhook.request_attempt ra
JOIN webhook.subscription s ON s.subscription__id = ra.subscription__id
WHERE s.subscription__id = 'YOUR_SUBSCRIPTION_ID'
  AND ra.picked_at IS NULL
  AND ra.cancelled_at IS NULL;
```

**Solutions**:

1. **Optimize Webhook Handler**:
   - Profile your endpoint's response time
   - Move heavy processing to background jobs
   - Return 200 immediately after validation

2. **Verify FIFO Necessity**:
   - Do you really need strict ordering?
   - Could you handle out-of-order events with timestamps?

3. **Split Event Types**:
   - Move non-critical events to separate subscription
   - Keep FIFO only for order-sensitive events

### Problem: Orphaned FIFO States

**Symptoms**:
- Subscription stuck after worker crash
- No active request attempt but still blocked

**Diagnosis**:
```sql
-- Find orphaned states (picked >5 min ago, not completed)
SELECT fss.subscription__id, fss.current_request_attempt__id
FROM webhook.fifo_subscription_state fss
JOIN webhook.request_attempt ra ON ra.request_attempt__id = fss.current_request_attempt__id
WHERE ra.picked_at IS NOT NULL
  AND ra.succeeded_at IS NULL
  AND ra.failed_at IS NULL
  AND ra.picked_at < NOW() - INTERVAL '5 minutes';
```

**Solution**:
```sql
-- Cleanup orphaned state
UPDATE webhook.fifo_subscription_state
SET current_request_attempt__id = NULL,
    updated_at = statement_timestamp()
WHERE subscription__id = 'YOUR_SUBSCRIPTION_ID';
```

## Migration Guide

### Enabling FIFO for Existing Subscription

1. **Assess Impact**:
   - Current webhook volume
   - Average webhook latency
   - Calculate expected FIFO throughput: 1 / latency

2. **Update Subscription**:
   ```bash
   curl -X PUT https://api.hook0.com/api/v1/subscriptions/{id} \
     -H "Authorization: Bearer TOKEN" \
     -d '{"fifo_mode": true, ...other fields...}'
   ```

3. **Monitor Closely**:
   - Watch queue depth for first hour
   - Verify webhook latency acceptable
   - Check for any blocking issues

4. **Optimize if Needed**:
   - Tune webhook handler for speed
   - Consider splitting subscriptions

### Disabling FIFO Mode

1. **Update Subscription**:
   ```bash
   curl -X PUT https://api.hook0.com/api/v1/subscriptions/{id} \
     -H "Authorization: Bearer TOKEN" \
     -d '{"fifo_mode": false, ...other fields...}'
   ```

2. **Verify**:
   - Check subscription shows `fifo_mode: false`
   - FIFO state will remain but won't block new webhooks
   - Old events still in queue will process

3. **Cleanup** (Optional):
   ```sql
   DELETE FROM webhook.fifo_subscription_state
   WHERE subscription__id = 'YOUR_SUBSCRIPTION_ID';
   ```

## API Reference

### Subscription Object

```typescript
interface Subscription {
  subscription__id: string;
  application__id: string;
  is_enabled: boolean;
  fifo_mode: boolean;  // FIFO mode flag
  target: Target;
  event_types: string[];
  created_at: string;
  // ... other fields
}
```

### Create Subscription Request

```typescript
interface SubscriptionPost {
  event_types: string[];
  target: Target;
  fifo_mode?: boolean;  // Default: false
  // ... other fields
}
```

### Update Subscription Request

```typescript
interface SubscriptionPut {
  event_types: string[];
  target: Target;
  fifo_mode?: boolean;  // Can toggle FIFO on/off
  // ... other fields
}
```

## FAQ

**Q: Can I enable FIFO for some events but not others in the same subscription?**
A: No. FIFO mode applies to all events in a subscription. Create separate subscriptions if you need different ordering guarantees.

**Q: What happens if my webhook endpoint is down?**
A: Hook0 will retry according to the retry schedule (up to 60 attempts over ~24 hours). All subsequent events remain blocked until retries succeed or are exhausted.

**Q: Does FIFO mode guarantee ordering across different subscriptions?**
A: No. FIFO mode only guarantees ordering within a single subscription. Different subscriptions process events independently.

**Q: Can I use FIFO mode with dedicated workers?**
A: Yes. FIFO mode works with both shared and dedicated workers. The blocking mechanism is database-level, not worker-level.

**Q: What's the maximum queue depth for FIFO subscriptions?**
A: No hard limit, but performance degrades with large queues. Monitor queue depth and alert on >100 pending events.

**Q: Will enabling FIFO mode affect my existing non-FIFO subscriptions?**
A: No. FIFO and non-FIFO subscriptions operate independently. Non-FIFO subscriptions maintain full concurrency.

**Q: Can I temporarily disable FIFO mode?**
A: Yes. Update the subscription with `fifo_mode: false`. Events will process immediately without ordering guarantee.

**Q: What happens to events already in queue when I disable FIFO?**
A: They process immediately without blocking. The FIFO state is cleared and events may complete out of order.

## Support

For issues or questions:
- Check monitoring queries in [FIFO_MONITORING_QUERIES.md](./FIFO_MONITORING_QUERIES.md)
- Review logs with `[FIFO]` prefix
- Contact support with subscription ID and relevant timeframe

## Related Documentation

- [FIFO Specification](./FIFO_SUBSCRIPTION_SPEC.md) - Technical implementation details
- [FIFO Monitoring Queries](./FIFO_MONITORING_QUERIES.md) - SQL queries for operations
- [FIFO Implementation Status](./FIFO_IMPLEMENTATION_STATUS.md) - Development progress
