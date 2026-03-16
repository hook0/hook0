<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { RefreshCw, Send } from 'lucide-vue-next';

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
import type { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';

import SubscriptionsRemove from './SubscriptionsRemove.vue';
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
import Hook0Code from '@/components/Hook0Code.vue';

const { t } = useI18n();
const { trackEvent } = useTracking();

// Permissions
const { canCreate, canEdit, canDelete } = usePermissions();

interface Props {
  tutorialMode?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  tutorialMode: false,
});

const emit = defineEmits(['tutorial-subscription-created']);

interface SelectableEventType extends EventType {
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
    labels.value = fromMap(sub.labels);
    labelsMap.value = { ...sub.labels };
    metadata.value = fromMap(sub.metadata);
    metadataMap.value = { ...sub.metadata };
    headersKv.value = fromMap(sub.target.headers);
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

function cancel2() {
  router.back();
}

function fromMap(headers: Record<string, unknown>): Hook0KeyValueKeyValuePair[] {
  return Object.entries(headers).map(([key, value]) => ({
    key,
    value: typeof value === 'string' ? value : JSON.stringify(value),
  }));
}

function toMap(
  pairs: Hook0KeyValueKeyValuePair[] | Record<string, string>
): Record<string, string> {
  if (Array.isArray(pairs)) {
    return pairs.reduce<Record<string, string>>((m, { key, value }) => {
      m[key] = value;
      return m;
    }, {});
  }
  return pairs;
}

function onHeadersUpdate(pairs: Hook0KeyValueKeyValuePair[] | Record<string, string>) {
  headersMap.value = toMap(pairs);
}

function onLabelsUpdate(pairs: Hook0KeyValueKeyValuePair[] | Record<string, string>) {
  labelsMap.value = toMap(pairs);
}

function onMetadataUpdate(pairs: Hook0KeyValueKeyValuePair[] | Record<string, string>) {
  metadataMap.value = toMap(pairs);
}

/**
 * Map 422 API validation errors to inline form field errors.
 */
function handleValidationError(err: unknown) {
  if (!isAxiosError(err)) {
    displayError(handleError(err as never));
    return;
  }

  const problem = handleError(err as never);
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
          headers: headersMap.value as unknown as Record<string, never>,
        },
        description: values.description,
        metadata: metadataMap.value as unknown as Record<string, never>,
        labels: labelsMap.value as unknown as Record<string, never>,
        is_enabled: isEnabled.value,
        event_types: EventTypeNamesFromSelectedEventTypes(eventTypes.value),
      },
      {
        onSuccess: () => {
          trackEvent('subscription', 'create', 'success');
          if (props.tutorialMode) {
            emit('tutorial-subscription-created');
          } else {
            cancel2();
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
          headers: headersMap.value as unknown as Record<string, never>,
        },
        description: values.description,
        metadata: metadataMap.value as unknown as Record<string, never>,
        labels: labelsMap.value as unknown as Record<string, never>,
        is_enabled: isEnabled.value,
        event_types: EventTypeNamesFromSelectedEventTypes(eventTypes.value),
        dedicated_workers: dedicatedWorkers.value.length > 0 ? dedicatedWorkers.value : undefined,
        application_id: applicationId.value,
      },
    },
    {
      onSuccess: () => {
        trackEvent('subscription', 'update', 'success');
        cancel2();
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

// Test endpoint state
interface TestEndpointResult {
  status: number;
  latencyMs: number;
  body: string;
  success: boolean;
}

const testEndpointLoading = ref(false);
const testEndpointResult = ref<TestEndpointResult | null>(null);
const testEndpointError = ref<string | null>(null);

function testEndpoint() {
  const url = targetUrl.value;
  if (!url) {
    testEndpointError.value = t('subscriptions.testUrlRequired');
    return;
  }

  testEndpointLoading.value = true;
  testEndpointResult.value = null;
  testEndpointError.value = null;

  const startTime = performance.now();

  fetch(url, {
    method: 'HEAD',
    mode: 'no-cors',
    signal: AbortSignal.timeout(10000),
  })
    .then((response) => {
      const latencyMs = Math.round(performance.now() - startTime);

      // In no-cors mode, response.type is 'opaque' and status is 0
      // This means the request reached the server but we can't read the response
      if (response.type === 'opaque') {
        testEndpointResult.value = {
          status: 0,
          latencyMs,
          body: '',
          success: true,
        };
        return;
      }

      void response.text().then((body) => {
        testEndpointResult.value = {
          status: response.status,
          latencyMs,
          body: body.slice(0, 2000),
          success: response.ok,
        };
      });
    })
    .catch((err: Error) => {
      const latencyMs = Math.round(performance.now() - startTime);
      testEndpointError.value = err.message;
      testEndpointResult.value = {
        status: 0,
        latencyMs,
        body: '',
        success: false,
      };
    })
    .finally(() => {
      testEndpointLoading.value = false;
    });
}

const testStatusVariant = computed(() => {
  if (!testEndpointResult.value) return '';
  const status = testEndpointResult.value.status;
  if (status === 0 && testEndpointResult.value.success) return 'opaque';
  if (status >= 200 && status < 300) return 'success';
  if (status >= 400 && status < 500) return 'warning';
  if (status >= 500) return 'error';
  return 'error';
});
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
                <div class="test-endpoint">
                  <Hook0Button
                    variant="secondary"
                    size="sm"
                    type="button"
                    :loading="testEndpointLoading"
                    :disabled="!targetUrl"
                    data-test="subscription-test-endpoint-button"
                    @click="testEndpoint()"
                  >
                    <template #left>
                      <Send :size="14" aria-hidden="true" />
                    </template>
                    {{ t('subscriptions.testEndpoint') }}
                  </Hook0Button>

                  <!-- Test Result -->
                  <div
                    v-if="testEndpointResult"
                    class="test-endpoint__result"
                    :class="`test-endpoint__result--${testStatusVariant}`"
                    data-test="subscription-test-endpoint-result"
                  >
                    <div class="test-endpoint__result-header">
                      <span class="test-endpoint__result-title">{{
                        t('subscriptions.testResponse')
                      }}</span>
                      <span
                        class="test-endpoint__status-badge"
                        :class="`test-endpoint__status-badge--${testStatusVariant}`"
                        data-test="subscription-test-endpoint-status"
                      >
                        {{
                          testEndpointResult.status === 0
                            ? t('subscriptions.testSuccess')
                            : testEndpointResult.status
                        }}
                      </span>
                    </div>

                    <dl class="test-endpoint__details">
                      <div class="test-endpoint__detail-row">
                        <dt class="test-endpoint__detail-label">
                          {{ t('subscriptions.testLatency') }}
                        </dt>
                        <dd
                          class="test-endpoint__detail-value test-endpoint__detail-value--mono"
                          data-test="subscription-test-endpoint-latency"
                        >
                          {{ testEndpointResult.latencyMs }}ms
                        </dd>
                      </div>
                    </dl>

                    <div v-if="testEndpointResult.body" class="test-endpoint__body">
                      <span class="test-endpoint__detail-label">{{
                        t('subscriptions.testBodyPreview')
                      }}</span>
                      <Hook0Code
                        :code="testEndpointResult.body"
                        data-test="subscription-test-endpoint-body"
                      />
                    </div>

                    <Hook0HelpText
                      v-if="testEndpointResult.status === 0 && testEndpointResult.success"
                      tone="info"
                    >
                      {{ t('subscriptions.testCorsWarning') }}
                    </Hook0HelpText>
                  </div>

                  <!-- Test Error -->
                  <div
                    v-if="testEndpointError && (!testEndpointResult || !testEndpointResult.success)"
                    class="test-endpoint__error"
                    data-test="subscription-test-endpoint-error"
                  >
                    <span class="test-endpoint__error-title">{{
                      t('subscriptions.testFailed')
                    }}</span>
                    <span class="test-endpoint__error-message">{{ testEndpointError }}</span>
                  </div>
                </div>
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine>
              <template #label>{{ t('subscriptions.endpointHeaders') }}</template>
              <template #content>
                <Hook0KeyValue
                  :value="headersKv"
                  key-placeholder="header name"
                  value-placeholder="value"
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
                  key-placeholder="Label key"
                  value-placeholder="Label value"
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
                    :aria-label="t('subscriptions.refreshEventTypes')"
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
                key-placeholder="key"
                value-placeholder="value"
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
              @click="cancel2()"
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
.test-endpoint {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  margin-top: 0.75rem;
}

.test-endpoint__result {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 0.75rem;
  background-color: var(--color-bg-secondary);
}

.test-endpoint__result--success {
  border-color: var(--color-success);
  background-color: var(--color-success-light);
}

.test-endpoint__result--warning {
  border-color: var(--color-warning);
  background-color: var(--color-warning-light);
}

.test-endpoint__result--error {
  border-color: var(--color-error);
  background-color: var(--color-error-light);
}

.test-endpoint__result--opaque {
  border-color: var(--color-info);
  background-color: var(--color-info-light);
}

.test-endpoint__result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.test-endpoint__result-title {
  font-weight: 600;
  font-size: 0.8125rem;
  color: var(--color-text-primary);
}

.test-endpoint__status-badge {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.5rem;
  border-radius: var(--radius-full);
  font-size: 0.6875rem;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.test-endpoint__status-badge--success {
  background-color: var(--color-success);
  color: #ffffff;
}

.test-endpoint__status-badge--warning {
  background-color: var(--color-warning);
  color: #ffffff;
}

.test-endpoint__status-badge--error {
  background-color: var(--color-error);
  color: #ffffff;
}

.test-endpoint__status-badge--opaque {
  background-color: var(--color-info);
  color: #ffffff;
}

.test-endpoint__details {
  margin: 0;
}

.test-endpoint__detail-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.test-endpoint__detail-label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.test-endpoint__detail-value {
  font-size: 0.75rem;
  color: var(--color-text-primary);
}

.test-endpoint__detail-value--mono {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
}

.test-endpoint__body {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  margin-top: 0.5rem;
}

.test-endpoint__error {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  border: 1px solid var(--color-error);
  border-radius: var(--radius-md);
  padding: 0.75rem;
  background-color: var(--color-error-light);
}

.test-endpoint__error-title {
  font-weight: 600;
  font-size: 0.8125rem;
  color: var(--color-error);
}

.test-endpoint__error-message {
  font-size: 0.75rem;
  color: var(--color-text-secondary);
  font-family: var(--font-mono);
  word-break: break-all;
}

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
