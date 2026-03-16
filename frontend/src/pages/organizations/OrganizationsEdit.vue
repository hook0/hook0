<script setup lang="ts">
import { computed, markRaw, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { push } from 'notivue';
import { Users, Rocket, FileText } from 'lucide-vue-next';

import {
  useOrganizationDetail,
  useCreateOrganization,
  useUpdateOrganization,
} from './useOrganizationQueries';
import { organizationSchema } from './organization.schema';
import { toTypedSchema } from '@/utils/zod-adapter';
import { routes } from '@/routes';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0ErrorCard from '@/components/Hook0ErrorCard.vue';
import Hook0Consumption, { type ComsumptionQuota } from '@/components/Hook0Consumption.vue';
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

interface Props {
  tutorialMode?: boolean;
}

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

// VeeValidate form with Zod schema
const { errors, defineField, handleSubmit, resetForm } = useForm({
  validationSchema: toTypedSchema(organizationSchema),
});

const [name, nameAttrs] = defineField('name');

// Populate form when org data loads
watch(orgDetail, (org) => {
  if (org) {
    resetForm({ values: { name: org.name } });
  }
});

// Consumptions computed from org detail
const consumptions = computed<ComsumptionQuota[]>(() => {
  if (!orgDetail.value) return [];
  return [
    {
      icon: markRaw(Users),
      name: t('organizations.consumptionMembers'),
      comsumption: orgDetail.value.consumption.members || 0,
      quota: orgDetail.value.quotas.members_per_organization_limit,
    },
    {
      icon: markRaw(Rocket),
      name: t('organizations.consumptionApplications'),
      comsumption: orgDetail.value.consumption.applications || 0,
      quota: orgDetail.value.quotas.applications_per_organization_limit,
    },
    {
      icon: markRaw(FileText),
      name: t('organizations.consumptionEventsPerDay'),
      comsumption: orgDetail.value.consumption.events_per_day || 0,
      quota: orgDetail.value.quotas.events_per_day_limit,
    },
  ];
});

// Mutations
const createMutation = useCreateOrganization();
const updateMutation = useUpdateOrganization();

const onSubmit = handleSubmit((values) => {
  if (isNew.value) {
    createMutation.mutate(
      { name: values.name },
      {
        onSuccess: (org) => {
          trackEvent('organization', 'create', 'success');
          if (props.tutorialMode) {
            emit('tutorial-organization-created', org.organization_id);
          } else {
            push.success({
              title: t('organizations.created'),
              message: t('organizations.createdMessage', { name: values.name }),
              duration: 5000,
            });
            void router.push({
              name: routes.TutorialCreateApplication,
              params: { organization_id: org.organization_id },
            });
          }
        },
        onError: (err) => {
          displayError(err as unknown as Problem);
        },
      }
    );
  } else {
    updateMutation.mutate(
      { organizationId: organizationId.value, organization: { name: values.name } },
      {
        onSuccess: () => {
          trackEvent('organization', 'update', 'success');
          push.success({
            title: t('organizations.updated'),
            message: t('organizations.updatedMessage', { name: values.name }),
            duration: 5000,
          });
          void router.push({
            name: routes.OrganizationsDashboard,
            params: { organization_id: organizationId.value },
          });
        },
        onError: (err) => {
          displayError(err as unknown as Problem);
        },
      }
    );
  }
});
</script>

<template>
  <Hook0PageLayout :title="isNew ? t('organizations.createTitle') : t('organizations.settings')">
    <Hook0Stack direction="column" gap="xl">
      <!-- Loading for edit mode -->
      <Hook0Card v-if="!isNew && isLoading">
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
          :consomptions="consumptions"
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
