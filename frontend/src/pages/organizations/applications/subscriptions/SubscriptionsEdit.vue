<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { RefreshCw } from 'lucide-vue-next';

import { subscriptionSchema } from './subscription.schema';
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
import { routes } from '@/routes';
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
import SubscriptionTestEndpoint from './SubscriptionTestEndpoint.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0KeyValue from '@/components/Hook0KeyValue.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Checkbox from '@/components/Hook0Checkbox.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';
import Hook0Form from '@/components/Hook0Form.vue';

const { t } = useI18n();
const { trackEvent } = useTracking();

// Permissions
const { canCreate, canEdit, canDelete } = usePermissions();

type Props = {
  tutorialMode?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  tutorialMode: false,
});

const emit = defineEmits(['tutorial-subscription-created']);

type SelectableEventType = EventType & {
  selected: boolean;
}

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
const route = useRoute();

const subscriptionId = computed(() => {
  const id = route.params.subscription_id;
  return typeof id === 'string' ? id : '';
});
const applicationId = computed(() => route.params.application_id as string);
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
  validationSchema: toTypedSchema(subscriptionSchema),
});

const [description, descriptionAttrs] = defineField('description');
const [targetMethod, targetMethodAttrs] = defineField('target_method');
const [targetUrl, targetUrlAttrs] = defineField('target_url');

// Non-validated form state (managed outside VeeValidate)
const secret = ref('');
const createdAt = ref('');
const isEnabled = ref(true);
const dedicatedWorkers = ref<string[]>([]);
const eventTypes = ref<SelectableEventType[]>([]);
const labels = ref<Hook0KeyValueKeyValuePair[]>([]);
const metadata = ref<Hook0KeyValueKeyValuePair[]>([]);
const headersKv = ref<Hook0KeyValueKeyValuePair[]>([]);
const headersMap = ref<Record<string, string>>({});
const labelsMap = ref<Record<string, string>>({});
const metadataMap = ref<Record<string, string>>({});

const httpMethods = 'GET,PATCH,POST,PUT,DELETE,OPTIONS,HEAD'.split(',').map(toOption);

// Populate form from subscription data (edit mode)
watch(subscriptionData, (sub) => {
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
  }
});

// Merge event types with subscription selection
watch(
  [rawEventTypes, subscriptionData],
  ([et]) => {
    if (!et) return;

    function mapper(eventType: EventType): SelectableEventType {
      return { ...eventType, selected: false };
    }

    const selectedNames = subscriptionData.value
      ? subscriptionData.value.event_types
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
const hasRequiredLabels = computed(() => Object.keys(labelsMap.value).length > 0);
const hasSelectedEventTypes = computed(() => eventTypes.value.some((et) => et.selected));
</script>

<template>
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
      <Hook0Form data-test="subscription-form" @submit="onSubmit">
        <Hook0Card data-test="subscription-card">
          <Hook0CardHeader>
            <template v-if="isNew" #header>{{ t('subscriptions.createTitle') }}</template>
            <template v-else #header>{{ t('subscriptions.editTitle') }}</template>
            <template #subtitle>{{ t('subscriptions.formSubtitle') }}</template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0CardContentLine>
              <template #label>{{ t('subscriptions.descriptionLabel') }}</template>
              <template #content>
                <Hook0Input
                  v-model="description"
                  v-bind="descriptionAttrs"
                  type="text"
                  :placeholder="t('subscriptions.descriptionPlaceholder')"
                  :error="errors.description"
                  data-test="subscription-description-input"
                >
                  <template #helpText>
                    {{ t('subscriptions.descriptionHelpText') }}
                  </template>
                </Hook0Input>
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine v-if="!isNew">
              <template #label>
                {{ t('subscriptions.secretLabel') }}

                <Hook0HelpText>
                  {{ t('subscriptions.secretDescription') }}
                  <Hook0Button
                    href="https://documentation.hook0.com/docs/verifying-webhook-signatures"
                    target="_blank"
                    >{{ t('subscriptions.authenticateWebhooks') }}</Hook0Button
                  >.
                  <Hook0HelpText tone="emphasis">{{
                    t('subscriptions.secretWarning')
                  }}</Hook0HelpText>
                </Hook0HelpText>
              </template>

              <template #content>
                <Hook0Stack direction="row" gap="xs">
                  <Hook0Input v-model="secret" type="text" disabled> </Hook0Input>
                </Hook0Stack>
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine>
              <template #label>
                {{ t('subscriptions.httpEndpoint') }}
                <Hook0HelpText>{{ t('subscriptions.httpEndpointHelp') }}</Hook0HelpText>
              </template>
              <template #content>
                <Hook0Stack direction="row" gap="xs">
                  <Hook0Select
                    v-model="targetMethod"
                    v-bind="targetMethodAttrs"
                    :options="httpMethods"
                    :error="errors.target_method"
                    data-test="subscription-method-select"
                  ></Hook0Select>
                  <Hook0Input
                    v-model="targetUrl"
                    v-bind="targetUrlAttrs"
                    type="text"
                    placeholder="https://"
                    :error="errors.target_url"
                    data-test="subscription-url-input"
                  >
                  </Hook0Input>
                </Hook0Stack>
                <Hook0HelpText>
                  <i18n-t keypath="subscriptions.webhookSiteHelp" tag="span">
                    <template #link>
                      <Hook0Button href="https://webhook.site" target="_blank">{{
                        t('subscriptions.webhookSiteName')
                      }}</Hook0Button>
                    </template>
                  </i18n-t>
                </Hook0HelpText>

                <!-- Test Endpoint -->
                <SubscriptionTestEndpoint :target-url="targetUrl || ''" />
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine>
              <template #label>{{ t('subscriptions.endpointHeaders') }}</template>
              <template #content>
                <Hook0KeyValue
                  :value="headersKv"
                  :key-placeholder="t('common.headerName')"
                  :value-placeholder="t('common.value')"
                  @update:model-value="onHeadersUpdate($event)"
                ></Hook0KeyValue>
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine>
              <template #label>
                {{ t('subscriptions.subscriptionLabels') }}
                <Hook0HelpText>{{ t('subscriptions.subscriptionLabelsHelp') }}</Hook0HelpText>
              </template>
              <template #content>
                <Hook0KeyValue
                  :value="labels"
                  :key-placeholder="t('common.labelKey')"
                  :value-placeholder="t('common.labelValue')"
                  data-test="subscription-labels"
                  @update:model-value="onLabelsUpdate($event)"
                ></Hook0KeyValue>
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine>
              <template #label>
                <Hook0Stack direction="column" gap="xs">
                  <span class="sub-edit__field-label">
                    <i18n-t keypath="subscriptions.selectEventTypesWithLink" tag="span">
                      <template #link>
                        <Hook0Button
                          variant="link"
                          :to="{ name: routes.EventTypesList }"
                          target="_blank"
                          >{{ t('eventTypes.title') }}</Hook0Button
                        >
                      </template>
                    </i18n-t>
                  </span>
                  <Hook0Button
                    variant="ghost"
                    size="sm"
                    :aria-label="t('subscriptions.refreshEventTypes')"
                    :title="t('subscriptions.refreshEventTypes')"
                    @click="refetchEt()"
                  >
                    <RefreshCw :size="14" aria-hidden="true" />
                  </Hook0Button>
                </Hook0Stack>
              </template>
              <template #content>
                <!-- Loading event types -->
                <Hook0Loader v-if="etLoading"></Hook0Loader>

                <!-- Error loading event types -->
                <Hook0ErrorCard v-else-if="etError" :error="etError" @retry="refetchEt()" />

                <!-- Event types list -->
                <ul v-else class="event-type-list" data-test="event-types-list">
                  <li
                    v-for="(eventType, index) in eventTypes"
                    :key="index"
                    class="event-type-list__item"
                    :data-test="`event-type-item-${index}`"
                  >
                    <Hook0Checkbox
                      v-model="eventType.selected"
                      :data-test="`event-type-checkbox-${index}`"
                    >
                      <span
                        class="sub-edit__event-type-label"
                        :data-test="`event-type-label-${index}`"
                      >
                        {{ eventType.event_type_name }}
                      </span>
                    </Hook0Checkbox>
                  </li>
                </ul>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>

          <Hook0CardContentLine>
            <template #label>{{ t('subscriptions.metadataLabel') }}</template>
            <template #content>
              <Hook0KeyValue
                :value="metadata"
                :key-placeholder="t('common.key')"
                :value-placeholder="t('common.value')"
                @update:model-value="onMetadataUpdate($event)"
              ></Hook0KeyValue>
            </template>
          </Hook0CardContentLine>
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
              v-if="!tutorialMode && (isNew ? canCreate('subscription') : canEdit('subscription'))"
              variant="primary"
              type="button"
              :loading="createMutation.isPending.value || updateMutation.isPending.value"
              :disabled="!targetUrl || !description || !hasRequiredLabels || !hasSelectedEventTypes"
              data-test="subscription-submit-button"
              @click="onSubmit"
              >{{ isNew ? t('common.create') : t('common.edit') }}
            </Hook0Button>

            <Hook0Button
              v-else
              variant="primary"
              type="submit"
              :loading="createMutation.isPending.value"
              :disabled="!targetUrl || !description || !hasRequiredLabels || !hasSelectedEventTypes"
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
</template>

<style scoped>
.sub-edit__field-label {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}

.sub-edit__event-type-label {
  color: var(--color-text-primary);
  font-weight: 500;
  font-size: 0.875rem;
  line-height: 1.5;
}

.event-type-list {
  list-style: none;
  padding: 0;
  margin: 0;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.event-type-list__item {
  padding: 0.5rem 0.75rem;
  display: flex;
  align-items: center;
  font-size: 0.875rem;
  transition: background-color 0.15s ease;
}

.event-type-list__item + .event-type-list__item {
  border-top: 1px solid var(--color-border);
}

.event-type-list__item:hover {
  background-color: var(--color-bg-secondary);
}
</style>
