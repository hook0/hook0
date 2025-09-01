# 4. Implement Operational Webhooks for System Monitoring

Date: 2025-08-29

## Status

Accepted

## Context

Hook0 is a webhook-as-a-service platform that manages webhook deliveries for various applications. As the platform scales and handles more critical webhook traffic, there's a growing need for observability into the webhook system itself. 

Currently, Hook0 lacks a mechanism to notify administrators and monitoring systems about important operational events such as:
- Endpoint failures and automatic disabling
- Message delivery exhaustion after retries
- System health degradation
- Configuration changes to webhook endpoints

Svix, a leading webhook-as-a-service provider, has successfully implemented "operational webhooks" that allow their platform to send webhooks about the webhook system itself. This meta-webhook approach provides excellent observability and enables proactive monitoring.

### Requirements

1. **Real-time Monitoring**: Need immediate notifications when webhook deliveries fail or endpoints are disabled
2. **System Health Visibility**: Track delivery success rates and identify problematic endpoints
3. **Audit Trail**: Record all configuration changes to webhook endpoints
4. **Integration with Monitoring Tools**: Allow external monitoring systems to receive operational events
5. **Security**: Ensure operational webhooks are authenticated and cannot be spoofed

## Decision

We will implement operational webhooks similar to Svix's approach, creating a meta-webhook system that sends notifications about the webhook infrastructure itself.

### Implementation Approach

1. **Dedicated Operational Webhook System**
   - Separate tables for operational endpoints and events
   - Independent from regular webhook subscriptions to avoid circular dependencies
   - Support for filtering which operational events an endpoint receives

2. **Event Types**
   - `endpoint.created` - When a new webhook endpoint is configured
   - `endpoint.updated` - When endpoint configuration changes
   - `endpoint.deleted` - When an endpoint is removed
   - `endpoint.disabled` - When an endpoint is automatically disabled due to failures
   - `message.attempt.exhausted` - When all retry attempts for a message fail
   - `message.attempt.failing` - When a message delivery is failing (but retries remain)
   - `message.attempt.recovered` - When a previously failing endpoint recovers

3. **Security Model**
   - HMAC-SHA256 signature verification (similar to Svix's v1 signature format)
   - Unique secret per operational endpoint
   - Timestamp validation to prevent replay attacks
   - Custom headers support for additional authentication

4. **Delivery Mechanism**
   - Dedicated background worker for operational webhook delivery
   - Exponential backoff retry logic (up to 5 attempts)
   - Automatic cleanup of old operational events (30+ days)
   - Rate limiting support per endpoint

5. **Database Schema**
   ```sql
   webhook.operational_endpoint - Stores operational webhook configurations
   webhook.operational_event - Stores operational events to be delivered
   webhook.operational_event_type - Defines available event types
   webhook.operational_attempt - Tracks delivery attempts
   webhook.message_stats - Aggregates delivery statistics
   ```

### API Design

New endpoints under `/api/v1/operational_webhooks/`:
- `GET /` - List operational webhook endpoints
- `POST /` - Create new operational webhook endpoint
- `GET /{id}` - Get specific endpoint details
- `PUT /{id}` - Update endpoint configuration
- `DELETE /{id}` - Delete endpoint
- `GET /event_types` - List available operational event types
- `GET /stats` - Get message delivery statistics

## Consequences

### Positive

1. **Enhanced Observability**: Real-time visibility into webhook system health and performance
2. **Proactive Monitoring**: Ability to detect and respond to issues before they impact users
3. **Integration Flexibility**: External monitoring tools can easily integrate via webhooks
4. **Audit Compliance**: Complete audit trail of endpoint configuration changes
5. **Industry Standard**: Following Svix's proven pattern increases familiarity for users
6. **Scalability**: Separate operational webhook system won't interfere with regular webhook traffic

### Negative

1. **Increased Complexity**: Additional tables, APIs, and background workers to maintain
2. **Resource Overhead**: Operational webhooks consume additional database and network resources
3. **Potential for Feedback Loops**: Must carefully prevent operational webhooks from triggering more operational events
4. **Migration Effort**: Existing monitoring solutions need to be updated to use operational webhooks

### Neutral

1. **Documentation Requirements**: Need comprehensive documentation for operational webhook integration
2. **Monitoring of Monitoring**: Operational webhooks themselves need monitoring (quis custodiet ipsos custodes?)
3. **Rate Limiting Considerations**: Operational webhooks could generate significant traffic during incidents

## Implementation Notes

### Phase 1 - Core Implementation (Completed)
- Database schema and migrations
- CRUD API for operational endpoints
- Webhook signature generation and verification
- Background worker for delivery

### Phase 2 - Enhanced Features (Future)
- Webhook event batching for high-volume scenarios
- Advanced filtering with complex predicates
- Webhook replay functionality
- Metrics and dashboard integration

### Phase 3 - Advanced Monitoring (Future)
- Predictive failure detection using ML
- Automatic remediation actions
- SLA tracking and reporting
- Multi-region operational webhook delivery

## References

- [Svix Operational Webhooks Documentation](https://docs.svix.com/incoming-webhooks)
- [Webhook Best Practices](https://webhooks.fyi/)
- [HMAC Authentication RFC 2104](https://tools.ietf.org/html/rfc2104)
- Original implementation: MR #149 in Hook0 repository

## Decision Makers

- Engineering Team Lead
- Platform Architecture Team
- DevOps/SRE Team

## Related ADRs

- None currently - this is the first webhook monitoring related ADR