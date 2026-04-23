import { computed } from 'vue';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import type { components } from '@/types';

export type RetryScheduleLimits = components['schemas']['InstanceConfig']['retry_schedule'];

/**
 * Reads the `retry_schedule` limits block from `/instance`.
 * Exposes both the fetch `error` and a dedicated `isLimitsMissing` flag so consumers can
 * distinguish a failed request from a successful response that simply lacks the limits block
 * (e.g. older backend deployments that don't publish `retry_schedule` yet).
 */
export function useRetryScheduleLimits() {
  const instance = useInstanceConfig();
  const limits = computed<RetryScheduleLimits | null>(
    () => instance.data.value?.retry_schedule ?? null
  );
  const isLimitsMissing = computed(
    () => !instance.isLoading.value && !instance.error.value && limits.value === null
  );
  return {
    limits,
    isLoading: instance.isLoading,
    error: instance.error,
    isLimitsMissing,
    refetch: instance.refetch,
  };
}
