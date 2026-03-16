<script setup lang="ts">
import {
  computed,
  defineAsyncComponent,
  markRaw,
  nextTick,
  ref,
  onMounted,
  onUnmounted,
} from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { push } from 'notivue';

import { routes } from '@/routes';
import type { UUID } from '@/http';
import { useTracking } from '@/composables/useTracking';
import { useCelebration } from '@/composables/useCelebration';

import {
  Rocket,
  Building2,
  AppWindow,
  FolderTree,
  Link,
  FileText,
  PartyPopper,
} from 'lucide-vue-next';

const TutorialWizardStepIntro = defineAsyncComponent(
  () => import('@/pages/tutorial/TutorialWizardStepIntro.vue')
);
const TutorialWizardStepOrganization = defineAsyncComponent(
  () => import('@/pages/tutorial/TutorialWizardStepOrganization.vue')
);
const TutorialWizardStepApplication = defineAsyncComponent(
  () => import('@/pages/tutorial/TutorialWizardStepApplication.vue')
);
const TutorialWizardStepForm = defineAsyncComponent(
  () => import('@/pages/tutorial/TutorialWizardStepForm.vue')
);
const TutorialWizardStepSuccess = defineAsyncComponent(
  () => import('@/pages/tutorial/TutorialWizardStepSuccess.vue')
);

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

// Progress bar steps (steps 1-5 only, mapped to 0-based for the progress component)
const PROGRESS_STEPS = computed(() => STEPS.value.slice(1, 6));

// ---------------------------------------------------------------------------
// Route params
// ---------------------------------------------------------------------------
const paramOrgId = computed(() => route.params.organization_id as UUID);
const paramAppId = computed(() => route.params.application_id as UUID);

// ---------------------------------------------------------------------------
// Unified advance step
// ---------------------------------------------------------------------------
type AdvanceConfig = {
  trackLabel: string;
  toastTitle: string;
  toastMessage: string;
  routeName: string;
  params: Record<string, string>;
};

function advanceStep(config: AdvanceConfig) {
  trackEvent('tutorial', 'step-complete', config.trackLabel);
  push.success({
    title: config.toastTitle,
    message: config.toastMessage,
    duration: 5000,
  });
  celebrate();
  void router.push({ name: config.routeName, params: config.params });
}

function handleIntroStart() {
  trackEvent('tutorial', 'start');
  void router.push({ name: routes.TutorialCreateOrganization });
}

function handleOrgAdvance(organizationId: UUID) {
  advanceStep({
    trackLabel: 'organization',
    toastTitle: t('tutorial.organizationCreated'),
    toastMessage: t('tutorial.continueToApplication'),
    routeName: routes.TutorialCreateApplication,
    params: { organization_id: organizationId },
  });
}

function handleAppAdvance(applicationId: UUID) {
  if (!paramOrgId.value) {
    push.error({
      title: t('tutorial.orgAppIdRequired'),
      message: t('common.somethingWentWrong'),
      duration: 5000,
    });
    return;
  }
  advanceStep({
    trackLabel: 'application',
    toastTitle: t('tutorial.applicationCreated'),
    toastMessage: t('tutorial.continueToEventType'),
    routeName: routes.TutorialCreateEventType,
    params: {
      organization_id: paramOrgId.value,
      application_id: applicationId,
    },
  });
}

function handleFormAdvance() {
  if (!paramOrgId.value || !paramAppId.value) {
    push.error({
      title: t('tutorial.orgAppIdRequired'),
      message: t('tutorial.somethingWentWrong'),
      duration: 5000,
    });
    return;
  }

  const step = currentStep.value;
  if (step === 3) {
    advanceStep({
      trackLabel: 'event-type',
      toastTitle: t('tutorial.step3.eventTypeCreated'),
      toastMessage: t('tutorial.step3.canCreateSubscription'),
      routeName: routes.TutorialCreateSubscription,
      params: {
        organization_id: paramOrgId.value,
        application_id: paramAppId.value,
      },
    });
  } else if (step === 4) {
    advanceStep({
      trackLabel: 'subscription',
      toastTitle: t('tutorial.step4.subscriptionCreated'),
      toastMessage: t('tutorial.step4.canSendEvent'),
      routeName: routes.TutorialSendEvent,
      params: {
        organization_id: paramOrgId.value,
        application_id: paramAppId.value,
      },
    });
  } else if (step === 5) {
    advanceStep({
      trackLabel: 'send-event',
      toastTitle: t('tutorial.step5.eventSent'),
      toastMessage: t('tutorial.step5.eventSentMessage'),
      routeName: routes.TutorialSuccess,
      params: {
        organization_id: paramOrgId.value,
        application_id: paramAppId.value,
      },
    });
  }
}

// ---------------------------------------------------------------------------
// Unified dismiss
// ---------------------------------------------------------------------------
function dismiss() {
  const step = currentStep.value;
  if (step === 0) {
    trackEvent('tutorial', 'skip');
    void router.push({ name: routes.Home });
  } else if (step <= 1) {
    void router.push({ name: routes.Home });
  } else if (step === 2) {
    void router.push({
      name: routes.OrganizationsDashboard,
      params: { organization_id: paramOrgId.value },
    });
  } else if (step === 6) {
    void router.push({
      name: routes.ApplicationsDashboard,
      params: { organization_id: paramOrgId.value },
    });
  } else {
    void router.push({
      name: routes.ApplicationsDashboard,
      params: {
        organization_id: paramOrgId.value,
        application_id: paramAppId.value,
      },
    });
  }
}

// ---------------------------------------------------------------------------
// Keyboard: Escape to dismiss
// ---------------------------------------------------------------------------
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    dismiss();
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
    dismiss();
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
    <div ref="overlayRef" class="wizard-overlay" @click="handleOverlayClick">
      <div
        ref="modalRef"
        class="wizard-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="wizard-step-title"
        @keydown="handleFocusTrap"
      >
        <TutorialWizardStepIntro
          v-if="currentStep === 0"
          @start="handleIntroStart"
          @skip="dismiss"
        />

        <TutorialWizardStepOrganization
          v-else-if="currentStep === 1"
          :progress-steps="PROGRESS_STEPS"
          @advance="handleOrgAdvance"
          @skip="dismiss"
        />

        <TutorialWizardStepApplication
          v-else-if="currentStep === 2"
          :organization-id="paramOrgId"
          :progress-steps="PROGRESS_STEPS"
          @advance="handleAppAdvance"
          @skip="dismiss"
        />

        <TutorialWizardStepForm
          v-else-if="currentStep >= 3 && currentStep <= 5"
          :key="currentStep"
          :step="(currentStep as 3 | 4 | 5)"
          :organization-id="paramOrgId"
          :application-id="paramAppId"
          :progress-steps="PROGRESS_STEPS"
          @advance="handleFormAdvance"
          @skip="dismiss"
        />

        <TutorialWizardStepSuccess
          v-else-if="currentStep === 6"
          :organization-id="paramOrgId"
          :application-id="paramAppId"
          @dismiss="dismiss"
        />
      </div>
    </div>
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
  max-width: 60rem;
  width: 100%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

@media (max-width: 1024px) {
  .wizard-modal {
    max-width: 90vw;
  }
}

.wizard-modal :deep(.wizard-modal__header) {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.wizard-modal :deep(.wizard-modal__title) {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.wizard-modal :deep(.wizard-modal__subtitle) {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.wizard-modal :deep(.wizard-modal__close) {
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

.wizard-modal :deep(.wizard-modal__close:hover) {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.wizard-modal :deep(.wizard-modal__close:focus-visible) {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.wizard-modal :deep(.wizard-modal__content) {
  padding: 1.5rem;
  overflow-y: auto;
  overscroll-behavior: contain;
  flex: 1;
}

.wizard-modal :deep(.wizard-modal__footer) {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
}

@media (prefers-reduced-motion: reduce) {
  .wizard-modal :deep(.wizard-modal__close) {
    transition: none;
  }

  .wizard-overlay {
    backdrop-filter: none;
  }
}
</style>
