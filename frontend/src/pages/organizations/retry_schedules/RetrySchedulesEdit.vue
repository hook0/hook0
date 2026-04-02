<script setup lang="ts">
import { computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { Zap, Timer, ListOrdered, Plus, Trash2 } from 'lucide-vue-next';
import { toast } from 'vue-sonner';

import { createRetryScheduleSchema, type RetryScheduleFormValues } from './retrySchedule.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import { handleMutationError } from '@/utils/handleMutationError';
import { formatDuration } from '@/utils/formatDuration';
import {
  useRetryScheduleDetail,
  useCreateRetrySchedule,
  useUpdateRetrySchedule,
} from './useRetryScheduleQueries';
import { routes } from '@/routes';
import { useRouteIds } from '@/composables/useRouteIds';
import { useTracking } from '@/composables/useTracking';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Slider from '@/components/Hook0Slider.vue';
import SelectableCard from '@/components/SelectableCard.vue';

const { t } = useI18n();
const { trackEvent } = useTracking();
const router = useRouter();
const { organizationId, retryScheduleId } = useRouteIds();
const isNew = computed(() => !retryScheduleId.value);

const {
  data: scheduleData,
  isLoading,
  error,
  refetch,
} = useRetryScheduleDetail(retryScheduleId, organizationId);

const createMutation = useCreateRetrySchedule();
const updateMutation = useUpdateRetrySchedule();

const { errors, defineField, handleSubmit, resetForm } = useForm({
  validationSchema: toTypedSchema(createRetryScheduleSchema()),
  initialValues: {
    name: '',
    strategy: 'increasing' as const,
    max_retries: 10,
    linear_delay: null,
    custom_intervals: [],
    increasing_base_delay: 3,
    increasing_wait_factor: 3,
  },
});

const [name, nameAttrs] = defineField('name');
const [strategy] = defineField('strategy');
const [maxRetries, maxRetriesAttrs] = defineField('max_retries');
const [linearDelay, linearDelayAttrs] = defineField('linear_delay');
const [customIntervals] = defineField('custom_intervals');
const [increasingBaseDelay, increasingBaseDelayAttrs] = defineField('increasing_base_delay');
const [increasingWaitFactor, increasingWaitFactorAttrs] = defineField('increasing_wait_factor');

// Typed accessors for template usage
const strategyValue = computed(() => (strategy.value as string) ?? 'increasing');
const maxRetriesValue = computed(() => (maxRetries.value as number) ?? 0);
const linearDelayValue = computed(() => (linearDelay.value as number) ?? 0);
const customIntervalsValue = computed(() => (customIntervals.value as number[] | null) ?? []);
const increasingBaseDelayValue = computed(() => (increasingBaseDelay.value as number) ?? 3);
const increasingWaitFactorValue = computed(() => (increasingWaitFactor.value as number) ?? 3);

// Sync max_retries from custom_intervals length
watch(
  () => customIntervalsValue.value.length,
  (len) => {
    if (strategyValue.value === 'custom' && len > 0) {
      maxRetries.value = len;
    }
  }
);

// Populate form in edit mode
watch(
  scheduleData,
  (data) => {
    if (data) {
      resetForm({
        values: {
          name: data.name,
          strategy: data.strategy as 'increasing' | 'linear' | 'custom',
          max_retries: data.max_retries,
          linear_delay: data.linear_delay,
          custom_intervals: data.custom_intervals ?? [],
          increasing_base_delay: data.increasing_base_delay ?? 3,
          increasing_wait_factor: data.increasing_wait_factor ?? 3,
        },
      });
    }
  },
  { immediate: true }
);

function cleanPayload(values: RetryScheduleFormValues, orgId: string) {
  const base = {
    organization_id: orgId,
    name: values.name,
    strategy: values.strategy,
    max_retries: values.max_retries,
  };
  switch (values.strategy) {
    case 'increasing':
      return {
        ...base,
        linear_delay: null,
        custom_intervals: null,
        increasing_base_delay: values.increasing_base_delay,
        increasing_wait_factor: values.increasing_wait_factor,
      };
    case 'linear':
      return {
        ...base,
        linear_delay: values.linear_delay,
        custom_intervals: null,
        increasing_base_delay: null,
        increasing_wait_factor: null,
      };
    case 'custom':
      return {
        ...base,
        linear_delay: null,
        custom_intervals: values.custom_intervals,
        increasing_base_delay: null,
        increasing_wait_factor: null,
        max_retries: values.custom_intervals!.length,
      };
  }
}

const onSubmit = handleSubmit((values) => {
  const payload = cleanPayload(values, organizationId.value);
  if (isNew.value) {
    createMutation.mutate(payload, {
      onSuccess: () => {
        trackEvent('retry-schedule', 'create', 'success');
        toast.success(t('common.success'), {
          description: t('retrySchedules.created'),
          duration: 3000,
        });
        void router.push({
          name: routes.RetrySchedulesList,
          params: { organization_id: organizationId.value },
        });
      },
      onError: (err) => handleMutationError(err),
    });
  } else {
    updateMutation.mutate(
      {
        retryScheduleId: retryScheduleId.value,
        organizationId: organizationId.value,
        schedule: payload,
      },
      {
        onSuccess: () => {
          trackEvent('retry-schedule', 'update', 'success');
          toast.success(t('common.success'), {
            description: t('retrySchedules.updated'),
            duration: 3000,
          });
        },
        onError: (err) => handleMutationError(err),
      }
    );
  }
});

function addInterval() {
  const current = customIntervalsValue.value;
  customIntervals.value = [...current, 60];
}

function removeInterval(index: number) {
  const current = customIntervalsValue.value;
  customIntervals.value = current.filter((_: number, i: number) => i !== index);
}

function updateInterval(index: number, value: string) {
  const current = [...customIntervalsValue.value];
  current[index] = Number(value) || 1;
  customIntervals.value = current;
}

// Preview computation
function buildPreviewRows(delaySecs: number[]) {
  let cumulative = 0;
  return delaySecs.map((s, i) => {
    cumulative += s;
    return {
      retry: i + 1,
      delay: formatDuration(s),
      cumulative: formatDuration(cumulative),
    };
  });
}

const previewRows = computed(() => {
  const strat = strategyValue.value;
  const max = maxRetriesValue.value;
  if (strat === 'increasing') {
    const bd = increasingBaseDelayValue.value;
    const wf = increasingWaitFactorValue.value;
    const delays = Array.from({ length: max }, (_, i) => Math.round(bd * Math.pow(wf, i)));
    return buildPreviewRows(delays);
  }
  if (strat === 'linear') {
    const d = linearDelayValue.value;
    const delays = Array.from({ length: max }, () => d);
    return buildPreviewRows(delays);
  }
  if (strat === 'custom') {
    return buildPreviewRows(customIntervalsValue.value);
  }
  return [];
});

const pageTitle = computed(() =>
  isNew.value ? t('retrySchedules.create') : t('retrySchedules.edit')
);
</script>

<template>
  <Hook0PageLayout :title="pageTitle">
    <Hook0ErrorCard v-if="error && !isLoading && !isNew" :error="error" @retry="refetch()" />

    <Hook0Card v-else-if="!isNew && (isLoading || !scheduleData)">
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="5" />
      </Hook0CardContent>
    </Hook0Card>

    <template v-else>
      <Hook0Form data-test="retry-schedule-form" @submit="onSubmit">
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>{{ pageTitle }}</template>
            <template #subtitle>{{ t('retrySchedules.aboutDescription') }}</template>
          </Hook0CardHeader>

          <Hook0CardContent>
            <div class="form-fields">
              <Hook0Input
                v-model="name"
                v-bind="nameAttrs"
                :label="t('retrySchedules.fields.name')"
                :placeholder="t('retrySchedules.fields.namePlaceholder')"
                :error="errors.name"
                data-test="retry-schedule-name-input"
              />

              <!-- Strategy selector -->
              <div class="form-fields__group">
                <label class="form-fields__label">{{ t('retrySchedules.fields.strategy') }}</label>
                <div class="strategy-cards">
                  <SelectableCard
                    :model-value="strategyValue === 'increasing'"
                    :label="t('retrySchedules.strategyIncreasing')"
                    :description="t('retrySchedules.fields.strategyIncreasingDesc')"
                    :icon="Zap"
                    name="strategy"
                    @update:model-value="strategy = 'increasing'"
                  />
                  <SelectableCard
                    :model-value="strategyValue === 'linear'"
                    :label="t('retrySchedules.strategyLinear')"
                    :description="t('retrySchedules.fields.strategyLinearDesc')"
                    :icon="Timer"
                    name="strategy"
                    @update:model-value="strategy = 'linear'"
                  />
                  <SelectableCard
                    :model-value="strategyValue === 'custom'"
                    :label="t('retrySchedules.strategyCustom')"
                    :description="t('retrySchedules.fields.strategyCustomDesc')"
                    :icon="ListOrdered"
                    name="strategy"
                    @update:model-value="strategy = 'custom'"
                  />
                </div>
              </div>

              <!-- Increasing fields -->
              <div v-if="strategyValue === 'increasing'" class="slider-row">
                <Hook0Slider
                  :model-value="increasingBaseDelayValue"
                  :min="1"
                  :max="300"
                  :label="t('retrySchedules.fields.increasingBaseDelay')"
                  :format-value="formatDuration"
                  :error="errors.increasing_base_delay"
                  @update:model-value="increasingBaseDelay = $event"
                />
                <Hook0Slider
                  :model-value="increasingWaitFactorValue"
                  :min="1.5"
                  :max="10"
                  :step="0.5"
                  :label="t('retrySchedules.fields.increasingWaitFactor')"
                  :format-value="(v: number) => '×' + v"
                  :error="errors.increasing_wait_factor"
                  @update:model-value="increasingWaitFactor = $event"
                />
                <Hook0Slider
                  :model-value="maxRetriesValue"
                  :min="1"
                  :max="25"
                  :label="t('retrySchedules.fields.maxRetries')"
                  :error="errors.max_retries"
                  @update:model-value="maxRetries = $event"
                />
              </div>

              <!-- Max retries (linear) -->
              <Hook0Slider
                v-if="strategyValue === 'linear'"
                :model-value="maxRetriesValue"
                :min="1"
                :max="25"
                :label="t('retrySchedules.fields.maxRetries')"
                :error="errors.max_retries"
                @update:model-value="maxRetries = $event"
              />

              <!-- Preview chips -->
              <div v-if="previewRows.length > 0" class="preview-chips">
                <span
                  v-for="row in previewRows"
                  :key="row.retry"
                  class="preview-chips__chip"
                  :title="t('retrySchedules.preview.cumulativeTooltip', { total: row.cumulative })"
                >
                  {{ row.delay }}
                </span>
              </div>

              <!-- Linear delay -->
              <Hook0Slider
                v-if="strategyValue === 'linear'"
                :model-value="linearDelayValue"
                :min="1"
                :max="86400"
                :step="1"
                :label="t('retrySchedules.fields.linearDelay')"
                :format-value="formatDuration"
                :error="errors.linear_delay"
                @update:model-value="linearDelay = $event"
              />

              <!-- Custom intervals editor -->
              <div v-if="strategyValue === 'custom'" class="form-fields__group">
                <label class="form-fields__label">
                  {{ t('retrySchedules.fields.customIntervals') }}
                </label>
                <div class="custom-intervals">
                  <div
                    v-for="(interval, index) in customIntervalsValue"
                    :key="index"
                    class="custom-intervals__row"
                  >
                    <span class="custom-intervals__label">
                      {{ t('retrySchedules.fields.retryNumber', { number: index + 1 }) }}
                    </span>
                    <Hook0Input
                      :model-value="String(interval as number)"
                      :label="t('retrySchedules.fields.intervalSeconds')"
                      type="number"
                      min="1"
                      max="604800"
                      @update:model-value="updateInterval(index, String($event))"
                    />
                    <Hook0Button variant="ghost" type="button" @click="removeInterval(index)">
                      <Trash2 :size="16" aria-hidden="true" />
                    </Hook0Button>
                  </div>
                </div>
                <Hook0Button variant="secondary" type="button" @click="addInterval()">
                  <template #left>
                    <Plus :size="16" aria-hidden="true" />
                  </template>
                  {{ t('retrySchedules.fields.addInterval') }}
                </Hook0Button>
                <p v-if="errors.custom_intervals" class="form-fields__error">
                  {{ errors.custom_intervals }}
                </p>
              </div>
            </div>
          </Hook0CardContent>

          <Hook0CardFooter>
            <Hook0Button
              variant="secondary"
              type="button"
              :to="{
                name: routes.RetrySchedulesList,
                params: { organization_id: organizationId },
              }"
            >
              {{ t('common.cancel') }}
            </Hook0Button>
            <Hook0Button variant="primary" type="submit">
              {{ isNew ? t('common.create') : t('common.save') }}
            </Hook0Button>
          </Hook0CardFooter>
        </Hook0Card>

      </Hook0Form>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.form-fields {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.form-fields__group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-fields__label {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.form-fields__hint {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.form-fields__error {
  font-size: 0.8125rem;
  color: var(--color-error);
}

.strategy-cards {
  display: flex;
  gap: 0.75rem;
}

.strategy-cards > * {
  flex: 1;
}

@media (max-width: 640px) {
  .strategy-cards {
    flex-direction: column;
  }
}

.custom-intervals {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.custom-intervals__row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.custom-intervals__label {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  min-width: 5rem;
  flex-shrink: 0;
}

.about-description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  line-height: 1.6;
  margin-bottom: 0.5rem;
}

.slider-row {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 1.5rem;
}

@media (max-width: 640px) {
  .slider-row {
    grid-template-columns: 1fr;
  }
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
}
</style>
