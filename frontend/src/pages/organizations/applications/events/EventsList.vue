<script setup lang="ts">
import { computed, h, markRaw, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import type { ColumnDef } from '@tanstack/vue-table';
import { RefreshCw, ExternalLink } from 'lucide-vue-next';

import { useEventList, useEventDetail, useReplayEvent, useSendEvent } from './useEventQueries';
import { useEventTypeList } from '../event_types/useEventTypeQueries';
import type { Event } from './EventsService';
import { routes } from '@/routes';
import { handleMutationError } from '@/utils/handleMutationError';
import { toast } from 'vue-sonner';

import Hook0PageLayout from '@/components/Hook0PageLayout.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Table from '@/components/Hook0Table.vue';
import Hook0TableCellLink from '@/components/Hook0TableCellLink.vue';
import Hook0TableCellCode from '@/components/Hook0TableCellCode.vue';
import Hook0TableCellDate from '@/components/Hook0TableCellDate.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0SidePanel from '@/components/Hook0SidePanel.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0KeyValue from '@/components/Hook0KeyValue.vue';
import { kvPairsToRecord, type Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import { Codemirror } from 'vue-codemirror';
import { json } from '@codemirror/lang-json';
import { EditorView } from 'codemirror';

type Props = {
  tutorialMode?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  tutorialMode: false,
});

const emit = defineEmits<{
  'tutorial-event-sent': [];
  'event-sent': [];
}>();

const { t } = useI18n();
const route = useRoute();
const router = useRouter();

const applicationId = computed(() => route.params.application_id as string);

// Side panel state
const sidePanelOpen = ref(false);
const selectedEventId = ref('');
const selectedEventIdRef = computed(() => selectedEventId.value);

const { data: selectedEventData, isLoading: selectedEventLoading } = useEventDetail(
  selectedEventIdRef,
  applicationId
);

function handleRowClick(row: Event) {
  selectedEventId.value = row.event_id;
  sidePanelOpen.value = true;
}

function closeSidePanel() {
  sidePanelOpen.value = false;
}

function openFullPage() {
  sidePanelOpen.value = false;
  void router.push({
    name: routes.EventsDetail,
    params: {
      application_id: route.params.application_id,
      organization_id: route.params.organization_id,
      event_id: selectedEventId.value,
    },
  });
}

const readOnlyExtensions = [json(), EditorView.lineWrapping, EditorView.editable.of(false)];

// Events list query
const { data: events, isLoading, error, refetch } = useEventList(applicationId);

// Event types query (for send event form)
const {
  data: rawEventTypes,
  isLoading: eventTypesLoading,
  error: eventTypesError,
  refetch: refetchEventTypes,
} = useEventTypeList(applicationId);

const eventTypeOptions = computed(() =>
  (rawEventTypes.value ?? []).map((et) => ({
    label: et.event_type_name,
    value: et.event_type_name,
  }))
);

// Mutations
const replayMutation = useReplayEvent();
const sendMutation = useSendEvent();

// Form state
const showEventForm = ref(props.tutorialMode);
const selectedEventType = ref<null | string>(null);
const labels = ref<Hook0KeyValueKeyValuePair[]>([{ key: 'all', value: 'yes' }]);
const occurredAt = ref<null | Date>(null);
const payload = ref<string>('{"test": true}');
const extensions = [json(), EditorView.lineWrapping];

function displayEventForm() {
  showEventForm.value = true;
}

function cancelTest() {
  showEventForm.value = false;
}

function sendTestEvent() {
  const eventType = selectedEventType.value;
  const currentLabels = labels.value;
  const currentOccurredAt = occurredAt.value;
  const currentPayload = payload.value;

  if (!eventType || currentLabels.length <= 0 || !currentOccurredAt || !currentPayload) {
    toast.error(t('events.invalidEvent'), {
      description: t('events.invalidEventMessage'),
      duration: 5000,
      });
    return;
  }

  sendMutation.mutate(
    {
      applicationId: applicationId.value,
      eventId: crypto.randomUUID(),
      eventType,
      labels: kvPairsToRecord(currentLabels),
      occurredAt: currentOccurredAt,
      payload: currentPayload,
    },
    {
      onSuccess: () => {
        if (props.tutorialMode) {
          emit('tutorial-event-sent');
        } else {
          showEventForm.value = false;
          toast.success(t('events.eventSentSuccess'), {
            description: t('events.eventSentMessage'),
            duration: 5000,
            });
          emit('event-sent');
        }
      },
      onError: (err) => {
        handleMutationError(err);
      },
    }
  );
}

function handleReplay(row: Event) {
  replayMutation.mutate(
    { eventId: row.event_id, applicationId: applicationId.value },
    {
      onSuccess: () => {
        toast.success(t('events.replay'), {
          description: t('events.replaySuccess'),
          duration: 5000,
          });
      },
      onError: (err) => {
        handleMutationError(err);
      },
    }
  );
}

const columns: ColumnDef<Event, unknown>[] = [
  {
    accessorKey: 'event_id',
    header: t('events.id'),
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: String(info.getValue()),
        dataTest: 'event-id-link',
        to: {
          name: routes.EventsDetail,
          params: {
            application_id: route.params.application_id,
            organization_id: route.params.organization_id,
            event_id: info.row.original.event_id,
          },
        },
      }),
  },
  {
    accessorKey: 'received_at',
    header: t('events.receivedAt'),
    enableSorting: true,
    cell: (info) => h(Hook0TableCellDate, { value: info.getValue() as string | null }),
  },
  {
    accessorKey: 'event_type_name',
    header: t('events.type'),
    cell: (info) => h(Hook0TableCellCode, { value: String(info.getValue()) }),
  },
  {
    accessorKey: 'labels',
    header: t('events.labels'),
    enableSorting: true,
    cell: (info) =>
      h(Hook0TableCellCode, {
        value: Object.entries((info.row.original.labels ?? {}) as Record<string, string>)
          .map(([key, value]) => `${key}=${value}`)
          .join(' '),
      }),
  },
  {
    id: 'options',
    header: t('common.actions'),
    cell: (info) =>
      h(Hook0TableCellLink, {
        value: t('events.replay'),
        icon: markRaw(RefreshCw),
        onClick: () => handleReplay(info.row.original),
      }),
  },
];
</script>

<template>
  <Hook0PageLayout :title="t('events.title')">
    <!-- Send event form -->
    <template v-if="showEventForm">
      <!-- Loading event types -->
      <Hook0Card v-if="eventTypesLoading" data-test="send-event-card">
        <Hook0CardHeader>
          <template #header>{{ t('events.sendTestEvent') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0SkeletonGroup :count="3" />
        </Hook0CardContent>
      </Hook0Card>

      <!-- Error loading event types -->
      <Hook0ErrorCard
        v-else-if="eventTypesError"
        :error="eventTypesError"
        @retry="refetchEventTypes()"
      />

      <!-- Form -->
      <Hook0Card v-else data-test="send-event-card">
        <Hook0CardHeader>
          <template #header>{{ t('events.sendTestEvent') }}</template>
          <template #subtitle>
            {{ t('events.sendTestEventSubtitle') }}
          </template>
        </Hook0CardHeader>

        <Hook0Form data-test="send-event-form" @submit="sendTestEvent">
          <Hook0CardContent>
            <Hook0CardContentLine>
              <template #label>{{ t('events.eventType') }}</template>
              <template #content>
                <Hook0Select
                  v-model="selectedEventType"
                  :options="eventTypeOptions"
                  data-test="send-event-type-select"
                />
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label>
                {{ t('events.eventLabels') }}
                <Hook0HelpText>{{ t('events.eventLabelsHelp') }}</Hook0HelpText>
              </template>
              <template #content>
                <Hook0KeyValue
                  :value="labels"
                  :key-placeholder="t('common.labelKey')"
                  :value-placeholder="t('common.labelValue')"
                  data-test="send-event-labels"
                />
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label>{{ t('events.occurredAt') }}</template>
              <template #content>
                <Hook0Input
                  v-model="occurredAt"
                  type="datetime-local"
                  data-test="send-event-occurred-at-input"
                />
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label>{{ t('events.payload') }}</template>
              <template #content>
                <Codemirror
                  v-model="payload"
                  :autofocus="true"
                  :indent-with-tab="true"
                  :tab-size="2"
                  :extensions="extensions"
                  data-test="send-event-payload-input"
                />
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>

          <Hook0CardFooter>
            <Hook0Button
              v-if="!props.tutorialMode"
              variant="secondary"
              data-test="send-event-cancel-button"
              @click="cancelTest"
            >
              {{ t('common.cancel') }}
            </Hook0Button>

            <Hook0Button
              v-if="!tutorialMode"
              variant="primary"
              submit
              data-test="send-event-submit-button"
            >
              {{ t('events.sendEvent') }}
            </Hook0Button>
            <Hook0Button v-else variant="primary" submit data-test="send-event-submit-button">
              {{ t('events.sendFirstEvent') }}
            </Hook0Button>
          </Hook0CardFooter>
        </Hook0Form>
      </Hook0Card>
    </template>

    <!-- Events list -->
    <template v-else>
      <!-- Error state (check FIRST - errors take priority) -->
      <Hook0ErrorCard v-if="error && !isLoading" :error="error" @retry="refetch()" />

      <!-- Loading skeleton (also shown when query is disabled and data is undefined) -->
      <Hook0Card v-else-if="isLoading || !events" data-test="events-card">
        <Hook0CardHeader>
          <template #header>{{ t('events.title') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0SkeletonGroup :count="4" />
        </Hook0CardContent>
      </Hook0Card>

      <!-- Data loaded (events is guaranteed to be defined here) -->
      <template v-else>
        <Hook0Card data-test="events-card">
          <Hook0CardHeader>
            <template #header>{{ t('events.title') }}</template>
            <template #subtitle>
              {{ t('events.subtitle') }}
            </template>
          </Hook0CardHeader>

          <Hook0CardContent v-if="events.length > 0">
            <Hook0Table
              data-test="events-table"
              :columns="columns"
              :data="events"
              row-id-field="event_id"
              clickable-rows
              @row-click="handleRowClick"
            />
          </Hook0CardContent>

          <Hook0CardContent v-else>
            <Hook0EmptyState
              :title="t('events.empty.title')"
              :description="t('events.empty.description')"
            >
              <template #action>
                <Hook0Button
                  variant="primary"
                  data-test="events-send-button"
                  @click="displayEventForm"
                >
                  {{ t('events.sendEvent') }}
                </Hook0Button>
              </template>
            </Hook0EmptyState>
          </Hook0CardContent>

          <Hook0CardFooter v-if="events.length > 0">
            <Hook0Button variant="primary" data-test="events-send-button" @click="displayEventForm">
              {{ t('events.sendEvent') }}
            </Hook0Button>
          </Hook0CardFooter>
        </Hook0Card>
      </template>
    </template>

    <!-- Event side panel -->
    <Hook0SidePanel
      :open="sidePanelOpen"
      :title="t('events.detail')"
      width="36rem"
      @close="closeSidePanel"
    >
      <template #header>
        <h2 class="event-panel__title">{{ t('events.detail') }}</h2>
        <Hook0Button
          variant="ghost"
          size="sm"
          :aria-label="t('events.openFullPage')"
          data-test="event-panel-full-page"
          @click="openFullPage"
        >
          <ExternalLink :size="16" aria-hidden="true" />
          {{ t('events.openFullPage') }}
        </Hook0Button>
      </template>

      <!-- Loading -->
      <template v-if="selectedEventLoading || !selectedEventData">
        <div class="event-panel__section">
          <Hook0Skeleton size="text-truncated" />
          <Hook0Skeleton size="text" />
          <Hook0Skeleton size="text-truncated" />
        </div>
        <div class="event-panel__section">
          <Hook0Skeleton size="block" />
        </div>
      </template>

      <!-- Event loaded -->
      <template v-else>
        <!-- Metadata -->
        <div class="event-panel__section">
          <h3 class="event-panel__section-title">{{ t('events.metadata') }}</h3>
          <div class="event-panel__meta">
            <div class="event-panel__meta-row">
              <span class="event-panel__meta-label">{{ t('events.id') }}</span>
              <code class="event-panel__meta-value event-panel__meta-value--mono">{{
                selectedEventData.event_id
              }}</code>
            </div>
            <div class="event-panel__meta-row">
              <span class="event-panel__meta-label">{{ t('events.type') }}</span>
              <code class="event-panel__meta-value event-panel__meta-value--mono">{{
                selectedEventData.event_type_name
              }}</code>
            </div>
            <div class="event-panel__meta-row">
              <span class="event-panel__meta-label">{{ t('events.receivedAt') }}</span>
              <span class="event-panel__meta-value">{{ selectedEventData.received_at }}</span>
            </div>
            <div class="event-panel__meta-row">
              <span class="event-panel__meta-label">{{ t('events.occurredAt') }}</span>
              <span class="event-panel__meta-value">{{ selectedEventData.occurred_at }}</span>
            </div>
          </div>
        </div>

        <!-- Labels -->
        <div
          v-if="
            selectedEventData.labels &&
            Object.keys(selectedEventData.labels as Record<string, string>).length > 0
          "
          class="event-panel__section"
        >
          <h3 class="event-panel__section-title">{{ t('events.labels') }}</h3>
          <div class="event-panel__labels">
            <span
              v-for="(val, key) in selectedEventData.labels as Record<string, string>"
              :key="String(key)"
              class="event-panel__label-badge"
            >
              {{ key }}={{ val }}
            </span>
          </div>
        </div>

        <!-- Payload -->
        <div class="event-panel__section">
          <h3 class="event-panel__section-title">{{ t('events.payload') }}</h3>
          <Codemirror
            :model-value="selectedEventData.payload_decoded"
            :extensions="readOnlyExtensions"
            :tab-size="2"
            class="event-panel__code"
          />
        </div>
      </template>
    </Hook0SidePanel>
  </Hook0PageLayout>
</template>

<style scoped>
.event-panel__title {
  flex: 1;
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.event-panel__section {
  margin-bottom: 1.25rem;
}

.event-panel__section:last-child {
  margin-bottom: 0;
}

.event-panel__section-title {
  font-size: 0.6875rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
  margin: 0 0 0.75rem;
}

.event-panel__meta {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.event-panel__meta-row {
  display: flex;
  align-items: baseline;
  gap: 0.75rem;
}

.event-panel__meta-label {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  min-width: 6rem;
  flex-shrink: 0;
}

.event-panel__meta-value {
  font-size: 0.8125rem;
  color: var(--color-text-primary);
  word-break: break-all;
}

.event-panel__meta-value--mono {
  font-family: var(--font-mono);
  font-size: 0.75rem;
}

.event-panel__labels {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
}

.event-panel__label-badge {
  display: inline-flex;
  padding: 0.125rem 0.5rem;
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  font-weight: 500;
  color: var(--color-primary);
  background-color: var(--color-primary-light);
  border-radius: var(--radius-full);
}

.event-panel__code {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
}
</style>
