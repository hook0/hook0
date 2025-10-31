## Summary
Implementation of automatic deactivation and retry policies for webhook subscriptions, including:
- Configurable retry schedule with exponential backoff
- Automatic deactivation after 5 days of continuous failures
- Proactive email notifications before deactivation
- Operational webhooks system for deactivation events

## Changes
- Added ADR documentation for automatic deactivation and retry policies specification
- Moved SPEC.md to proper ADR location (0004-Automatic-deactivation-and-retry-policies.md)

## Related Issues
This MR implements the webhook delivery retry policy and automatic subscription deactivation system as specified in the ADR.

## Testing
- [ ] Unit tests for retry logic
- [ ] Integration tests for deactivation workflow
- [ ] Email notification testing
- [ ] Webhook event testing

## Implementation Details
The specification follows the Svix/Stripe model for retry policies with:
1. Exponential backoff retry schedule
2. Automatic deactivation after 5 days of failures
3. Email notifications at 1 day and 4 days before deactivation
4. Webhook events for subscription status changes