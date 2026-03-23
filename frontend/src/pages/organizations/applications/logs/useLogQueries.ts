import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as LogService from './LogService';
import { logKeys } from '@/queries/keys';

export function useLogList(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => logKeys.list(applicationId.value)),
    queryFn: () => LogService.list(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}
