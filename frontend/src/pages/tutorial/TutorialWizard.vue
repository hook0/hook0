<script setup lang="ts">
import { computed, defineAsyncComponent, markRaw, ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { toast } from 'vue-sonner';

import { routes } from '@/routes';
import type { UUID } from '@/http';
import { useTracking } from '@/composables/useTracking';
import { useCelebration } from '@/composables/useCelebration';
import { useFocusTrap } from '@/composables/useFocusTrap';

import {
  Rocket,
  Building2,
  AppWindow,
  FolderTree,
  Link,
  FileText,
  PartyPopper,
} from 'lucide-vue-next';

// Lazy-load each step component so only the active step is in the bundle
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

const TOAST_DURATION_MS = 5000;

// Step mapping from route name
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

// Step definitions for progress bar — icons are module-level (markRaw once)
const STEP_ICONS = [
  markRaw(Rocket),
  markRaw(Building2),
  markRaw(AppWindow),
  markRaw(FolderTree),
  markRaw(Link),
  markRaw(FileText),
  markRaw(PartyPopper),
] as const;

const STEPS = computed(() => [
  { icon: STEP_ICONS[0], label: t('tutorial.steps.intro') },
  { icon: STEP_ICONS[1], label: t('tutorial.steps.organization') },
  { icon: STEP_ICONS[2], label: t('tutorial.steps.application') },
  { icon: STEP_ICONS[3], label: t('tutorial.steps.eventType') },
  { icon: STEP_ICONS[4], label: t('tutorial.steps.subscription') },
  { icon: STEP_ICONS[5], label: t('tutorial.steps.sendEvent') },
  { icon: STEP_ICONS[6], label: t('tutorial.steps.success') },
]);

// Progress bar steps (steps 1-5 only, mapped to 0-based for the progress component)
const PROGRESS_STEPS = computed(() => STEPS.value.slice(1, 6));

// Route params
const paramOrgId = computed(() => route.params.organization_id as UUID);
const paramAppId = computed(() => route.params.application_id as UUID);

// Unified advance step
type AdvanceConfig = {
  trackLabel: string;
  toastTitle: string;
  toastMessage: string;
  routeName: string;
  params: Record<string, string>;
};

function advanceStep(config: AdvanceConfig) {
  trackEvent('tutorial', 'step-complete', config.trackLabel);
  toast.success(config.toastTitle, {
    description: config.toastMessage,
    duration: TOAST_DURATION_MS,
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
    toast.error(t('tutorial.orgAppIdRequired'), {
      description: t('common.somethingWentWrong'),
      duration: TOAST_DURATION_MS,
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

// Config map for form steps (3, 4, 5)
const FORM_STEP_CONFIG: Record<
  3 | 4 | 5,
  { trackLabel: string; toastTitle: string; toastMessage: string; routeName: string }
> = {
  3: {
    trackLabel: 'event-type',
    toastTitle: 'tutorial.step3.eventTypeCreated',
    toastMessage: 'tutorial.step3.canCreateSubscription',
    routeName: routes.TutorialCreateSubscription,
  },
  4: {
    trackLabel: 'subscription',
    toastTitle: 'tutorial.step4.subscriptionCreated',
    toastMessage: 'tutorial.step4.canSendEvent',
    routeName: routes.TutorialSendEvent,
  },
  5: {
    trackLabel: 'send-event',
    toastTitle: 'tutorial.step5.eventSent',
    toastMessage: 'tutorial.step5.eventSentMessage',
    routeName: routes.TutorialSuccess,
  },
};

function handleFormAdvance() {
  if (!paramOrgId.value || !paramAppId.value) {
    toast.error(t('tutorial.orgAppIdRequired'), {
      description: t('tutorial.somethingWentWrong'),
      duration: TOAST_DURATION_MS,
    });
    return;
  }

  const step = currentStep.value as 3 | 4 | 5;
  const config = FORM_STEP_CONFIG[step];
  if (!config) return;

  advanceStep({
    trackLabel: config.trackLabel,
    toastTitle: t(config.toastTitle),
    toastMessage: t(config.toastMessage),
    routeName: config.routeName,
    params: {
      organization_id: paramOrgId.value,
      application_id: paramAppId.value,
    },
  });
}

// Config map for dismiss routes
type DismissRouteEntry = {
  track?: boolean;
  routeName: string;
  params?: Record<string, string>;
};

function getDismissRoute(): DismissRouteEntry {
  switch (currentStep.value) {
    case 0:
      return { track: true, routeName: routes.Home };
    case 1:
      return { routeName: routes.Home };
    case 2:
      return {
        routeName: routes.OrganizationsDashboard,
        params: { organization_id: paramOrgId.value },
      };
    case 6:
      return {
        routeName: routes.ApplicationsDashboard,
        params: { organization_id: paramOrgId.value },
      };
    default:
      // Steps 3-5: dismiss to the app dashboard
      return {
        routeName: routes.ApplicationsDashboard,
        params: {
          organization_id: paramOrgId.value,
          application_id: paramAppId.value,
        },
      };
  }
}

function dismiss() {
  const entry = getDismissRoute();
  if (entry.track) {
    trackEvent('tutorial', 'skip');
  }
  void router.push({ name: entry.routeName, params: entry.params });
}

// Focus trap & keyboard handling
const overlayRef = ref<HTMLElement | null>(null);
const modalRef = ref<HTMLElement | null>(null);

const { activate, handleKeydown } = useFocusTrap(modalRef, { onEscape: dismiss });

onMounted(() => {
  activate();
});

// Overlay click to dismiss
function handleOverlayClick(e: MouseEvent) {
  if (e.target === overlayRef.value) {
    dismiss();
  }
}
</script>

<template>
  <Teleport to="body">
    <div ref="overlayRef" class="wizard-overlay" @click="handleOverlayClick">
      <div
        ref="modalRef"
        class="wizard-modal"
        data-test="tutorial-wizard-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="wizard-step-title"
        @keydown="handleKeydown"
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

@media (max-width: 767px) {
  .wizard-overlay {
    padding: 0;
    align-items: flex-end;
  }

  .wizard-modal {
    max-width: 100%;
    max-height: 90dvh;
    border-radius: var(--radius-xl) var(--radius-xl) 0 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .wizard-overlay {
    backdrop-filter: none;
  }
}
</style>
