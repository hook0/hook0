<script setup lang="ts">
import { computed, markRaw, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useForm } from 'vee-validate';
import { push } from 'notivue';
import { FileText } from 'lucide-vue-next';

import {
  useApplicationDetail,
  useCreateApplication,
  useUpdateApplication,
} from './useApplicationQueries';
import { applicationSchema } from './application.schema';
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
import ApplicationsRemove from '@/pages/organizations/applications/ApplicationsRemove.vue';
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

const emit = defineEmits(['tutorial-application-created']);

const organizationId = computed(() => {
  const id = route.params.organization_id;
  return typeof id === 'string' ? id : '';
});

const applicationId = computed(() => {
  const id = route.params.application_id;
  return typeof id === 'string' ? id : '';
});

const isNew = computed(() => !applicationId.value);

// Load existing application for edit mode
const {
  data: appDetail,
  isLoading,
  error: loadError,
  refetch,
} = useApplicationDetail(applicationId);

// VeeValidate form with Zod schema
const { errors, defineField, handleSubmit, resetForm } = useForm({
  validationSchema: toTypedSchema(applicationSchema),
});

const [name, nameAttrs] = defineField('name');

// Populate form when app data loads
watch(appDetail, (app) => {
  if (app) {
    resetForm({ values: { name: app.name } });
  }
});

// Consumptions computed from app detail
const consumptions = computed<ComsumptionQuota[]>(() => {
  if (!appDetail.value) return [];
  return [
    {
      icon: markRaw(FileText),
      name: t('applications.consumptionEventsPerDay'),
      comsumption: appDetail.value.consumption.events_per_day || 0,
      quota: appDetail.value.quotas.events_per_day_limit,
    },
  ];
});

function cancel() {
  router.back();
}

// Mutations
const createMutation = useCreateApplication();
const updateMutation = useUpdateApplication();

const onSubmit = handleSubmit((values) => {
  if (isNew.value) {
    createMutation.mutate(
      { name: values.name, organization_id: organizationId.value },
      {
        onSuccess: (app) => {
          trackEvent('application', 'create', 'success');
          if (props.tutorialMode) {
            emit('tutorial-application-created', app.application_id);
          } else {
            push.success({
              title: t('applications.created'),
              message: t('applications.createdMessage', { name: values.name }),
              duration: 5000,
            });
            void router.push({
              name: routes.TutorialCreateEventType,
              params: {
                organization_id: organizationId.value,
                application_id: app.application_id,
              },
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
      {
        applicationId: applicationId.value,
        application: { name: values.name, organization_id: organizationId.value },
      },
      {
        onSuccess: () => {
          trackEvent('application', 'update', 'success');
          cancel();
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
  <Hook0PageLayout :title="isNew ? t('applications.createTitle') : t('applications.settings')">
    <Hook0Stack direction="column" gap="xl">
      <!-- Loading for edit mode -->
      <Hook0Card v-if="!isNew && isLoading">
        <Hook0CardHeader>
          <template #header>{{ t('applications.editTitle') }}</template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0SkeletonGroup :count="2" />
        </Hook0CardContent>
      </Hook0Card>

      <!-- Error loading app -->
      <Hook0ErrorCard v-else-if="!isNew && loadError" :error="loadError" @retry="refetch()" />

      <!-- Form -->
      <template v-else>
        <Hook0Form data-test="application-form" @submit="onSubmit">
          <Hook0Card data-test="application-card">
            <Hook0CardHeader>
              <template #header>{{
                isNew ? t('applications.createTitle') : t('applications.editTitle')
              }}</template>
              <template #subtitle>{{ t('applications.formSubtitle') }}</template>
            </Hook0CardHeader>
            <Hook0CardContent>
              <Hook0CardContentLine>
                <template #label>{{ t('applications.name') }}</template>
                <template #content>
                  <Hook0Input
                    v-model="name"
                    v-bind="nameAttrs"
                    type="text"
                    :placeholder="t('applications.namePlaceholder')"
                    :error="errors.name"
                    data-test="application-name-input"
                  >
                    <template #helpText>{{ t('applications.nameHelpText') }}</template>
                  </Hook0Input>
                </template>
              </Hook0CardContentLine>
            </Hook0CardContent>

            <Hook0CardFooter>
              <Hook0Button
                v-if="!tutorialMode"
                variant="secondary"
                type="button"
                data-test="application-cancel-button"
                @click="cancel()"
                >{{ t('common.cancel') }}</Hook0Button
              >
              <Hook0Button
                v-if="!tutorialMode && (isNew ? canCreate('application') : canEdit('application'))"
                variant="primary"
                type="button"
                :loading="createMutation.isPending.value || updateMutation.isPending.value"
                :disabled="!name"
                data-test="application-submit-button"
                @click="onSubmit"
                >{{ isNew ? t('common.create') : t('common.edit') }}
              </Hook0Button>

              <Hook0Button
                v-else
                variant="primary"
                type="button"
                :loading="createMutation.isPending.value"
                :disabled="!name"
                :tooltip="t('applications.createTooltip')"
                data-test="application-submit-button"
                @click="onSubmit"
                >{{ t('applications.createFirstApp') }}
              </Hook0Button>
            </Hook0CardFooter>
          </Hook0Card>
        </Hook0Form>

        <Hook0Consumption
          v-if="!isNew && applicationId && appDetail"
          :title="t('applications.consumptionTitle', { name: appDetail.name })"
          entity-type="application"
          :consomptions="consumptions"
        />

        <ApplicationsRemove
          v-if="!isNew && applicationId && canDelete('application')"
          :application-id="applicationId"
          :application-name="appDetail?.name ?? ''"
        ></ApplicationsRemove>
      </template>
    </Hook0Stack>
  </Hook0PageLayout>
</template>

<style scoped>
/* Hook0Stack handles all layout */
</style>
