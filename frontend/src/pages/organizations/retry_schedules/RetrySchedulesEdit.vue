<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { toast } from 'vue-sonner';
import { Plus } from 'lucide-vue-next';

import { useRouteIds } from '@/composables/useRouteIds';
import { usePermissions } from '@/composables/usePermissions';
import { handleMutationError } from '@/utils/handleMutationError';
import { routes } from '@/routes';

import {
  useCreateRetrySchedule,
  useRetryScheduleDetail,
  useUpdateRetrySchedule,
} from './useRetryScheduleQueries';
import { useRetryScheduleLimits } from './useRetryScheduleLimits';
import type { RetrySchedulePayload, RetryStrategy } from './retrySchedule.types';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Slider from '@/components/Hook0Slider.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0CardSkeleton from '@/components/Hook0CardSkeleton.vue';
import Hook0IntervalChips from './Hook0IntervalChips.vue';

const { t } = useI18n();
const router = useRouter();
const { organizationId, retryScheduleId } = useRouteIds();
const perms = usePermissions();

const isEdit = computed(() => !!retryScheduleId.value);
const canSubmit = computed(() =>
  isEdit.value ? perms.canEdit('retry_schedule') : perms.canCreate('retry_schedule')
);

const { limits } = useRetryScheduleLimits();

const detailQuery = useRetryScheduleDetail(retryScheduleId);

// Form state
const name = ref('');
const strategy = ref<RetryStrategy>('exponential_increasing');
const maxRetries = ref(5);
const baseDelay = ref(60);
const waitFactor = ref(2);
const linearDelay = ref(60);
const customIntervals = ref<number[]>([30, 120, 600]);
const newIntervalValue = ref<number | null>(null);
const formError = ref<string | null>(null);

// Populate form when editing
watch(
  () => detailQuery.data.value,
  (schedule) => {
    if (!schedule) return;
    name.value = schedule.name;
    strategy.value = schedule.strategy;
    maxRetries.value = schedule.max_retries;
    if (schedule.strategy === 'exponential_increasing') {
      baseDelay.value = schedule.increasing_base_delay ?? 60;
      waitFactor.value = schedule.increasing_wait_factor ?? 2;
    } else if (schedule.strategy === 'linear') {
      linearDelay.value = schedule.linear_delay ?? 60;
    } else {
      customIntervals.value = [...(schedule.custom_intervals ?? [])];
    }
  },
  { immediate: true }
);

// Initialize defaults once when limits arrive — do NOT clobber user edits on refetch.
const didInit = ref(false);
watch(
  limits,
  (lim) => {
    if (!lim || isEdit.value || didInit.value) return;
    maxRetries.value = Math.min(5, lim.max_retries);
    baseDelay.value = Math.max(lim.exponential_base_delay_min_secs, 60);
    waitFactor.value = Math.max(lim.exponential_wait_factor_min, 2);
    linearDelay.value = Math.max(lim.min_single_delay_secs, 60);
    didInit.value = true;
  },
  { immediate: true }
);

function addInterval() {
  const lim = limits.value;
  if (newIntervalValue.value === null || !lim) return;
  const v = newIntervalValue.value;
  if (v < lim.min_single_delay_secs || v > lim.max_single_delay_secs) {
    formError.value = t('retrySchedules.validation.intervalOutOfRange', {
      min: lim.min_single_delay_secs,
      max: lim.max_single_delay_secs,
    });
    return;
  }
  if (customIntervals.value.length >= lim.max_custom_intervals_length) return;
  customIntervals.value.push(v);
  newIntervalValue.value = null;
  formError.value = null;
}

function removeInterval(index: number) {
  customIntervals.value.splice(index, 1);
}

function buildPayload(): RetrySchedulePayload | null {
  const trimmed = name.value.trim();
  if (!trimmed) {
    formError.value = t('retrySchedules.validation.nameRequired');
    return null;
  }
  switch (strategy.value) {
    case 'exponential_increasing':
      return {
        strategy: 'exponential_increasing',
        name: trimmed,
        max_retries: maxRetries.value,
        base_delay: baseDelay.value,
        wait_factor: waitFactor.value,
      };
    case 'linear':
      return {
        strategy: 'linear',
        name: trimmed,
        max_retries: maxRetries.value,
        delay: linearDelay.value,
      };
    case 'custom':
      if (customIntervals.value.length === 0) {
        formError.value = t('retrySchedules.validation.intervalsRequired');
        return null;
      }
      return {
        strategy: 'custom',
        name: trimmed,
        intervals: [...customIntervals.value],
      };
  }
}

const createMutation = useCreateRetrySchedule();
const updateMutation = useUpdateRetrySchedule();

function handleSubmit() {
  formError.value = null;
  const payload = buildPayload();
  if (!payload) return;

  const target = {
    name: routes.RetrySchedulesList,
    params: { organization_id: organizationId.value },
  };

  if (isEdit.value) {
    updateMutation.mutate(
      { retryScheduleId: retryScheduleId.value, payload },
      {
        onSuccess: () => {
          toast.success(t('common.success'), { description: t('retrySchedules.updated') });
          void router.push(target);
        },
        onError: (err) => handleMutationError(err),
      }
    );
  } else {
    createMutation.mutate(
      { organization_id: organizationId.value, payload },
      {
        onSuccess: () => {
          toast.success(t('common.success'), { description: t('retrySchedules.created') });
          void router.push(target);
        },
        onError: (err) => handleMutationError(err),
      }
    );
  }
}

function handleCancel() {
  void router.push({
    name: routes.RetrySchedulesList,
    params: { organization_id: organizationId.value },
  });
}

const isSubmitting = computed(
  () => createMutation.isPending.value || updateMutation.isPending.value
);

const title = computed(() =>
  isEdit.value ? t('retrySchedules.editTitle') : t('retrySchedules.newTitle')
);

const secondsUnit = computed(() => t('retrySchedules.units.seconds'));
const retriesUnit = computed(() => t('retrySchedules.units.retries'));
</script>

<template>
  <Hook0PageLayout :title="title">
    <Hook0ErrorCard
      v-if="detailQuery.error.value && isEdit"
      :error="detailQuery.error.value"
      @retry="detailQuery.refetch()"
    />

    <Hook0CardSkeleton v-else-if="isEdit && detailQuery.isLoading.value" />

    <template v-else>
      <form @submit.prevent="handleSubmit">
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>{{ t('retrySchedules.cardHeader') }}</template>
            <template #subtitle>{{ t('retrySchedules.cardSubtitle') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <div class="form-grid">
              <Hook0Input
                v-model="name"
                :label="t('retrySchedules.fields.name')"
                :placeholder="t('retrySchedules.fields.namePlaceholder')"
                maxlength="200"
                required
              />

              <Hook0Select
                v-model="strategy"
                :label="t('retrySchedules.fields.strategy')"
                :options="[
                  {
                    value: 'exponential_increasing',
                    label: t('retrySchedules.strategies.exponentialIncreasing'),
                  },
                  { value: 'linear', label: t('retrySchedules.strategies.linear') },
                  { value: 'custom', label: t('retrySchedules.strategies.custom') },
                ]"
              />

              <!-- Exponential fields -->
              <template v-if="strategy === 'exponential_increasing' && limits">
                <Hook0Slider
                  v-model="maxRetries"
                  :label="t('retrySchedules.fields.maxRetries')"
                  :min="1"
                  :max="limits.max_retries"
                  :unit="retriesUnit"
                />
                <Hook0Slider
                  v-model="baseDelay"
                  :label="t('retrySchedules.fields.baseDelay')"
                  :min="limits.exponential_base_delay_min_secs"
                  :max="limits.exponential_base_delay_max_secs"
                  :unit="secondsUnit"
                  :help-text="t('retrySchedules.help.baseDelay')"
                />
                <Hook0Slider
                  v-model="waitFactor"
                  :label="t('retrySchedules.fields.waitFactor')"
                  :min="limits.exponential_wait_factor_min"
                  :max="limits.exponential_wait_factor_max"
                  :step="0.1"
                  :help-text="t('retrySchedules.help.waitFactor')"
                />
              </template>

              <!-- Linear fields -->
              <template v-if="strategy === 'linear' && limits">
                <Hook0Slider
                  v-model="maxRetries"
                  :label="t('retrySchedules.fields.maxRetries')"
                  :min="1"
                  :max="limits.max_retries"
                  :unit="retriesUnit"
                />
                <Hook0Slider
                  v-model="linearDelay"
                  :label="t('retrySchedules.fields.linearDelay')"
                  :min="limits.min_single_delay_secs"
                  :max="limits.max_single_delay_secs"
                  :unit="secondsUnit"
                  :help-text="t('retrySchedules.help.linearDelay')"
                />
              </template>

              <!-- Custom intervals — no "preview" block; the chips are the preview. -->
              <template v-if="strategy === 'custom' && limits">
                <div class="custom-intervals">
                  <label class="custom-intervals__label">
                    {{ t('retrySchedules.fields.intervals') }}
                  </label>
                  <p class="custom-intervals__help">
                    {{
                      t('retrySchedules.help.intervals', {
                        max: limits.max_custom_intervals_length,
                      })
                    }}
                  </p>
                  <Hook0IntervalChips
                    :values="customIntervals"
                    removable
                    @remove="removeInterval"
                  />
                  <div class="custom-intervals__add">
                    <input
                      v-model.number="newIntervalValue"
                      type="number"
                      :min="limits.min_single_delay_secs"
                      :max="limits.max_single_delay_secs"
                      :placeholder="t('retrySchedules.fields.intervalPlaceholder')"
                      class="custom-intervals__input"
                    />
                    <Hook0Button
                      variant="secondary"
                      type="button"
                      :disabled="
                        newIntervalValue === null ||
                        customIntervals.length >= limits.max_custom_intervals_length
                      "
                      @click="addInterval"
                    >
                      <Plus :size="14" aria-hidden="true" />
                      {{ t('common.add') }}
                    </Hook0Button>
                  </div>
                </div>
              </template>

              <p v-if="formError" class="form-error">{{ formError }}</p>
            </div>
          </Hook0CardContent>
          <Hook0CardFooter>
            <Hook0Button variant="secondary" type="button" @click="handleCancel">
              {{ t('common.cancel') }}
            </Hook0Button>
            <Hook0Button variant="primary" type="submit" :disabled="isSubmitting || !canSubmit">
              {{ isEdit ? t('common.save') : t('common.create') }}
            </Hook0Button>
          </Hook0CardFooter>
        </Hook0Card>
      </form>
    </template>
  </Hook0PageLayout>
</template>

<style scoped>
.form-grid {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.custom-intervals {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.custom-intervals__label {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

.custom-intervals__help {
  margin: 0;
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}

.custom-intervals__add {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.custom-intervals__input {
  flex: 1;
  min-width: 0;
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background-color: var(--color-bg-primary);
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}

.custom-intervals__input:focus {
  outline: 2px solid var(--color-primary);
  outline-offset: 1px;
  border-color: var(--color-primary);
}

.form-error {
  margin: 0;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-md);
  background-color: var(--color-error-light);
  color: var(--color-error);
  font-size: 0.8125rem;
}
</style>
