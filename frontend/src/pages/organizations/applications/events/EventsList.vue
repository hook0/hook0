<script setup lang="ts">
import { ColDef } from 'ag-grid-community';
import { onMounted, onUpdated, ref, defineEmits, defineProps } from 'vue';
import { useRoute } from 'vue-router';

import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import { Problem, UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import { routes } from '@/routes';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import * as EventsService from './EventsService';
import { Event } from './EventsService';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0Error from '@/components/Hook0Error.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { list } from '@/pages/organizations/applications/event_types/EventTypeService.ts';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import { push } from 'notivue';
import { v4 as uuidv4 } from 'uuid';
import { Codemirror } from 'vue-codemirror';
import { json } from '@codemirror/lang-json';
import { EditorView } from 'codemirror';
import Hook0KeyValue from '@/components/Hook0KeyValue.vue';
import { Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';

interface Props {
  // cache-burst
  burst?: string | string[];
  tutorialMode?: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits(['tutorial-event-sent', 'event-sent']);

const route = useRoute();

const show_event_form = ref<boolean>(false);

const event_type$ = ref<Promise<Array<{ label: string; value: string }>>>(Promise.resolve([]));

const selected_event_type = ref<null | string>();
const labels = ref<Hook0KeyValueKeyValuePair[]>([{ key: 'all', value: 'yes' }]);
const occurred_at = ref<null | Date>();
const payload = ref<undefined | string>('{"test": true}');

const extensions = [json(), EditorView.lineWrapping];

const columnDefs: ColDef[] = [
  {
    field: 'event_id',
    headerName: 'Event ID',
    suppressMovable: true,
    resizable: true,
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value(row: Event) {
        return row.event_id;
      },

      to(row: Event) {
        return {
          name: routes.EventsDetail,
          params: {
            application_id: route.params.application_id,
            organization_id: route.params.organization_id,
            event_id: row.event_id,
          },
        };
      },
    },
  },
  {
    field: 'received_at',
    suppressMovable: true,
    suppressSizeToFit: true,
    width: 175,
    sortable: true,
    resizable: true,
    headerName: 'Received At',
    cellRenderer: Hook0TableCellDate,
  },
  {
    field: 'event_type_name',
    headerName: 'Event Type',
    suppressMovable: true,
    resizable: true,
    cellRenderer: Hook0TableCellCode,
  },
  {
    field: 'labels',
    suppressMovable: true,
    sortable: true,
    resizable: true,
    width: 100,
    headerName: 'Labels',
    cellRenderer: Hook0TableCellCode,
    cellRendererParams: {
      value(row: Event) {
        return Object.entries(row.labels as Record<string, string>)
          .map(([key, value]) => `${key}=${value}`)
          .join(' ');
      },
    },
    // This seems useless but triggers a warning if not set
    valueFormatter: () => 'unreachable',
  },
  {
    suppressMovable: true,
    headerName: 'Options',
    suppressSizeToFit: true,
    width: 115,
    cellRenderer: Hook0TableCellLink,
    cellRendererParams: {
      value: 'Replay',
      icon: 'arrows-rotate',
      onClick: (row: Event): void => {
        EventsService.replay(row.event_id, application_id.value as UUID)
          .then(() => {
            push.success({
              title: 'Event replayed',
              message: 'The event was replayed successfully',
              duration: 5000,
            });
          })
          .catch(displayError);
      },
    },
  },
];

const events$ = ref<Promise<Array<Event>>>();
const application_id = ref<null | UUID>(null);

function _forceLoad() {
  application_id.value = route.params.application_id as UUID;
  events$.value = EventsService.list(application_id.value);

  event_type$.value = list(application_id.value)
    .then((event_types) =>
      event_types.map((et) => ({ label: et.event_type_name, value: et.event_type_name }))
    )
    .catch((error) => {
      displayError(error as Problem);
      return [];
    });

  if (props.tutorialMode) {
    display_event_form();
  }
}

function _load() {
  if (application_id.value !== route.params.application_id) {
    _forceLoad();
  }
}

function display_event_form() {
  show_event_form.value = true;
}

function send_test_event() {
  if (
    !selected_event_type.value ||
    labels.value.length <= 0 ||
    !occurred_at.value ||
    !payload.value
  ) {
    push.error({
      title: 'Invalid test event',
      message: 'The test event form is invalid',
      duration: 5000,
    });
    return;
  }

  EventsService.send_json_event(
    application_id.value as UUID,
    uuidv4(),
    selected_event_type.value,
    toMap(labels.value),
    occurred_at.value,
    payload.value
  )
    .then(() => {
      if (props.tutorialMode) {
        emit('tutorial-event-sent');
      } else {
        show_event_form.value = false;
        push.success({
          title: 'Test event sent',
          message: 'The test event was sent successfully',
          duration: 5000,
        });
        _forceLoad();
        emit('event-sent');
      }
    })
    .catch(displayError);
}

function cancel_test() {
  show_event_form.value = false;
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

function toMap(pairs: Hook0KeyValueKeyValuePair[]): Record<string, string> {
  return pairs.reduce((m, { key, value }) => {
    // @ts-ignore
    m[key] = value;
    return m;
  }, {});
}
</script>

<template>
  <Promised v-if="show_event_form" :promise="event_type$">
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <template #default="event_types">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header> Send a test event </template>
          <template #subtitle>
            For sending a test event, you need to
            <Hook0Button href="https://documentation.hook0.com/docs/getting-started#event-types"
              >create an event types</Hook0Button
            >
            first. After that you can
            <Hook0Button
              href="https://documentation.hook0.com/docs/getting-started#creating-a-subscription"
              >create a subscription</Hook0Button
            >
            for this event type. Finally you can send a test event.
          </template>
        </Hook0CardHeader>

        <form @submit.prevent="send_test_event">
          <Hook0CardContent>
            <Hook0CardContentLine>
              <template #label> Event Type </template>
              <template #content>
                <Hook0Select v-model="selected_event_type" :options="event_types"></Hook0Select>
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label>
                Event labels

                <Hook0Text class="helpText mt-2 block">
                  Hook0 will only forward events to subscriptions which labels are all part of the
                  event's labels. Events can have extra labels and still trigger a subscription, but
                  subscriptions cannot have labels that are not part of the event or they won't be
                  triggered by it.
                </Hook0Text>

                <Hook0Text class="helpText mt-2 block"> </Hook0Text>
              </template>
              <template #content>
                <Hook0KeyValue
                  :value="labels"
                  key-placeholder="Label key"
                  value-placeholder="Label value"
                ></Hook0KeyValue>
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label> Occurred At </template>
              <template #content>
                <Hook0Input v-model="occurred_at" type="datetime-local" />
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label> Payload </template>
              <template #content>
                <Codemirror
                  v-model="payload"
                  :autofocus="true"
                  :indent-with-tab="true"
                  :tab-size="2"
                  :extensions="extensions"
                />
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>

          <Hook0CardFooter>
            <Hook0Button v-if="!props.tutorialMode" class="secondary" @click="cancel_test"
              >Cancel</Hook0Button
            >

            <Hook0Button
              v-if="!tutorialMode"
              tooltip="ℹ️ To continue, you need to fill all fields"
              class="primary"
              submit
              >Send event</Hook0Button
            >
            <Hook0Button
              v-else
              tooltip="ℹ️ To continue, you need to fill all fields"
              class="primary"
              submit
              >Send Your First Event 🎉</Hook0Button
            >
          </Hook0CardFooter>
        </form>
      </Hook0Card>
    </template>
    <template #rejected="error">
      <Hook0Error :error="error"></Hook0Error>
    </template>
  </Promised>
  <Promised v-else :promise="events$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="event_types">
      <Hook0Card>
        <Hook0CardHeader>
          <template #header> Events </template>
          <template #subtitle>
            Events that Hook0 receive from your application and that Hook0 forwarded to
            subscriptions (webhooks).
          </template>
        </Hook0CardHeader>

        <Hook0CardContent v-if="event_types.length > 0">
          <Hook0Table
            :context="{ events$, columnDefs }"
            :column-defs="columnDefs"
            :row-data="event_types"
          >
          </Hook0Table>
        </Hook0CardContent>

        <Hook0CardContent v-else>
          <Hook0CardContentLines>
            <Hook0CardContentLine type="full-width">
              <template #content>
                <Hook0Text class="center block" style="text-align: center"
                  >Your application did not send any events. Time to send the first one!
                </Hook0Text>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContentLines>
        </Hook0CardContent>

        <Hook0CardFooter>
          <Hook0Button class="primary" @click="display_event_form">Send an event</Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <Hook0Error :error="error"></Hook0Error>
    </template>
  </Promised>
</template>
