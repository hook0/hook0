# Configurable Retry Per Subscription

## Summary
This MR implements configurable retry settings per subscription, allowing each webhook subscription to have custom retry behavior tailored to its specific requirements.

## Changes
- ✅ Added `retry_config` JSONB column to subscription table
- ✅ Extended API models and handlers for retry configuration
- ✅ Modified output-worker to use per-subscription retry settings
- ✅ Added UI components for configuring retry settings
- ✅ Implemented comprehensive test coverage

## Features
Each subscription can now configure:
- **Max Fast Retries** (0-100): Number of retries with exponential backoff
- **Max Slow Retries** (0-100): Number of retries with fixed interval
- **Fast Retry Delay** (1-3600s): Initial delay for exponential backoff
- **Max Fast Retry Delay** (1-86400s): Maximum cap for exponential backoff
- **Slow Retry Delay** (60-604800s): Fixed interval between slow retries

## Technical Details

### Database
- New `retry_config` JSONB column with default values matching current behavior
- Backward compatible - existing subscriptions use sensible defaults

### API
- Added `RetryConfig` struct with validation
- Updated subscription endpoints (create/update/get/list)
- Maintains backward compatibility

### Output Worker
- Dynamic retry configuration per subscription
- Falls back to defaults when config not present
- Exponential backoff for fast retries, fixed interval for slow retries

### Frontend
- Intuitive UI for retry configuration
- Input validation matching backend constraints
- Clear help text for each setting

## Testing
- Unit tests for retry calculation logic
- Various scenarios including custom configurations
- Edge cases covered (exhausted retries, boundary conditions)

## Migration Guide
No action required - existing subscriptions will continue using default retry settings. Users can optionally customize retry behavior through the UI.

## Screenshots
The new retry configuration section appears in the subscription edit form with clear labels and help text for each setting.

---
**Type**: Feature
**Breaking Changes**: None
**Related Issues**: N/A