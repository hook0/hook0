import { computed } from 'vue';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import type { RetryScheduleLimits } from './retrySchedule.types';

/**
 * Reads the `retry_schedule` limits block from `/instance`.
 * `error` is exposed so consumers can warn the user when limits are missing.
 */
export function useRetryScheduleLimits() {
  const instance = useInstanceConfig();
  const limits = computed<RetryScheduleLimits | null>(
    () => instance.data.value?.retry_schedule ?? null
  );
  return { limits, isLoading: instance.isLoading, error: instance.error };
}
