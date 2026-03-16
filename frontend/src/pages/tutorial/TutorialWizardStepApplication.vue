<script setup lang="ts">
import type { Component } from 'vue';
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { AppWindow } from 'lucide-vue-next';

import type { UUID } from '@/http';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';
import ApplicationsEdit from '@/pages/organizations/applications/ApplicationsEdit.vue';
import TutorialWizardEntityStep from './TutorialWizardEntityStep.vue';

type ProgressStep = { icon: Component; label: string };
const props = defineProps<{ organizationId: string; progressSteps: ProgressStep[] }>();
const emit = defineEmits<{ advance: [applicationId: UUID]; skip: [] }>();
const { t } = useI18n();

const appListOrgId = computed(() => props.organizationId || '');
const { data: rawApps, isLoading, error, refetch } = useApplicationList(appListOrgId);
const options = computed(() => [
  { label: t('tutorial.selectApplication'), value: '' },
  ...(rawApps.value ?? []).map((a) => ({ label: a.name, value: a.application_id })),
]);
</script>

<template>
  <TutorialWizardEntityStep
    :step-number="2"
    :step-title="t('tutorial.step2Title')"
    :step-description="t('tutorial.step2Description')"
    :choose-label="t('tutorial.chooseApplication')"
    :create-label="t('tutorial.createNewApplication')"
    :select-existing-label="t('tutorial.selectExistingApplication')"
    :select-label="t('tutorial.selectApplication')"
    :continue-label="t('tutorial.continueStep3')"
    :skip-label="t('tutorial.skip')"
    :progress-steps="progressSteps"
    :progress-current="1"
    :entity-icon="AppWindow"
    :entity-options="options"
    :entities-loading="isLoading"
    :entities-error="error"
    :require-options="true"
    selection-name="application_selection"
    @advance="emit('advance', $event)"
    @skip="emit('skip')"
    @retry="refetch()"
  >
    <template #edit="{ onCreated }">
      <ApplicationsEdit :tutorial-mode="true" @tutorial-application-created="onCreated($event)" />
    </template>
  </TutorialWizardEntityStep>
</template>
