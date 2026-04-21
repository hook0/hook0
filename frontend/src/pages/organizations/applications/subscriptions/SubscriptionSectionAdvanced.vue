<script setup lang="ts">
// Advanced settings sub-form — custom HTTP headers, metadata key-value pairs, and retry schedule selection.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { routes } from '@/routes';
import { useRouteIds } from '@/composables/useRouteIds';

import type { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import type { Hook0SelectSingleOption } from '@/components/Hook0Select';

import { useRetryScheduleLimits } from '../../../retry_schedules/useRetryScheduleLimits';
import { formatDelay } from '../../../retry_schedules/retryScheduleFormatters';

import Hook0KeyValue from '@/components/Hook0KeyValue.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Button from '@/components/Hook0Button.vue';

const { organizationId } = useRouteIds();

const { t } = useI18n();

const { limits } = useRetryScheduleLimits();

const defaultScheduleDelayLabels = computed(() => {
  const delays = limits.value?.default_schedule_delays_secs ?? [];
  return delays.map((secs) => formatDelay(secs)).join(', ');
});

type Props = {
  headersKv: Hook0KeyValueKeyValuePair[];
  metadata: Hook0KeyValueKeyValuePair[];
  retryScheduleId?: string | null;
  retryScheduleOptions?: Hook0SelectSingleOption[];
};

withDefaults(defineProps<Props>(), {
  retryScheduleId: null,
  retryScheduleOptions: () => [],
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

    <!-- Retry schedule -->
    <div v-if="retryScheduleOptions.length > 0" class="sub-row">
      <div class="sub-row__label">
        <span class="sub-row__title sub-row__title--muted">{{
          t('subscriptions.retryScheduleLabel')
        }}</span>
        <span class="sub-row__hint">
          <i18n-t keypath="subscriptions.retryScheduleHint" tag="span">
            <template #link>
              <Hook0Button
                variant="link"
                :to="{
                  name: routes.RetrySchedulesList,
                  params: { organization_id: organizationId },
                }"
                >{{ t('subscriptions.retryScheduleLabel').toLowerCase() }}</Hook0Button
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

.sub-row__content {
  min-width: 0;
}
</style>
