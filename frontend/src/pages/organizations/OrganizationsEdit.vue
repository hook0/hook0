<script setup lang="ts">
import { computed, markRaw } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { Users, Rocket, FileText } from 'lucide-vue-next';

import {
  useOrganizationDetail,
  useCreateOrganization,
  useUpdateOrganization,
} from './useOrganizationQueries';
import { organizationSchema } from './organization.schema';
import type { OrganizationInfo } from './OrganizationService';
import { routes } from '@/routes';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import { useEntityForm } from '@/composables/useEntityForm';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Consumption, { type ConsumptionQuota } from '@/components/Hook0Consumption.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import OrganizationRemove from './OrganizationsRemove.vue';
import Hook0Form from '@/components/Hook0Form.vue';
import Hook0PageLayout from '@/components/Hook0PageLayout.vue';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const { trackEvent } = useTracking();

// Permissions
const { canCreate, canEdit, canDelete } = usePermissions();

type Props = {
  tutorialMode?: boolean;
};

const props = withDefaults(defineProps<Props>(), {
  tutorialMode: false,
});

const emit = defineEmits(['tutorial-organization-created']);

const organizationId = computed(() => {
  const id = route.params.organization_id;
  return typeof id === 'string' ? id : '';
});
const isNew = computed(() => !organizationId.value);

// Load existing organization for edit mode
const {
  data: orgDetail,
  isLoading,
  error: loadError,
  refetch,
} = useOrganizationDetail(organizationId);

// Mutations
const createMutation = useCreateOrganization();
const updateMutation = useUpdateOrganization();

// Form via composable
const { errors, defineField, onSubmit } = useEntityForm<{ name: string }, OrganizationInfo>({
  schema: organizationSchema,
  isNew,
  existingValues: computed(() => (orgDetail.value ? { name: orgDetail.value.name } : undefined)),
  createFn: (values) => createMutation.mutateAsync({ name: values.name }),
  updateFn: (values) =>
    updateMutation.mutateAsync({
      organizationId: organizationId.value,
      organization: { name: values.name },
    }),
  skipToast: () => props.tutorialMode,
  successCreateTitle: t('organizations.created'),
  successCreateMessage: (v) => t('organizations.createdMessage', { name: v.name }),
  successUpdateTitle: t('organizations.updated'),
  successUpdateMessage: (v) => t('organizations.updatedMessage', { name: v.name }),
  onCreated: (org) => {
    trackEvent('organization', 'create', 'success');
    if (props.tutorialMode) {
      emit('tutorial-organization-created', org.organization_id);
    } else {
      void router.push({
        name: routes.TutorialCreateApplication,
        params: { organization_id: org.organization_id },
      });
    }
  },
  onUpdated: () => {
    trackEvent('organization', 'update', 'success');
    void router.push({
      name: routes.OrganizationsDashboard,
      params: { organization_id: organizationId.value },
    });
  },
});

const [name, nameAttrs] = defineField('name');

// Consumptions computed from org detail
const consumptions = computed<ConsumptionQuota[]>(() => {
  if (!orgDetail.value) return [];
  return [
    {
      icon: markRaw(Users),
      name: t('organizations.consumptionMembers'),
      consumption: orgDetail.value.consumption.members || 0,
      quota: orgDetail.value.quotas.members_per_organization_limit,
    },
    {
      icon: markRaw(Rocket),
      name: t('organizations.consumptionApplications'),
      consumption: orgDetail.value.consumption.applications || 0,
      quota: orgDetail.value.quotas.applications_per_organization_limit,
    },
    {
      icon: markRaw(FileText),
      name: t('organizations.consumptionEventsPerDay'),
      consumption: orgDetail.value.consumption.events_per_day || 0,
      quota: orgDetail.value.quotas.events_per_day_limit,
    },
  ];
});
</script>

<template>
  <Hook0PageLayout :title="isNew ? t('organizations.createTitle') : t('organizations.settings')">
    <Hook0Stack direction="column" gap="xl">
      <!-- Loading for edit mode (also shown when query is disabled and data is undefined) -->
      <Hook0Card v-if="!isNew && (isLoading || !orgDetail)">
        <Hook0CardHeader>
          <template #header>{{ t('organizations.editTitle') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0SkeletonGroup :count="2" />
        </Hook0CardContent>
      </Hook0Card>

      <!-- Error loading org -->
      <Hook0ErrorCard v-else-if="!isNew && loadError" :error="loadError" @retry="refetch()" />

      <!-- Form -->
      <template v-else>
        <Hook0Form data-test="organization-form" @submit="onSubmit">
          <Hook0Card data-test="organization-card">
            <Hook0CardHeader>
              <template #header>{{
                isNew ? t('organizations.createTitle') : t('organizations.editTitle')
              }}</template>
              <template #subtitle>{{ t('organizations.formSubtitle') }}</template>
            </Hook0CardHeader>
            <Hook0CardContent>
              <Hook0CardContentLine>
                <template #label>{{ t('organizations.name') }}</template>
                <template #content>
                  <Hook0Input
                    v-model="name"
                    v-bind="nameAttrs"
                    type="text"
                    :placeholder="t('organizations.namePlaceholder')"
                    :error="errors.name"
                    data-test="organization-name-input"
                  >
                    <template #helpText></template>
                  </Hook0Input>
                </template>
              </Hook0CardContentLine>
            </Hook0CardContent>

            <Hook0CardFooter>
              <Hook0Button
                v-if="
                  !tutorialMode && (isNew ? canCreate('organization') : canEdit('organization'))
                "
                variant="primary"
                type="button"
                :loading="createMutation.isPending.value || updateMutation.isPending.value"
                :disabled="!name"
                data-test="organization-submit-button"
                @click="onSubmit"
                >{{ isNew ? t('common.create') : t('common.edit') }}
              </Hook0Button>

              <Hook0Button
                v-else
                variant="primary"
                :loading="createMutation.isPending.value"
                :disabled="!name"
                :tooltip="t('organizations.createTooltip')"
                type="button"
                data-test="organization-submit-button"
                @click="onSubmit"
              >
                {{ t('organizations.createFirstOrg') }}
              </Hook0Button>
            </Hook0CardFooter>
          </Hook0Card>
        </Hook0Form>

        <Hook0Consumption
          v-if="!isNew && organizationId && orgDetail"
          :title="t('organizations.consumptionTitle', { name: orgDetail.name })"
          entity-type="organization"
          :consumptions="consumptions"
        />

        <OrganizationRemove
          v-if="!isNew && canDelete('organization')"
          :organization-id="organizationId"
          :organization-name="orgDetail?.name ?? ''"
        />
      </template>
    </Hook0Stack>
  </Hook0PageLayout>
</template>

<style scoped>
/* Hook0Stack handles all layout */
</style>
