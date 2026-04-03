import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as LogService from './LogService';
import { logKeys } from '@/queries/keys';

export function useLogList(applicationId: Ref<string>, subscriptionId?: Ref<string | undefined>) {
  return useQuery({
    queryKey: computed(() => logKeys.list(applicationId.value, subscriptionId?.value)),
    queryFn: () =>
      LogService.list({
        application_id: applicationId.value,
        subscription_id: subscriptionId?.value,
      }),
    enabled: computed(() => !!applicationId.value),
  });
}
