<script setup lang="ts">
import { computed, ref, watch, nextTick, type Component } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useRouteIds } from '@/composables/useRouteIds';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { toTypedSchema } from '@/utils/zod-adapter';
import { FolderTree, FormInput, Terminal } from 'lucide-vue-next';
import RustIcon from './RustIcon.vue';
import JavaScriptIcon from './JavaScriptIcon.vue';
import PythonIcon from './PythonIcon.vue';
import GoIcon from './GoIcon.vue';
import PhpIcon from './PhpIcon.vue';
import RubyIcon from './RubyIcon.vue';
import JavaIcon from './JavaIcon.vue';
import CSharpIcon from './CSharpIcon.vue';
import KotlinIcon from './KotlinIcon.vue';
import ZigIcon from './ZigIcon.vue';
import LuaIcon from './LuaIcon.vue';
import { toast } from 'vue-sonner';

import { sendEventSchema, type SendEventForm } from './sendEvent.schema';
import { useSendEvent } from './useEventQueries';
import { useEventTypeList } from '../event_types/useEventTypeQueries';
import { useSecretList } from '../application_secrets/useSecretQueries';
import { useSubscriptionList } from '../subscriptions/useSubscriptionQueries';
import {
  kvPairsToRecord,
  recordToKvPairs,
  type Hook0KeyValueKeyValuePair,
} from '@/components/Hook0KeyValue';
import featureFlags from '@/feature-flags';
import { routes } from '@/routes';
import { handleMutationError } from '@/utils/handleMutationError';
import { useAuthStore } from '@/stores/auth';
import { useOnboardingStore } from '@/stores/onboarding';
import { usePermissions } from '@/composables/usePermissions';
import { useInstanceConfig } from '@/composables/useInstanceConfig';
import { useTracking } from '@/composables/useTracking';
import { getUseCasePreset, formatEventTypeName } from '@/utils/usecasePreset';
import { HOOK0_SDKS } from '@/generated/sdkExamples';
import type { Hook0CodeLanguage } from '@/components/Hook0Code';
import {
  CURL_PANEL,
  FEATURED_TARGETS,
  FORM_PANEL,
  hashForPanel,
  panelFromHash,
  partitionSdks,
  renderCurlSnippet,
  renderSdkSnippets,
  type Hook0IconTarget,
} from './sendEventSnippets';

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
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0EmptyState from '@/components/Hook0EmptyState.vue';
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
const { trackEvent } = useTracking();
const { canCreate } = usePermissions();

/** Everything this screen sends to Matomo, so the funnel reads under one name. */
const TRACKING_CATEGORY = 'send-event';

/**
 * The mark shown beside a language.
 *
 * Keyed exhaustively by `ICON_TARGETS`, which is what makes this map impossible to get wrong in
 * either direction: a missing entry and a leftover one are both type errors here, and the tests
 * hold that same list against the registry. A target the map somehow did not answer to would show
 * no mark rather than break, which is why the lookup keeps saying it might not answer.
 */
const LANGUAGE_ICONS: Record<Hook0IconTarget, Component> = {
  typescript: JavaScriptIcon,
  python: PythonIcon,
  java: JavaIcon,
  csharp: CSharpIcon,
  php: PhpIcon,
  go: GoIcon,
  rust: RustIcon,
  kotlin: KotlinIcon,
  lua: LuaIcon,
  ruby: RubyIcon,
  zig: ZigIcon,
};

function iconFor(target: string): Component | undefined {
  return (LANGUAGE_ICONS as Record<string, Component | undefined>)[target];
}

// Throws rather than quietly dropping a tab the screen promises: the SDKs are a compile-time
// constant, so a featured target no SDK answers to is a mistake, not a state to render.
const { featured: featuredSdks, others: pickerSdks } = partitionSdks(HOOK0_SDKS, FEATURED_TARGETS);

const sdkByTarget = new Map(HOOK0_SDKS.map((sdk) => [sdk.target, sdk]));

// Panels — driven by the URL hash (no remount since App.vue uses route.path as key)
const tabRefs = ref<HTMLElement[]>([]);

const tabs = computed(() => [
  { id: FORM_PANEL, label: t('events.tabs.easy'), icon: FormInput as Component },
  { id: CURL_PANEL, label: t('events.tabs.curl'), icon: Terminal as Component },
  ...featuredSdks.map((sdk) => ({
    id: sdk.target,
    label: sdk.displayName,
    icon: iconFor(sdk.target),
  })),
]);

const activePanel = computed(() => panelFromHash(route.hash, HOOK0_SDKS));
const activeSdk = computed(() => sdkByTarget.get(activePanel.value));

/**
 * Whether the open panel is one the strip carries a tab for.
 *
 * False for the nine languages the picker holds, which a fragment in a shared URL opens as readily
 * as the picker itself does.
 */
const someTabIsOpen = computed(() => tabs.value.some((tab) => tab.id === activePanel.value));

/**
 * Whether this tab is the one a Tab press reaches.
 *
 * A tablist keeps exactly one element focusable and moves within itself with the arrow keys, which
 * is what stops a strip of twelve from filling the page's tab order. Letting that follow the open
 * panel alone leaves the strip with no focusable element at all while a picker language is open:
 * Tab and Shift+Tab step over it, and the arrow handler cannot hand the focus back because it only
 * ever fires from a tab that already holds it. So the first tab stands in whenever no tab is open,
 * and the strip is reachable from the keyboard in every state. `aria-selected` deliberately does
 * not follow: with a picker language open no tab is in fact open, and saying one is would name a
 * panel that is not on screen.
 */
function isRovingTab(target: string, index: number): boolean {
  return someTabIsOpen.value ? target === activePanel.value : index === 0;
}

/**
 * The ids a tab and its panel point at each other with, in the shape `Hook0Tabs` uses.
 *
 * One id for the panel rather than one per tab, which is where this parts company with that
 * component: it renders every panel and hides the closed ones, so each `aria-controls` always
 * resolves, while this screen mounts a single panel with `v-if`. An `aria-controls` naming an
 * element that is not in the document leads a reader following it to nothing at all, so every tab
 * names the one panel that is always there, and whichever branch is rendered — the skeleton, the
 * error card, the form, the empty state or the code block — answers to that id.
 *
 * A tab's own id is derived from the panel it opens, so a language added to the registry arrives
 * with its pair already made and no list here can fall behind it.
 */
const TAB_PANEL_ID = 'send-event-tabpanel';

function tabButtonId(panel: string): string {
  return `send-event-tab-${panel}`;
}

/**
 * What a code panel is called when no tab is there to call it anything.
 *
 * A panel is named by the tab that opens it, and the languages behind the picker have none — so
 * they carry the language's own name instead. Pointing `aria-labelledby` at an id that is not on
 * the page would leave the panel nameless, which is worse than the role alone.
 *
 * Read off the picker's own list, which holds every language reaching this state and nothing else,
 * so there is no second name written here for a panel that is never asked.
 */
const codePanelName = computed(() =>
  pickerSdks
    .filter((sdk) => sdk.target === activePanel.value)
    .map((sdk) => sdk.displayName)
    .join('')
);

/**
 * Whether the payload editor may still take the focus for itself.
 *
 * Arriving at this screen is what that autofocus is for: the cursor waits in the editor and a
 * reader types. Coming back to the form is not the same thing. The form panel is mounted by
 * `v-if`, so every return to its tab builds the editor afresh and it claims the focus again —
 * including the focus an arrow key has just moved onto the tab itself. The tab strip then dead-ends
 * there: one press reaches the form and no further press is heard, because the keydown handler
 * lives on a button nothing is focused on any more. Silent, too — the tabs still look right.
 */
const payloadEditorMayTakeFocus = ref(true);

/** The picker shows the open language, or nothing at all while a tab is open. */
const pickedTarget = computed(() =>
  pickerSdks.some((sdk) => sdk.target === activePanel.value) ? activePanel.value : ''
);

/**
 * Opens the language the picker was moved to, and puts it back when it was moved to nothing.
 *
 * The picker's first entry is its own label rather than a language, so choosing it opens nothing —
 * and nothing is not the same as no change. The control has already moved to the label by the time
 * this is heard, while the panel stays where it was; and since the value bound to it has not moved
 * either, nothing re-renders to put it right. Measured: with a picker language open, choosing the
 * label left the select reading "More languages…", still marked as though a language were picked,
 * above that language's example. So the control is put back on the language that is in fact open.
 * Written as a listener rather than as a `v-model` setter for that reason: a setter that declines a
 * value has no way to say so to a control that has already taken it.
 */
function pickLanguage(event: Event) {
  const picker = event.target as HTMLSelectElement;
  if (picker.value.length > 0) {
    openPanel(picker.value);
    return;
  }
  picker.value = pickedTarget.value;
}

function setTabRef(el: unknown, index: number) {
  if (el instanceof HTMLElement) {
    tabRefs.value[index] = el;
  }
}

function openPanel(panel: string) {
  payloadEditorMayTakeFocus.value = false;
  void router.replace({ ...route, hash: hashForPanel(panel) });
}

function activateTab(panel: string, index: number) {
  openPanel(panel);
  void nextTick(() => {
    tabRefs.value[index]?.focus();
  });
}

function handleTabKeydown(event: KeyboardEvent, index: number) {
  const count = tabs.value.length;
  let newIndex = index;
  if (event.key === 'ArrowRight') {
    newIndex = (index + 1) % count;
    event.preventDefault();
  } else if (event.key === 'ArrowLeft') {
    newIndex = (index - 1 + count) % count;
    event.preventDefault();
  } else if (event.key === 'Home') {
    newIndex = 0;
    event.preventDefault();
  } else if (event.key === 'End') {
    newIndex = count - 1;
    event.preventDefault();
  } else {
    return;
  }
  activateTab(tabs.value[newIndex].id, newIndex);
}

function trackCopy(block: string) {
  trackEvent(TRACKING_CATEGORY, 'copy', `${activePanel.value}:${block}`);
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

/**
 * Whether the application has no event type to send.
 *
 * An application that has just been created has none, and the empty events list sends a reader
 * straight here from it. An event is sent as a type, so there is nothing for the select to offer
 * and nothing the form could send — a state of its own, and not the slow fetch or the failed one
 * the two panels above stand for. Left to fall through, it renders a select with no option under a
 * submit button whose tooltip asks for fields that are all already filled.
 */
const hasNoEventType = computed(() => eventTypeOptions.value.length === 0);

/**
 * The event types never arrived at all.
 *
 * Guarded on the list being absent rather than on the error alone. Queries here are refetched on a
 * window regaining the focus once their data has gone stale, and retried once; so a refused refresh
 * — a rate limit, a session that has just expired — sets the error while the good list is still in
 * hand. An error card guarded on the error alone would then take away the form a reader is filling
 * in, or the snippet they are half way through copying, over a request that changed nothing.
 */
const eventTypesFailure = computed(() => {
  const failure = eventTypesError.value;
  if (failure === null || rawEventTypes.value !== undefined) {
    return undefined;
  }
  return failure;
});

/** The event types are still on their way, and no panel can say anything until they are here. */
const showsEventTypesSkeleton = computed(
  () =>
    eventTypesFailure.value === undefined &&
    (eventTypesLoading.value || rawEventTypes.value === undefined)
);

/**
 * Whether the event types are known and there is one to send.
 *
 * Every panel is written against them — the form offers them, the examples name one — so this is
 * what all of them wait for.
 */
const eventTypesReady = computed(
  () =>
    eventTypesFailure.value === undefined && !showsEventTypesSkeleton.value && !hasNoEventType.value
);

/**
 * Whether this instance still takes an application secret as a Bearer credential.
 *
 * `middleware_biscuit.rs` accepts one only under a compatibility setting the API marks deprecated,
 * and `GET /instance` reports that setting. With it off, every example built on an application
 * secret is answered with a 401 and the navigation hides the page the secret is created on — so the
 * screen has to offer something else rather than print a credential the server refuses.
 *
 * Three answers, not two: accepted, refused, and unknown. Unknown is not a default to fall back on —
 * printing a credential on the strength of a guess is the very thing this stands against — so it is
 * held both while the call has not come back yet (a skeleton) and when it came back refused (the
 * state below, which withholds every credential-bearing example and offers the call again).
 */
const {
  data: instanceConfig,
  error: instanceConfigError,
  refetch: refetchInstanceConfig,
} = useInstanceConfig();

const instanceSettled = computed(
  () => instanceConfig.value !== undefined || instanceConfigError.value !== null
);

const applicationSecretRefused = computed(() => {
  const config = instanceConfig.value;
  return config !== undefined && !config.application_secret_compatibility;
});

/**
 * The instance never said whether it takes an application secret.
 *
 * Guarded on the config being absent rather than on the error alone, the way the event types and the
 * secrets are: a refused refresh once the config is in hand changed nothing on screen, and the last
 * good answer is better than a card thrown over it. What is left — the config still undefined with an
 * error against it — is the third answer, neither accepted nor refused, and the screen withholds
 * every credential-bearing example in it rather than guessing the setting in either direction.
 */
const instanceConfigFailure = computed(() => {
  const failure = instanceConfigError.value;
  if (failure === null || instanceConfig.value !== undefined) {
    return undefined;
  }
  return failure;
});

const applicationSecretUnknown = computed(() => instanceConfigFailure.value !== undefined);

/**
 * Where a reader goes for a credential their own code can carry, and what that page is called.
 *
 * A service token where an application secret is no longer taken: it is a biscuit, which the
 * middleware accepts whatever the compatibility setting says, and which the application-secrets
 * concept page names as the credential to use in that configuration.
 */
const credentialRoute = computed(() =>
  applicationSecretRefused.value ? routes.ServicesTokenList : routes.ApplicationSecretsList
);

const credentialCta = computed(() =>
  applicationSecretRefused.value ? t('serviceTokens.create') : t('events.tabs.noSecretCta')
);

/**
 * Whether the instance has said, in so many words, that it accepts an application secret.
 *
 * The positive of the two states above, and not merely their negation: a loading instance is neither
 * refused nor failed either, and asking it for a secret before it has answered would race that
 * answer — the list would go out, and only then would the instance say it is never read. So the
 * question is put positively, and is false until the config is in hand with the setting on.
 */
const applicationSecretAccepted = computed(() => {
  const config = instanceConfig.value;
  return config !== undefined && config.application_secret_compatibility;
});

// Secrets query. Not asked for at all where its answer is never read: the tutorial authenticates its
// examples with the session's own token, and outside it the list is worth fetching only once the
// instance has said it takes an application secret — a refused, failed or not-yet-answered instance
// has no example to put one in.
const secretsWanted = computed(() => !props.tutorialMode && applicationSecretAccepted.value);
const {
  data: secrets,
  error: secretsError,
  refetch: refetchSecrets,
} = useSecretList(applicationId, secretsWanted);

/** The secrets never arrived, guarded on their absence for the reason the event types are. */
const secretsFailure = computed(() => {
  const failure = secretsError.value;
  if (failure === null || secrets.value !== undefined) {
    return undefined;
  }
  return failure;
});

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

// In the onboarding tutorial, pre-select the event type created a step earlier and
// seed matching labels + payload from the use-case chosen at intro. Outside the
// tutorial (or when "Other"/skipped), keep the generic defaults.
const onboarding = useOnboardingStore();
const preset = props.tutorialMode ? getUseCasePreset(onboarding.useCase) : undefined;

const { errors, meta, values, setFieldValue } = useForm<SendEventForm>({
  validationSchema: toTypedSchema(sendEventSchema),
  initialValues: {
    eventType: preset ? formatEventTypeName(preset.eventType) : '',
    labels: preset
      ? preset.labels.map((label) => ({ ...label }))
      : [{ key: 'user_id', value: '1' }],
    occurredAt: formatDateTimeLocal(new Date()),
    payload: preset ? preset.payload : '{"test": true}',
  },
});

// The subscription created at the previous tutorial step is the contract this event
// has to satisfy: Hook0 routes an event to a subscription only when the subscription
// listens to the event's type and the event's labels contain all of the
// subscription's. The use case picked at the intro lives in the SPA session alone, so
// a reload — or a tab restored straight on this step — would seed a generic `user_id`
// label against a subscription filtering on `customer_id`, and the wizard would end on
// "your webhook pipeline is live" with nothing ever delivered. Reading the persisted
// subscription instead makes both sides agree whatever the session state, so the
// preset above is only ever an optimistic first paint.
// When that read is refused there is nothing to make them agree with, and the preset is all the
// form has: the state below says so and offers the read again, rather than letting the wizard end
// on "your webhook pipeline is live" over an event no subscription was listening for.
const isTutorial = computed(() => props.tutorialMode);
const {
  data: tutorialSubscriptions,
  error: tutorialSubscriptionsError,
  refetch: refetchTutorialSubscriptions,
} = useSubscriptionList(applicationId, isTutorial);
const seededFromSubscription = ref(false);

const showsTutorialSubscriptionsError = computed(
  () => tutorialSubscriptionsError.value !== null && tutorialSubscriptions.value === undefined
);

watch(
  tutorialSubscriptions,
  (subscriptions) => {
    if (seededFromSubscription.value || !subscriptions || subscriptions.length === 0) {
      return;
    }
    // The most recent one: the tutorial just created it, and a subscription that
    // already existed on the application must not win over it.
    const target = subscriptions.reduce((newest, candidate) =>
      candidate.created_at > newest.created_at ? candidate : newest
    );
    const targetLabels = recordToKvPairs(target.labels);
    // A subscription without labels accepts every event, so the generic default
    // already satisfies it — and the form requires at least one label to submit.
    if (targetLabels.length > 0) {
      void setFieldValue('labels', targetLabels);
    }
    if (target.event_types.length > 0) {
      void setFieldValue('eventType', target.event_types[0]);
    }
    seededFromSubscription.value = true;
  },
  { immediate: true }
);

// Bind form fields to refs for template v-model
const selectedEventType = computed({
  get: () => values.eventType ?? '',
  set: (v: string) => void setFieldValue('eventType', v),
});

const labels = computed({
  get: () => (values.labels ?? []) as Hook0KeyValueKeyValuePair[],
  set: (v: Hook0KeyValueKeyValuePair[]) => void setFieldValue('labels', v),
});

/**
 * Takes a copy of the editor's rows rather than the rows themselves.
 *
 * `Hook0KeyValue` emits the very objects its inputs are bound to, so without the copy the form's
 * value is an alias of another component's internal state and every keystroke mutates it in place,
 * underneath the validation. What that costs was measured: raise a message about one row — a blank
 * key, which the schema has always refused — and it never goes away again. Typing a perfectly good
 * value back in leaves the message on screen and the Send button disabled, with nothing left on the
 * form to explain either, and no keystroke anywhere recovers it. With a copy, the form has a value
 * of its own to compare against and the message clears the moment the row is good.
 */
function onLabelsUpdate(pairs: Hook0KeyValueKeyValuePair[] | Record<string, string>) {
  const rows = Array.isArray(pairs) ? pairs : recordToKvPairs(pairs);
  labels.value = rows.map((row) => ({ key: row.key, value: row.value }));
}

const occurredAt = computed({
  get: () => values.occurredAt ?? formatDateTimeLocal(new Date()),
  set: (v: string) => void setFieldValue('occurredAt', v),
});

const payload = computed({
  get: () => values.payload ?? '{"test": true}',
  set: (v: string) => void setFieldValue('payload', v),
});

/**
 * What is wrong with the labels, in the schema's own words.
 *
 * `useForm` reports a row of the editor under its own path — `labels[0].key` and its siblings — and
 * the editor draws the rows itself rather than as fields this screen can hand a message to, so the
 * first thing the schema has to say about any of them is said once, beneath it.
 */
const labelsError = computed(() => {
  for (const [path, message] of Object.entries(errors.value)) {
    if (path.startsWith('labels') && typeof message === 'string') {
      return message;
    }
  }
  return '';
});

/**
 * What is wrong with the payload, in the schema's own words.
 *
 * `useForm` reports a field that is fine by leaving its key out of `errors` entirely; the field
 * below reads an empty message as nothing to say, so the two shapes are reconciled here and not
 * carried any further.
 */
const payloadError = computed(() => {
  const reported = errors.value.payload;
  return typeof reported === 'string' ? reported : '';
});

/**
 * What is wrong with the instant, in the schema's own words.
 *
 * The schema has always required it and a datetime field is cleared as easily as any other, so the
 * only sign was the submit button going disabled under a tooltip asking for fields that are all
 * filled — the two below it say what is wrong, and this one said nothing at all.
 */
const occurredAtError = computed(() => {
  const reported = errors.value.occurredAt;
  return typeof reported === 'string' ? reported : '';
});

// Authorization token for snippets
const snippetToken = computed(() => {
  if (props.tutorialMode) {
    return authStore.accessToken ?? '';
  }
  return effectiveSecretToken.value;
});

// An application keeps its secrets only as long as nobody deletes the last one, and the ones
// created before secrets were provisioned at all may never have had any. Until the list has come
// back there is nothing to say either way, so the panel waits rather than accusing.
// A failed fetch leaves `data` undefined for good, so waiting on it alone cannot tell a request in
// flight from one that came back refused — and the panels below would sit on a skeleton forever,
// which reads as slow rather than as broken. The error is the other half of the answer.
// Whether a secret would be accepted at all is the question before that one, so the instance has to
// have answered too; where the answer is no or never came, and in the tutorial, there is no list to
// wait for.
const secretsSettled = computed(() => {
  if (props.tutorialMode) {
    return true;
  }
  if (!instanceSettled.value) {
    return false;
  }
  return (
    applicationSecretRefused.value ||
    applicationSecretUnknown.value ||
    secrets.value !== undefined ||
    secretsError.value !== null
  );
});
const hasToken = computed(() => snippetToken.value.length > 0);

/**
 * The origin every example is written against.
 *
 * Read the way `http.ts` reads it rather than straight off the build, because the two have to be
 * the same origin. `?API_ENDPOINT=` on this page's own URL points the dashboard at another server —
 * validated against an allowlist, and documented in `self-hosting/bare-metal.md` as the way to work
 * against one. Under that override the Send button posted to the overridden origin and succeeded
 * while all twelve examples went on naming the one this bundle was built with, so a reader copied a
 * snippet addressed to a server the page in front of them was not using.
 *
 * No fallback behind the reader: `env.d.ts` types the variable as a plain string, every build path
 * sets it, and a build that did not would have broken `http.ts` long before this screen.
 */
const apiBaseUrl = computed(() =>
  featureFlags.getOrElse('API_ENDPOINT', import.meta.env.VITE_API_ENDPOINT)
);

/**
 * The instant the form names, in the shape the API reads. Only cURL carries it: the SDK examples
 * leave `occurred_at` unset and let the server date the event, so no example asks for it.
 *
 * A half-typed datetime parses to nothing, and asking that for its ISO form throws. The field is
 * being typed into while this is read, so the present instant stands in until it parses.
 */
const occurredAtIso = computed(() => {
  const chosen = new Date(occurredAt.value);
  return Number.isNaN(chosen.getTime()) ? new Date().toISOString() : chosen.toISOString();
});

/**
 * The labels this screen will post, and the very same ones every example prints.
 *
 * One value rather than two. The request was built from a trimmed copy of the rows while the
 * examples were built from the rows as typed, so a label whose key or value was nothing but spaces
 * appeared in all twelve snippets and was then dropped from the call — and when it was the only
 * label, the send was refused outright by a button that had never looked disabled. The schema now
 * refuses a blank row, so nothing reaches here to be discarded quietly; the trim stays because a
 * row is half-typed for as long as someone is typing it, and an example has no business printing a
 * label the request would not carry.
 */
const labelsToSend = computed(() =>
  labels.value
    .map((label) => ({ key: label.key.trim(), value: label.value.trim() }))
    .filter((label) => label.key.length > 0 && label.value.length > 0)
);

/**
 * The event type every example names, which is always one this application has.
 *
 * The form's select settles on its first option the moment it mounts, and for as long as the
 * examples read the form alone that was enough — on the form's own tab. A code tab never mounts it:
 * a fragment in a shared link opens one directly, and the form's value is still empty there. What
 * stood in was a name invented for the occasion, and `event_type` is a foreign key — so the snippet
 * a reader pasted was refused by the API for naming a type that does not exist, on an application
 * whose real types were sitting one tab away. The first of those is read here instead. The panels
 * below render no snippet at all until the list has come back with something in it, so there is
 * always one to read.
 */
const snippetEventType = computed(() => {
  const options = eventTypeOptions.value;
  const chosen = values.eventType;
  // Held against the list rather than taken on trust. The tutorial seeds the form with the type its
  // use-case preset names, so the value is filled from the first render and nothing on a code tab
  // ever confronts it: the strip is clickable while the list is still out, and a language opened
  // then keeps the preset for good. `event_type` is a foreign key, so a name the application does
  // not have is a snippet the API refuses.
  if (options.some((option) => option.value === chosen)) {
    return chosen;
  }
  const [first] = options;
  return first === undefined ? '' : first.value;
});

/** What the form is worth to an example. */
const snippetValues = computed(() => ({
  apiUrl: apiBaseUrl.value,
  applicationId: applicationId.value,
  token: snippetToken.value,
  eventType: snippetEventType.value,
  payload: payload.value || '{}',
  labels: labelsToSend.value,
}));

const curlSnippet = computed(() =>
  renderCurlSnippet({
    ...snippetValues.value,
    eventId: crypto.randomUUID(),
    occurredAt: occurredAtIso.value,
  })
);

/**
 * The open language together with its three blocks, or nothing when the open panel is not one.
 *
 * One value rather than two, so the panel below cannot render half of it: the SDK is what names the
 * package and its registry, the render is what fills the blocks, and neither is worth showing alone.
 */
const activeSdkExample = computed(() => {
  const sdk = activeSdk.value;
  if (sdk === undefined) {
    return undefined;
  }
  return { sdk, snippets: renderSdkSnippets(sdk, snippetValues.value) };
});

/**
 * What the API would refuse about the request every example prints, in the schema's own words.
 *
 * Held against what the examples carry rather than against what the form holds, which are not the
 * same thing: a code panel names the application's first event type where the form's select has not
 * mounted to choose one, dates the event itself where the field is half typed, and drops a row that
 * is nothing but spaces. Reading the form's own state instead would refuse a panel a link opened
 * directly, over fields nobody has been offered.
 *
 * Read from the schema rather than from the form, for the same reason: the fields are mounted by the
 * tab that holds them, so on a code panel the form has nothing to say. The schema answers either
 * way.
 *
 * What it is for: an event with no label, or a payload that is not JSON, is refused by the API, and
 * the form says so where it is on screen while the code panels went on printing the request as
 * though it would run. `LABELS_MIN_SIZE` is 1, and clearing the key of the only row is enough.
 */
const unsendableReason = computed(() => {
  const checked = sendEventSchema.safeParse({
    eventType: snippetValues.value.eventType,
    labels: snippetValues.value.labels,
    occurredAt: occurredAtIso.value,
    payload: snippetValues.value.payload,
  });
  return checked.success ? '' : checked.error.issues[0].message;
});

/**
 * The target name doubles as the grammar name, which is what lets a language be added without this
 * screen learning anything about it. A target no grammar answers to renders as plain text, so the
 * narrowing cannot turn into a broken block.
 */
const activeLanguage = computed(() => activePanel.value as Hook0CodeLanguage);

/** Whether a code panel is showing an example rather than one of the states that stand in for one. */
const panelShowsSnippet = computed(() => {
  if (!eventTypesReady.value || activePanel.value === FORM_PANEL || !secretsSettled.value) {
    return false;
  }
  // In the tutorial the snippet carries the session's own token, present and accepted whatever the
  // instance says, so none of the durable-credential states withhold it. Outside it, each one does.
  const credentialReady = props.tutorialMode
    ? hasToken.value
    : !applicationSecretRefused.value &&
      instanceConfigFailure.value === undefined &&
      secretsFailure.value === undefined &&
      hasToken.value;
  return credentialReady && unsendableReason.value.length === 0;
});

/**
 * The panel a reader is in fact reading, or nothing while a state stands in for it.
 *
 * The figure below is read as "this language was opened and its example was there", and the order
 * the picker lists languages in is meant to be revisited on it. Reported from the fragment alone it
 * also counted the openings that showed a skeleton, an error card or the empty state: on an
 * application with no event type, every fragment scored an opening and no panel ever showed an
 * example. The form counts as a panel too, or the language figures have no denominator.
 */
const openedPanel = computed(() => {
  const panel = activePanel.value;
  if (panel === FORM_PANEL) {
    return eventTypesReady.value ? panel : '';
  }
  return panelShowsSnippet.value ? panel : '';
});

watch(
  openedPanel,
  (panel) => {
    if (panel.length > 0) {
      trackEvent(TRACKING_CATEGORY, 'open', panel);
    }
  },
  { immediate: true }
);

// Submit
function sendTestEvent() {
  if (!meta.value.valid) {
    toast.error(t('events.invalidEvent'), {
      description: t('events.invalidEventMessage'),
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
      labels: kvPairsToRecord(labelsToSend.value),
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
      </Hook0CardHeader>

      <!-- SHARED: Always visible tabs, plus the picker holding the languages a bar cannot fit -->
      <div class="send-event__tabs">
        <div class="send-event__tablist" role="tablist" :aria-label="t('events.sendTestEvent')">
          <button
            v-for="(tab, index) in tabs"
            :id="tabButtonId(tab.id)"
            :key="tab.id"
            :ref="(el) => setTabRef(el, index)"
            role="tab"
            :aria-selected="activePanel === tab.id"
            :aria-controls="TAB_PANEL_ID"
            :tabindex="isRovingTab(tab.id, index) ? 0 : -1"
            class="send-event__tab"
            :class="{ 'send-event__tab--active': activePanel === tab.id }"
            :data-test="`send-event-tab-${tab.id}`"
            @click="activateTab(tab.id, index)"
            @keydown="handleTabKeydown($event, index)"
          >
            <component :is="tab.icon" v-if="tab.icon" :size="16" aria-hidden="true" />
            {{ tab.label }}
          </button>
        </div>

        <select
          :value="pickedTarget"
          class="send-event__picker"
          :class="{ 'send-event__picker--active': pickedTarget.length > 0 }"
          :aria-label="t('events.tabs.moreLanguages')"
          data-test="send-event-language-select"
          @change="pickLanguage"
        >
          <option value="">{{ t('events.tabs.moreLanguages') }}</option>
          <option v-for="sdk in pickerSdks" :key="sdk.target" :value="sdk.target">
            {{ sdk.displayName }}
          </option>
        </select>
      </div>

      <!-- CONDITIONAL CONTENT -->

      <!-- The event types answer for every panel, not for the form alone: the form offers them and
           the examples name one, so until the list is known there is nothing any panel can honestly
           show. Held back to the form's own tab, the three states below left the code panels to
           print a name of their own invention.

           Error before skeleton, and that order is the whole of it: a failed fetch leaves `data`
           undefined for good, so a skeleton guarded on its absence claims the refused state as well
           as the one still in flight and never gives it up. The card below was unreachable —
           measured, not reasoned: with the event types refused, this panel showed three skeletons
           and went on showing them. The screen already says as much about the secrets panel, a few
           lines down; this one had the same hole.

           None of the three is a code panel, so none of them is named after the open language: with
           a picker language open, "No event type yet" announced itself as "Rust". -->
      <Hook0CardContent
        v-if="eventTypesFailure"
        :id="TAB_PANEL_ID"
        role="tabpanel"
        :aria-labelledby="someTabIsOpen ? tabButtonId(activePanel) : undefined"
        :aria-label="someTabIsOpen ? undefined : t('events.panels.eventTypesRefused')"
      >
        <Hook0ErrorCard :error="eventTypesFailure" @retry="refetchEventTypes()" />
      </Hook0CardContent>

      <!-- Loading skeleton -->
      <Hook0CardContent
        v-else-if="showsEventTypesSkeleton"
        :id="TAB_PANEL_ID"
        role="tabpanel"
        :aria-labelledby="someTabIsOpen ? tabButtonId(activePanel) : undefined"
        :aria-label="someTabIsOpen ? undefined : t('events.panels.eventTypesLoading')"
        data-test="send-event-loading"
      >
        <Hook0SkeletonGroup :count="3" />
      </Hook0CardContent>

      <!-- Nothing to send yet. The form would render a select with no option in it, and refuse to
           submit for a reason it states as an unfilled field; an example would name a type the
           application does not have, and `event_type` is a foreign key. -->
      <Hook0CardContent
        v-else-if="hasNoEventType"
        :id="TAB_PANEL_ID"
        role="tabpanel"
        :aria-labelledby="someTabIsOpen ? tabButtonId(activePanel) : undefined"
        :aria-label="someTabIsOpen ? undefined : t('events.noEventType.title')"
        data-test="send-event-no-event-type"
      >
        <Hook0EmptyState
          :title="t('events.noEventType.title')"
          :description="t('events.noEventType.description')"
          :icon="FolderTree"
        >
          <template v-if="canCreate('event_type')" #action>
            <Hook0Button
              variant="primary"
              data-test="send-event-create-event-type-button"
              :to="{ name: routes.EventTypesNew, params: route.params }"
            >
              {{ t('events.noEventType.cta') }}
            </Hook0Button>
          </template>
        </Hook0EmptyState>
      </Hook0CardContent>

      <!-- Easy way: Form -->
      <Hook0Form
        v-else-if="activePanel === FORM_PANEL"
        :id="TAB_PANEL_ID"
        role="tabpanel"
        :aria-labelledby="tabButtonId(FORM_PANEL)"
        data-test="send-event-form"
        @submit="sendTestEvent"
      >
        <Hook0CardContent>
          <!-- The subscription created a step earlier is what this event has to satisfy, and it
               could not be read. The preset below is a guess at it, so it is named as one. -->
          <div
            v-if="showsTutorialSubscriptionsError"
            class="send-event__notice"
            data-test="send-event-tutorial-subscription-error"
          >
            <p>{{ t('events.tutorialSubscriptionRefused') }}</p>
            <Hook0Button
              variant="secondary"
              data-test="send-event-tutorial-subscription-retry"
              @click="refetchTutorialSubscriptions()"
            >
              {{ t('common.retry') }}
            </Hook0Button>
          </div>
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
                @update:model-value="onLabelsUpdate($event)"
              />
              <p
                v-if="labelsError.length > 0"
                class="send-event__field-error"
                role="alert"
                data-test="send-event-labels-error"
              >
                {{ labelsError }}
              </p>
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine>
            <template #label>{{ t('events.occurredAt') }}</template>
            <template #content>
              <!-- Through the field's own message rather than beside it, the way the rest of the
                   dashboard's inputs say what is wrong: the field then also announces itself as
                   invalid and points at the message, which the two paragraphs below cannot do for
                   controls that are not fields. The id is written out so that pointer resolves to
                   the same element on every render. -->
              <Hook0Input
                id="send-event-occurred-at"
                v-model="occurredAt"
                type="datetime-local"
                :error="occurredAtError"
                data-test="send-event-occurred-at-input"
              />
            </template>
          </Hook0CardContentLine>
          <Hook0CardContentLine>
            <template #label>{{ t('events.payload') }}</template>
            <template #content>
              <div data-test="send-event-payload-input">
                <!-- Tab moves on rather than indenting. `vue-codemirror` turns that binding on by
                     default, so it is switched off here rather than left out: with it on, Tab was
                     measured staying inside the editor, and the editor is the last field of the
                     form — Cancel and Send sit right after it, and nothing but a mouse reached
                     them. A JSON payload gains little from tab-indentation, and most of it is
                     pasted. -->
                <Codemirror
                  v-model="payload"
                  :autofocus="payloadEditorMayTakeFocus"
                  :indent-with-tab="false"
                  :tab-size="2"
                  :extensions="extensions"
                />
              </div>
              <!-- The schema refuses a payload that is not JSON and, until now, said so nowhere:
                   the only sign was the submit button going disabled under a tooltip asking for
                   fields that are all filled. Shown the way the select and the inputs above show
                   theirs, since the editor is not one of them and carries no message of its own. -->
              <p
                v-if="payloadError.length > 0"
                class="send-event__field-error"
                role="alert"
                data-test="send-event-payload-error"
              >
                {{ payloadError }}
              </p>
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

      <!-- Every code panel, cURL and the SDKs alike -->
      <Hook0CardContent
        v-else
        :id="TAB_PANEL_ID"
        role="tabpanel"
        :aria-labelledby="someTabIsOpen ? tabButtonId(activePanel) : undefined"
        :aria-label="someTabIsOpen ? undefined : codePanelName"
        :data-test="
          activePanel === CURL_PANEL ? 'send-event-curl-panel' : `send-event-panel-${activePanel}`
        "
      >
        <!-- The credential a snippet on this screen carries during the tutorial is the dashboard's
             own session token: it works the minute it is copied and is refused shortly after, with
             nothing on screen to connect the two. The note points at the durable credential to move
             to. While the config is loading it names the default one, application secrets, which
             almost every instance accepts and which stays right once the config lands, so the common
             path never swaps. Only a confirmed instance failure drops the recommendation, because
             then the screen genuinely cannot know which credential the instance takes. -->
        <p
          v-if="tutorialMode && panelShowsSnippet && instanceConfigFailure === undefined"
          class="send-event__block-note"
          data-test="send-event-tutorial-token"
        >
          {{ t('events.tutorialTokenNote') }}
          <router-link :to="{ name: credentialRoute, params: route.params }">
            {{ credentialCta }}
          </router-link>
        </p>
        <!-- On a confirmed instance failure the screen cannot learn which credential the instance
             accepts, so the note states the need and names no page. -->
        <p
          v-else-if="tutorialMode && panelShowsSnippet"
          class="send-event__block-note"
          data-test="send-event-tutorial-token"
        >
          {{ t('events.tutorialTokenNoteUnknown') }}
        </p>

        <Hook0SkeletonGroup v-if="!secretsSettled" :count="2" />

        <!-- An application secret authenticates nothing on this instance, so there is no snippet to
             show: one carrying it would be answered with a 401, and the page it is created on is
             not even in the navigation here. -->
        <div
          v-else-if="!tutorialMode && applicationSecretRefused"
          class="send-event__notice"
          data-test="send-event-secret-not-accepted"
        >
          <p>{{ t('events.tabs.secretNotAccepted') }}</p>
          <router-link
            :to="{ name: routes.ServicesTokenList, params: route.params }"
            class="send-event__notice-link"
          >
            {{ t('serviceTokens.create') }}
          </router-link>
        </div>

        <!-- The instance never said whether it takes an application secret, so which credential every
             example should carry is unknown. Rather than print one on a guess — a Bearer the server
             may answer with a 401, or a blanket "use a service token" when secrets may well be taken
             — the panel says the call failed and offers it again, the way it does for a refused
             secrets fetch. -->
        <div
          v-else-if="!tutorialMode && instanceConfigFailure"
          data-test="send-event-instance-config-error"
        >
          <Hook0ErrorCard :error="instanceConfigFailure" @retry="refetchInstanceConfig()" />
        </div>

        <!-- The secrets never arrived. Saying so, with the way back, rather than showing a snippet
             built on a token this screen does not have. The name stays the card's own: passing one
             in here replaced it, and the state the house selector reaches was then this one alone. -->
        <div v-else-if="!tutorialMode && secretsFailure" data-test="send-event-secrets-error">
          <Hook0ErrorCard :error="secretsFailure" @retry="refetchSecrets()" />
        </div>

        <!-- No secret, so no snippet: one showing `Bearer ` with nothing after it reads as working
             code and fails on the first call. -->
        <div v-else-if="!hasToken" class="send-event__notice" data-test="send-event-no-secret">
          <p>{{ t('events.tabs.noSecret') }}</p>
          <router-link
            :to="{ name: routes.ApplicationSecretsList, params: route.params }"
            class="send-event__notice-link"
          >
            {{ t('events.tabs.noSecretCta') }}
          </router-link>
        </div>

        <!-- The request the form describes is one the API refuses, so no example prints it. The
             form says what is wrong where it is on screen, and a reader who arrived here from a
             link never sees that — they see a command that cannot run. -->
        <div
          v-else-if="unsendableReason.length > 0"
          class="send-event__notice"
          data-test="send-event-unsendable"
        >
          <p>{{ t('events.tabs.unsendable', { reason: unsendableReason }) }}</p>
          <Hook0Button
            variant="secondary"
            data-test="send-event-unsendable-back"
            @click="openPanel(FORM_PANEL)"
          >
            {{ t('events.tabs.unsendableCta') }}
          </Hook0Button>
        </div>

        <Hook0Code
          v-else-if="activePanel === CURL_PANEL"
          :code="curlSnippet"
          language="bash"
          :editable="false"
          @copy="trackCopy('send')"
        />

        <Hook0Stack v-else-if="activeSdkExample" direction="column" gap="lg">
          <section class="send-event__block" data-test="send-event-install">
            <h4 class="send-event__block-title">{{ t('events.tabs.install') }}</h4>
            <!--
              Which package, at which version. The command alone does not say: nine of the eleven
              install by name and take whatever the registry serves today, so a reader comparing the
              screen against a lockfile has nothing to compare.
            -->
            <p class="send-event__block-package" data-test="send-event-install-package">
              {{
                activeSdkExample.sdk.publishedToRegistry
                  ? t('events.tabs.installPackage', {
                      package: activeSdkExample.sdk.packageName,
                      version: activeSdkExample.sdk.version,
                      registry: activeSdkExample.sdk.registry,
                    })
                  : t('events.tabs.installPackageUnpublished', {
                      package: activeSdkExample.sdk.packageName,
                      version: activeSdkExample.sdk.version,
                    })
              }}
            </p>
            <p v-if="!activeSdkExample.sdk.publishedToRegistry" class="send-event__block-note">
              {{ t('events.tabs.notOnRegistry', { registry: activeSdkExample.sdk.registry }) }}
            </p>
            <Hook0Code
              :code="activeSdkExample.snippets.install"
              language="bash"
              :editable="false"
              @copy="trackCopy('install')"
            />
          </section>

          <section class="send-event__block" data-test="send-event-send">
            <h4 class="send-event__block-title">{{ t('events.tabs.send') }}</h4>
            <!-- The one field of the form no example on this tab carries, said once where a reader
                 comparing the two tabs would otherwise read the omission as a mistake. The API
                 requires the instant, so a raw call has to state it; every SDK dates the event
                 itself when the caller leaves it unset. -->
            <p class="send-event__block-note" data-test="send-event-send-occurred-at">
              {{ t('events.tabs.sendOccurredAt') }}
            </p>
            <Hook0Code
              :code="activeSdkExample.snippets.send"
              :language="activeLanguage"
              :editable="false"
              @copy="trackCopy('send')"
            />
          </section>

          <section class="send-event__block" data-test="send-event-verify">
            <h4 class="send-event__block-title">{{ t('events.tabs.verify') }}</h4>
            <p class="send-event__block-note">
              <i18n-t keypath="events.tabs.verifyHint" tag="span">
                <template #subscription>
                  <router-link :to="{ name: routes.SubscriptionsList, params: route.params }">
                    {{ t('events.tabs.verifyHintSubscription') }}
                  </router-link>
                </template>
              </i18n-t>
            </p>
            <Hook0Code
              :code="activeSdkExample.snippets.verify"
              :language="activeLanguage"
              :editable="false"
              @copy="trackCopy('verify')"
            />
          </section>
        </Hook0Stack>

        <!-- The panel would otherwise be an empty box carrying a role, an id and a name: whatever
             is open here is neither the form, nor cURL, nor a language the registry declares. -->
        <p v-else class="send-event__block-note" data-test="send-event-no-example">
          {{ t('events.tabs.noExample') }}
        </p>
      </Hook0CardContent>
    </Hook0Card>
  </div>
</template>

<style scoped>
.send-event__tabs {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding-right: 1rem;
  border-bottom: 1px solid var(--color-border);
}

.send-event__tablist {
  display: flex;
  gap: 0;
  flex: 1 1 auto;
  overflow-x: auto;
  scrollbar-width: none;
}

.send-event__tablist::-webkit-scrollbar {
  display: none;
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
  white-space: nowrap;
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

.send-event__picker {
  flex: 0 0 auto;
  max-width: 14rem;
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-sm);
  background-color: var(--color-bg-secondary);
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
}

.send-event__picker--active {
  color: var(--color-primary);
  border-color: var(--color-primary);
}

.send-event__picker:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 1px;
}

.send-event__block-title {
  margin: 0 0 0.375rem;
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.send-event__block-package {
  margin: 0 0 0.5rem;
  font-family: var(--font-mono, monospace);
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.send-event__block-note {
  margin: 0 0 0.5rem;
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
}

.send-event__field-error {
  margin-top: 0.375rem;
  font-size: 0.75rem;
  color: var(--color-danger);
}

/* Every panel that stands in for what a reader came for, drawn the same way so the difference
   between them is what they say rather than how they look. */
.send-event__notice {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.5rem;
  padding: 1rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  background-color: var(--color-bg-secondary);
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.send-event__notice p {
  margin: 0;
}

.send-event__notice-link {
  align-self: flex-start;
  font-weight: 600;
}
</style>
