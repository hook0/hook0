<script setup lang="ts">
import { computed, defineComponent, h, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { toast } from 'vue-sonner';

import { useRetryScheduleLimits } from './useRetryScheduleLimits';
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
import SelectableCard from '@/components/SelectableCard.vue';
import RetrySchedulePreview from './RetrySchedulePreview.vue';

import type { components } from '@/types';

type RetryStrategy = components['schemas']['RetrySchedule']['strategy'];
type RetryScheduleFormValues = {
  name: string;
  strategy: RetryStrategy;
  max_retries: number;
  linear_delay_secs: number | null;
  custom_intervals_secs: number[];
  increasing_base_delay_secs: number | null;
  increasing_wait_factor: number | null;
};

// Retry schedule create/edit form.
//
// How it works:
// 1. Detects create vs edit from route params (retryScheduleId presence)
// 2. Fetches bounds from /instance (useRetryScheduleLimits) — sliders enforce the same policy as the backend
// 3. Three strategy branches (exponential_increasing/linear/custom) show different slider/input fields
// 4. cleanPayload() strips fields irrelevant to the chosen strategy before submission — the API rejects mixed fields
// 5. RetrySchedulePreview renders the projected delay sequence and reports whether any term exceeds max_single_delay_secs
// 6. Validation is delegated to the backend — 422 responses are surfaced via handleMutationError

const { t } = useI18n();
const { trackEvent } = useTracking();
const router = useRouter();
const { organizationId, retryScheduleId } = useRouteIds();
const isNew = computed(() => !retryScheduleId.value);

const { limits, isLoading: limitsLoading, error: limitsError } = useRetryScheduleLimits();

const {
  data: scheduleData,
  isLoading: detailLoading,
  error: detailError,
  refetch,
} = useRetryScheduleDetail(retryScheduleId, organizationId);

const createMutation = useCreateRetrySchedule();
const updateMutation = useUpdateRetrySchedule();

// Sparkline icons for the strategy picker. Each hints at the shape of delay over retry #.
const svgProps = {
  width: 18,
  height: 18,
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': 2,
  'stroke-linecap': 'round',
  'stroke-linejoin': 'round',
  'aria-hidden': 'true',
} as const;

const ExponentialStrategyIcon = defineComponent({
  name: 'ExponentialStrategyIcon',
  setup: () => () =>
    h('svg', svgProps, [
      h('polyline', { points: '3,21 8,20 12,17 16,11 20,3' }),
      h('circle', { cx: 3, cy: 21, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 20, cy: 3, r: 1, fill: 'currentColor' }),
    ]),
});

const LinearStrategyIcon = defineComponent({
  name: 'LinearStrategyIcon',
  setup: () => () =>
    h('svg', svgProps, [
      h('polyline', { points: '3,12 21,12' }),
      h('circle', { cx: 3, cy: 12, r: 1.3, fill: 'currentColor' }),
      h('circle', { cx: 8, cy: 12, r: 1.3, fill: 'currentColor' }),
      h('circle', { cx: 12, cy: 12, r: 1.3, fill: 'currentColor' }),
      h('circle', { cx: 16, cy: 12, r: 1.3, fill: 'currentColor' }),
      h('circle', { cx: 21, cy: 12, r: 1.3, fill: 'currentColor' }),
    ]),
});

const CustomStrategyIcon = defineComponent({
  name: 'CustomStrategyIcon',
  setup: () => () =>
    h('svg', svgProps, [
      h('polyline', { points: '3,16 8,8 12,14 16,5 20,11' }),
      h('circle', { cx: 3, cy: 16, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 8, cy: 8, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 12, cy: 14, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 16, cy: 5, r: 1, fill: 'currentColor' }),
      h('circle', { cx: 20, cy: 11, r: 1, fill: 'currentColor' }),
    ]),
});

const { errors, defineField, handleSubmit, resetForm } = useForm<RetryScheduleFormValues>({
  initialValues: {
    name: '',
    strategy: 'exponential_increasing',
    max_retries: 8,
    linear_delay_secs: 300,
    custom_intervals_secs: [],
    increasing_base_delay_secs: 10,
    increasing_wait_factor: 3,
  },
});

const [name, nameAttrs] = defineField('name');
const [strategy] = defineField('strategy');
const [maxRetries] = defineField('max_retries');
const [linearDelay] = defineField('linear_delay_secs');
const [customIntervals] = defineField('custom_intervals_secs');
const [increasingBaseDelay] = defineField('increasing_base_delay_secs');
const [increasingWaitFactor] = defineField('increasing_wait_factor');

// VeeValidate fields can be undefined between resets — these computed refs prevent template crashes by providing type-safe fallbacks
const strategyValue = computed<RetryStrategy>(() => strategy.value ?? 'exponential_increasing');
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
  (length) => {
    if (strategyValue.value === 'custom' && length > 0) {
      maxRetries.value = length;
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
          linear_delay_secs: data.linear_delay_secs ?? null,
          custom_intervals_secs: data.custom_intervals_secs ?? [],
          increasing_base_delay_secs: data.increasing_base_delay_secs ?? 3,
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
 * cleanPayload({ strategy: 'linear', linear_delay_secs: 60, ... }, 'org-1')
 * // => { ...base, linear_delay_secs: 60 }
 */
function cleanPayload(values: RetryScheduleFormValues, orgId: string) {
  const base = {
    organization_id: orgId,
    name: values.name,
    strategy: values.strategy,
    max_retries: values.max_retries,
  };
  switch (values.strategy) {
    case 'exponential_increasing':
      return {
        ...base,
        increasing_base_delay_secs: values.increasing_base_delay_secs ?? undefined,
        increasing_wait_factor: values.increasing_wait_factor ?? undefined,
      };
    case 'linear':
      return {
        ...base,
        linear_delay_secs: values.linear_delay_secs ?? undefined,
      };
    case 'custom':
      return {
        ...base,
        custom_intervals_secs: values.custom_intervals_secs ?? undefined,
        max_retries: (values.custom_intervals_secs ?? []).length,
      };
  }
}

const hasExceedingRetries = ref(false);

function handleSubmitSuccess(action: 'create' | 'update') {
  trackEvent('retry-schedule', action, 'success');
  toast.success(t('common.success'), {
    description: t(action === 'create' ? 'retrySchedules.created' : 'retrySchedules.updated'),
    duration: 3000,
  });
  void router.push({
    name: routes.RetrySchedulesList,
    params: { organization_id: organizationId.value },
  });
}

const onSubmit = handleSubmit((values) => {
  if (hasExceedingRetries.value) return;
  const payload = cleanPayload(values, organizationId.value);
  if (isNew.value) {
    createMutation.mutate(payload, {
      onSuccess: () => handleSubmitSuccess('create'),
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
        onSuccess: () => handleSubmitSuccess('update'),
        onError: (err) => handleMutationError(err),
      }
    );
  }
});

function addInterval() {
  if (!limits.value) return;
  const current = customIntervalsValue.value;
  const last = current.at(-1) ?? 60;
  const next = Math.min(last * 2, limits.value.max_single_delay_secs);
  customIntervals.value = [...current, next];
}

function removeInterval(index: number) {
  const current = customIntervalsValue.value;
  customIntervals.value = current.filter((_, position) => position !== index);
}

function updateInterval(index: number, value: string) {
  if (!limits.value) return;
  const current = [...customIntervalsValue.value];
  const clamped = Math.min(
    Math.max(
      Number(value) || limits.value.min_single_delay_secs,
      limits.value.min_single_delay_secs
    ),
    limits.value.max_single_delay_secs
  );
  current[index] = clamped;
  customIntervals.value = current;
}

function handleIntervalBlur(event: FocusEvent, index: number) {
  if (!limits.value) return;
  const target = event.target as HTMLInputElement;
  const parsed = parseDuration(target.value);
  if (parsed !== null && parsed >= limits.value.min_single_delay_secs) {
    updateInterval(index, String(parsed));
  }
  target.value = formatDuration(customIntervalsValue.value[index] ?? 0);
}

const pageTitle = computed(() =>
  isNew.value ? t('retrySchedules.create') : t('retrySchedules.edit')
);

const isSkeleton = computed(
  () => limitsLoading.value || (!isNew.value && (detailLoading.value || !scheduleData.value))
);

const pageError = computed(() => limitsError.value ?? (!isNew.value ? detailError.value : null));
</script>

<template>
  <Hook0PageLayout :title="pageTitle">
    <Hook0ErrorCard v-if="pageError && !isSkeleton" :error="pageError" @retry="refetch()" />

    <Hook0Card v-else-if="isSkeleton">
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="5" />
      </Hook0CardContent>
    </Hook0Card>

    <template v-else-if="limits">
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
                    :model-value="strategyValue === 'exponential_increasing'"
                    :label="t('retrySchedules.strategyIncreasing')"
                    :description="t('retrySchedules.fields.strategyIncreasingDesc')"
                    :icon="ExponentialStrategyIcon"
                    name="strategy"
                    @update:model-value="strategy = 'exponential_increasing'"
                  />
                  <SelectableCard
                    :model-value="strategyValue === 'linear'"
                    :label="t('retrySchedules.strategyLinear')"
                    :description="t('retrySchedules.fields.strategyLinearDesc')"
                    :icon="LinearStrategyIcon"
                    name="strategy"
                    @update:model-value="strategy = 'linear'"
                  />
                  <SelectableCard
                    :model-value="strategyValue === 'custom'"
                    :label="t('retrySchedules.strategyCustom')"
                    :description="t('retrySchedules.fields.strategyCustomDesc')"
                    :icon="CustomStrategyIcon"
                    name="strategy"
                    @update:model-value="strategy = 'custom'"
                  />
                </div>
              </fieldset>

              <div v-if="strategyValue === 'exponential_increasing'" class="slider-row">
                <Hook0Slider
                  :model-value="increasingBaseDelayValue"
                  :min="limits.exponential_base_delay_min_secs"
                  :max="limits.exponential_base_delay_max_secs"
                  :label="t('retrySchedules.fields.increasingBaseDelay')"
                  :format-value="formatDuration"
                  :editable="true"
                  :error="errors.increasing_base_delay_secs"
                  @update:model-value="increasingBaseDelay = $event"
                />
                <Hook0Slider
                  :model-value="increasingWaitFactorValue"
                  :min="limits.exponential_wait_factor_min"
                  :max="limits.exponential_wait_factor_max"
                  :step="0.01"
                  :label="t('retrySchedules.fields.increasingWaitFactor')"
                  :format-value="(v: number) => '×' + v.toFixed(2)"
                  :error="errors.increasing_wait_factor"
                  @update:model-value="increasingWaitFactor = $event"
                />
                <Hook0Slider
                  :model-value="maxRetriesValue"
                  :min="1"
                  :max="limits.max_retries"
                  :label="t('retrySchedules.fields.maxRetries')"
                  :error="errors.max_retries"
                  @update:model-value="maxRetries = $event"
                />
              </div>

              <div v-if="strategyValue === 'linear'" class="slider-row slider-row--two">
                <Hook0Slider
                  :model-value="linearDelayValue"
                  :min="limits.min_single_delay_secs"
                  :max="limits.max_single_delay_secs"
                  :step="1"
                  :label="t('retrySchedules.fields.linearDelay')"
                  :format-value="formatDuration"
                  :editable="true"
                  :error="errors.linear_delay_secs"
                  @update:model-value="linearDelay = $event"
                />
                <Hook0Slider
                  :model-value="maxRetriesValue"
                  :min="1"
                  :max="limits.max_retries"
                  :label="t('retrySchedules.fields.maxRetries')"
                  :error="errors.max_retries"
                  @update:model-value="maxRetries = $event"
                />
              </div>

              <RetrySchedulePreview
                v-if="strategyValue !== 'custom'"
                :strategy="strategyValue"
                :max-retries="maxRetriesValue"
                :linear-delay="linearDelayValue"
                :custom-intervals="customIntervalsValue"
                :increasing-base-delay="increasingBaseDelayValue"
                :increasing-wait-factor="increasingWaitFactorValue"
                :max-interval-seconds="limits.max_single_delay_secs"
                @update:has-exceeding="hasExceedingRetries = $event"
              />

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
                      @blur="(event) => handleIntervalBlur(event as FocusEvent, index)"
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
                <p v-if="errors.custom_intervals_secs" class="form-fields__error">
                  {{ errors.custom_intervals_secs }}
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
  min-width: 0;
}

@media (max-width: 640px) {
  .strategy-cards {
    gap: 0.375rem;
  }

  .strategy-cards :deep(.selectable-card) {
    padding: 0.625rem 0.5rem;
    gap: 0.5rem;
  }

  .strategy-cards :deep(.selectable-card__icon) {
    width: 2rem;
    height: 2rem;
  }

  .strategy-cards :deep(.selectable-card__label) {
    font-size: 0.8125rem;
  }

  .strategy-cards :deep(.selectable-card__indicator) {
    display: none;
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
</style>
