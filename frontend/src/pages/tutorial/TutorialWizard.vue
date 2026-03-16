<script setup lang="ts">
import { computed, markRaw, nextTick, ref, watch, onMounted, onUnmounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { push } from 'notivue';

import { routes } from '@/routes';
import { UUID } from '@/http';
import { useTracking } from '@/composables/useTracking';
import { useCelebration } from '@/composables/useCelebration';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Illustration from '@/components/Hook0Illustration.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';

import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import ApplicationsEdit from '@/pages/organizations/applications/ApplicationsEdit.vue';
import EventTypesNew from '@/pages/organizations/applications/event_types/EventTypesNew.vue';
import SubscriptionsEdit from '@/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue';
import EventsList from '@/pages/organizations/applications/events/EventsList.vue';

import TutorialStepProgress from '@/pages/tutorial/TutorialStepProgress.vue';

import {
  Rocket,
  Building2,
  AppWindow,
  FolderTree,
  Link,
  FileText,
  PartyPopper,
  ArrowRight,
  X,
  Plus,
  List,
  Check,
  MessageSquare,
  Github,
  BookOpen,
  Newspaper,
} from 'lucide-vue-next';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const { trackEvent } = useTracking();
const { celebrate } = useCelebration();

// ---------------------------------------------------------------------------
// Step mapping from route name
// ---------------------------------------------------------------------------
const STEP_MAP: Record<string, number> = {
  [routes.Tutorial]: 0,
  [routes.TutorialCreateOrganization]: 1,
  [routes.TutorialCreateApplication]: 2,
  [routes.TutorialCreateEventType]: 3,
  [routes.TutorialCreateSubscription]: 4,
  [routes.TutorialSendEvent]: 5,
  [routes.TutorialSuccess]: 6,
};

const currentStep = computed(() => STEP_MAP[route.name as string] ?? 0);

// ---------------------------------------------------------------------------
// Step definitions for progress bar
// ---------------------------------------------------------------------------
const STEPS = computed(() => [
  { icon: markRaw(Rocket), label: t('tutorial.steps.intro') },
  { icon: markRaw(Building2), label: t('tutorial.steps.organization') },
  { icon: markRaw(AppWindow), label: t('tutorial.steps.application') },
  { icon: markRaw(FolderTree), label: t('tutorial.steps.eventType') },
  { icon: markRaw(Link), label: t('tutorial.steps.subscription') },
  { icon: markRaw(FileText), label: t('tutorial.steps.sendEvent') },
  { icon: markRaw(PartyPopper), label: t('tutorial.steps.success') },
]);

// Progress bar steps (steps 1–5 only, mapped to 0-based for the progress component)
const PROGRESS_STEPS = computed(() => STEPS.value.slice(1, 6));

// ---------------------------------------------------------------------------
// Intro step list (step 0)
// ---------------------------------------------------------------------------
type TutorialStepItem = {
  icon: typeof Building2;
  label: string;
};

const tutorialSteps: TutorialStepItem[] = [
  { icon: Building2, label: 'tutorial.intro.anOrganization' },
  { icon: AppWindow, label: 'tutorial.intro.anApplication' },
  { icon: FolderTree, label: 'tutorial.intro.anEventType' },
  { icon: Link, label: 'tutorial.intro.aSubscription' },
  { icon: FileText, label: 'tutorial.intro.anEvent' },
];

// ---------------------------------------------------------------------------
// Step 0 — Intro
// ---------------------------------------------------------------------------
function startTutorial() {
  trackEvent('tutorial', 'start');
  void router.push({ name: routes.TutorialCreateOrganization });
}

function skipTutorial() {
  trackEvent('tutorial', 'skip');
  void router.push({ name: routes.Home });
}

// ---------------------------------------------------------------------------
// Step 1 — Organization
// ---------------------------------------------------------------------------
const enum OrgSection {
  CreateOrganization = 'create_organization',
  SelectExistingOrganization = 'select_existing_organization',
}

const orgId = ref<UUID | null>(null);
const selectedOrgId = ref<UUID | null>(null);
const orgSection = ref<OrgSection | null>(null);

const {
  data: rawOrganizations,
  isLoading: orgLoading,
  error: orgError,
  refetch: orgRefetch,
} = useOrganizationList();

const organizationOptions = computed(() => [
  { label: '', value: '' },
  ...(rawOrganizations.value ?? []).map((o) => ({ label: o.name, value: o.organization_id })),
]);

function goToStep2(organization_id: UUID) {
  orgId.value = organization_id;
  if (selectedOrgId.value) {
    trackEvent('tutorial', 'step-complete', 'organization');
    push.success({
      title: t('tutorial.organizationSelected'),
      message: t('tutorial.continueToApplication'),
      duration: 5000,
    });
    celebrate();
    void router.push({
      name: routes.TutorialCreateApplication,
      params: { organization_id: selectedOrgId.value },
    });
  } else if (orgId.value) {
    trackEvent('tutorial', 'step-complete', 'organization');
    push.success({
      title: t('tutorial.organizationCreated'),
      message: t('tutorial.continueToApplication'),
      duration: 5000,
    });
    celebrate();
    void router.push({
      name: routes.TutorialCreateApplication,
      params: { organization_id: orgId.value },
    });
  } else {
    push.error({
      title: t('tutorial.organizationIdRequired'),
      message: t('common.somethingWentWrong'),
      duration: 5000,
    });
  }
}

// ---------------------------------------------------------------------------
// Step 2 — Application
// ---------------------------------------------------------------------------
const enum AppSection {
  CreateApplication,
  SelectExistingApplication,
}

const paramOrgId = computed(() => route.params.organization_id as UUID);
const appId = ref<UUID | null>(null);
const selectedAppId = ref<UUID | null>(null);
const appSection = ref<AppSection | null>(null);

const appListOrgId = computed(() =>
  currentStep.value === 2 && paramOrgId.value ? paramOrgId.value : ('' as UUID)
);

const {
  data: rawApplications,
  isLoading: appLoading,
  error: appError,
  refetch: appRefetch,
} = useApplicationList(appListOrgId);

const applicationOptions = computed(() => {
  const apps = rawApplications.value ?? [];
  return [
    { label: '', value: '' },
    ...apps.map((a) => ({ label: a.name, value: a.application_id })),
  ];
});

// Auto-select "create" if no applications exist
watch(rawApplications, (apps) => {
  if ((apps ?? []).length <= 0 && appSection.value === null) {
    appSection.value = AppSection.CreateApplication;
  }
});

function goToStep3(application_id: UUID) {
  appId.value = application_id;
  if (paramOrgId.value && selectedAppId.value) {
    trackEvent('tutorial', 'step-complete', 'application');
    push.success({
      title: t('tutorial.applicationSelected'),
      message: t('tutorial.continueToEventType'),
      duration: 5000,
    });
    celebrate();
    void router.push({
      name: routes.TutorialCreateEventType,
      params: {
        organization_id: paramOrgId.value,
        application_id: selectedAppId.value,
      },
    });
  } else if (paramOrgId.value && appId.value) {
    trackEvent('tutorial', 'step-complete', 'application');
    push.success({
      title: t('tutorial.applicationCreated'),
      message: t('tutorial.continueToEventType'),
      duration: 5000,
    });
    celebrate();
    void router.push({
      name: routes.TutorialCreateEventType,
      params: {
        organization_id: paramOrgId.value,
        application_id: appId.value,
      },
    });
  } else {
    push.error({
      title: t('tutorial.orgAppIdRequired'),
      message: t('common.somethingWentWrong'),
      duration: 5000,
    });
  }
}

// ---------------------------------------------------------------------------
// Steps 3–5 shared state
// ---------------------------------------------------------------------------
const paramAppId = computed(() => route.params.application_id as UUID);

const step3Alert = ref<Alert>({ visible: false, type: 'alert', title: '', description: '' });
const step4Alert = ref<Alert>({ visible: false, type: 'alert', title: '', description: '' });
const step5Alert = ref<Alert>({ visible: false, type: 'alert', title: '', description: '' });
const step6Alert = ref<Alert>({ visible: false, type: 'alert', title: '', description: '' });

const step3Done = ref(false);
const step4Done = ref(false);
const step5Done = ref(false);

function validateParamsForStep(alertRef: typeof step3Alert) {
  if (!paramOrgId.value || !paramAppId.value) {
    alertRef.value.visible = true;
    alertRef.value.type = 'warning';
    alertRef.value.title = t('tutorial.orgAppIdRequired');
    alertRef.value.description = t('tutorial.somethingWentWrong');
  }
}

// Reset alerts on step change
watch(
  currentStep,
  (step) => {
    if (step === 3) {
      step3Alert.value.visible = false;
      step3Done.value = false;
      validateParamsForStep(step3Alert);
    }
    if (step === 4) {
      step4Alert.value.visible = false;
      step4Done.value = false;
      validateParamsForStep(step4Alert);
    }
    if (step === 5) {
      step5Alert.value.visible = false;
      step5Done.value = false;
      validateParamsForStep(step5Alert);
    }
    if (step === 6) {
      step6Alert.value.visible = false;
      validateParamsForStep(step6Alert);
      celebrate(100);
      trackEvent('tutorial', 'complete');
    }

    focusFirstElement();
  },
  { immediate: true }
);

// ---------------------------------------------------------------------------
// Step 3 — Event Type
// ---------------------------------------------------------------------------
function goToStep4() {
  if (paramOrgId.value && paramAppId.value) {
    trackEvent('tutorial', 'step-complete', 'event-type');
    step3Done.value = true;
    push.success({
      title: t('tutorial.step3.eventTypeCreated'),
      message: t('tutorial.step3.canCreateSubscription'),
      duration: 5000,
    });
    celebrate();
    void router.push({
      name: routes.TutorialCreateSubscription,
      params: {
        organization_id: paramOrgId.value,
        application_id: paramAppId.value,
      },
    });
  } else {
    push.error({
      title: t('tutorial.orgAppIdRequired'),
      message: t('tutorial.somethingWentWrong'),
      duration: 5000,
    });
  }
}

// ---------------------------------------------------------------------------
// Step 4 — Subscription
// ---------------------------------------------------------------------------
function goToStep5() {
  if (paramOrgId.value && paramAppId.value) {
    trackEvent('tutorial', 'step-complete', 'subscription');
    step4Done.value = true;
    push.success({
      title: t('tutorial.step4.subscriptionCreated'),
      message: t('tutorial.step4.canSendEvent'),
      duration: 5000,
    });
    celebrate();
    void router.push({
      name: routes.TutorialSendEvent,
      params: {
        organization_id: paramOrgId.value,
        application_id: paramAppId.value,
      },
    });
  } else {
    push.error({
      title: t('tutorial.orgAppIdRequired'),
      message: t('tutorial.somethingWentWrong'),
      duration: 5000,
    });
  }
}

// ---------------------------------------------------------------------------
// Step 5 — Send Event
// ---------------------------------------------------------------------------
function goToStep6() {
  trackEvent('tutorial', 'step-complete', 'send-event');
  push.success({
    title: t('tutorial.step5.eventSent'),
    message: t('tutorial.step5.eventSentMessage'),
    duration: 5000,
  });
  celebrate();
  void router.push({
    name: routes.TutorialSuccess,
    params: {
      organization_id: paramOrgId.value,
      application_id: paramAppId.value,
    },
  });
}

// ---------------------------------------------------------------------------
// Step 6 — Success
// ---------------------------------------------------------------------------
function goToApplicationDashboard() {
  void router.push({
    name: routes.ApplicationsDashboard,
    params: { organization_id: paramOrgId.value },
  });
}

// ---------------------------------------------------------------------------
// Skip helpers (per-step)
// ---------------------------------------------------------------------------
function skipToHome() {
  void router.push({ name: routes.Home });
}

function skipToOrgDashboard() {
  void router.push({
    name: routes.OrganizationsDashboard,
    params: { organization_id: paramOrgId.value },
  });
}

function skipToAppDashboard() {
  void router.push({
    name: routes.ApplicationsDashboard,
    params: {
      organization_id: paramOrgId.value,
      application_id: paramAppId.value,
    },
  });
}

// ---------------------------------------------------------------------------
// Keyboard: Escape to dismiss
// ---------------------------------------------------------------------------
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (currentStep.value === 0) {
      skipTutorial();
    } else if (currentStep.value <= 1) {
      skipToHome();
    } else if (currentStep.value === 2) {
      skipToOrgDashboard();
    } else {
      skipToAppDashboard();
    }
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown);
  focusFirstElement();
});

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown);
});

// ---------------------------------------------------------------------------
// Overlay click to dismiss
// ---------------------------------------------------------------------------
const overlayRef = ref<HTMLElement | null>(null);
const modalRef = ref<HTMLElement | null>(null);

function handleOverlayClick(e: MouseEvent) {
  if (e.target === overlayRef.value) {
    if (currentStep.value === 0) {
      skipTutorial();
    } else if (currentStep.value <= 1) {
      skipToHome();
    } else if (currentStep.value === 2) {
      skipToOrgDashboard();
    } else {
      skipToAppDashboard();
    }
  }
}

// ---------------------------------------------------------------------------
// Focus trap
// ---------------------------------------------------------------------------
const FOCUSABLE_SELECTOR =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

function handleFocusTrap(e: KeyboardEvent) {
  if (e.key !== 'Tab' || !modalRef.value) return;

  const focusable = Array.from(modalRef.value.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
  if (focusable.length === 0) return;

  const first = focusable[0];
  const last = focusable[focusable.length - 1];

  if (e.shiftKey) {
    if (document.activeElement === first) {
      e.preventDefault();
      last.focus();
    }
  } else {
    if (document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }
}

function focusFirstElement() {
  void nextTick(() => {
    if (!modalRef.value) return;
    const first = modalRef.value.querySelector<HTMLElement>(FOCUSABLE_SELECTOR);
    first?.focus();
  });
}
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog-overlay" appear>
      <div v-show="true" ref="overlayRef" class="wizard-overlay" @click="handleOverlayClick">
        <Transition name="dialog" appear>
          <div
            v-show="true"
            ref="modalRef"
            class="wizard-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="wizard-step-title"
            @keydown="handleFocusTrap"
          >
            <!-- ============================================================ -->
            <!-- STEP 0 — Intro                                               -->
            <!-- ============================================================ -->
            <template v-if="currentStep === 0">
              <div class="wizard-modal__header">
                <Hook0Stack direction="row" align="center" gap="sm">
                  <Hook0IconBadge variant="primary" size="md">
                    <Rocket :size="18" aria-hidden="true" />
                  </Hook0IconBadge>
                  <span id="wizard-step-title" class="wizard-modal__title">{{
                    t('tutorial.intro.title')
                  }}</span>
                </Hook0Stack>
                <button
                  class="wizard-modal__close"
                  type="button"
                  :aria-label="t('tutorial.intro.skipButton')"
                  @click="skipTutorial"
                >
                  <X :size="18" aria-hidden="true" />
                </button>
              </div>

              <div class="wizard-modal__content" data-test="tutorial-card">
                <Hook0Stack direction="column" gap="lg">
                  <span class="wizard-modal__subtitle">{{ t('tutorial.intro.subtitle') }}</span>

                  <Hook0Illustration
                    variant="tutorial"
                    size="hero"
                    :alt="t('tutorial.intro.illustrationAlt')"
                  />

                  <Hook0Stack direction="column" gap="md" data-test="tutorial-header">
                    {{ t('tutorial.intro.inThisTutorial') }}
                  </Hook0Stack>

                  <ul class="tutorial-steps">
                    <li
                      v-for="step in tutorialSteps"
                      :key="step.label"
                      class="tutorial-steps__item"
                    >
                      <span class="tutorial-steps__icon">
                        <component :is="step.icon" :size="16" aria-hidden="true" />
                      </span>
                      <span class="tutorial-steps__label">{{ t(step.label) }}</span>
                    </li>
                  </ul>

                  <Hook0Stack direction="column" gap="none">
                    <i18n-t keypath="tutorial.intro.skipNote" tag="span">
                      <template #link>
                        <Hook0Button
                          variant="link"
                          target="_blank"
                          href="https://documentation.hook0.com/docs/getting-started"
                          >{{ t('tutorial.intro.programmatically') }}</Hook0Button
                        >
                      </template>
                    </i18n-t>
                  </Hook0Stack>
                </Hook0Stack>
              </div>

              <div class="wizard-modal__footer">
                <Hook0Button
                  variant="secondary"
                  type="button"
                  data-test="tutorial-skip-button"
                  @click="skipTutorial"
                >
                  <X :size="16" aria-hidden="true" />
                  {{ t('tutorial.intro.skipButton') }}
                </Hook0Button>
                <Hook0Button
                  variant="primary"
                  type="button"
                  data-test="tutorial-start-button"
                  @click="startTutorial"
                >
                  {{ t('tutorial.intro.startStep1') }}
                  <ArrowRight :size="16" aria-hidden="true" />
                </Hook0Button>
              </div>
            </template>
            <template v-else-if="currentStep === 1">
              <!-- STEP 1 — Organization -->
              <div class="wizard-modal__header">
                <Hook0Stack direction="row" align="center" gap="sm">
                  <Hook0Badge display="step" variant="primary">1</Hook0Badge>
                  <span id="wizard-step-title" class="wizard-modal__title">{{
                    t('tutorial.step1Title')
                  }}</span>
                </Hook0Stack>
                <button
                  class="wizard-modal__close"
                  type="button"
                  :aria-label="t('tutorial.skip')"
                  @click="skipToHome"
                >
                  <X :size="18" aria-hidden="true" />
                </button>
              </div>

              <div class="wizard-modal__content">
                <Hook0Stack direction="column" gap="lg">
                  <span class="wizard-modal__subtitle">{{ t('tutorial.step1Description') }}</span>

                  <TutorialStepProgress :steps="PROGRESS_STEPS" :current="0" />

                  <Hook0Card v-if="!orgId">
                    <Hook0CardHeader>
                      <template #header>
                        <Hook0Stack direction="row" align="center" gap="sm">
                          <Building2 :size="18" aria-hidden="true" />
                          <Hook0Stack direction="row" align="center" gap="none">
                            {{ t('tutorial.chooseOrganization') }}
                          </Hook0Stack>
                        </Hook0Stack>
                      </template>
                    </Hook0CardHeader>
                    <Hook0CardContent>
                      <Hook0Stack layout="grid" gap="md" grid-size="compact">
                        <label
                          class="selectable-card"
                          :class="{
                            'selectable-card--selected':
                              orgSection === OrgSection.CreateOrganization,
                          }"
                          data-test="tutorial-create-org-option"
                          @click="orgSection = OrgSection.CreateOrganization"
                        >
                          <input
                            type="radio"
                            name="organization_selection"
                            :checked="orgSection === OrgSection.CreateOrganization"
                            class="selectable-card__radio"
                          />
                          <span
                            class="selectable-card__icon"
                            :class="{
                              'selectable-card__icon--selected':
                                orgSection === OrgSection.CreateOrganization,
                            }"
                          >
                            <Plus :size="18" aria-hidden="true" />
                          </span>
                          <span class="selectable-card__label">
                            {{ t('tutorial.createNewOrganization') }}
                          </span>
                          <span class="selectable-card__indicator">
                            <Check
                              v-if="orgSection === OrgSection.CreateOrganization"
                              :size="16"
                              aria-hidden="true"
                            />
                          </span>
                        </label>
                        <label
                          class="selectable-card"
                          :class="{
                            'selectable-card--selected':
                              orgSection === OrgSection.SelectExistingOrganization,
                          }"
                          data-test="tutorial-select-org-option"
                          @click="orgSection = OrgSection.SelectExistingOrganization"
                        >
                          <input
                            type="radio"
                            name="organization_selection"
                            :checked="orgSection === OrgSection.SelectExistingOrganization"
                            class="selectable-card__radio"
                          />
                          <span
                            class="selectable-card__icon"
                            :class="{
                              'selectable-card__icon--selected':
                                orgSection === OrgSection.SelectExistingOrganization,
                            }"
                          >
                            <List :size="18" aria-hidden="true" />
                          </span>
                          <span class="selectable-card__label">
                            {{ t('tutorial.selectExistingOrganization') }}
                          </span>
                          <span class="selectable-card__indicator">
                            <Check
                              v-if="orgSection === OrgSection.SelectExistingOrganization"
                              :size="16"
                              aria-hidden="true"
                            />
                          </span>
                        </label>
                      </Hook0Stack>
                    </Hook0CardContent>
                  </Hook0Card>

                  <OrganizationsEdit
                    v-if="!orgId && orgSection === OrgSection.CreateOrganization"
                    :tutorial-mode="true"
                    @tutorial-organization-created="goToStep2($event)"
                  />

                  <!-- Select existing organization -->
                  <template v-if="orgSection === OrgSection.SelectExistingOrganization">
                    <!-- Loading -->
                    <Hook0Stack v-if="orgLoading" direction="column" gap="md">
                      <Hook0Skeleton size="hero" />
                      <Hook0Skeleton size="heading" />
                    </Hook0Stack>

                    <!-- Error -->
                    <Hook0ErrorCard v-else-if="orgError" :error="orgError" @retry="orgRefetch()" />

                    <!-- Organization select -->
                    <Hook0Card v-else>
                      <Hook0CardContent>
                        <Hook0CardContentLine type="full-width">
                          <template #label>{{ t('tutorial.selectOrganization') }}</template>
                          <template #content>
                            <Hook0Select v-model="selectedOrgId" :options="organizationOptions" />
                          </template>
                        </Hook0CardContentLine>
                      </Hook0CardContent>
                    </Hook0Card>
                  </template>
                </Hook0Stack>
              </div>

              <div class="wizard-modal__footer">
                <Hook0Button variant="secondary" type="button" @click="skipToHome">
                  <X :size="16" aria-hidden="true" />
                  {{ t('tutorial.skip') }}
                </Hook0Button>
                <Hook0Button
                  v-if="orgId || selectedOrgId"
                  variant="primary"
                  type="button"
                  @click="goToStep2((orgId ?? selectedOrgId)!)"
                >
                  {{ t('tutorial.continueStep2') }}
                  <ArrowRight :size="16" aria-hidden="true" />
                </Hook0Button>
              </div>
            </template>
            <template v-else-if="currentStep === 2">
              <!-- STEP 2 — Application -->
              <div class="wizard-modal__header">
                <Hook0Stack direction="row" align="center" gap="sm">
                  <Hook0Badge display="step" variant="primary">2</Hook0Badge>
                  <span id="wizard-step-title" class="wizard-modal__title">{{
                    t('tutorial.step2Title')
                  }}</span>
                </Hook0Stack>
                <button
                  class="wizard-modal__close"
                  type="button"
                  :aria-label="t('tutorial.skip')"
                  @click="skipToOrgDashboard"
                >
                  <X :size="18" aria-hidden="true" />
                </button>
              </div>

              <div class="wizard-modal__content">
                <!-- Loading -->
                <Hook0Stack v-if="appLoading" direction="column" gap="md">
                  <Hook0Skeleton size="hero" />
                  <Hook0Skeleton size="heading" />
                  <Hook0Skeleton size="heading" />
                </Hook0Stack>

                <!-- Error -->
                <Hook0ErrorCard v-else-if="appError" :error="appError" @retry="appRefetch()" />

                <!-- Data loaded -->
                <Hook0Stack v-else direction="column" gap="lg">
                  <span class="wizard-modal__subtitle">{{ t('tutorial.step2Description') }}</span>

                  <TutorialStepProgress :steps="PROGRESS_STEPS" :current="1" />

                  <Hook0Card v-if="paramOrgId && !appId && applicationOptions.length > 1">
                    <Hook0CardHeader>
                      <template #header>
                        <Hook0Stack direction="row" align="center" gap="sm">
                          <AppWindow :size="18" aria-hidden="true" />
                          <Hook0Stack direction="row" align="center" gap="none">
                            {{ t('tutorial.chooseApplication') }}
                          </Hook0Stack>
                        </Hook0Stack>
                      </template>
                    </Hook0CardHeader>
                    <Hook0CardContent>
                      <Hook0Stack layout="grid" gap="md" grid-size="compact">
                        <label
                          class="selectable-card"
                          :class="{
                            'selectable-card--selected':
                              appSection === AppSection.CreateApplication,
                          }"
                          @click="appSection = AppSection.CreateApplication"
                        >
                          <input
                            type="radio"
                            name="application_selection"
                            :checked="appSection === AppSection.CreateApplication"
                            class="selectable-card__radio"
                          />
                          <span
                            class="selectable-card__icon"
                            :class="{
                              'selectable-card__icon--selected':
                                appSection === AppSection.CreateApplication,
                            }"
                          >
                            <Plus :size="18" aria-hidden="true" />
                          </span>
                          <span class="selectable-card__label">
                            {{ t('tutorial.createNewApplication') }}
                          </span>
                          <span class="selectable-card__indicator">
                            <Check
                              v-if="appSection === AppSection.CreateApplication"
                              :size="16"
                              aria-hidden="true"
                            />
                          </span>
                        </label>
                        <label
                          class="selectable-card"
                          :class="{
                            'selectable-card--selected':
                              appSection === AppSection.SelectExistingApplication,
                          }"
                          @click="appSection = AppSection.SelectExistingApplication"
                        >
                          <input
                            type="radio"
                            name="application_selection"
                            :checked="appSection === AppSection.SelectExistingApplication"
                            class="selectable-card__radio"
                          />
                          <span
                            class="selectable-card__icon"
                            :class="{
                              'selectable-card__icon--selected':
                                appSection === AppSection.SelectExistingApplication,
                            }"
                          >
                            <List :size="18" aria-hidden="true" />
                          </span>
                          <span class="selectable-card__label">
                            {{ t('tutorial.selectExistingApplication') }}
                          </span>
                          <span class="selectable-card__indicator">
                            <Check
                              v-if="appSection === AppSection.SelectExistingApplication"
                              :size="16"
                              aria-hidden="true"
                            />
                          </span>
                        </label>
                      </Hook0Stack>
                    </Hook0CardContent>
                  </Hook0Card>

                  <ApplicationsEdit
                    v-if="paramOrgId && appSection === AppSection.CreateApplication"
                    :tutorial-mode="true"
                    @tutorial-application-created="goToStep3($event)"
                  />

                  <!-- Select existing application -->
                  <template
                    v-if="paramOrgId && appSection === AppSection.SelectExistingApplication"
                  >
                    <Hook0Card>
                      <Hook0CardContent>
                        <Hook0CardContentLine type="full-width">
                          <template #label>
                            {{ t('tutorial.selectApplication') }}
                          </template>
                          <template #content>
                            <Hook0Select v-model="selectedAppId" :options="applicationOptions" />
                          </template>
                        </Hook0CardContentLine>
                      </Hook0CardContent>
                    </Hook0Card>
                  </template>
                </Hook0Stack>
              </div>

              <div class="wizard-modal__footer">
                <Hook0Button variant="secondary" type="button" @click="skipToOrgDashboard">
                  <X :size="16" aria-hidden="true" />
                  {{ t('tutorial.skip') }}
                </Hook0Button>
                <Hook0Button
                  v-if="paramOrgId && (appId || selectedAppId)"
                  variant="primary"
                  type="button"
                  @click="goToStep3((appId ?? selectedAppId)!)"
                >
                  {{ t('tutorial.continueStep3') }}
                  <ArrowRight :size="16" aria-hidden="true" />
                </Hook0Button>
              </div>
            </template>
            <template v-else-if="currentStep === 3">
              <!-- STEP 3 — Event Type -->
              <div class="wizard-modal__header">
                <Hook0Stack direction="row" align="center" gap="sm">
                  <Hook0Badge display="step" variant="primary">3</Hook0Badge>
                  <span id="wizard-step-title" class="wizard-modal__title">{{
                    t('tutorial.step3.title')
                  }}</span>
                </Hook0Stack>
                <button
                  class="wizard-modal__close"
                  type="button"
                  :aria-label="t('tutorial.step3.skip')"
                  @click="skipToAppDashboard"
                >
                  <X :size="18" aria-hidden="true" />
                </button>
              </div>

              <div class="wizard-modal__content">
                <template v-if="step3Alert.visible">
                  <Hook0Alert
                    :type="step3Alert.type"
                    :title="step3Alert.title"
                    :description="step3Alert.description"
                  />
                  <Hook0Button variant="secondary" type="button" @click="router.back()">
                    {{ t('tutorial.close') }}
                  </Hook0Button>
                </template>

                <Hook0Stack v-else direction="column" gap="lg">
                  <span class="wizard-modal__subtitle">{{ t('tutorial.step3.subtitle') }}</span>

                  <TutorialStepProgress :steps="PROGRESS_STEPS" :current="2" />

                  <Hook0Stack
                    v-if="paramOrgId && paramAppId && !step3Done"
                    direction="column"
                    gap="md"
                  >
                    <Hook0Stack direction="row" align="center" gap="sm">
                      <Hook0IconBadge variant="primary">
                        <FolderTree :size="18" aria-hidden="true" />
                      </Hook0IconBadge>
                      <Hook0Stack direction="row" align="center" gap="none">
                        {{ t('tutorial.step3.title') }}
                      </Hook0Stack>
                    </Hook0Stack>
                    <EventTypesNew :tutorial-mode="true" @tutorial-event-type-created="goToStep4" />
                  </Hook0Stack>
                </Hook0Stack>
              </div>

              <div class="wizard-modal__footer">
                <Hook0Button variant="secondary" type="button" @click="skipToAppDashboard">
                  <X :size="16" aria-hidden="true" />
                  {{ t('tutorial.step3.skip') }}
                </Hook0Button>
                <Hook0Button
                  v-if="paramOrgId && paramAppId && step3Done"
                  variant="primary"
                  type="button"
                  @click="goToStep4"
                >
                  {{ t('tutorial.step3.continueStep4') }}
                  <ArrowRight :size="16" aria-hidden="true" />
                </Hook0Button>
              </div>
            </template>
            <template v-else-if="currentStep === 4">
              <!-- STEP 4 — Subscription -->
              <div class="wizard-modal__header">
                <Hook0Stack direction="row" align="center" gap="sm">
                  <Hook0Badge display="step" variant="primary">4</Hook0Badge>
                  <span id="wizard-step-title" class="wizard-modal__title">{{
                    t('tutorial.step4.title')
                  }}</span>
                </Hook0Stack>
                <button
                  class="wizard-modal__close"
                  type="button"
                  :aria-label="t('tutorial.step4.skip')"
                  @click="skipToAppDashboard"
                >
                  <X :size="18" aria-hidden="true" />
                </button>
              </div>

              <div class="wizard-modal__content">
                <template v-if="step4Alert.visible">
                  <Hook0Alert
                    :type="step4Alert.type"
                    :title="step4Alert.title"
                    :description="step4Alert.description"
                  />
                  <Hook0Button variant="secondary" type="button" @click="router.back()">
                    {{ t('tutorial.close') }}
                  </Hook0Button>
                </template>

                <Hook0Stack v-else direction="column" gap="lg">
                  <span class="wizard-modal__subtitle">{{ t('tutorial.step4.subtitle') }}</span>

                  <TutorialStepProgress :steps="PROGRESS_STEPS" :current="3" />

                  <Hook0Stack
                    v-if="paramOrgId && paramAppId && !step4Done"
                    direction="column"
                    gap="md"
                  >
                    <Hook0Stack direction="row" align="center" gap="sm">
                      <Hook0IconBadge variant="primary">
                        <Link :size="18" aria-hidden="true" />
                      </Hook0IconBadge>
                      <Hook0Stack direction="row" align="center" gap="none">
                        {{ t('tutorial.step4.title') }}
                      </Hook0Stack>
                    </Hook0Stack>
                    <SubscriptionsEdit
                      :tutorial-mode="true"
                      @tutorial-subscription-created="goToStep5"
                    />
                  </Hook0Stack>
                </Hook0Stack>
              </div>

              <div class="wizard-modal__footer">
                <Hook0Button variant="secondary" type="button" @click="skipToAppDashboard">
                  <X :size="16" aria-hidden="true" />
                  {{ t('tutorial.step4.skip') }}
                </Hook0Button>
                <Hook0Button
                  v-if="paramOrgId && paramAppId && step4Done"
                  variant="primary"
                  type="button"
                  @click="goToStep5"
                >
                  {{ t('tutorial.step4.continueStep5') }}
                  <ArrowRight :size="16" aria-hidden="true" />
                </Hook0Button>
              </div>
            </template>
            <template v-else-if="currentStep === 5">
              <!-- STEP 5 — Send Event -->
              <div class="wizard-modal__header">
                <Hook0Stack direction="row" align="center" gap="sm">
                  <Hook0Badge display="step" variant="primary">5</Hook0Badge>
                  <span id="wizard-step-title" class="wizard-modal__title">{{
                    t('tutorial.step5.title')
                  }}</span>
                </Hook0Stack>
                <button
                  class="wizard-modal__close"
                  type="button"
                  :aria-label="t('tutorial.step5.skip')"
                  @click="skipToAppDashboard"
                >
                  <X :size="18" aria-hidden="true" />
                </button>
              </div>

              <div class="wizard-modal__content">
                <template v-if="step5Alert.visible">
                  <Hook0Alert
                    :type="step5Alert.type"
                    :title="step5Alert.title"
                    :description="step5Alert.description"
                  />
                  <Hook0Button variant="secondary" type="button" @click="router.back()">
                    {{ t('tutorial.close') }}
                  </Hook0Button>
                </template>

                <Hook0Stack v-else direction="column" gap="lg">
                  <span class="wizard-modal__subtitle">{{ t('tutorial.step5.subtitle') }}</span>

                  <TutorialStepProgress :steps="PROGRESS_STEPS" :current="4" />

                  <Hook0Stack
                    v-if="paramOrgId && paramAppId && !step5Done"
                    direction="column"
                    gap="md"
                  >
                    <Hook0Stack direction="row" align="center" gap="sm">
                      <Hook0IconBadge variant="primary">
                        <FileText :size="18" aria-hidden="true" />
                      </Hook0IconBadge>
                      <Hook0Stack direction="row" align="center" gap="none">
                        {{ t('tutorial.step5.title') }}
                      </Hook0Stack>
                    </Hook0Stack>
                    <EventsList :tutorial-mode="true" @tutorial-event-sent="goToStep6" />
                  </Hook0Stack>
                </Hook0Stack>
              </div>

              <div class="wizard-modal__footer">
                <Hook0Button variant="secondary" type="button" @click="skipToAppDashboard">
                  <X :size="16" aria-hidden="true" />
                  {{ t('tutorial.step5.skip') }}
                </Hook0Button>
                <Hook0Button v-if="step5Done" variant="primary" type="button" @click="goToStep6">
                  {{ t('tutorial.step5.backToApplication') }}
                  <ArrowRight :size="16" aria-hidden="true" />
                </Hook0Button>
              </div>
            </template>
            <template v-else-if="currentStep === 6">
              <!-- STEP 6 — Success -->
              <div class="wizard-modal__header">
                <Hook0Stack direction="row" align="center" gap="sm">
                  <Hook0IconBadge variant="success" size="lg">
                    <PartyPopper :size="20" aria-hidden="true" />
                  </Hook0IconBadge>
                  <span id="wizard-step-title" class="wizard-modal__title">{{
                    t('tutorial.congrats.title')
                  }}</span>
                </Hook0Stack>
                <button
                  class="wizard-modal__close"
                  type="button"
                  :aria-label="t('tutorial.close')"
                  @click="goToApplicationDashboard"
                >
                  <X :size="18" aria-hidden="true" />
                </button>
              </div>

              <div class="wizard-modal__content">
                <template v-if="step6Alert.visible">
                  <Hook0Alert
                    :type="step6Alert.type"
                    :title="step6Alert.title"
                    :description="step6Alert.description"
                  />
                  <Hook0Button variant="secondary" type="button" @click="router.back()">
                    {{ t('tutorial.close') }}
                  </Hook0Button>
                </template>

                <Hook0Stack v-else direction="column" gap="lg">
                  <i18n-t keypath="tutorial.congrats.subtitle" tag="span">
                    <template #discord>
                      <Hook0Button
                        variant="link"
                        href="https://discord.com/invite/hook0"
                        target="_blank"
                      >
                        <MessageSquare :size="14" aria-hidden="true" />
                        Discord
                      </Hook0Button>
                    </template>
                    <template #github>
                      <Hook0Button
                        variant="link"
                        href="https://github.com/hook0/hook0"
                        target="_blank"
                      >
                        <Github :size="14" aria-hidden="true" />
                        GitHub
                      </Hook0Button>
                    </template>
                  </i18n-t>

                  <i18n-t keypath="tutorial.congrats.feedback" tag="span">
                    <template #discussions>
                      <Hook0Button
                        variant="link"
                        href="https://documentation.hook0.com/discuss"
                        target="_blank"
                      >
                        <BookOpen :size="14" aria-hidden="true" />
                        {{ t('tutorial.congrats.discussions') }}
                      </Hook0Button>
                    </template>
                    <template #changelog>
                      <Hook0Button
                        variant="link"
                        href="https://documentation.hook0.com/changelog"
                        target="_blank"
                      >
                        <Newspaper :size="14" aria-hidden="true" />
                        {{ t('tutorial.congrats.changelog') }}
                      </Hook0Button>
                    </template>
                    <template #documentation>
                      <Hook0Button
                        variant="link"
                        href="https://documentation.hook0.com/docs/events"
                        target="_blank"
                      >
                        <BookOpen :size="14" aria-hidden="true" />
                        {{ t('tutorial.congrats.documentation') }}
                      </Hook0Button>
                    </template>
                  </i18n-t>
                </Hook0Stack>
              </div>

              <div class="wizard-modal__footer">
                <Hook0Button variant="primary" type="button" @click="goToApplicationDashboard">
                  {{ t('tutorial.congrats.goToDashboard') }}
                  <ArrowRight :size="16" aria-hidden="true" />
                </Hook0Button>
              </div>
            </template>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.wizard-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-overlay, rgba(0, 0, 0, 0.5));
  backdrop-filter: blur(4px);
  padding: 1rem;
}

.wizard-modal {
  background-color: var(--color-bg-primary);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-xl);
  max-width: 42rem;
  width: 100%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

.wizard-modal__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.wizard-modal__title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.wizard-modal__subtitle {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.wizard-modal__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 2.75rem;
  height: 2.75rem;
  border: none;
  background: none;
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
  flex-shrink: 0;
}

.wizard-modal__close:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.wizard-modal__close:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.wizard-modal__content {
  padding: 1.5rem;
  overflow-y: auto;
  overscroll-behavior: contain;
  flex: 1;
}

.wizard-modal__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

/* Intro step list */
.tutorial-steps {
  list-style: none;
  padding: 0;
  margin: 0;
}

.tutorial-steps__item {
  display: flex;
  align-items: center;
  padding: 0.5rem 0.75rem;
  font-size: 0.875rem;
  border-radius: var(--radius-md);
  transition: background-color 0.15s ease;
}

.tutorial-steps__icon {
  flex-shrink: 0;
  width: 1.75rem;
  height: 1.75rem;
  margin-right: 0.625rem;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
}

.tutorial-steps__label {
  color: var(--color-text-primary);
}

/* Selectable cards (org/app step radio cards) */
.selectable-card {
  display: flex;
  align-items: center;
  padding: 1rem 1.25rem;
  border: 2px solid var(--color-border);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-primary);
  cursor: pointer;
  transition:
    border-color 0.15s ease,
    background-color 0.15s ease;
  gap: 0.75rem;
}

.selectable-card:hover {
  border-color: var(--color-border-strong);
  background-color: var(--color-bg-secondary);
}

.selectable-card:focus-within {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.selectable-card--selected {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.selectable-card--selected:hover {
  border-color: var(--color-primary);
  background-color: var(--color-primary-light);
}

.selectable-card__radio {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.selectable-card__icon {
  flex-shrink: 0;
  width: 2.5rem;
  height: 2.5rem;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
}

.selectable-card__icon--selected {
  background-color: var(--color-primary);
  color: var(--color-bg-primary);
}

.selectable-card__label {
  flex: 1;
  min-width: 0;
  font-size: 0.875rem;
  color: var(--color-text-primary);
}

.selectable-card__indicator {
  flex-shrink: 0;
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-left: auto;
  color: var(--color-primary);
}

@media (prefers-reduced-motion: reduce) {
  .wizard-modal__close,
  .selectable-card,
  .step-progress__circle {
    transition: none;
  }

  .wizard-overlay {
    backdrop-filter: none;
  }
}
</style>
