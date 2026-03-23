import { useQuery } from '@tanstack/vue-query';
import { getInstanceConfig } from '@/utils/instance-config';
import { instanceConfigKeys } from '@/queries/keys';

export function useInstanceConfig() {
  return useQuery({
    queryKey: instanceConfigKeys.all,
    queryFn: () => getInstanceConfig(),
    staleTime: Infinity,
  });
}
