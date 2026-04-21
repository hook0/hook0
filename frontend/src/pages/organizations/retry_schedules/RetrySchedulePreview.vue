<script setup lang="ts">
import { computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { formatDuration, SECONDS_PER_YEAR } from '@/utils/duration';
import Hook0Tooltip from '@/components/Hook0Tooltip.vue';
import { computeDelaysRaw, type RetryStrategy } from './retryScheduleFormatters';

type Props = {
  strategy: RetryStrategy;
  maxRetries: number;
  linearDelay: number;
  customIntervals: number[];
  increasingBaseDelay: number;
  increasingWaitFactor: number;
  maxIntervalSeconds: number;
};

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:has-exceeding': [value: boolean];
}>();

const { t } = useI18n();

type PreviewRow = {
  retry: number;
  delaySecs: number;
  delay: string;
  cumulative: string;
  exceeds: boolean;
  wayTooMuch: boolean;
};

function buildPreviewRows(delaySecs: number[]): PreviewRow[] {
  let cumulative = 0;
  return delaySecs.map((seconds, index) => {
    cumulative += seconds;
    return {
      retry: index + 1,
      delaySecs: seconds,
      delay: formatDuration(seconds),
      cumulative: formatDuration(cumulative),
      exceeds: seconds > props.maxIntervalSeconds,
      wayTooMuch: seconds > SECONDS_PER_YEAR,
    };
  });
}

const previewRows = computed<PreviewRow[]>(() =>
  buildPreviewRows(
    computeDelaysRaw({
      strategy: props.strategy,
      max_retries: props.maxRetries,
      linear_delay_secs: props.linearDelay,
      custom_intervals_secs: props.customIntervals,
      increasing_base_delay_secs: props.increasingBaseDelay,
      increasing_wait_factor: props.increasingWaitFactor,
    })
  )
);

const firstExceedingIndex = computed(() => previewRows.value.findIndex((row) => row.exceeds));
const hasExceedingRetries = computed(() => firstExceedingIndex.value !== -1);
const firstExceedingRetry = computed(
  () => previewRows.value[firstExceedingIndex.value]?.retry ?? 0
);

const totalCumulativeSecs = computed(() =>
  previewRows.value.reduce((sum, row) => sum + row.delaySecs, 0)
);
const totalCumulativeFormatted = computed(() => formatDuration(totalCumulativeSecs.value));

const maxIntervalFormatted = computed(() => formatDuration(props.maxIntervalSeconds));

// immediate: true — first emit during the initial render so the parent's disabled state is
// aligned with the real preview from frame 0. Without it the parent starts with the default
// `false` and only syncs on the first value change, leaving a transient mismatch.
watch(hasExceedingRetries, (value) => emit('update:has-exceeding', value), { immediate: true });
</script>

<template>
  <div v-if="previewRows.length > 0" class="preview-section">
    <div class="preview-section__header">
      <label class="preview-section__label">{{ t('retrySchedules.preview.label') }}</label>
      <span class="preview-section__total">
        <span class="preview-section__total-value">
          {{
            t('retrySchedules.preview.totalDuration', {
              duration: totalCumulativeFormatted,
            })
          }}
        </span>
        <span class="preview-section__total-separator" aria-hidden="true">·</span>
        <span class="preview-section__total-retries">
          {{
            t(
              'retrySchedules.preview.retriesCount',
              { count: previewRows.length },
              previewRows.length
            )
          }}
        </span>
      </span>
    </div>

    <div class="preview-chips">
      <Hook0Tooltip
        v-for="row in previewRows"
        :key="row.retry"
        :content="
          row.exceeds
            ? t('retrySchedules.preview.exceedsMaxDelayTooltip', { max: maxIntervalFormatted })
            : t('retrySchedules.preview.cumulativeTooltip', { total: row.cumulative })
        "
        position="top"
      >
        <span class="preview-chips__chip" :class="{ 'preview-chips__chip--exceeds': row.exceeds }">
          {{ row.wayTooMuch ? t('retrySchedules.preview.overOneYear') : row.delay }}
        </span>
      </Hook0Tooltip>
    </div>
    <p v-if="hasExceedingRetries" class="preview-section__error">
      {{
        t('retrySchedules.preview.exceedsMaxDelay', {
          n: firstExceedingRetry,
          max: maxIntervalFormatted,
        })
      }}
    </p>
  </div>
</template>

<style scoped>
.preview-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.preview-section__label {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.preview-section__error {
  font-size: 0.8125rem;
  color: var(--color-error);
}

.preview-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.preview-chips__chip {
  display: inline-flex;
  align-items: center;
  padding: 0.25rem 0.625rem;
  border-radius: var(--radius-full);
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background-color: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  font-variant-numeric: tabular-nums;
  cursor: default;
}

.preview-chips__chip--exceeds {
  color: var(--color-error);
  background-color: var(--color-error-light);
  border-color: var(--color-error);
}

.preview-section__header {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.75rem;
  flex-wrap: wrap;
}

.preview-section__total {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.75rem;
  font-variant-numeric: tabular-nums;
  color: var(--color-text-secondary);
}

.preview-section__total-value {
  font-weight: 600;
  color: var(--color-text-primary);
}

.preview-section__total-separator {
  color: var(--color-text-tertiary);
}
</style>
