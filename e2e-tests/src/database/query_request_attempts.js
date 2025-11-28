// Helper to query request attempts filtered by subscription_id via API
// The API already exposes failed_at field, so we use the REST API
import list_request_attempt_api from '../events/list_request_attempt.js';

export default function (
  base_url,
  service_token,
  application_id,
  subscription_id = null,
  event_id = null
) {
  // Use the existing API endpoint to get request attempts
  // Note: API returns attempts filtered by event_id or all for application
  const attempts = list_request_attempt_api(base_url, service_token, application_id, event_id);

  if (!attempts || !Array.isArray(attempts)) {
    return [];
  }

  // Filter by subscription_id if provided
  if (subscription_id) {
    return attempts.filter(
      (attempt) => attempt.subscription && attempt.subscription.subscription_id === subscription_id
    );
  }

  return attempts;
}
