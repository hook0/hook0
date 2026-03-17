<script setup lang="ts">
import { computed, defineAsyncComponent, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import TutorialStepProgress from '@/pages/tutorial/TutorialStepProgress.vue';
import WizardStepLayout from '@/pages/tutorial/WizardStepLayout.vue';
import type { ProgressStep } from '@/pages/tutorial/types';

import { FolderTree, Link, FileText } from 'lucide-vue-next';

const EventTypesNew = defineAsyncComponent(
  () => import('@/pages/organizations/applications/event_types/EventTypesNew.vue')
);
const SubscriptionsEdit = defineAsyncComponent(
  () => import('@/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue')
);
const EventsList = defineAsyncComponent(
  () => import('@/pages/organizations/applications/events/EventsList.vue')
);

type Props = {
  step: 3 | 4 | 5;
  organizationId: string;
  applicationId: string;
  progressSteps: ProgressStep[];
};

const props = defineProps<Props>();

const emit = defineEmits<{
  advance: [];
  skip: [];
}>();

const { t } = useI18n();

const alert = ref<Alert>({ visible: false, type: 'alert', title: '', description: '' });
const done = ref(false);

const STEP_CONFIG = computed(() => {
  const configs: Record<
    3 | 4 | 5,
    {
      badge: string;
      titleKey: string;
      subtitleKey: string;
      skipKey: string;
      continueKey: string;
      icon: typeof FolderTree;
      progressIndex: number;
    }
  > = {
    3: {
      badge: '3',
      titleKey: 'tutorial.step3.title',
      subtitleKey: 'tutorial.step3.subtitle',
      skipKey: 'tutorial.step3.skip',
      continueKey: 'tutorial.step3.continueStep4',
      icon: FolderTree,
      progressIndex: 2,
    },
    4: {
      badge: '4',
      titleKey: 'tutorial.step4.title',
      subtitleKey: 'tutorial.step4.subtitle',
      skipKey: 'tutorial.step4.skip',
      continueKey: 'tutorial.step4.continueStep5',
      icon: Link,
      progressIndex: 3,
    },
    5: {
      badge: '5',
      titleKey: 'tutorial.step5.title',
      subtitleKey: 'tutorial.step5.subtitle',
      skipKey: 'tutorial.step5.skip',
      continueKey: 'tutorial.step5.backToApplication',
      icon: FileText,
      progressIndex: 4,
    },
  };
  return configs[props.step];
});

function validateParams() {
  if (!props.organizationId || !props.applicationId) {
    alert.value.visible = true;
    alert.value.type = 'warning';
    alert.value.title = t('tutorial.orgAppIdRequired');
    alert.value.description = t('tutorial.somethingWentWrong');
  }
}

// Reset alert and done state when step changes
watch(
  () => props.step,
  () => {
    alert.value.visible = false;
    done.value = false;
    validateParams();
  },
  { immediate: true }
);

function handleFormDone() {
  done.value = true;
  emit('advance');
}
</script>

<template>
  <WizardStepLayout
    data-test="tutorial-step-form"
    :step-number="Number(STEP_CONFIG.badge)"
    :title="t(STEP_CONFIG.titleKey)"
    :continue-label="t(STEP_CONFIG.continueKey)"
    :continue-disabled="!(organizationId && applicationId && done)"
    @skip="emit('skip')"
    @continue="emit('advance')"
  >
    <template v-if="alert.visible">
      <Hook0Alert :type="alert.type" :title="alert.title" :description="alert.description" />
      <Hook0Button variant="secondary" type="button" @click="emit('skip')">
        {{ t('tutorial.close') }}
      </Hook0Button>
    </template>

    <Hook0Stack v-else direction="column" gap="lg">
      <span class="wizard-subtitle">{{ t(STEP_CONFIG.subtitleKey) }}</span>

      <TutorialStepProgress :steps="progressSteps" :current="STEP_CONFIG.progressIndex" />

      <Hook0Stack v-if="organizationId && applicationId && !done" direction="column" gap="md">
        <Hook0Stack direction="row" align="center" gap="sm">
          <Hook0IconBadge variant="primary">
            <component :is="STEP_CONFIG.icon" :size="18" aria-hidden="true" />
          </Hook0IconBadge>
          <Hook0Stack direction="row" align="center" gap="none">
            {{ t(STEP_CONFIG.titleKey) }}
          </Hook0Stack>
        </Hook0Stack>

        <EventTypesNew
          v-if="step === 3"
          :tutorial-mode="true"
          @tutorial-event-type-created="handleFormDone"
        />
        <SubscriptionsEdit
          v-else-if="step === 4"
          :tutorial-mode="true"
          @tutorial-subscription-created="handleFormDone"
        />
        <EventsList
          v-else-if="step === 5"
          :tutorial-mode="true"
          @tutorial-event-sent="handleFormDone"
        />
      </Hook0Stack>
    </Hook0Stack>
  </WizardStepLayout>
</template>

<style scoped>
.wizard-subtitle {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}
</style>
