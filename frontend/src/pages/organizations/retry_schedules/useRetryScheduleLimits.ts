import { computed } from 'vue';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import type { RetryScheduleLimits } from './retrySchedule.types';

/**
 * Reads the `retry_schedule` limits block from `/instance`.
 * TODO: drop the narrowing helper once types.ts is regenerated from the running API
 * and `InstanceConfig.retry_schedule` is picked up natively.
 */
export function useRetryScheduleLimits() {
  const instance = useInstanceConfig();
  const limits = computed<RetryScheduleLimits | null>(() => {
    const cfg = instance.data.value as { retry_schedule?: RetryScheduleLimits } | undefined;
    return cfg?.retry_schedule ?? null;
  });
  return { limits, isLoading: instance.isLoading, error: instance.error };
}
