<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { onMounted, onUpdated, ref } from 'vue';
import { head } from 'ramda';

import { Problem, UUID } from '@/http';
import * as SubscriptionService from './SubscriptionService';
import { Subscription, Target } from './SubscriptionService';
import { routes } from '@/routes';
import SubscriptionsRemove from './SubscriptionsRemove.vue';
import * as EventTypeService from '../event_types/EventTypeService';
import { EventType } from '../event_types/EventTypeService';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0List from '@/components/Hook0List.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0KeyValue from '@/components/Hook0KeyValue.vue';
import { Hook0SelectSingleOption } from '@/components/Hook0Select';
import { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import { intersectWith } from '@/utils/fp';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import { push } from 'notivue';

interface SelectableEventType extends EventType {
  selected: boolean;
}

function EventTypeNamesFromSelectedEventTypes(
  selectableEventTypes: SelectableEventType[]
): string[] {
  return selectableEventTypes
    .filter((eventType) => eventType.selected)
    .map((eventType) => EventTypeFromSelectableEventType(eventType).event_type_name);
}

function EventTypeFromSelectableEventType(selectableEventTypes: SelectableEventType) {
  return {
    event_type_name: selectableEventTypes.event_type_name,
    resource_type_name: selectableEventTypes.resource_type_name,
    service_name: selectableEventTypes.service_name,
    verb_name: selectableEventTypes.verb_name,
  };
}

function SelectedEventTypesFromEventTypeNames(eventTypeNames: string[]): SelectableEventType[] {
  return eventTypeNames.map((eventTypeName) => SelectableEventTypeFromEventTypeName(eventTypeName));
}

function SelectableEventTypeFromEventTypeName(eventTypeName: string): SelectableEventType {
  const [resource_type_name, service_name, verb_name] = eventTypeName.split('.');

  return {
    event_type_name: eventTypeName,
    resource_type_name,
    service_name,
    verb_name,
    selected: true,
  };
}

function toOption(val: string): Hook0SelectSingleOption {
  return {
    value: val,
    label: val,
  };
}

const router = useRouter();
const route = useRoute();

const isNew = ref(true);
const subscription_id = ref<UUID | null>(null);
const subscription = ref({
  created_at: '',
  secret: '',
  description: '',
  application_id: '',
  event_types: [] as string[],
  dedicated_workers: [] as string[],
  is_enabled: true,
  label_key: '',
  label_value: '',
  metadata: {}, // K/V as object
  target: {
    type: 'http',
    method: '',
    url: '',
    headers: {},
  } as Target,
});
const eventTypes = ref<SelectableEventType[]>([]);

const httpTarget = ref({
  METHODS: 'GET,PATCH,POST,PUT,DELETE,OPTIONS,HEAD'.split(',').map(toOption),
  headers: [] as Hook0KeyValueKeyValuePair[], // K/V
});

function _load() {
  function mapper(eventType: EventType): SelectableEventType {
    return {
      ...eventType,
      selected: false,
    };
  }

  if (subscription_id.value !== route.params.subscription_id) {
    subscription_id.value = route.params.subscription_id as UUID;
    isNew.value = !subscription_id.value;
    eventTypes.value = [];

    // first load the subscription if in edit mode
    (!isNew.value
      ? SubscriptionService.get(subscription_id.value).then((sub: Subscription) => {
          // We do the mapping bellow (instead of a single-line assignment) to stay in control of what we manage
          subscription.value.secret = sub.secret;
          subscription.value.created_at = sub.created_at;
          subscription.value.description = sub.description || '';

          subscription.value.event_types = sub.event_types;

          subscription.value.is_enabled = sub.is_enabled;
          subscription.value.label_key = sub.label_key;
          subscription.value.label_value = sub.label_value;
          subscription.value.metadata = sub.metadata;
          subscription.value.dedicated_workers = sub.dedicated_workers;

          // currently PaperClip does not handle our "enum Target" on Rust-side and yield a string
          subscription.value.target = sub.target as unknown as Target;
          httpTarget.value.headers = fromMap(
            (subscription.value.target as unknown as Target).headers
          );
        })
      : Promise.resolve()
    )
      // then (always) load the eventTypes
      .then(() => EventTypeService.list(route.params.application_id as string))
      .then((et) => {
        // apply selection if any event_types were already selected (edit mode)
        // this will display previously selected event type names that are not available anymore
        eventTypes.value = intersectWith<SelectableEventType, SelectableEventType, string>(
          (a) => a.event_type_name,

          // depending on the user event type selection, for a single event type we might have one or more events
          (selectableType: SelectableEventType[]) => {
            return {
              ...(head(selectableType) as SelectableEventType),
              selected: selectableType.some((type) => type.selected),
            };
          },

          et.map(mapper),
          SelectedEventTypesFromEventTypeNames(subscription.value.event_types)
        );
      })
      .catch(displayError);
  }
}

function cancel2() {
  router.back();
}

function fromMap(headers: Record<string, string>): Hook0KeyValueKeyValuePair[] {
  return Object.entries(headers).map(([key, value]) => ({ key, value }));
}

function toMap(pairs: Hook0KeyValueKeyValuePair[]): Record<string, string> {
  return pairs.reduce((m, { key, value }) => {
    // @ts-ignore
    m[key] = value;
    return m;
  }, {});
}

function upsert(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  if (isNew.value) {
    SubscriptionService.create({
      application_id: route.params.application_id as string,

      target: {
        type: 'http',
        method: subscription.value.target.method,
        url: subscription.value.target.url,
        headers: subscription.value.target.headers,
      },
      description: subscription.value.description,
      metadata: subscription.value.metadata,
      label_value: subscription.value.label_value,
      label_key: subscription.value.label_key,
      is_enabled: subscription.value.is_enabled,
      event_types: EventTypeNamesFromSelectedEventTypes(eventTypes.value),
    }).then((_resp) => {
      cancel2();
    }, displayError);
    return;
  }

  SubscriptionService.update(subscription_id.value as UUID, {
    target: {
      type: 'http',
      method: subscription.value.target.method,
      url: subscription.value.target.url,
      headers: subscription.value.target.headers,
    },
    description: subscription.value.description,
    metadata: subscription.value.metadata,
    label_value: subscription.value.label_value,
    label_key: subscription.value.label_key,
    is_enabled: subscription.value.is_enabled,
    event_types: EventTypeNamesFromSelectedEventTypes(eventTypes.value),
    dedicated_workers:
      subscription.value.dedicated_workers.length > 0
        ? subscription.value.dedicated_workers
        : undefined,
    application_id: route.params.application_id as string,
  }).then((_resp) => {
    cancel2();
  }, displayError);
}

function displayError(err: Problem) {
  console.error(err);
  let options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}

onMounted(() => {
  _load();
});

onUpdated(() => {
  _load();
});
</script>

<template>
  <div>
    <form @submit="upsert">
      <Hook0Card>
        <Hook0CardHeader>
          <template v-if="isNew" #header> Create new subscription (webhook) </template>
          <template v-else #header> Edit subscription (webhook) </template>
          <template #subtitle>
            A subscription (webhook) receives all events sent to Hook0 that match its filters.
          </template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine v-if="!isNew">
            <template #label>
              Subscription secret

              <Hook0Text class="helpText mt-2 block">
                The secret is used to
                <Hook0Button
                  href="https://documentation.hook0.com/docs/verifying-webhook-signatures"
                  target="_blank"
                  >authenticate the webhooks</Hook0Button
                >
                sent by Hook0.
                <em>Do not share this secret with anyone.</em>
              </Hook0Text>
            </template>

            <template #content>
              <div class="flex flex-row">
                <Hook0Input
                  v-model="subscription.secret"
                  type="text"
                  class="w-full disabled:bg-slate-50 disabled:text-slate-500 disabled:border-slate-200 disabled:shadow-none"
                  disabled
                >
                </Hook0Input>
              </div>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label> Endpoint HTTP verb and url </template>
            <template #content>
              <div class="flex flex-row">
                <Hook0Select
                  v-model="subscription.target.method"
                  class="flex-none width-small"
                  :options="httpTarget.METHODS"
                ></Hook0Select>
                <Hook0Input
                  v-model="subscription.target.url"
                  type="text"
                  class="w-full ml-1"
                  placeholder="https://"
                  required
                >
                </Hook0Input>
              </div>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label> Endpoint headers </template>
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
            <template #label> Subscription description </template>
            <template #content>
              <Hook0Input
                v-model="subscription.description"
                type="text"
                placeholder="my awesome api - production"
                required
              >
                <template #helpText>Describe what your subscription will do </template>
              </Hook0Input>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label>
              Subscription labels

              <Hook0Text class="helpText mt-2 block">
                Hook0 will only forward events to subscriptions that have the same
                <Hook0Text class="code">label_key</Hook0Text>
                and
                <Hook0Text class="code">label_value</Hook0Text>
                as specified in the event.
              </Hook0Text>

              <Hook0Text class="helpText mt-2 block"> </Hook0Text>
            </template>
            <template #content>
              <div class="flex flex-row">
                <Hook0Input
                  v-model="subscription.label_key"
                  type="text"
                  class="w-full"
                  placeholder="label_key"
                  required
                >
                </Hook0Input>

                <Hook0Input
                  v-model="subscription.label_value"
                  type="text"
                  class="w-full ml-1"
                  placeholder="label_value"
                  required
                >
                </Hook0Input>
              </div>
            </template>
          </Hook0CardContentLine>

          <Hook0CardContentLine>
            <template #label>
              <Hook0Text>
                Select
                <Hook0Button :to="{ name: routes.EventTypesList }" target="_blank"
                  >event types</Hook0Button
                >
                to listen to
                <Hook0Button @click="_load()">
                  <Hook0Icon name="fa-arrows-rotate"></Hook0Icon>
                </Hook0Button>
              </Hook0Text>
            </template>
            <template #content>
              <Hook0Loader v-if="eventTypes === null"></Hook0Loader>
              <Hook0List v-else>
                <Hook0ListItem v-for="(eventType, index) in eventTypes" :key="index">
                  <template #left>
                    <Hook0Input
                      :id="'event_' + index"
                      type="checkbox"
                      :value="eventType.selected"
                      @input="eventType.selected = !eventType.selected"
                    ></Hook0Input>
                    <label
                      :for="'event_' + index"
                      class="font-medium text-gray-700 select-none ml-2 cursor-pointer"
                    >
                      <Hook0Text>{{ eventType.event_type_name }}</Hook0Text>
                    </label>
                  </template>
                </Hook0ListItem>
              </Hook0List>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>

        <Hook0CardContentLine>
          <template #label> Metadata </template>
          <template #content>
            <Hook0KeyValue
              :value="subscription.metadata"
              key-placeholder="key"
              value-placeholder="value"
              @update:model-value="subscription.metadata = $event"
            ></Hook0KeyValue>
          </template>
        </Hook0CardContentLine>
        <Hook0CardFooter>
          <Hook0Button class="secondary" type="button" @click="cancel2()">Cancel</Hook0Button>
          <Hook0Button class="primary" type="button" @click="upsert($event)"
            >{{ isNew ? 'Create' : 'Update' }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </form>

    <SubscriptionsRemove
      v-if="!isNew && subscription_id"
      :subscription-id="subscription_id"
      :subscription-name="subscription.description"
      :application-id="
        Array.isArray(route.params.application_id)
          ? route.params.application_id[0]
          : route.params.application_id
      "
    ></SubscriptionsRemove>
  </div>
</template>
