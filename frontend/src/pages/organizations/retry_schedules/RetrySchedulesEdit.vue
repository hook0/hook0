<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { toast } from 'vue-sonner';

import { useRetryScheduleLimits } from './useRetryScheduleLimits';
import { handleMutationError } from '@/utils/handleMutationError';
import { handleError } from '@/http';
import { isAxiosError } from '@/http';
import { displayError } from '@/utils/displayError';
import { formatDuration } from '@/utils/duration';
import {
  useRetryScheduleDetail,
  useCreateRetrySchedule,
  useUpdateRetrySchedule,
} from './useRetryScheduleQueries';
import type { RetrySchedulePost, RetrySchedulePut } from './RetryScheduleService';
import { formatDelayList, type RetryStrategy } from './retryScheduleFormatters';
import { routes } from '@/routes';
import { useRouteIds } from '@/composables/useRouteIds';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';

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
import CustomIntervalChip from './CustomIntervalChip.vue';
import { CustomStrategyIcon, ExponentialStrategyIcon, LinearStrategyIcon } from './StrategyIcons';

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

const { canCreate, canEdit } = usePermissions();
const canSubmit = computed(() =>
  isNew.value ? canCreate('retry_schedules') : canEdit('retry_schedules')
);

const {
  limits,
  isLoading: limitsLoading,
  error: limitsError,
  isLimitsMissing,
  refetch: refetchLimits,
} = useRetryScheduleLimits();

const {
  data: scheduleData,
  isLoading: detailLoading,
  error: detailError,
  refetch,
} = useRetryScheduleDetail(retryScheduleId, organizationId);

const createMutation = useCreateRetrySchedule();
const updateMutation = useUpdateRetrySchedule();

const { errors, defineField, handleSubmit, resetForm, setFieldError } =
  useForm<RetryScheduleFormValues>({
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

// VeeValidate fields can be undefined between resets — these computed refs prevent template crashes by providing type-safe fallbacks.
// Use ?? (not ||) so a legitimate 0 from the user is not replaced by the fallback.
const strategyValue = computed<RetryStrategy>(() => strategy.value ?? 'exponential_increasing');
const maxRetriesValue = computed(() => maxRetries.value ?? 0);
const linearDelayValue = computed(() => linearDelay.value ?? 0);
const customIntervalsValue = computed((): number[] =>
  Array.isArray(customIntervals.value) ? customIntervals.value : []
);
const increasingBaseDelayValue = computed(() => increasingBaseDelay.value ?? 10);
const increasingWaitFactorValue = computed(() => increasingWaitFactor.value ?? 3);

const defaultScheduleDelayLabels = computed(() =>
  formatDelayList(limits.value?.default_schedule_delays_secs ?? [])
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
          increasing_base_delay_secs: data.increasing_base_delay_secs ?? 10,
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
function cleanPayload(values: RetryScheduleFormValues, orgId: string): RetrySchedulePost {
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

// Strip the POST-only organization_id so the PUT body matches the RetrySchedulePut schema
function toPutBody(post: RetrySchedulePost): RetrySchedulePut {
  const { organization_id: _, ...put } = post;
  return put;
}

const previewExceedsCap = ref(false);

// Reset previewExceedsCap when strategy changes — the preview component is v-if'd out for custom,
// so it won't emit updates. Without this, switching from exponential/linear to custom leaves
// the stale flag and shows a bogus warning.
watch(strategy, () => {
  previewExceedsCap.value = false;
});

// Custom strategy has no preview — detect cap violations here instead. Preview-emitted state covers exponential/linear.
const customExceedsCap = computed(() => {
  if (strategyValue.value !== 'custom' || !limits.value) return false;
  const cap = limits.value.max_single_delay_secs;
  return customIntervalsValue.value.some((interval) => interval > cap);
});

const hasExceedingRetries = computed(() => previewExceedsCap.value || customExceedsCap.value);

// API 422 responses mention the offending field by name in `detail`. Form field names match the
// API field names 1:1, so we scan for each one with a word boundary (so `name` does not match
// inside `custom_intervals_secs`). Regexes precompiled once, and we stop at the first match —
// the API flags one field per response, so a second match would spray the same error everywhere.
const FORM_FIELD_PATTERNS: readonly (readonly [keyof RetryScheduleFormValues, RegExp])[] = [
  ['custom_intervals_secs', /\bcustom_intervals_secs\b/],
  ['increasing_base_delay_secs', /\bincreasing_base_delay_secs\b/],
  ['increasing_wait_factor', /\bincreasing_wait_factor\b/],
  ['linear_delay_secs', /\blinear_delay_secs\b/],
  ['max_retries', /\bmax_retries\b/],
  ['strategy', /\bstrategy\b/],
  ['name', /\bname\b/],
];

function handleValidationError(err: unknown) {
  if (!isAxiosError(err)) {
    handleMutationError(err);
    return;
  }
  const problem = handleError(err as Parameters<typeof handleError>[0]);
  if (problem.status === 422) {
    const detail = (problem.detail ?? '').toLowerCase();
    for (const [field, pattern] of FORM_FIELD_PATTERNS) {
      if (pattern.test(detail)) {
        setFieldError(field, problem.detail ?? problem.title);
        return;
      }
    }
    displayError(problem);
    return;
  }
  displayError(problem);
}

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
  if (values.strategy === 'custom' && values.custom_intervals_secs.length === 0) {
    setFieldError('custom_intervals_secs', t('retrySchedules.fields.customIntervalsRequired'));
    return;
  }
  const payload = cleanPayload(values, organizationId.value);
  if (isNew.value) {
    createMutation.mutate(payload, {
      onSuccess: () => handleSubmitSuccess('create'),
      onError: (err) => handleValidationError(err),
    });
  } else {
    updateMutation.mutate(
      {
        retryScheduleId: retryScheduleId.value,
        organizationId: organizationId.value,
        schedule: toPutBody(payload),
      },
      {
        onSuccess: () => handleSubmitSuccess('update'),
        onError: (err) => handleValidationError(err),
      }
    );
  }
});

const canAddInterval = computed(() => {
  if (!limits.value) return false;
  if (customIntervalsValue.value.length >= limits.value.max_custom_intervals_length) return false;
  // At the single-delay cap, another chip would duplicate the last one — surface as disabled instead
  const last = customIntervalsValue.value.at(-1);
  return last === undefined || last < limits.value.max_single_delay_secs;
});

const addIntervalHint = computed(() => {
  if (!limits.value) return '';
  if (customIntervalsValue.value.length >= limits.value.max_custom_intervals_length) {
    return t('retrySchedules.fields.intervalsMaxReached', {
      max: limits.value.max_custom_intervals_length,
    });
  }
  return '';
});

function addInterval() {
  if (!canAddInterval.value || !limits.value) return;
  const current = customIntervalsValue.value;
  const cap = limits.value.max_single_delay_secs;
  const last = current.at(-1) ?? 60;
  customIntervals.value = [...current, Math.min(last * 2, cap)];
}

const addIntervalButton = ref<HTMLButtonElement | null>(null);

function removeInterval(index: number) {
  const current = customIntervalsValue.value;
  customIntervals.value = current.filter((_, position) => position !== index);
  void nextTick(() => {
    addIntervalButton.value?.focus();
  });
}

function setCustomInterval(index: number, seconds: number) {
  const current = [...customIntervalsValue.value];
  current[index] = seconds;
  customIntervals.value = current;
}

const pageTitle = computed(() =>
  isNew.value ? t('retrySchedules.create') : t('retrySchedules.edit')
);

const isSkeleton = computed(
  () => limitsLoading.value || (!isNew.value && (detailLoading.value || !scheduleData.value))
);

// Surface a dedicated error when the backend responded without a retry_schedule limits block —
// the form depends on those bounds and cannot render sensible sliders without them.
const pageError = computed(() => {
  if (limitsError.value) return limitsError.value;
  if (isLimitsMissing.value) return t('retrySchedules.limitsUnavailable');
  if (!isNew.value) return detailError.value;
  return null;
});
</script>

<template>
  <Hook0PageLayout :title="pageTitle">
    <Hook0ErrorCard
      v-if="pageError && !isSkeleton"
      :error="pageError"
      :retryable="!isLimitsMissing"
      @retry="limitsError || isLimitsMissing ? refetchLimits() : refetch()"
    />

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
            <template #subtitle>{{
              t('retrySchedules.aboutDescription', { delays: defaultScheduleDelayLabels })
            }}</template>
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
                    data-test="retry-schedule-strategy-exponential"
                    @update:model-value="strategy = 'exponential_increasing'"
                  />
                  <SelectableCard
                    :model-value="strategyValue === 'linear'"
                    :label="t('retrySchedules.strategyLinear')"
                    :description="t('retrySchedules.fields.strategyLinearDesc')"
                    :icon="LinearStrategyIcon"
                    name="strategy"
                    data-test="retry-schedule-strategy-linear"
                    @update:model-value="strategy = 'linear'"
                  />
                  <SelectableCard
                    :model-value="strategyValue === 'custom'"
                    :label="t('retrySchedules.strategyCustom')"
                    :description="t('retrySchedules.fields.strategyCustomDesc')"
                    :icon="CustomStrategyIcon"
                    name="strategy"
                    data-test="retry-schedule-strategy-custom"
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
                @update:has-exceeding="previewExceedsCap = $event"
              />

              <div v-if="strategyValue === 'custom'" class="form-fields__group">
                <label class="form-fields__label">
                  {{ t('retrySchedules.fields.customIntervals') }}
                </label>
                <div class="custom-chips">
                  <CustomIntervalChip
                    v-for="(interval, index) in customIntervalsValue"
                    :key="`chip-${index}`"
                    :model-value="interval"
                    :min="limits.min_single_delay_secs"
                    :max="limits.max_single_delay_secs"
                    :index="index"
                    @update:model-value="setCustomInterval(index, $event)"
                    @remove="removeInterval(index)"
                  />
                  <button
                    ref="addIntervalButton"
                    type="button"
                    class="custom-chips__add"
                    :aria-label="t('retrySchedules.fields.addInterval')"
                    :disabled="!canAddInterval"
                    data-test="retry-schedule-add-interval"
                    @click="addInterval()"
                  >
                    +
                  </button>
                </div>
                <p v-if="addIntervalHint" class="form-fields__hint">{{ addIntervalHint }}</p>
                <p v-if="customExceedsCap" class="form-fields__error">
                  {{
                    t('retrySchedules.fields.customExceedsCap', {
                      max: formatDuration(limits.max_single_delay_secs),
                    })
                  }}
                </p>
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
              v-if="canSubmit"
              variant="primary"
              type="submit"
              data-test="retry-schedule-submit"
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

.form-fields__hint {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
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

.custom-chips__add:hover:not(:disabled) {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.custom-chips__add:disabled {
  opacity: 0.5;
  cursor: not-allowed;
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
