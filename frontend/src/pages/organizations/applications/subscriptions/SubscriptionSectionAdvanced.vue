<script setup lang="ts">
// Advanced settings sub-form — custom HTTP headers, metadata key-value pairs, and retry schedule selection.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { routes } from '@/routes';
import { useRouteIds } from '@/composables/useRouteIds';

import type { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import type { Hook0SelectSingleOption } from '@/components/Hook0Select';

import { useRetryScheduleLimits } from '../../retry_schedules/useRetryScheduleLimits';
import { useRetryScheduleList } from '../../retry_schedules/useRetryScheduleQueries';
import { formatDelayList } from '../../retry_schedules/retryScheduleFormatters';

import Hook0KeyValue from '@/components/Hook0KeyValue.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

const { organizationId } = useRouteIds();

const { t } = useI18n();

const { limits, isLoading: limitsLoading } = useRetryScheduleLimits();

// Fetch the schedule list locally so the <select> stays unmounted until the options are ready.
// Hook0Select.initValue() normalizes the bound value to '' when the current id isn't in options —
// if we let it render with an empty list, the saved schedule silently reverts to "default" on edit.
// TanStack Query deduplicates this call with any sibling consumer (e.g. the parent page).
const { data: retrySchedules, isLoading: schedulesLoading } = useRetryScheduleList(organizationId);

const retryScheduleOptions = computed<Hook0SelectSingleOption[]>(() => {
  const defaultOption: Hook0SelectSingleOption = {
    value: '',
    label: t('retrySchedules.defaultSchedule'),
  };
  return [
    defaultOption,
    ...(retrySchedules.value ?? []).map((schedule) => ({
      value: schedule.retry_schedule_id,
      label: schedule.name,
    })),
  ];
});

const retryScheduleReady = computed(() => !schedulesLoading.value && !limitsLoading.value);

const defaultScheduleDelayLabels = computed(() =>
  formatDelayList(limits.value?.default_schedule_delays_secs ?? [])
);

type Props = {
  headersKv: Hook0KeyValueKeyValuePair[];
  metadata: Hook0KeyValueKeyValuePair[];
  retryScheduleId?: string | null;
};

withDefaults(defineProps<Props>(), {
  retryScheduleId: null,
});

const emit = defineEmits<{
  'update:headers': [value: Hook0KeyValueKeyValuePair[] | Record<string, string>];
  'update:metadata': [value: Hook0KeyValueKeyValuePair[] | Record<string, string>];
  'update:retryScheduleId': [value: string | null];
}>();
</script>

<template>
  <div class="sub-section">
    <!-- Headers -->
    <div class="sub-row">
      <div class="sub-row__label">
        <span class="sub-row__title sub-row__title--muted">{{
          t('subscriptions.endpointHeaders')
        }}</span>
        <span class="sub-row__hint">{{ t('subscriptions.headersHint') }}</span>
      </div>
      <div class="sub-row__content">
        <Hook0KeyValue
          :value="headersKv"
          :key-placeholder="t('common.headerName')"
          :value-placeholder="t('common.value')"
          @update:model-value="emit('update:headers', $event)"
        />
      </div>
    </div>

    <!-- Metadata -->
    <div class="sub-row">
      <div class="sub-row__label">
        <span class="sub-row__title sub-row__title--muted">{{
          t('subscriptions.metadataLabel')
        }}</span>
        <span class="sub-row__hint">{{ t('subscriptions.metadataHint') }}</span>
      </div>
      <div class="sub-row__content">
        <Hook0KeyValue
          :value="metadata"
          :key-placeholder="t('common.key')"
          :value-placeholder="t('common.value')"
          @update:model-value="emit('update:metadata', $event)"
        />
      </div>
    </div>

    <!-- Retry schedule — render the select only once schedules + limits are loaded so Hook0Select doesn't normalize the bound value to '' against an empty options list. -->
    <div v-if="!retryScheduleReady" class="sub-row">
      <div class="sub-row__label">
        <Hook0Skeleton class="sub-row__skeleton-label" />
      </div>
      <div class="sub-row__content">
        <Hook0Skeleton class="sub-row__skeleton-field" />
      </div>
    </div>
    <div v-else class="sub-row">
      <div class="sub-row__label">
        <span id="retry-schedule-label" class="sub-row__title sub-row__title--muted">{{
          t('subscriptions.retryScheduleLabel')
        }}</span>
        <span class="sub-row__hint">
          <i18n-t keypath="subscriptions.retryScheduleHint" tag="span">
            <template #link>
              <router-link
                class="sub-row__link"
                :to="{
                  name: routes.RetrySchedulesList,
                  params: { organization_id: organizationId },
                }"
                >{{ t('subscriptions.retryScheduleLinkText') }}</router-link
              >
            </template>
          </i18n-t>
        </span>
      </div>
      <div class="sub-row__content">
        <!-- Empty string from Hook0Select means "default schedule" — convert to null for the API. -->
        <Hook0Select
          :model-value="retryScheduleId ?? ''"
          :options="retryScheduleOptions"
          aria-labelledby="retry-schedule-label"
          @update:model-value="emit('update:retryScheduleId', $event || null)"
        />
        <p v-if="!retryScheduleId" class="sub-row__default-hint">
          {{ t('retrySchedules.defaultScheduleDesc', { delays: defaultScheduleDelayLabels }) }}
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sub-section {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.sub-row {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.5rem;
}

@media (min-width: 640px) {
  .sub-row {
    grid-template-columns: 2fr 3fr;
    gap: 1.5rem;
  }
}

.sub-row__label {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  padding-top: 0.25rem;
}

.sub-row__title {
  font-size: 0.875rem;
  font-weight: 700;
  color: var(--color-text-primary);
}

.sub-row__title--muted {
  color: var(--color-text-muted);
  font-weight: 600;
}

.sub-row__hint {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.sub-row__default-hint {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  line-height: 1.4;
  margin-top: 0.25rem;
}

.sub-row__link {
  color: var(--color-primary);
  text-decoration: underline;
  transition: color 0.15s ease;
}

.sub-row__link:hover {
  color: var(--color-primary-hover);
}

.sub-row__link:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-sm);
}

.sub-row__content {
  min-width: 0;
}

.sub-row__skeleton-label {
  height: 1rem;
  width: 8rem;
}

.sub-row__skeleton-field {
  height: 2.25rem;
  width: 100%;
}
</style>
