<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { push } from 'notivue';

import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';
import { routes } from '@/routes';
import { UUID } from '@/http';
import { progressItems } from '@/pages/tutorial/TutorialService';
import { useTracking } from '@/composables/useTracking';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0ProgressBar from '@/components/Hook0ProgressBar.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import ApplicationsEdit from '@/pages/organizations/applications/ApplicationsEdit.vue';
import { AppWindow, Plus, List, ArrowRight, X, Check } from 'lucide-vue-next';
import { useCelebration } from '@/composables/useCelebration';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const { trackEvent } = useTracking();

const enum Section {
  CreateApplication,
  SelectExistingApplication,
}

const organizationId = computed(() => route.params.organization_id as UUID);
const applicationId = ref<UUID | null>(null);
const selectedApplicationId = ref<UUID | null>(null);
const currentSection = ref<Section | null>(null);

const { data: rawApplications, isLoading, error, refetch } = useApplicationList(organizationId);

const applicationOptions = computed(() => {
  const apps = rawApplications.value ?? [];
  return [
    { label: '', value: '' },
    ...apps.map((a) => ({ label: a.name, value: a.application_id })),
  ];
});

// Auto-select "create" if no applications exist
watch(rawApplications, (apps) => {
  if ((apps ?? []).length <= 0 && currentSection.value === null) {
    currentSection.value = Section.CreateApplication;
  }
});

const { celebrate } = useCelebration();

function celebrateStep() {
  celebrate();
}

function goThirdStep(application_id: UUID) {
  applicationId.value = application_id;
  if (organizationId.value && selectedApplicationId.value) {
    trackEvent('tutorial', 'step-complete', 'application');
    push.success({
      title: t('tutorial.applicationSelected'),
      message: t('tutorial.continueToEventType'),
      duration: 5000,
    });
    celebrateStep();
    void router.push({
      name: routes.TutorialCreateEventType,
      params: {
        organization_id: organizationId.value,
        application_id: selectedApplicationId.value,
      },
    });
  } else if (organizationId.value && applicationId.value) {
    trackEvent('tutorial', 'step-complete', 'application');
    push.success({
      title: t('tutorial.applicationCreated'),
      message: t('tutorial.continueToEventType'),
      duration: 5000,
    });
    celebrateStep();
    void router.push({
      name: routes.TutorialCreateEventType,
      params: {
        organization_id: organizationId.value,
        application_id: applicationId.value,
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
</script>

<template>
  <!-- Loading -->
  <Hook0Stack direction="column" gap="none">
    <Hook0Card v-if="isLoading">
      <Hook0CardHeader>
        <template #header>
          <Hook0Stack direction="row" align="center" gap="sm">
            <Hook0Badge display="step" variant="primary">2</Hook0Badge>
            <Hook0Stack direction="row" align="center" gap="none">
              {{ t('tutorial.step2Title') }}
            </Hook0Stack>
          </Hook0Stack>
        </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0Stack direction="column" gap="md">
          <Hook0Skeleton size="hero" />
          <Hook0Skeleton size="heading" />
          <Hook0Skeleton size="heading" />
        </Hook0Stack>
      </Hook0CardContent>
    </Hook0Card>

    <!-- Error -->
    <Hook0ErrorCard v-else-if="error" :error="error" @retry="refetch()" />

    <!-- Data loaded -->
    <Hook0Card v-else>
      <Hook0CardHeader>
        <template #header>
          <Hook0Stack direction="row" align="center" gap="sm">
            <Hook0Badge display="step" variant="primary">2</Hook0Badge>
            <Hook0Stack direction="row" align="center" gap="none">
              {{ t('tutorial.step2Title') }}
            </Hook0Stack>
          </Hook0Stack>
        </template>
        <template #subtitle>
          {{ t('tutorial.step2Description') }}
        </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Stack direction="column" gap="lg">
              <Hook0ProgressBar :current="2" :items="progressItems" />

              <Hook0Card v-if="organizationId && !applicationId && applicationOptions.length > 1">
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
                        'selectable-card--selected': currentSection === Section.CreateApplication,
                      }"
                      @click="currentSection = Section.CreateApplication"
                    >
                      <input
                        type="radio"
                        name="application_selection"
                        :checked="currentSection === Section.CreateApplication"
                        class="selectable-card__radio"
                      />
                      <span
                        class="selectable-card__icon"
                        :class="{
                          'selectable-card__icon--selected':
                            currentSection === Section.CreateApplication,
                        }"
                      >
                        <Plus :size="18" />
                      </span>
                      <span class="selectable-card__label">
                        {{ t('tutorial.createNewApplication') }}
                      </span>
                      <span class="selectable-card__indicator">
                        <Check
                          v-if="currentSection === Section.CreateApplication"
                          :size="16"
                          aria-hidden="true"
                        />
                      </span>
                    </label>
                    <label
                      class="selectable-card"
                      :class="{
                        'selectable-card--selected':
                          currentSection === Section.SelectExistingApplication,
                      }"
                      @click="currentSection = Section.SelectExistingApplication"
                    >
                      <input
                        type="radio"
                        name="application_selection"
                        :checked="currentSection === Section.SelectExistingApplication"
                        class="selectable-card__radio"
                      />
                      <span
                        class="selectable-card__icon"
                        :class="{
                          'selectable-card__icon--selected':
                            currentSection === Section.SelectExistingApplication,
                        }"
                      >
                        <List :size="18" />
                      </span>
                      <span class="selectable-card__label">
                        {{ t('tutorial.selectExistingApplication') }}
                      </span>
                      <span class="selectable-card__indicator">
                        <Check
                          v-if="currentSection === Section.SelectExistingApplication"
                          :size="16"
                          aria-hidden="true"
                        />
                      </span>
                    </label>
                  </Hook0Stack>
                </Hook0CardContent>
              </Hook0Card>

              <ApplicationsEdit
                v-if="organizationId && currentSection === Section.CreateApplication"
                :tutorial-mode="true"
                @tutorial-application-created="goThirdStep($event)"
              />
            </Hook0Stack>
          </template>
        </Hook0CardContentLine>

        <!-- Select existing application -->
        <template v-if="organizationId && currentSection === Section.SelectExistingApplication">
          <Hook0Stack direction="column" gap="none">
            <Hook0Card>
              <Hook0CardContent>
                <Hook0CardContentLine type="full-width">
                  <template #label>
                    {{ t('tutorial.selectApplication') }}
                  </template>
                  <template #content>
                    <Hook0Select
                      v-model="selectedApplicationId"
                      :options="applicationOptions"
                    ></Hook0Select>
                  </template>
                </Hook0CardContentLine>
              </Hook0CardContent>
            </Hook0Card>
          </Hook0Stack>
        </template>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button
          variant="secondary"
          type="button"
          @click="
            router.push({
              name: routes.OrganizationsDashboard,
              params: { organization_id: organizationId },
            })
          "
        >
          <X :size="16" />
          {{ t('tutorial.skip') }}
        </Hook0Button>
        <Hook0Button
          v-if="organizationId && (applicationId || selectedApplicationId)"
          variant="primary"
          type="button"
          @click="goThirdStep(applicationId ?? selectedApplicationId ?? ('' as UUID))"
        >
          {{ t('tutorial.continueStep3') }}
          <ArrowRight :size="16" />
        </Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>
  </Hook0Stack>
</template>

<style scoped>
.selectable-card {
  display: flex;
  align-items: center;
  padding: 1rem 1.25rem;
  border: 2px solid var(--color-border);
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-primary);
  cursor: pointer;
  transition: all 0.15s ease;
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
  color: #ffffff;
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
</style>
