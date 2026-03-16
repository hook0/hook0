<script setup lang="ts">
import type { Component } from 'vue';
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

import type { UUID } from '@/http';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import SelectableCard from '@/components/SelectableCard.vue';
import TutorialStepProgress from '@/pages/tutorial/TutorialStepProgress.vue';
import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';

import { Building2, Plus, List, ArrowRight, X } from 'lucide-vue-next';

type ProgressStep = {
  icon: Component;
  label: string;
};

type Props = {
  progressSteps: ProgressStep[];
};

defineProps<Props>();

const emit = defineEmits<{
  advance: [organizationId: UUID];
  skip: [];
}>();

const { t } = useI18n();

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
  { label: t('tutorial.selectOrganization'), value: '' },
  ...(rawOrganizations.value ?? []).map((o) => ({ label: o.name, value: o.organization_id })),
]);

// Auto-select "create" if no organizations exist
watch(rawOrganizations, (orgs) => {
  if ((orgs ?? []).length <= 0 && orgSection.value === null) {
    orgSection.value = OrgSection.CreateOrganization;
  }
});

function handleCreated(organization_id: UUID) {
  orgId.value = organization_id;
}

function handleAdvance() {
  const id = orgId.value ?? selectedOrgId.value;
  if (id) {
    emit('advance', id);
  }
}
</script>

<template>
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
      @click="$emit('skip')"
    >
      <X :size="18" aria-hidden="true" />
    </button>
  </div>

  <div class="wizard-modal__content">
    <Hook0Stack direction="column" gap="lg">
      <span class="wizard-modal__subtitle">{{ t('tutorial.step1Description') }}</span>

      <TutorialStepProgress :steps="progressSteps" :current="0" />

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
          <Hook0Stack layout="grid" gap="md" grid-size="compact" role="radiogroup">
            <SelectableCard
              :model-value="orgSection === OrgSection.CreateOrganization"
              :label="t('tutorial.createNewOrganization')"
              :icon="Plus"
              name="organization_selection"
              data-test="tutorial-create-org-option"
              @update:model-value="orgSection = OrgSection.CreateOrganization"
            />
            <SelectableCard
              :model-value="orgSection === OrgSection.SelectExistingOrganization"
              :label="t('tutorial.selectExistingOrganization')"
              :icon="List"
              name="organization_selection"
              data-test="tutorial-select-org-option"
              @update:model-value="orgSection = OrgSection.SelectExistingOrganization"
            />
          </Hook0Stack>
        </Hook0CardContent>
      </Hook0Card>

      <OrganizationsEdit
        v-if="!orgId && orgSection === OrgSection.CreateOrganization"
        :tutorial-mode="true"
        @tutorial-organization-created="handleCreated($event)"
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
    <Hook0Button variant="secondary" type="button" @click="$emit('skip')">
      <X :size="16" aria-hidden="true" />
      {{ t('tutorial.skip') }}
    </Hook0Button>
    <Hook0Button
      v-if="orgId || (selectedOrgId && selectedOrgId !== '')"
      variant="primary"
      type="button"
      @click="handleAdvance"
    >
      {{ t('tutorial.continueStep2') }}
      <ArrowRight :size="16" aria-hidden="true" />
    </Hook0Button>
  </div>
</template>
