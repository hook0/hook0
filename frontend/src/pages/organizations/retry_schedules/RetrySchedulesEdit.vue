<script setup lang="ts">
import { computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { Zap, Timer, ListOrdered } from 'lucide-vue-next';
import { toast } from 'vue-sonner';

import {
  createRetryScheduleSchema,
  type RetryScheduleFormValues,
  MAX_RETRIES,
  MAX_INTERVAL_SECONDS,
  SLIDER_MAX_BASE_DELAY,
  SLIDER_MAX_LINEAR_DELAY,
} from './retrySchedule.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import { handleMutationError } from '@/utils/handleMutationError';
import { formatDuration } from '@/utils/formatDuration';
import { parseDuration } from '@/utils/parseDuration';
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
import Hook0Tooltip from '@/components/Hook0Tooltip.vue';
import SelectableCard from '@/components/SelectableCard.vue';

// Retry schedule create/edit form.
//
// How it works:
// 1. Detects create vs edit from route params (retryScheduleId presence)
// 2. Three strategy branches (increasing/linear/custom) show different slider/input fields
// 3. cleanPayload() nulls out fields irrelevant to the chosen strategy before submission — the API rejects mixed fields
// 4. Preview chips compute the actual delay sequence so users see what they're configuring

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
    max_retries: 8,
    linear_delay: 300,
    custom_intervals: [],
    increasing_base_delay: 10,
    increasing_wait_factor: 3,
  },
});

const [name, nameAttrs] = defineField('name');
const [strategy] = defineField('strategy');
const [maxRetries] = defineField('max_retries');
const [linearDelay] = defineField('linear_delay');
const [customIntervals] = defineField('custom_intervals');
const [increasingBaseDelay] = defineField('increasing_base_delay');
const [increasingWaitFactor] = defineField('increasing_wait_factor');

// VeeValidate fields can be undefined between resets — these computed refs prevent template crashes by providing type-safe fallbacks
const strategyValue = computed(() => strategy.value ?? 'increasing');
const maxRetriesValue = computed(() => Number(maxRetries.value) || 0);
const linearDelayValue = computed(() => Number(linearDelay.value) || 0);
const customIntervalsValue = computed((): number[] =>
  Array.isArray(customIntervals.value) ? customIntervals.value.map(Number) : []
);
const increasingBaseDelayValue = computed(() => Number(increasingBaseDelay.value) || 10);
const increasingWaitFactorValue = computed(() => Number(increasingWaitFactor.value) || 3);

// Custom strategy derives max_retries from the intervals array — the API rejects payloads where they disagree, so we keep them in lockstep
watch(
  () => customIntervalsValue.value.length,
  (len) => {
    if (strategyValue.value === 'custom' && len > 0) {
      maxRetries.value = len;
    }
  }
);

// When the query resolves (edit mode), hydrate the form — without this, the user sees blank fields even though data loaded
watch(
  scheduleData,
  (data) => {
    if (data) {
      resetForm({
        values: {
          name: data.name,
          strategy: data.strategy,
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

/**
 * Strips fields that don't belong to the selected strategy.
 * The API validates that only the active strategy's fields are non-null —
 * sending stale values from a previously selected strategy causes a 422.
 *
 * @example
 * cleanPayload({ strategy: 'linear', linear_delay: 60, ... }, 'org-1')
 * // => { ...base, linear_delay: 60, custom_intervals: null, increasing_base_delay: null, ... }
 */
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
        max_retries: (values.custom_intervals ?? []).length,
      };
  }
}

const onSubmit = handleSubmit((values) => {
  if (hasExceedingRetries.value) return;
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
    const { organization_id: _, ...schedule } = payload;
    updateMutation.mutate(
      {
        retryScheduleId: retryScheduleId.value,
        organizationId: organizationId.value,
        schedule,
      },
      {
        onSuccess: () => {
          trackEvent('retry-schedule', 'update', 'success');
          toast.success(t('common.success'), {
            description: t('retrySchedules.updated'),
            duration: 3000,
          });
          void router.push({
            name: routes.RetrySchedulesList,
            params: { organization_id: organizationId.value },
          });
        },
        onError: (err) => handleMutationError(err),
      }
    );
  }
});

function addInterval() {
  const current = customIntervalsValue.value;
  const last = current.length > 0 ? current[current.length - 1] : 60;
  const next = Math.min(last * 2, MAX_INTERVAL_SECONDS);
  customIntervals.value = [...current, next];
}

function removeInterval(index: number) {
  const current = customIntervalsValue.value;
  customIntervals.value = current.filter((_, i) => i !== index);
}

function updateInterval(index: number, value: string) {
  const current = [...customIntervalsValue.value];
  current[index] = Math.min(Math.max(Number(value) || 1, 1), MAX_INTERVAL_SECONDS);
  customIntervals.value = current;
}

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
  return delaySecs.map((s, i) => {
    cumulative += s;
    return {
      retry: i + 1,
      delaySecs: s,
      delay: formatDuration(s),
      cumulative: formatDuration(cumulative),
      exceeds: s > MAX_INTERVAL_SECONDS,
      wayTooMuch: s > 365 * 86400,
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
  // Unreachable unless a new strategy is added without updating this switch
  return [];
});

const hasExceedingRetries = computed(() => previewRows.value.some((r) => r.exceeds));
const firstExceedingRetry = computed(() => {
  const row = previewRows.value.find((r) => r.exceeds);
  return row ? row.retry : 0;
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

              <fieldset class="form-fields__group form-fields__group--no-border">
                <legend class="form-fields__label">
                  {{ t('retrySchedules.fields.strategy') }}
                </legend>
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
              </fieldset>

              <div v-if="strategyValue === 'increasing'" class="slider-row">
                <Hook0Slider
                  :model-value="increasingBaseDelayValue"
                  :min="1"
                  :max="SLIDER_MAX_BASE_DELAY"
                  :label="t('retrySchedules.fields.increasingBaseDelay')"
                  :format-value="formatDuration"
                  :editable="true"
                  :error="errors.increasing_base_delay"
                  @update:model-value="increasingBaseDelay = $event"
                />
                <Hook0Slider
                  :model-value="increasingWaitFactorValue"
                  :min="1.5"
                  :max="10"
                  :step="0.01"
                  :label="t('retrySchedules.fields.increasingWaitFactor')"
                  :format-value="(v: number) => '×' + v.toFixed(2)"
                  :error="errors.increasing_wait_factor"
                  @update:model-value="increasingWaitFactor = $event"
                />
                <Hook0Slider
                  :model-value="maxRetriesValue"
                  :min="1"
                  :max="MAX_RETRIES"
                  :label="t('retrySchedules.fields.maxRetries')"
                  :error="errors.max_retries"
                  @update:model-value="maxRetries = $event"
                />
              </div>

              <!-- Linear: both sliders grouped before preview -->
              <div v-if="strategyValue === 'linear'" class="slider-row slider-row--two">
                <Hook0Slider
                  :model-value="linearDelayValue"
                  :min="1"
                  :max="SLIDER_MAX_LINEAR_DELAY"
                  :step="1"
                  :label="t('retrySchedules.fields.linearDelay')"
                  :format-value="formatDuration"
                  :editable="true"
                  :error="errors.linear_delay"
                  @update:model-value="linearDelay = $event"
                />
                <Hook0Slider
                  :model-value="maxRetriesValue"
                  :min="1"
                  :max="MAX_RETRIES"
                  :label="t('retrySchedules.fields.maxRetries')"
                  :error="errors.max_retries"
                  @update:model-value="maxRetries = $event"
                />
              </div>

              <!-- Preview: retry delay sequence visualization -->
              <div v-if="previewRows.length > 0" class="preview-section">
                <label class="form-fields__label">{{ t('retrySchedules.preview.label') }}</label>
                <div class="preview-chips">
                  <Hook0Tooltip
                    v-for="row in previewRows"
                    :key="row.retry"
                    :content="
                      row.exceeds
                        ? t('retrySchedules.preview.exceedsMaxDelayTooltip')
                        : t('retrySchedules.preview.cumulativeTooltip', { total: row.cumulative })
                    "
                    position="top"
                  >
                    <span
                      class="preview-chips__chip"
                      :class="{ 'preview-chips__chip--exceeds': row.exceeds }"
                    >
                      {{ row.wayTooMuch ? t('retrySchedules.preview.overOneYear') : row.delay }}
                    </span>
                  </Hook0Tooltip>
                </div>
                <p v-if="hasExceedingRetries" class="form-fields__error">
                  {{ t('retrySchedules.preview.exceedsMaxDelay', { n: firstExceedingRetry }) }}
                </p>
              </div>

              <div v-if="strategyValue === 'custom'" class="form-fields__group">
                <label class="form-fields__label">
                  {{ t('retrySchedules.fields.customIntervals') }}
                </label>
                <div class="custom-chips">
                  <div
                    v-for="(interval, index) in customIntervalsValue"
                    :key="`chip-${index}`"
                    class="custom-chips__chip"
                  >
                    <input
                      type="text"
                      class="custom-chips__input"
                      :value="formatDuration(interval)"
                      :aria-label="t('retrySchedules.fields.retryNumber', { number: index + 1 })"
                      @focus="($event.target as HTMLInputElement).value = String(interval)"
                      @blur="
                        (e) => {
                          const parsed = parseDuration((e.target as HTMLInputElement).value);
                          if (parsed !== null && parsed >= 1) {
                            updateInterval(index, String(Math.min(parsed, MAX_INTERVAL_SECONDS)));
                          }
                          (e.target as HTMLInputElement).value = formatDuration(
                            customIntervalsValue[index]
                          );
                        }
                      "
                      @keydown.enter="($event.target as HTMLInputElement).blur()"
                    />
                    <button
                      type="button"
                      class="custom-chips__remove"
                      :aria-label="t('retrySchedules.fields.removeInterval', { number: index + 1 })"
                      @click="removeInterval(index)"
                    >
                      ×
                    </button>
                  </div>
                  <button type="button" class="custom-chips__add" @click="addInterval()">+</button>
                </div>
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
            <Hook0Button
              variant="primary"
              type="submit"
              :disabled="
                hasExceedingRetries ||
                createMutation.isPending.value ||
                updateMutation.isPending.value
              "
              :loading="createMutation.isPending.value || updateMutation.isPending.value"
            >
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

.form-fields__group--no-border {
  border: none;
  padding: 0;
  margin: 0;
}

.form-fields__label {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
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

.custom-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  align-items: center;
}

.custom-chips__chip {
  display: inline-flex;
  align-items: center;
  border-radius: var(--radius-full);
  font-size: 0.75rem;
  font-weight: 500;
  background-color: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  overflow: hidden;
  transition: border-color 0.15s ease;
}

.custom-chips__chip:focus-within {
  border-color: var(--color-primary);
}

.custom-chips__input {
  border: none;
  background: transparent;
  font-size: 0.75rem;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  color: var(--color-text-secondary);
  padding: 0.25rem 0.5rem;
  width: 5rem;
  text-align: center;
  outline: none;
}

.custom-chips__input:focus {
  color: var(--color-text-primary);
  width: 4rem;
  text-align: center;
}

.custom-chips__remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-left: 1px solid var(--color-border);
  color: var(--color-text-tertiary);
  cursor: pointer;
  padding: 0.25rem 0.375rem;
  font-size: 0.875rem;
  line-height: 1;
  transition: all 0.15s ease;
}

.custom-chips__remove:hover {
  background-color: var(--color-error-light);
  color: var(--color-error);
}

.custom-chips__add {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: var(--radius-full);
  border: 1px dashed var(--color-border);
  background: transparent;
  color: var(--color-text-tertiary);
  font-size: 1rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.custom-chips__add:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.slider-row {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 1.5rem;
}

.slider-row--two {
  grid-template-columns: 1fr 1fr;
}

@media (max-width: 640px) {
  .slider-row,
  .slider-row--two {
    grid-template-columns: 1fr;
  }
}

.preview-section {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
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
</style>
