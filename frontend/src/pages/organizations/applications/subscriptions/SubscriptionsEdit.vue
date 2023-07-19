<template>
  <div>
    <form @submit="upsert">
      <hook0-card>
        <hook0-card-header>
          <template #header v-if="isNew"> Create new subscription (webhook) </template>
          <template #header v-else> Edit subscription (webhook) </template>
          <template #subtitle>
            A subscription (webhook) receives all events sent to Hook0 that match its filters
          </template>
        </hook0-card-header>
        <hook0-card-content>
          <hook0-card-content-line>
            <template #label> Endpoint HTTP verb and url </template>
            <template #content>
              <div class="flex flex-row">
                <hook0-select
                  class="flex-none width-small"
                  :options="httpTarget.METHODS"
                  v-model="subscription.target.method"
                ></hook0-select>
                <hook0-input
                  type="text"
                  class="w-full ml-1"
                  v-model="subscription.target.url"
                  placeholder="https://"
                  required
                >
                </hook0-input>
              </div>
            </template>
          </hook0-card-content-line>

          <hook0-card-content-line>
            <template #label> Endpoint headers </template>
            <template #content>
              <hook0-key-value
                :value="httpTarget.headers"
                @update:modelValue="subscription.target.headers = toMap($event)"
                key-placeholder="header name"
                value-placeholder="value"
              ></hook0-key-value>
            </template>
          </hook0-card-content-line>

          <hook0-card-content-line>
            <template #label> Subscription description </template>
            <template #content>
              <hook0-input
                type="text"
                v-model="subscription.description"
                placeholder="my awesome api - production"
                required
              >
                <template #helpText>Describe what your subscription will do </template>
              </hook0-input>
            </template>
          </hook0-card-content-line>

          <hook0-card-content-line>
            <template #label>
              Subscription labels

              <hook0-text class="helpText mt-2 block">
                Hook0 will only forward events to subscriptions that have the same
                <hook0-text class="code">label_key</hook0-text>
                and
                <hook0-text class="code">label_value</hook0-text>
                as specified in the event.
              </hook0-text>

              <hook0-text class="helpText mt-2 block"> </hook0-text>
            </template>
            <template #content>
              <div class="flex flex-row">
                <hook0-input
                  type="text"
                  class="w-full"
                  v-model="subscription.label_key"
                  placeholder="label_key"
                  required
                >
                </hook0-input>

                <hook0-input
                  type="text"
                  class="w-full ml-1"
                  v-model="subscription.label_value"
                  placeholder="label_value"
                  required
                >
                </hook0-input>
              </div>
            </template>
          </hook0-card-content-line>

          <hook0-card-content-line>
            <template #label>
              <hook0-text>
                Select
                <hook0-button :to="{ name: routes.EventTypesList }" target="_blank"
                  >event types</hook0-button
                >
                to listen to
                <hook0-button @click="_load()">
                  <hook0-icon name="fa-arrows-rotate"></hook0-icon>
                </hook0-button>
              </hook0-text>
            </template>
            <template #content>
              <hook0-loader v-if="eventTypes === null"></hook0-loader>
              <hook0-list v-else>
                <hook0-list-item v-for="(eventType, index) in eventTypes" :key="index">
                  <template v-slot:left>
                    <hook0-input
                      type="checkbox"
                      :value="eventType.selected"
                      @input="eventType.selected = !eventType.selected"
                      :id="'event_' + index"
                    ></hook0-input>
                    <label
                      :for="'event_' + index"
                      class="font-medium text-gray-700 select-none ml-2 cursor-pointer"
                    >
                      <hook0-text>{{ eventType.event_type_name }}</hook0-text>
                    </label>
                  </template>
                </hook0-list-item>
              </hook0-list>
            </template>
          </hook0-card-content-line>
        </hook0-card-content>

        <hook0-card-content-line>
          <template #label> Metadata </template>
          <template #content>
            <hook0-key-value
              :value="subscription.metadata"
              @update:modelValue="subscription.metadata = $event"
              key-placeholder="key"
              value-placeholder="value"
            ></hook0-key-value>
          </template>
        </hook0-card-content-line>

        <hook0-card-content v-if="alert.visible">
          <hook0-alert
            :type="alert.type"
            :title="alert.title"
            :description="alert.description"
          ></hook0-alert>
        </hook0-card-content>
        <hook0-card-footer>
          <hook0-button class="secondary" type="button" @click="cancel2()">Cancel</hook0-button>
          <hook0-button class="primary" type="button" @click="upsert($event)"
            >{{ isNew ? 'Create' : 'Update' }}
          </hook0-button>
        </hook0-card-footer>
      </hook0-card>
    </form>

    <SubscriptionsRemove
      v-if="!isNew"
      :subscription-id="subscription_id"
      :subscription-name="subscription.description"
      :application-id="this.$route.params.application_id"
    ></SubscriptionsRemove>
  </div>
</template>

<script lang="ts">
import { AxiosError } from 'axios';
import * as SubscriptionService from './SubscriptionService';
import { HttpTarget, Subscription, SubscriptionPost, Target } from './SubscriptionService';
import { Options, Vue } from 'vue-class-component';
import { routes } from '@/routes';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { isAxiosError, Problem, UUID } from '@/http';
import { Alert } from '@/components/Hook0Alert';
import SubscriptionsRemove from './SubscriptionsRemove.vue';
import * as EventTypeService from '../event_types/EventTypeService';
import { EventType } from '../event_types/EventTypeService';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Error from '@/components/Hook0Error.vue';
import Hook0List from '@/components/Hook0List.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0KeyValue from '@/components/Hook0KeyValue.vue';
import { Hook0SelectSingleOption } from '@/components/Hook0Select';
import { head, KeyValuePair } from 'ramda';
import { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import { intersectWith } from '@/utils/fp';

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

@Options({
  components: {
    Hook0KeyValue,
    Hook0Select,
    Hook0Button,
    Hook0Text,
    Hook0Input,
    Hook0ListItem,
    Hook0List,
    Hook0Error,
    Hook0Loader,
    SubscriptionsRemove,
    Hook0Alert,
  },
})
export default class SubscriptionsEdit extends Vue {
  private isNew = true;

  subscription_id: UUID | null = null;

  routes = routes;

  public eventTypes: Array<SelectableEventType> = [];

  httpTarget = {
    METHODS: 'GET,PATCH,POST,PUT,DELETE,OPTIONS,HEAD'.split(',').map(toOption),
    headers: [] as Hook0KeyValueKeyValuePair[], // K/V
  };

  subscription = {
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
  };

  alert: Alert = {
    visible: false,
    type: 'alert',
    title: '',
    description: '',
  };

  mounted() {
    this._load();
  }

  updated() {
    this._load();
  }

  _load() {
    this.alert.visible = false;

    function mapper(eventType: EventType): SelectableEventType {
      return {
        ...eventType,
        selected: false,
      };
    }

    if (this.subscription_id !== this.$route.params.subscription_id) {
      this.subscription_id = this.$route.params.subscription_id as UUID;
      this.isNew = !this.subscription_id;
      this.eventTypes = [];

      // first load the subscription if in edit mode
      (!this.isNew
        ? SubscriptionService.get(this.subscription_id).then((subscription: Subscription) => {
            // We do the mapping bellow (instead of a single-line assignment) to stay in control of what we manage
            this.subscription.secret = subscription.secret;
            this.subscription.created_at = subscription.created_at;
            this.subscription.description = subscription.description || '';

            this.subscription.event_types = subscription.event_types;

            this.subscription.is_enabled = subscription.is_enabled;
            this.subscription.label_key = subscription.label_key;
            this.subscription.label_value = subscription.label_value;
            this.subscription.metadata = subscription.metadata;
            this.subscription.dedicated_workers = subscription.dedicated_workers;

            // currently PaperClip does not handle our "enum Target" on Rust-side and yield a string
            this.subscription.target = subscription.target as unknown as Target;
            this.httpTarget.headers = this.fromMap(
              (subscription.target as unknown as Target).headers
            );
          })
        : Promise.resolve()
      )
        // then (always) load the eventTypes
        .then(() => EventTypeService.list(this.$route.params.application_id as string))
        .then((eventTypes) => {
          // apply selection if any event_types were already selected (edit mode)
          // this will display previously selected event type names that are not available anymore
          this.eventTypes = intersectWith<SelectableEventType, SelectableEventType, string>(
            (a) => a.event_type_name,

            // depending on the user event type selection, for a single event type we might have one or more events
            (selectableType: SelectableEventType[]) => {
              return {
                ...(head(selectableType) as SelectableEventType),
                selected: selectableType.some((type) => type.selected),
              };
            },

            eventTypes.map(mapper),
            SelectedEventTypesFromEventTypeNames(this.subscription.event_types)
          );
        })
        .catch(this.displayError.bind(this));
    }
  }

  cancel2() {
    this.$router.back();
  }

  fromMap(headers: Record<string, string>): Hook0KeyValueKeyValuePair[] {
    return Object.entries(headers).map(([key, value]) => ({ key, value }));
  }

  toMap(pairs: Hook0KeyValueKeyValuePair[]): Record<string, string> {
    return pairs.reduce((m, { key, value }) => {
      // @ts-ignore
      m[key] = value;
      return m;
    }, {});
  }

  upsert(e: Event) {
    e.preventDefault();
    e.stopImmediatePropagation();

    this.alert.visible = false; // reset alert

    if (this.isNew) {
      SubscriptionService.create({
        application_id: this.$route.params.application_id as string,

        // eslint-disable-next-line
        target: {
          type: 'http',
          method: this.subscription.target.method,
          url: this.subscription.target.url,
          headers: this.subscription.target.headers,
        },
        description: this.subscription.description,
        metadata: this.subscription.metadata,
        label_value: this.subscription.label_value,
        label_key: this.subscription.label_key,
        is_enabled: this.subscription.is_enabled,
        event_types: EventTypeNamesFromSelectedEventTypes(this.eventTypes),
      }).then((_resp: any) => {
        this.cancel2();
      }, this.displayError.bind(this));
      return;
    }

    SubscriptionService.update(this.subscription_id as UUID, {
      // eslint-disable-next-line
      target: {
        type: 'http',
        method: this.subscription.target.method,
        url: this.subscription.target.url,
        headers: this.subscription.target.headers,
      },
      description: this.subscription.description,
      metadata: this.subscription.metadata,
      label_value: this.subscription.label_value,
      label_key: this.subscription.label_key,
      is_enabled: this.subscription.is_enabled,
      event_types: EventTypeNamesFromSelectedEventTypes(this.eventTypes),
      dedicated_workers:
        this.subscription.dedicated_workers.length > 0
          ? this.subscription.dedicated_workers
          : undefined,
      application_id: this.$route.params.application_id as string,
    }).then((_resp: any) => {
      this.cancel2();
    }, this.displayError.bind(this));
  }

  displayError(err: AxiosError | unknown) {
    console.error(err);
    this.alert.visible = true;

    if (isAxiosError(err) && err.response) {
      const problem: Problem = err.response.data as Problem;
      this.alert.type = problem.status >= 500 ? 'alert' : 'warning';
      this.alert.title = problem.title;
      this.alert.description = problem.detail;
    } else {
      this.alert.type = 'alert';
      this.alert.title = 'An error occurred';
      this.alert.description = String(err);
    }
  }
}
</script>

<style scoped></style>
