<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { push } from 'notivue';

import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
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
import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import { Building2, Plus, List, ArrowRight, X, Check } from 'lucide-vue-next';
import { useCelebration } from '@/composables/useCelebration';

const { t } = useI18n();
const router = useRouter();
const { trackEvent } = useTracking();

const enum Sections {
  CreateOrganization = 'create_organization',
  SelectExistingOrganization = 'select_existing_organization',
}

const organizationId = ref<UUID | null>(null);
const selectedOrganizationId = ref<UUID | null>(null);
const currentSection = ref<Sections | null>(null);

const { data: rawOrganizations, isLoading, error, refetch } = useOrganizationList();

const organizationOptions = computed(() => [
  { label: '', value: '' },
  ...(rawOrganizations.value ?? []).map((o) => ({ label: o.name, value: o.organization_id })),
]);

const { celebrate } = useCelebration();

function celebrateStep() {
  celebrate();
}

function goSecondStep(organization_id: UUID) {
  organizationId.value = organization_id;
  if (selectedOrganizationId.value) {
    trackEvent('tutorial', 'step-complete', 'organization');
    push.success({
      title: t('tutorial.organizationSelected'),
      message: t('tutorial.continueToApplication'),
      duration: 5000,
    });
    celebrateStep();
    void router.push({
      name: routes.TutorialCreateApplication,
      params: { organization_id: selectedOrganizationId.value },
    });
  } else if (organizationId.value) {
    trackEvent('tutorial', 'step-complete', 'organization');
    push.success({
      title: t('tutorial.organizationCreated'),
      message: t('tutorial.continueToApplication'),
      duration: 5000,
    });
    celebrateStep();
    void router.push({
      name: routes.TutorialCreateApplication,
      params: { organization_id: organizationId.value },
    });
  } else {
    push.error({
      title: t('tutorial.organizationIdRequired'),
      message: t('common.somethingWentWrong'),
      duration: 5000,
    });
  }
}
</script>

<template>
  <Hook0Stack direction="column" gap="none">
    <Hook0Card>
      <Hook0CardHeader>
        <template #header>
          <Hook0Stack direction="row" align="center" gap="sm">
            <Hook0Badge display="step" variant="primary">1</Hook0Badge>
            <Hook0Stack direction="row" align="center" gap="none">
              {{ t('tutorial.step1Title') }}
            </Hook0Stack>
          </Hook0Stack>
        </template>
        <template #subtitle>
          {{ t('tutorial.step1Description') }}
        </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Stack direction="column" gap="lg">
              <Hook0ProgressBar :current="1" :items="progressItems" />

              <Hook0Card v-if="!organizationId">
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
                        'selectable-card--selected': currentSection === Sections.CreateOrganization,
                      }"
                      data-test="tutorial-create-org-option"
                      @click="currentSection = Sections.CreateOrganization"
                    >
                      <input
                        type="radio"
                        name="organization_selection"
                        :checked="currentSection === Sections.CreateOrganization"
                        class="selectable-card__radio"
                      />
                      <span
                        class="selectable-card__icon"
                        :class="{
                          'selectable-card__icon--selected':
                            currentSection === Sections.CreateOrganization,
                        }"
                      >
                        <Plus :size="18" />
                      </span>
                      <span class="selectable-card__label">
                        {{ t('tutorial.createNewOrganization') }}
                      </span>
                      <span class="selectable-card__indicator">
                        <Check
                          v-if="currentSection === Sections.CreateOrganization"
                          :size="16"
                          aria-hidden="true"
                        />
                      </span>
                    </label>
                    <label
                      class="selectable-card"
                      :class="{
                        'selectable-card--selected':
                          currentSection === Sections.SelectExistingOrganization,
                      }"
                      data-test="tutorial-select-org-option"
                      @click="currentSection = Sections.SelectExistingOrganization"
                    >
                      <input
                        type="radio"
                        name="organization_selection"
                        :checked="currentSection === Sections.SelectExistingOrganization"
                        class="selectable-card__radio"
                      />
                      <span
                        class="selectable-card__icon"
                        :class="{
                          'selectable-card__icon--selected':
                            currentSection === Sections.SelectExistingOrganization,
                        }"
                      >
                        <List :size="18" />
                      </span>
                      <span class="selectable-card__label">
                        {{ t('tutorial.selectExistingOrganization') }}
                      </span>
                      <span class="selectable-card__indicator">
                        <Check
                          v-if="currentSection === Sections.SelectExistingOrganization"
                          :size="16"
                          aria-hidden="true"
                        />
                      </span>
                    </label>
                  </Hook0Stack>
                </Hook0CardContent>
              </Hook0Card>

              <OrganizationsEdit
                v-if="!organizationId && currentSection === Sections.CreateOrganization"
                :tutorial-mode="true"
                @tutorial-organization-created="goSecondStep($event)"
              />
            </Hook0Stack>
          </template>
        </Hook0CardContentLine>

        <!-- Select existing organization -->
        <template v-if="currentSection === Sections.SelectExistingOrganization">
          <!-- Loading -->
          <Hook0CardContentLine v-if="isLoading" type="full-width">
            <template #content>
              <Hook0Stack direction="column" gap="md">
                <Hook0Skeleton size="hero" />
                <Hook0Skeleton size="heading" />
              </Hook0Stack>
            </template>
          </Hook0CardContentLine>

          <!-- Error -->
          <Hook0ErrorCard v-else-if="error" :error="error" @retry="refetch()" />

          <!-- Organization select -->
          <template v-else>
            <Hook0Stack direction="column" gap="none">
              <Hook0Card>
                <Hook0CardContent>
                  <Hook0CardContentLine type="full-width">
                    <template #label>{{ t('tutorial.selectOrganization') }}</template>
                    <template #content>
                      <Hook0Select
                        v-model="selectedOrganizationId"
                        :options="organizationOptions"
                      ></Hook0Select>
                    </template>
                  </Hook0CardContentLine>
                </Hook0CardContent>
              </Hook0Card>
            </Hook0Stack>
          </template>
        </template>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button variant="secondary" type="button" @click="router.push({ name: routes.Home })">
          <X :size="16" />
          {{ t('tutorial.skip') }}
        </Hook0Button>
        <Hook0Button
          v-if="organizationId || selectedOrganizationId"
          variant="primary"
          type="button"
          @click="goSecondStep(organizationId ?? selectedOrganizationId ?? ('' as UUID))"
        >
          {{ t('tutorial.continueStep2') }}
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
