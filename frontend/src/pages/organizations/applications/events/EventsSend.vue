<script setup lang="ts">
import { computed, ref, nextTick } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { toTypedSchema } from '@/utils/zod-adapter';
import { FormInput, Terminal, Copy } from 'lucide-vue-next';
import RustIcon from './RustIcon.vue';
import JavaScriptIcon from './JavaScriptIcon.vue';
import { toast } from 'vue-sonner';

import { sendEventSchema, type SendEventForm } from './sendEvent.schema';
import { useSendEvent } from './useEventQueries';
import { useEventTypeList } from '../event_types/useEventTypeQueries';
import { useSecretList } from '../application_secrets/useSecretQueries';
import { kvPairsToRecord, type Hook0KeyValueKeyValuePair } from '@/components/Hook0KeyValue';
import { routes } from '@/routes';
import { handleMutationError } from '@/utils/handleMutationError';
import { useAuthStore } from '@/stores/auth';
import { useClipboardCopy } from '@/composables/useClipboardCopy';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0KeyValue from '@/components/Hook0KeyValue.vue';
import Hook0HelpText from '@/components/Hook0HelpText.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0Code from '@/components/Hook0Code.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
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
  'event-sent': [];
}>();

const { t } = useI18n();
const route = useRoute();
const router = useRouter();
const { applicationId } = useRouteIds();
const authStore = useAuthStore();
const { copy: clipboardCopy } = useClipboardCopy();

// Tabs — driven by URL hash (no remount since App.vue uses route.path as key)
type TabId = 'easy' | 'curl' | 'javascript' | 'rust';
const tabs: TabId[] = ['easy', 'curl', 'javascript', 'rust'];
const tabRefs = ref<HTMLElement[]>([]);

const hashTabMap: Record<string, TabId> = {
  '#curl': 'curl',
  '#js': 'javascript',
  '#rust': 'rust',
};

const activeTab = computed(() => hashTabMap[route.hash] ?? 'easy');

function setTabRef(el: unknown, index: number) {
  if (el instanceof HTMLElement) {
    tabRefs.value[index] = el;
  }
}

function activateTab(tab: TabId, index: number) {
  const hash = tab === 'easy' ? '' : `#${tab === 'javascript' ? 'js' : tab}`;
  void router.replace({ ...route, hash });
  void nextTick(() => {
    tabRefs.value[index]?.focus();
  });
}

function handleTabKeydown(event: KeyboardEvent, index: number) {
  let newIndex = index;
  if (event.key === 'ArrowRight') {
    newIndex = (index + 1) % tabs.length;
    event.preventDefault();
  } else if (event.key === 'ArrowLeft') {
    newIndex = (index - 1 + tabs.length) % tabs.length;
    event.preventDefault();
  } else if (event.key === 'Home') {
    newIndex = 0;
    event.preventDefault();
  } else if (event.key === 'End') {
    newIndex = tabs.length - 1;
    event.preventDefault();
  } else {
    return;
  }
  activateTab(tabs[newIndex], newIndex);
}

// Event types query
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

// Secrets query
const { data: secrets } = useSecretList(applicationId);

const effectiveSecretToken = computed(() => {
  if (secrets.value && secrets.value.length > 0) return secrets.value[0].token;
  return '';
});

// Mutation
const sendMutation = useSendEvent();

// Form setup
const extensions = [json(), EditorView.lineWrapping];

// Helper to format Date to datetime-local string
function formatDateTimeLocal(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

const { meta, values, setFieldValue } = useForm<SendEventForm>({
  validationSchema: toTypedSchema(sendEventSchema),
  initialValues: {
    eventType: '',
    labels: [{ key: 'user_id', value: '1' }],
    occurredAt: formatDateTimeLocal(new Date()),
    payload: '{"test": true}',
  },
});

// Bind form fields to refs for template v-model
const selectedEventType = computed({
  get: () => values.eventType ?? '',
  set: (v: string) => void setFieldValue('eventType', v),
});

const labels = computed({
  get: () => (values.labels ?? []) as Hook0KeyValueKeyValuePair[],
  set: (v: Hook0KeyValueKeyValuePair[]) => void setFieldValue('labels', v),
});

const occurredAt = computed({
  get: () => values.occurredAt ?? formatDateTimeLocal(new Date()),
  set: (v: string) => void setFieldValue('occurredAt', v),
});

const payload = computed({
  get: () => values.payload ?? '{"test": true}',
  set: (v: string) => void setFieldValue('payload', v),
});

// Authorization token for snippets
const snippetToken = computed(() => {
  if (props.tutorialMode) {
    return authStore.accessToken ?? '';
  }
  return effectiveSecretToken.value;
});

// API base URL for snippets
const apiBaseUrl = computed(() => {
  return import.meta.env.VITE_API_ENDPOINT ?? '';
});

// Code snippets
function buildLabelsJs(labelsRecord: Record<string, string>): string {
  const entries = Object.entries(labelsRecord);
  if (entries.length === 0) return '{}';
  const inner = entries
    .map(([k, v]) => `    ${JSON.stringify(k)}: ${JSON.stringify(v)}`)
    .join(',\n');
  return `{\n${inner}\n  }`;
}

function buildLabelsRust(labelsRecord: Record<string, string>): string {
  const entries = Object.entries(labelsRecord);
  if (entries.length === 0) return 'vec![]';
  const inner = entries
    .map(
      ([k, v]) =>
        `            (${JSON.stringify(k)}.to_string(), ${JSON.stringify(v)}.to_string()),`
    )
    .join('\n');
  return `vec![\n${inner}\n        ]`;
}

const curlSnippet = computed(() => {
  const labelsRecord = kvPairsToRecord(labels.value);
  const body = JSON.stringify(
    {
      application_id: applicationId.value,
      event_id: crypto.randomUUID(),
      event_type: values.eventType || 'your.event.type',
      labels: labelsRecord,
      occurred_at: new Date().toISOString(),
      payload: payload.value || '{}',
      payload_content_type: 'application/json',
    },
    null,
    2
  )
    .split('\n')
    .map((l, i) => (i === 0 ? l : `  ${l}`))
    .join('\n');
  return [
    `curl -X POST '${apiBaseUrl.value}/event' \\`,
    `  -H 'Content-Type: application/json' \\`,
    `  -H 'Authorization: Bearer ${snippetToken.value}' \\`,
    `  -d '${body}'`,
  ].join('\n');
});

const jsSnippet = computed(() => {
  const labelsRecord = kvPairsToRecord(labels.value);
  const payloadStr = payload.value || '{}';
  return [
    `import { Hook0Client, Event } from "hook0-client";`,
    ``,
    `const hook0 = new Hook0Client(`,
    `  ${JSON.stringify(apiBaseUrl.value)},`,
    `  ${JSON.stringify(applicationId.value)},`,
    `  ${JSON.stringify(snippetToken.value)},`,
    `);`,
    ``,
    `const event = new Event(`,
    `  ${JSON.stringify(values.eventType || 'user.account.created')},`,
    `  JSON.stringify(${payloadStr}),`,
    `  "application/json",`,
    `  ${buildLabelsJs(labelsRecord)},`,
    `);`,
    ``,
    `const eventId = await hook0.sendEvent(event);`,
    `console.log("Event sent:", eventId);`,
  ].join('\n');
});

const rustSnippet = computed(() => {
  const labelsRecord = kvPairsToRecord(labels.value);
  const payloadStr = payload.value || '{}';
  return [
    `use hook0_client::{Hook0Client, Event};`,
    `use reqwest::Url;`,
    `use uuid::Uuid;`,
    `use std::borrow::Cow;`,
    ``,
    `#[tokio::main]`,
    `async fn main() -> Result<(), Box<dyn std::error::Error>> {`,
    `    let client = Hook0Client::new(`,
    `        Url::parse(${JSON.stringify(apiBaseUrl.value)})?,`,
    `        Uuid::parse_str(${JSON.stringify(applicationId.value)})?,`,
    `        ${JSON.stringify(snippetToken.value)},`,
    `    )?;`,
    ``,
    `    let event = Event {`,
    `        event_id: &None,`,
    `        event_type: ${JSON.stringify(values.eventType || 'user.account.created')},`,
    `        payload: Cow::Borrowed(r#"${payloadStr}"#),`,
    `        payload_content_type: "application/json",`,
    `        metadata: None,`,
    `        occurred_at: None,`,
    `        labels: ${buildLabelsRust(labelsRecord)},`,
    `    };`,
    ``,
    `    let event_id = client.send_event(&event).await?;`,
    `    println!("Event sent: {event_id}");`,
    `    Ok(())`,
    `}`,
  ].join('\n');
});

function copySnippet() {
  let code = '';
  if (activeTab.value === 'curl') code = curlSnippet.value;
  else if (activeTab.value === 'javascript') code = jsSnippet.value;
  else if (activeTab.value === 'rust') code = rustSnippet.value;
  clipboardCopy(code, t('common.codeCopied'));
}

// Submit
function sendTestEvent() {
  if (!meta.value.valid) {
    toast.error(t('events.invalidEvent'), {
      description: t('events.invalidEventMessage'),
      duration: 5000,
    });
    return;
  }

  const validLabels = labels.value.filter(
    (l) => l.key.trim().length > 0 && l.value.trim().length > 0,
  );
  if (validLabels.length === 0) {
    toast.error(t('events.invalidEvent'), {
      description: t('events.labelsRequired'),
      duration: 5000,
    });
    return;
  }
  const labelsToSend = kvPairsToRecord(validLabels);

  const eventId = crypto.randomUUID();

  sendMutation.mutate(
    {
      applicationId: applicationId.value,
      eventId,
      eventType: values.eventType,
      labels: labelsToSend,
      occurredAt: new Date(values.occurredAt),
      payload: values.payload,
    },
    {
      onSuccess: () => {
        if (props.tutorialMode) {
          emit('event-sent');
        } else {
          toast.success(t('events.eventSentSuccess'), {
            description: t('events.eventSentMessage'),
            duration: 5000,
          });
          void router.push({
            name: routes.EventsDetail,
            params: {
              ...route.params,
              event_id: eventId,
            },
          });
        }
      },
      onError: (err) => {
        handleMutationError(err);
      },
    }
  );
}

function handleCancel() {
  router.back();
}
</script>

<template>
  <div data-test="send-event-card">
    <Hook0Card>
      <!-- SHARED: Always visible header -->
      <Hook0CardHeader>
        <template #header>{{ t('events.sendTestEvent') }}</template>
        <template #subtitle>
          <i18n-t keypath="events.sendTestEventSubtitle" tag="span">
            <template #eventType>
              <router-link :to="{ name: routes.EventTypesList, params: route.params }">
                {{ t('events.sendTestEventCreateEventType') }}
              </router-link>
            </template>
            <template #subscription>
              <router-link :to="{ name: routes.SubscriptionsList, params: route.params }">
                {{ t('events.sendTestEventCreateSubscription') }}
              </router-link>
            </template>
          </i18n-t>
        </template>
        <template v-if="activeTab !== 'easy'" #actions>
          <Hook0Button variant="ghost" @click="copySnippet">
            <Copy :size="14" aria-hidden="true" />
          </Hook0Button>
        </template>
      </Hook0CardHeader>

      <!-- SHARED: Always visible tabs -->
      <div class="send-event__tabs" role="tablist" :aria-label="t('events.sendTestEvent')">
        <button
          v-for="(tab, index) in tabs"
          :key="tab"
          :ref="(el) => setTabRef(el, index)"
          role="tab"
          :aria-selected="activeTab === tab"
          :tabindex="activeTab === tab ? 0 : -1"
          class="send-event__tab"
          :class="{ 'send-event__tab--active': activeTab === tab }"
          @click="activateTab(tab, index)"
          @keydown="handleTabKeydown($event, index)"
        >
          <FormInput v-if="tab === 'easy'" :size="16" aria-hidden="true" />
          <Terminal v-else-if="tab === 'curl'" :size="16" aria-hidden="true" />
          <JavaScriptIcon v-else-if="tab === 'javascript'" :size="16" />
          <RustIcon v-else-if="tab === 'rust'" :size="16" />
          {{ t(`events.tabs.${tab}`) }}
        </button>
      </div>

      <!-- CONDITIONAL CONTENT -->

      <!-- Easy way: Loading skeleton -->
      <Hook0CardContent v-if="activeTab === 'easy' && (eventTypesLoading || !rawEventTypes)">
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>

      <!-- Easy way: Error -->
      <Hook0CardContent v-else-if="activeTab === 'easy' && eventTypesError">
        <Hook0ErrorCard :error="eventTypesError" @retry="refetchEventTypes()" />
      </Hook0CardContent>

      <!-- Easy way: Form -->
      <Hook0Form
        v-else-if="activeTab === 'easy'"
        data-test="send-event-form"
        @submit="sendTestEvent"
      >
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
                :show-separator="true"
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
              <div data-test="send-event-payload-input">
                <Codemirror
                  v-model="payload"
                  :autofocus="true"
                  :indent-with-tab="true"
                  :tab-size="2"
                  :extensions="extensions"
                />
              </div>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>

        <Hook0CardFooter>
          <Hook0Button
            v-if="!props.tutorialMode"
            variant="secondary"
            data-test="send-event-cancel-button"
            @click="handleCancel"
          >
            {{ t('common.cancel') }}
          </Hook0Button>

          <Hook0Button
            v-if="!tutorialMode"
            variant="primary"
            submit
            :disabled="!meta.valid"
            :tooltip="!meta.valid ? t('forms.fillRequiredFields') : undefined"
            data-test="send-event-submit-button"
          >
            {{ t('events.sendEvent') }}
          </Hook0Button>
          <Hook0Button
            v-else
            variant="primary"
            submit
            :disabled="!meta.valid"
            :tooltip="!meta.valid ? t('forms.fillRequiredFields') : undefined"
            data-test="send-event-submit-button"
          >
            {{ t('events.sendFirstEvent') }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Form>

      <!-- Code snippet panels -->
      <Hook0CardContent v-else-if="activeTab === 'curl'" role="tabpanel">
        <Hook0Code :code="curlSnippet" language="bash" :editable="false" />
      </Hook0CardContent>
      <Hook0CardContent v-else-if="activeTab === 'javascript'" role="tabpanel">
        <Hook0Code :code="jsSnippet" language="javascript" :editable="false" />
      </Hook0CardContent>
      <Hook0CardContent v-else-if="activeTab === 'rust'" role="tabpanel">
        <Hook0Code :code="rustSnippet" language="rust" :editable="false" />
      </Hook0CardContent>
    </Hook0Card>
  </div>
</template>

<style scoped>
.send-event__tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--color-border);
}

.send-event__tab {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.625rem 1rem;
  border: none;
  border-bottom: 2px solid transparent;
  background: none;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.send-event__tab:hover {
  color: var(--color-text-primary);
  background-color: var(--color-bg-secondary);
}

.send-event__tab:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.send-event__tab--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}
</style>
