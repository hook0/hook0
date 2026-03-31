<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';

import { createSubscriptionSchema } from './subscription.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import { displayError } from '@/utils/displayError';
import { isAxiosError, handleError } from '@/http';
import {
  useSubscriptionDetail,
  useCreateSubscription,
  useUpdateSubscription,
} from './useSubscriptionQueries';
import { useEventTypeList } from '../event_types/useEventTypeQueries';
import type { EventType } from '../event_types/EventTypeService';
import { intersectWith } from '@/utils/fp';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import type { Hook0SelectSingleOption } from '@/components/Hook0Select';
import {
  kvPairsToRecord,
  recordToKvPairs,
  type Hook0KeyValueKeyValuePair,
} from '@/components/Hook0KeyValue';

import SubscriptionsRemove from './SubscriptionsRemove.vue';
import SubscriptionSectionBasics from './SubscriptionSectionBasics.vue';
import SubscriptionSectionEventTypes from './SubscriptionSectionEventTypes.vue';
import type { SelectableEventType } from './subscription.types';
import SubscriptionSectionLabels from './SubscriptionSectionLabels.vue';
import SubscriptionSectionAdvanced from './SubscriptionSectionAdvanced.vue';
import { useRetryScheduleList } from '../../retry_schedules/useRetryScheduleQueries';

import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CopyField from '@/components/Hook0CopyField.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';

import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

import Hook0Form from '@/components/Hook0Form.vue';
import Hook0PageLayout from '@/components/Hook0PageLayout.vue';

const { t } = useI18n();
const { trackEvent } = useTracking();

// Permissions
const { canCreate, canEdit, canDelete } = usePermissions();

type Props = {
  tutorialMode?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  tutorialMode: false,
});

const emit = defineEmits(['tutorial-subscription-created']);

function EventTypeNamesFromSelectedEventTypes(
  selectableEventTypes: SelectableEventType[]
): string[] {
  return selectableEventTypes
    .filter((eventType) => eventType.selected)
    .map((eventType) => eventType.event_type_name);
}

function SelectedEventTypesFromEventTypeNames(eventTypeNames: string[]): SelectableEventType[] {
  return eventTypeNames.map((eventTypeName) => {
    const [resource_type_name, service_name, verb_name] = eventTypeName.split('.');
    return {
      event_type_name: eventTypeName,
      resource_type_name,
      service_name,
      verb_name,
      selected: true,
    };
  });
}

function toOption(val: string): Hook0SelectSingleOption {
  return { value: val, label: val };
}

/**
 * Centralized cast for API record types that expect Record<string, never>.
 * The API schema uses `never` as the value type for open-ended string maps,
 * but our form state uses Record<string, string>. This helper isolates
 * the unavoidable cast to a single location.
 */
function toApiRecord(map: Record<string, string>): Record<string, never> {
  return map as unknown as Record<string, never>;
}

const router = useRouter();
const { organizationId, applicationId, subscriptionId } = useRouteIds();
const isNew = computed(() => !subscriptionId.value);

// Queries
const {
  data: subscriptionData,
  isLoading: subLoading,
  error: subError,
  refetch: refetchSub,
} = useSubscriptionDetail(subscriptionId);

const {
  data: rawEventTypes,
  isLoading: etLoading,
  error: etError,
  refetch: refetchEt,
} = useEventTypeList(applicationId);

// Mutations
const createMutation = useCreateSubscription();
const updateMutation = useUpdateSubscription();

// VeeValidate form with Zod schema
const { errors, defineField, handleSubmit, resetForm, setFieldError } = useForm({
  validationSchema: toTypedSchema(createSubscriptionSchema()),
});

const [description, descriptionAttrs] = defineField('description');
const [targetMethod, targetMethodAttrs] = defineField('target_method');
targetMethod.value = 'POST';
const [targetUrl, targetUrlAttrs] = defineField('target_url');

// Non-validated form state (managed outside VeeValidate)
const secret = ref('');
const createdAt = ref('');
const isEnabled = ref(true);
const dedicatedWorkers = ref<string[]>([]);
const eventTypes = ref<SelectableEventType[]>([]);
const labels = ref<Hook0KeyValueKeyValuePair[]>([{ key: 'user_id', value: '1' }]);
const metadata = ref<Hook0KeyValueKeyValuePair[]>([]);
const headersKv = ref<Hook0KeyValueKeyValuePair[]>([]);
const headersMap = ref<Record<string, string>>({});
const labelsMap = ref<Record<string, string>>({ user_id: '1' });
const metadataMap = ref<Record<string, string>>({});

const selectedRetryScheduleId = ref<string | null>(null);

// Retry schedule list for selector
const { data: retrySchedules } = useRetryScheduleList(organizationId);
const retryScheduleOptions = computed(() => {
  const options: Hook0SelectSingleOption[] = [
    { value: '', label: t('retrySchedules.defaultSchedule') },
  ];
  if (retrySchedules.value) {
    retrySchedules.value.forEach((s) => {
      options.push({ value: s.retry_schedule_id, label: s.name });
    });
  }
  return options;
});

const httpMethods = 'GET,PATCH,POST,PUT,DELETE,OPTIONS,HEAD'.split(',').map(toOption);

// Populate form from subscription data (edit mode)
watch(
  subscriptionData,
  (sub) => {
    if (sub) {
      resetForm({
        values: {
          description: sub.description || '',
          target_method: sub.target.method,
          target_url: sub.target.url,
        },
      });
      secret.value = sub.secret;
      createdAt.value = sub.created_at;
      isEnabled.value = sub.is_enabled;
      dedicatedWorkers.value = [...sub.dedicated_workers];
      labels.value = recordToKvPairs(sub.labels);
      labelsMap.value = { ...sub.labels };
      metadata.value = recordToKvPairs(sub.metadata);
      metadataMap.value = { ...sub.metadata };
      headersKv.value = recordToKvPairs(sub.target.headers);
      headersMap.value = { ...sub.target.headers } as unknown as Record<string, string>;
      selectedRetryScheduleId.value = sub.retry_schedule_id ?? null;
    }
  },
  { immediate: true }
);

// Merge event types with subscription selection
watch(
  [rawEventTypes, subscriptionData],
  ([et]) => {
    if (!et) return;

    function mapper(eventType: EventType): SelectableEventType {
      return { ...eventType, selected: false };
    }

    // In tutorial mode, pre-select all event types (the one just created in the previous step)
    const selectedNames = subscriptionData.value
      ? subscriptionData.value.event_types
      : props.tutorialMode
        ? et.map((e) => e.event_type_name)
        : ([] as string[]);

    eventTypes.value = intersectWith<SelectableEventType, SelectableEventType, string>(
      (a) => a.event_type_name,
      (selectableType: SelectableEventType[]) => ({
        ...selectableType[0],
        selected: selectableType.some((type) => type.selected),
      }),
      et.map(mapper),
      SelectedEventTypesFromEventTypeNames(selectedNames)
    );
  },
  { immediate: true }
);

function navigateBack() {
  router.back();
}

function onHeadersUpdate(pairs: Hook0KeyValueKeyValuePair[] | Record<string, string>) {
  headersMap.value = Array.isArray(pairs) ? kvPairsToRecord(pairs) : pairs;
}

function onLabelsUpdate(pairs: Hook0KeyValueKeyValuePair[] | Record<string, string>) {
  labelsMap.value = Array.isArray(pairs) ? kvPairsToRecord(pairs) : pairs;
}

function onMetadataUpdate(pairs: Hook0KeyValueKeyValuePair[] | Record<string, string>) {
  metadataMap.value = Array.isArray(pairs) ? kvPairsToRecord(pairs) : pairs;
}

/**
 * Map 422 API validation errors to inline form field errors.
 */
function handleValidationError(err: unknown) {
  if (!isAxiosError(err)) {
    displayError(handleError(err as Parameters<typeof handleError>[0]));
    return;
  }

  const problem = handleError(err as Parameters<typeof handleError>[0]);
  if (problem.status === 422) {
    const detail = problem.detail || '';
    // Map known API field names to VeeValidate field names
    const fieldMap: Record<string, 'description' | 'target_method' | 'target_url'> = {
      description: 'description',
      'target.url': 'target_url',
      'target.method': 'target_method',
      url: 'target_url',
      method: 'target_method',
    };

    let mapped = false;
    for (const [apiField, formField] of Object.entries(fieldMap)) {
      if (detail.toLowerCase().includes(apiField.toLowerCase())) {
        setFieldError(formField, detail);
        mapped = true;
      }
    }

    if (!mapped) {
      displayError(problem);
    }
    return;
  }

  displayError(problem);
}

const onSubmit = handleSubmit((values) => {
  if (isNew.value) {
    createMutation.mutate(
      {
        application_id: applicationId.value,
        target: {
          type: 'http' as const,
          method: values.target_method,
          url: values.target_url,
          headers: toApiRecord(headersMap.value),
        },
        description: values.description,
        metadata: toApiRecord(metadataMap.value),
        labels: toApiRecord(labelsMap.value),
        is_enabled: isEnabled.value,
        event_types: EventTypeNamesFromSelectedEventTypes(eventTypes.value),
        retry_schedule_id: selectedRetryScheduleId.value || null,
      },
      {
        onSuccess: () => {
          trackEvent('subscription', 'create', 'success');
          if (props.tutorialMode) {
            emit('tutorial-subscription-created');
          } else {
            navigateBack();
          }
        },
        onError: (err) => {
          handleValidationError(err);
        },
      }
    );
    return;
  }

  updateMutation.mutate(
    {
      subscriptionId: subscriptionId.value,
      subscription: {
        target: {
          type: 'http' as const,
          method: values.target_method,
          url: values.target_url,
          headers: toApiRecord(headersMap.value),
        },
        description: values.description,
        metadata: toApiRecord(metadataMap.value),
        labels: toApiRecord(labelsMap.value),
        is_enabled: isEnabled.value,
        event_types: EventTypeNamesFromSelectedEventTypes(eventTypes.value),
        dedicated_workers: dedicatedWorkers.value.length > 0 ? dedicatedWorkers.value : undefined,
        application_id: applicationId.value,
        retry_schedule_id: selectedRetryScheduleId.value || null,
      },
    },
    {
      onSuccess: () => {
        trackEvent('subscription', 'update', 'success');
        navigateBack();
      },
      onError: (err) => {
        handleValidationError(err);
      },
    }
  );
});

// Computed: whether the non-validated parts are ready
const hasRequiredLabels = computed(() => {
  const entries = Object.entries(labelsMap.value);
  return (
    entries.length > 0 && entries.every(([k, v]) => k.trim().length > 0 && v.trim().length > 0)
  );
});
const hasSelectedEventTypes = computed(() => eventTypes.value.some((et) => et.selected));

const missingFieldsTooltip = computed(() => {
  const missing: string[] = [];
  if (!targetUrl.value) missing.push(t('subscriptions.fields.endpoint'));
  if (!description.value) missing.push(t('subscriptions.fields.name'));
  if (!hasSelectedEventTypes.value) missing.push(t('subscriptions.fields.eventTypes'));
  if (!hasRequiredLabels.value) missing.push(t('subscriptions.fields.labels'));
  if (missing.length === 0) return undefined;
  return t('forms.missingFields', { fields: missing.join(', ') });
});
</script>

<template>
  <Hook0PageLayout :title="isNew ? t('subscriptions.createTitle') : t('subscriptions.editTitle')">
    <Hook0Stack direction="column" gap="lg">
      <!-- Loading subscription for edit mode -->
      <Hook0Card v-if="!isNew && subLoading">
        <Hook0CardHeader>
          <template #header>{{ t('subscriptions.editTitle') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0Stack direction="column" gap="md">
            <Hook0Skeleton size="hero" />
            <Hook0Skeleton size="heading" />
            <Hook0Skeleton size="heading" />
          </Hook0Stack>
        </Hook0CardContent>
      </Hook0Card>

      <!-- Error loading subscription -->
      <Hook0ErrorCard v-else-if="!isNew && subError" :error="subError" @retry="refetchSub()" />

      <!-- Form -->
      <template v-else>
        <!-- Secret (edit mode only, above the form card) -->
        <Hook0Card v-if="!isNew" data-test="subscription-secret-card">
          <Hook0CardContent>
            <div class="sub-secret">
              <span class="sub-secret__title">{{ t('subscriptions.secretLabel') }}</span>
              <span class="sub-secret__hint">
                <i18n-t keypath="subscriptions.secretHint" tag="span">
                  <template #link>
                    <Hook0Button
                      variant="link"
                      href="https://documentation.hook0.com/docs/verifying-webhook-signatures"
                      target="_blank"
                      >{{ t('subscriptions.authenticateWebhooks') }}</Hook0Button
                    >
                  </template>
                </i18n-t>
              </span>
              <Hook0CopyField :value="secret" maskable />
            </div>
          </Hook0CardContent>
        </Hook0Card>

        <Hook0Form data-test="subscription-form" @submit="onSubmit">
          <Hook0Card data-test="subscription-card">
            <Hook0CardHeader>
              <template v-if="isNew" #header>{{ t('subscriptions.createTitle') }}</template>
              <template v-else #header>{{ t('subscriptions.editTitle') }}</template>
              <template #subtitle>{{ t('subscriptions.formSubtitle') }}</template>
            </Hook0CardHeader>

            <Hook0CardContent>
              <div class="sub-form">
                <!-- Section 1: Basics -->
                <div class="sub-form__section-header">
                  <span class="sub-form__section-num">1</span>
                  <span class="sub-form__section-title">{{
                    t('subscriptions.sectionBasics')
                  }}</span>
                </div>
                <SubscriptionSectionBasics
                  :description="description"
                  :description-attrs="descriptionAttrs"
                  :description-error="errors.description"
                  :target-method="targetMethod"
                  :target-method-attrs="targetMethodAttrs"
                  :target-method-error="errors.target_method"
                  :target-url="targetUrl"
                  :target-url-attrs="targetUrlAttrs"
                  :target-url-error="errors.target_url"
                  :http-methods="httpMethods"
                  :autofocus="isNew"
                  @update:description="description = $event"
                  @update:target-method="targetMethod = $event"
                  @update:target-url="targetUrl = $event"
                />

                <div class="sub-form__divider" />

                <!-- Section 2: Filtering -->
                <div class="sub-form__section-header">
                  <span class="sub-form__section-num">2</span>
                  <span class="sub-form__section-title">{{
                    t('subscriptions.sectionFiltering')
                  }}</span>
                </div>

                <Hook0Stack direction="column" gap="md">
                  <SubscriptionSectionEventTypes
                    :event-types="eventTypes"
                    :loading="etLoading"
                    :error="etError"
                    @update:event-types="eventTypes = $event"
                    @refresh="refetchEt()"
                  />

                  <SubscriptionSectionLabels
                    :labels="labels"
                    @update:labels="onLabelsUpdate($event)"
                  />
                </Hook0Stack>

                <div class="sub-form__divider" />

                <!-- Section 3: Optional -->
                <div class="sub-form__section-header sub-form__section-header--muted">
                  <span class="sub-form__section-num sub-form__section-num--muted">3</span>
                  <span class="sub-form__section-title">{{
                    t('subscriptions.sectionAdvanced')
                  }}</span>
                  <span class="sub-form__section-badge">{{ t('common.optional') }}</span>
                </div>
                <SubscriptionSectionAdvanced
                  :headers-kv="headersKv"
                  :metadata="metadata"
                  :retry-schedule-id="selectedRetryScheduleId"
                  :retry-schedule-options="retryScheduleOptions"
                  @update:headers="onHeadersUpdate($event)"
                  @update:metadata="onMetadataUpdate($event)"
                  @update:retry-schedule-id="selectedRetryScheduleId = $event"
                />
              </div>
            </Hook0CardContent>

            <Hook0CardFooter>
              <Hook0Button
                v-if="!props.tutorialMode"
                variant="secondary"
                type="button"
                data-test="subscription-cancel-button"
                @click="navigateBack()"
                >{{ t('common.cancel') }}</Hook0Button
              >
              <Hook0Button
                v-if="
                  !tutorialMode && (isNew ? canCreate('subscription') : canEdit('subscription'))
                "
                variant="primary"
                type="button"
                :loading="createMutation.isPending.value || updateMutation.isPending.value"
                :disabled="
                  !targetUrl || !description || !hasRequiredLabels || !hasSelectedEventTypes
                "
                :tooltip="missingFieldsTooltip"
                data-test="subscription-submit-button"
                @click="onSubmit"
                >{{ isNew ? t('common.create') : t('common.save') }}
              </Hook0Button>

              <Hook0Button
                v-else
                variant="primary"
                type="submit"
                :loading="createMutation.isPending.value"
                :disabled="
                  !targetUrl || !description || !hasRequiredLabels || !hasSelectedEventTypes
                "
                :tooltip="missingFieldsTooltip"
                data-test="subscription-submit-button"
                @click="onSubmit"
                >{{ t('subscriptions.createFirstSubscription') }}
              </Hook0Button>
            </Hook0CardFooter>
          </Hook0Card>
        </Hook0Form>

        <SubscriptionsRemove
          v-if="!isNew && subscriptionId && canDelete('subscription')"
          :subscription-id="subscriptionId"
          :subscription-name="description || ''"
          :application-id="applicationId"
        ></SubscriptionsRemove>
      </template>
    </Hook0Stack>
  </Hook0PageLayout>
</template>

<style scoped>
/* Secret card */
.sub-secret {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.sub-secret__title {
  font-size: 0.875rem;
  font-weight: 700;
  color: var(--color-text-primary);
}

.sub-secret__hint {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  line-height: 1.5;
}

.sub-secret__hint :deep(.hook0-button) {
  padding-top: 0;
  padding-bottom: 0;
}

.sub-secret__warning {
  font-size: 0.75rem;
  font-style: italic;
  font-weight: 500;
  color: var(--color-warning);
}

.sub-form__row-hint--emphasis {
  font-style: italic;
  font-weight: 500;
}

/* Form sections layout */
.sub-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem 0;
}

.sub-form__section-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.sub-form__section-num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 1.375rem;
  height: 1.375rem;
  border-radius: var(--radius-full);
  background-color: var(--color-primary);
  color: var(--color-primary-text, #fff);
  font-size: 0.6875rem;
  font-weight: 700;
  flex-shrink: 0;
}

.sub-form__section-num--muted {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-muted);
}

.sub-form__section-title {
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.sub-form__section-header--muted .sub-form__section-title {
  color: var(--color-text-muted);
}

.sub-form__section-badge {
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--color-text-muted);
  padding: 0.0625rem 0.625rem;
  border-radius: var(--radius-full);
  background-color: var(--color-bg-tertiary);
}

.sub-form__divider {
  height: 1px;
  background-color: var(--color-border);
  margin: 1.5rem 0;
}

@media (max-width: 639px) {
  .sub-form__divider {
    margin: 0.75rem 0;
  }
}
</style>
