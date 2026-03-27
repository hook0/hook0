import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as ResponseService from './ResponseService';
import { responseKeys } from '@/queries/keys';

export function useResponseDetail(responseId: Ref<string>, applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => responseKeys.detail(responseId.value, applicationId.value)),
    queryFn: () => ResponseService.get(responseId.value, applicationId.value),
    enabled: computed(() => !!responseId.value && !!applicationId.value),
  });
}
