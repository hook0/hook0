import { computed } from 'vue';
import { useRoute } from 'vue-router';

export function useRouteIds() {
  const route = useRoute();

  const get = (name: string) =>
    computed(() => {
      const v = route.params[name];
      return Array.isArray(v) ? (v[0] ?? '') : (v ?? '');
    });

  return {
    organizationId: get('organization_id'),
    applicationId: get('application_id'),
    serviceTokenId: get('service_token_id'),
    eventId: get('event_id'),
    subscriptionId: get('subscription_id'),
    responseId: get('response_id'),
    requestAttemptId: get('request_attempt_id'),
  };
}
