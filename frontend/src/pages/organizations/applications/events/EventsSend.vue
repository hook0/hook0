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
import Hook0CopyField from '@/components/Hook0CopyField.vue';
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

// Tabs
type TabId = 'easy' | 'curl' | 'javascript' | 'rust';
const tabs: TabId[] = ['easy', 'curl', 'javascript', 'rust'];
const activeTab = ref<TabId>('easy');
const tabRefs = ref<HTMLElement[]>([]);

function setTabRef(el: unknown, index: number) {
  if (el instanceof HTMLElement) {
    tabRefs.value[index] = el;
  }
}

function activateTab(tab: TabId, index: number) {
  activeTab.value = tab;
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

const secretOptions = computed(() =>
  (secrets.value ?? []).map((s) => ({
    label: s.name ?? s.token.slice(0, 12) + '...',
    value: s.token,
  }))
);

const selectedSecretToken = ref<string | null>(null);

const effectiveSecretToken = computed(() => {
  if (selectedSecretToken.value) return selectedSecretToken.value;
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
const curlSnippet = computed(() => {
  const labelsRecord = kvPairsToRecord(labels.value);
  const payloadStr = payload.value || '{}';
  return `curl -X POST '${apiBaseUrl.value}/event' \\
  -H 'Content-Type: application/json' \\
  -H 'Authorization: Bearer ${snippetToken.value}' \\
  -d '${JSON.stringify(
    {
      application_id: applicationId.value,
      event_id: crypto.randomUUID(),
      event_type: selectedEventType.value || 'your.event.type',
      labels: labelsRecord,
      occurred_at: new Date().toISOString(),
      payload: payloadStr,
      payload_content_type: 'application/json',
    },
    null,
    2
  )}'`;
});

const jsSnippet = computed(() => {
  const labelsRecord = kvPairsToRecord(labels.value);
  const payloadStr = payload.value || '{}';
  return `fetch('${apiBaseUrl.value}/event', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': 'Bearer ${snippetToken.value}',
  },
  body: JSON.stringify(${JSON.stringify(
    {
      application_id: applicationId.value,
      event_id: crypto.randomUUID(),
      event_type: selectedEventType.value || 'your.event.type',
      labels: labelsRecord,
      occurred_at: new Date().toISOString(),
      payload: payloadStr,
      payload_content_type: 'application/json',
    },
    null,
    4
  )}),
});`;
});

const rustSnippet = computed(() => {
  const labelsRecord = kvPairsToRecord(labels.value);
  const payloadStr = payload.value || '{}';
  const bodyJson = JSON.stringify(
    {
      application_id: applicationId.value,
      event_id: crypto.randomUUID(),
      event_type: selectedEventType.value || 'your.event.type',
      labels: labelsRecord,
      occurred_at: new Date().toISOString(),
      payload: payloadStr,
      payload_content_type: 'application/json',
    },
    null,
    4
  );
  return `use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

let client = reqwest::Client::new();
let res = client
    .post("${apiBaseUrl.value}/event")
    .header(CONTENT_TYPE, "application/json")
    .header(AUTHORIZATION, "Bearer ${snippetToken.value}")
    .body(r#"${bodyJson}"#)
    .send()
    .await?;`;
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

  const labelsToSend = kvPairsToRecord(labels.value);
  if (Object.keys(labelsToSend).length === 0) {
    toast.error(t('events.invalidEvent'), {
      description: t('events.labelsRequired'),
      duration: 5000,
    });
    return;
  }

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
    <Hook0Card v-if="activeTab === 'easy' && (eventTypesLoading || !rawEventTypes)">
      <!-- Tabs inside card -->
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

      <Hook0CardHeader>
        <template #header>{{ t('events.sendTestEvent') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>
    </Hook0Card>

    <!-- Error loading event types -->
    <Hook0ErrorCard
      v-else-if="activeTab === 'easy' && eventTypesError"
      :error="eventTypesError"
      @retry="refetchEventTypes()"
    />

    <!-- Easy way tab panel: Form -->
    <Hook0Card v-else-if="activeTab === 'easy'" role="tabpanel" :aria-label="t('events.tabs.easy')">
      <!-- Tabs inside card -->
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
    </Hook0Card>

    <!-- Code snippet tab panels -->
    <Hook0Card
      v-if="activeTab !== 'easy'"
      role="tabpanel"
      :aria-label="t(`events.tabs.${activeTab}`)"
    >
      <!-- Tabs inside card -->
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

      <Hook0CardHeader>
        <template #header>{{ t(`events.tabs.${activeTab}`) }}</template>
        <template #actions>
          <Hook0Button variant="ghost" @click="copySnippet">
            <Copy :size="14" aria-hidden="true" />
          </Hook0Button>
        </template>
      </Hook0CardHeader>

      <Hook0CardContent>
        <!-- Secret selector -->
        <div class="send-event__secret-row">
          <template v-if="tutorialMode">
            <span class="send-event__secret-note">
              {{ t('events.tutorialTokenNote') }}
            </span>
          </template>
          <template v-else>
            <label class="send-event__secret-label">
              {{ t('events.selectSecret') }}
            </label>
            <div v-if="secrets && secrets.length > 0" class="send-event__secret-controls">
              <Hook0Select v-model="selectedSecretToken" :options="secretOptions" />
              <Hook0CopyField
                :value="effectiveSecretToken"
                :maskable="true"
                :copy-message="t('common.idCopied')"
              />
            </div>
            <div v-else class="send-event__no-secrets">
              <span>{{ t('events.noSecretsYet') }}</span>
              <router-link
                :to="{
                  name: routes.ApplicationSecretsNew,
                  params: route.params,
                }"
                class="send-event__create-secret-link"
              >
                {{ t('events.createSecret') }}
              </router-link>
            </div>
          </template>
        </div>

        <!-- Snippet -->
        <Hook0Code v-if="activeTab === 'curl'" :code="curlSnippet" />
        <Hook0Code v-else-if="activeTab === 'javascript'" :code="jsSnippet" />
        <Hook0Code v-else-if="activeTab === 'rust'" :code="rustSnippet" />
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

.send-event__secret-row {
  margin-bottom: 1rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--color-border);
}

.send-event__secret-label {
  display: block;
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-bottom: 0.375rem;
}

.send-event__secret-controls {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.send-event__secret-note {
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  font-style: italic;
}

.send-event__no-secrets {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.send-event__create-secret-link {
  color: var(--color-primary);
  font-weight: 500;
  text-decoration: none;
  transition: color 0.15s ease;
}

.send-event__create-secret-link:hover {
  color: var(--color-primary-hover);
}
</style>
