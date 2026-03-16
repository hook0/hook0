<script setup lang="ts">
import type { Component } from 'vue';
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

import type { UUID } from '@/http';
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
import Hook0Stack from '@/components/Hook0Stack.vue';
import SelectableCard from '@/components/SelectableCard.vue';
import TutorialStepProgress from '@/pages/tutorial/TutorialStepProgress.vue';
import ApplicationsEdit from '@/pages/organizations/applications/ApplicationsEdit.vue';

import { AppWindow, Plus, List, ArrowRight, X } from 'lucide-vue-next';

type ProgressStep = {
  icon: Component;
  label: string;
};

type Props = {
  organizationId: string;
  progressSteps: ProgressStep[];
};

const props = defineProps<Props>();

const emit = defineEmits<{
  advance: [applicationId: UUID];
  skip: [];
}>();

const { t } = useI18n();

const enum AppSection {
  CreateApplication,
  SelectExistingApplication,
}

const appId = ref<UUID | null>(null);
const selectedAppId = ref<UUID | null>(null);
const appSection = ref<AppSection | null>(null);

const appListOrgId = computed(() => props.organizationId || '');

const {
  data: rawApplications,
  isLoading: appLoading,
  error: appError,
  refetch: appRefetch,
} = useApplicationList(appListOrgId);

const applicationOptions = computed(() => {
  const apps = rawApplications.value ?? [];
  return [
    { label: t('tutorial.selectApplication'), value: '' },
    ...apps.map((a) => ({ label: a.name, value: a.application_id })),
  ];
});

// Auto-select "create" if no applications exist
watch(rawApplications, (apps) => {
  if ((apps ?? []).length <= 0 && appSection.value === null) {
    appSection.value = AppSection.CreateApplication;
  }
});

function handleCreated(application_id: UUID) {
  appId.value = application_id;
}

function handleAdvance() {
  const id = appId.value ?? selectedAppId.value;
  if (id) {
    emit('advance', id);
  }
}
</script>

<template>
  <div class="wizard-modal__header">
    <Hook0Stack direction="row" align="center" gap="sm">
      <Hook0Badge display="step" variant="primary">2</Hook0Badge>
      <span id="wizard-step-title" class="wizard-modal__title">{{ t('tutorial.step2Title') }}</span>
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

      <TutorialStepProgress :steps="progressSteps" :current="1" />

      <Hook0Card v-if="organizationId && !appId && applicationOptions.length > 1">
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
          <Hook0Stack layout="grid" gap="md" grid-size="compact" role="radiogroup">
            <SelectableCard
              :model-value="appSection === AppSection.CreateApplication"
              :label="t('tutorial.createNewApplication')"
              :icon="Plus"
              name="application_selection"
              @update:model-value="appSection = AppSection.CreateApplication"
            />
            <SelectableCard
              :model-value="appSection === AppSection.SelectExistingApplication"
              :label="t('tutorial.selectExistingApplication')"
              :icon="List"
              name="application_selection"
              @update:model-value="appSection = AppSection.SelectExistingApplication"
            />
          </Hook0Stack>
        </Hook0CardContent>
      </Hook0Card>

      <ApplicationsEdit
        v-if="organizationId && appSection === AppSection.CreateApplication"
        :tutorial-mode="true"
        @tutorial-application-created="handleCreated($event)"
      />

      <!-- Select existing application -->
      <template v-if="organizationId && appSection === AppSection.SelectExistingApplication">
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
    <Hook0Button variant="secondary" type="button" @click="$emit('skip')">
      <X :size="16" aria-hidden="true" />
      {{ t('tutorial.skip') }}
    </Hook0Button>
    <Hook0Button
      v-if="organizationId && (appId || (selectedAppId && selectedAppId !== ''))"
      variant="primary"
      type="button"
      @click="handleAdvance"
    >
      {{ t('tutorial.continueStep3') }}
      <ArrowRight :size="16" aria-hidden="true" />
    </Hook0Button>
  </div>
</template>
