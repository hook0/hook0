import { computed } from 'vue';
import { useInstanceConfig } from './useInstanceConfig';

/**
 * Fallback values used while the /instance config is loading or when the
 * health monitor is disabled on this instance. The authoritative defaults
 * live in api/src/main.rs (`health_monitor_warning_failure_percent`,
 * `health_monitor_disable_failure_percent`).
 */
const DEFAULT_WARNING_THRESHOLD = 80;
const DEFAULT_CRITICAL_THRESHOLD = 95;

/**
 * Reads the health monitor thresholds from the /instance endpoint so the UI
 * mirrors the actual server-side configuration. Falls back to the documented
 * defaults while the instance config is loading or when the health monitor is
 * disabled on this instance.
 */
export function useHealthThresholds() {
  const { data: instanceConfig } = useInstanceConfig();

  const warning = computed(
    () =>
      instanceConfig.value?.subscription_health_monitor?.failure_percent_for_warning ??
      DEFAULT_WARNING_THRESHOLD
  );
  const critical = computed(
    () =>
      instanceConfig.value?.subscription_health_monitor?.failure_percent_for_disable ??
      DEFAULT_CRITICAL_THRESHOLD
  );

  return { warning, critical };
}
