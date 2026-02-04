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
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0ProgressBar from '@/components/Hook0ProgressBar.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import { Building2, Plus, List, ArrowRight, X } from 'lucide-vue-next';

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

function goSecondStep(organization_id: UUID) {
  organizationId.value = organization_id;
  if (selectedOrganizationId.value) {
    trackEvent('tutorial', 'step-complete', 'organization');
    push.success({
      title: t('tutorial.organizationSelected'),
      message: t('tutorial.continueToApplication'),
      duration: 5000,
    });
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
                    <Hook0ListItem
                      variant="selectable"
                      name="organization_selection"
                      :selected="currentSection === Sections.CreateOrganization"
                      data-test="tutorial-create-org-option"
                      @select="currentSection = Sections.CreateOrganization"
                    >
                      <template #icon>
                        <Plus :size="18" />
                      </template>
                      <template #left>
                        {{ t('tutorial.createNewOrganization') }}
                      </template>
                    </Hook0ListItem>
                    <Hook0ListItem
                      variant="selectable"
                      name="organization_selection"
                      :selected="currentSection === Sections.SelectExistingOrganization"
                      data-test="tutorial-select-org-option"
                      @select="currentSection = Sections.SelectExistingOrganization"
                    >
                      <template #icon>
                        <List :size="18" />
                      </template>
                      <template #left>
                        {{ t('tutorial.selectExistingOrganization') }}
                      </template>
                    </Hook0ListItem>
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
                  <Hook0CardContentLines>
                    <Hook0CardContentLine type="full-width">
                      <template #label>{{ t('tutorial.selectOrganization') }}</template>
                      <template #content>
                        <Hook0Select
                          v-model="selectedOrganizationId"
                          :options="organizationOptions"
                        ></Hook0Select>
                      </template>
                    </Hook0CardContentLine>
                  </Hook0CardContentLines>
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
/* No custom styles - using Hook0* components only */
</style>
