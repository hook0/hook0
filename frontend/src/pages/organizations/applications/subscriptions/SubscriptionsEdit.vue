<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { RefreshCw } from 'lucide-vue-next';

import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';
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
import type { Hook0SelectSingleOption } from '@/components/Hook0Select';
import type { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';

import SubscriptionsRemove from './SubscriptionsRemove.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0List from '@/components/Hook0List.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
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
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Form from '@/components/Hook0Form.vue';

const { t } = useI18n();
const { trackEvent } = useTracking();

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

// Form state
const subscription = ref({
  created_at: '',
  secret: '',
  description: '',
  application_id: '',
  event_types: [] as string[],
  dedicated_workers: [] as string[],
  is_enabled: true,
  labels: {} as Record<string, string>,
  metadata: {} as Record<string, string>,
  target: {
    type: 'http',
    method: '',
    url: '',
    headers: {} as Record<string, string>,
  },
});

const eventTypes = ref<SelectableEventType[]>([]);
const labels = ref<Hook0KeyValueKeyValuePair[]>([]);
const metadata = ref<Hook0KeyValueKeyValuePair[]>([]);

const httpTarget = ref({
  METHODS: 'GET,PATCH,POST,PUT,DELETE,OPTIONS,HEAD'.split(',').map(toOption),
  headers: [] as Hook0KeyValueKeyValuePair[],
});

// Populate form from subscription data (edit mode)
// Clone data to avoid readonly proxy from TanStack Query cache
watch(subscriptionData, (sub) => {
  if (sub) {
    subscription.value.secret = sub.secret;
    subscription.value.created_at = sub.created_at;
    subscription.value.description = sub.description || '';
    subscription.value.event_types = [...sub.event_types];
    subscription.value.is_enabled = sub.is_enabled;
    labels.value = fromMap(sub.labels);
    subscription.value.labels = { ...sub.labels };
    metadata.value = fromMap(sub.metadata);
    subscription.value.metadata = { ...sub.metadata };
    subscription.value.dedicated_workers = [...sub.dedicated_workers];
    subscription.value.target = {
      type: sub.target.type,
      method: sub.target.method,
      url: sub.target.url,
      headers: { ...sub.target.headers },
    };
    httpTarget.value.headers = fromMap(sub.target.headers);
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

    eventTypes.value = intersectWith<SelectableEventType, SelectableEventType, string>(
      (a) => a.event_type_name,
      (selectableType: SelectableEventType[]) => ({
        ...selectableType[0],
        selected: selectableType.some((type) => type.selected),
      }),
      et.map(mapper),
      SelectedEventTypesFromEventTypeNames(subscription.value.event_types)
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

function upsert(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  if (!subscription.value.metadata) {
    subscription.value.metadata = toMap(metadata.value);
  }

  if (isNew.value) {
    createMutation.mutate(
      {
        application_id: applicationId.value,
        target: {
          type: 'http' as const,
          method: subscription.value.target.method,
          url: subscription.value.target.url,
          headers: subscription.value.target.headers as unknown as Record<string, never>,
        },
        description: subscription.value.description,
        metadata: subscription.value.metadata as unknown as Record<string, never>,
        labels: subscription.value.labels as unknown as Record<string, never>,
        is_enabled: subscription.value.is_enabled,
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
          displayError(err as unknown as Problem);
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
          method: subscription.value.target.method,
          url: subscription.value.target.url,
          headers: subscription.value.target.headers as unknown as Record<string, never>,
        },
        description: subscription.value.description,
        metadata: subscription.value.metadata as unknown as Record<string, never>,
        labels: subscription.value.labels as unknown as Record<string, never>,
        is_enabled: subscription.value.is_enabled,
        event_types: EventTypeNamesFromSelectedEventTypes(eventTypes.value),
        dedicated_workers:
          subscription.value.dedicated_workers.length > 0
            ? subscription.value.dedicated_workers
            : undefined,
        application_id: applicationId.value,
      },
    },
    {
      onSuccess: () => {
        trackEvent('subscription', 'update', 'success');
        cancel2();
      },
      onError: (err) => {
        displayError(err as unknown as Problem);
      },
    }
  );
}
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
      <Hook0Form data-test="subscription-form" @submit="upsert">
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
                  v-model="subscription.description"
                  type="text"
                  :placeholder="t('subscriptions.descriptionPlaceholder')"
                  required
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
                  <Hook0Input v-model="subscription.secret" type="text" disabled> </Hook0Input>
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
                    v-model="subscription.target.method"
                    :options="httpTarget.METHODS"
                    data-test="subscription-method-select"
                  ></Hook0Select>
                  <Hook0Input
                    v-model="subscription.target.url"
                    type="text"
                    placeholder="https://"
                    required
                    data-test="subscription-url-input"
                  >
                  </Hook0Input>
                </Hook0Stack>
                <Hook0HelpText>
                  If you just want to run some tests, you can go to
                  <Hook0Button href="https://webhook.site" target="_blank"
                    >Webhook.site</Hook0Button
                  >
                  to obtain a unique URL. Keep the page open and use the unique URL here with any
                  HTTP verb!
                </Hook0HelpText>
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine>
              <template #label>{{ t('subscriptions.endpointHeaders') }}</template>
              <template #content>
                <Hook0KeyValue
                  :value="httpTarget.headers"
                  key-placeholder="header name"
                  value-placeholder="value"
                  @update:model-value="subscription.target.headers = toMap($event)"
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
                  @update:model-value="subscription.labels = toMap($event)"
                ></Hook0KeyValue>
              </template>
            </Hook0CardContentLine>

            <Hook0CardContentLine>
              <template #label>
                <Hook0Stack direction="row" gap="xs" align="center">
                  <Hook0Text variant="primary">{{ t('subscriptions.selectEventTypes') }}</Hook0Text>
                  <Hook0Button :to="{ name: routes.EventTypesList }" target="_blank">{{
                    t('eventTypes.title')
                  }}</Hook0Button>
                  <Hook0Button @click="refetchEt()">
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
                <Hook0List v-else data-test="event-types-list">
                  <Hook0ListItem
                    v-for="(eventType, index) in eventTypes"
                    :key="index"
                    :data-test="`event-type-item-${index}`"
                  >
                    <template #left>
                      <Hook0Checkbox
                        v-model="eventType.selected"
                        :data-test="`event-type-checkbox-${index}`"
                      >
                        <Hook0Text
                          variant="primary"
                          weight="medium"
                          :data-test="`event-type-label-${index}`"
                        >
                          {{ eventType.event_type_name }}
                        </Hook0Text>
                      </Hook0Checkbox>
                    </template>
                  </Hook0ListItem>
                </Hook0List>
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
                @update:model-value="subscription.metadata = toMap($event)"
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
              v-if="!tutorialMode"
              variant="primary"
              type="button"
              :loading="createMutation.isPending.value || updateMutation.isPending.value"
              :disabled="
                !subscription.target.url ||
                !subscription.description ||
                Object.keys(subscription.labels).length <= 0 ||
                !eventTypes.some((et) => et.selected)
              "
              data-test="subscription-submit-button"
              @click="upsert($event)"
              >{{ isNew ? t('common.create') : t('common.edit') }}
            </Hook0Button>

            <Hook0Button
              v-else
              variant="primary"
              type="submit"
              :loading="createMutation.isPending.value"
              :disabled="
                !subscription.target.url ||
                !subscription.description ||
                Object.keys(subscription.labels).length <= 0 ||
                !eventTypes.some((et) => et.selected)
              "
              data-test="subscription-submit-button"
              @click="upsert($event)"
              >{{ t('subscriptions.createFirstSubscription') }}
            </Hook0Button>
          </Hook0CardFooter>
        </Hook0Card>
      </Hook0Form>

      <SubscriptionsRemove
        v-if="!isNew && subscriptionId"
        :subscription-id="subscriptionId"
        :subscription-name="subscription.description"
        :application-id="applicationId"
      ></SubscriptionsRemove>
    </template>
  </Hook0Stack>
</template>

<style scoped>
/* No custom CSS - using Hook0* components only */
</style>
