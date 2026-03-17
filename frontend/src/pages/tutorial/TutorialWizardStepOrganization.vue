<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Building2 } from 'lucide-vue-next';

import type { UUID } from '@/http';
import type { ProgressStep } from '@/pages/tutorial/types';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import TutorialWizardEntityStep from './TutorialWizardEntityStep.vue';

defineProps<{ progressSteps: ProgressStep[] }>();
const emit = defineEmits<{ advance: [organizationId: UUID]; skip: [] }>();
const { t } = useI18n();

const { data: rawOrgs, isLoading, error, refetch } = useOrganizationList();
const options = computed(() => [
  { label: t('tutorial.selectOrganization'), value: '' },
  ...(rawOrgs.value ?? []).map((o) => ({ label: o.name, value: o.organization_id })),
]);
</script>

<template>
  <TutorialWizardEntityStep
    :step-number="1"
    :step-title="t('tutorial.step1Title')"
    :step-description="t('tutorial.step1Description')"
    :choose-label="t('tutorial.chooseOrganization')"
    :create-label="t('tutorial.createNewOrganization')"
    :select-existing-label="t('tutorial.selectExistingOrganization')"
    :select-label="t('tutorial.selectOrganization')"
    :continue-label="t('tutorial.continueStep2')"
    :skip-label="t('tutorial.skip')"
    :progress-steps="progressSteps"
    :progress-current="0"
    :entity-icon="Building2"
    :entity-options="options"
    :entities-loading="isLoading"
    :entities-error="error"
    selection-name="organization_selection"
    create-data-test="tutorial-create-org-option"
    select-data-test="tutorial-select-org-option"
    @advance="emit('advance', $event)"
    @skip="emit('skip')"
    @retry="refetch()"
  >
    <template #edit="{ onCreated }">
      <OrganizationsEdit :tutorial-mode="true" @tutorial-organization-created="onCreated($event)" />
    </template>
  </TutorialWizardEntityStep>
</template>
