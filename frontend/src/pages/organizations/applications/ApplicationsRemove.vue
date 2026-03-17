<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';

import * as ApplicationsService from './ApplicationService';
import { routes } from '@/routes';
import { handleMutationError } from '@/utils/handleMutationError';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import Hook0DangerZoneCard from '@/components/Hook0DangerZoneCard.vue';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const { trackEvent } = useTracking();
const { canDelete } = usePermissions();

type Props = {
  applicationId: string;
  applicationName: string;
}

const props = defineProps<Props>();
const loading = ref(false);

function confirmRemove() {
  loading.value = true;
  ApplicationsService.remove(props.applicationId)
    .then(() => {
      trackEvent('application', 'delete', 'success');
      return router.push({
        name: routes.OrganizationsDashboard,
        params: {
          organization_id: route.params.organization_id,
        },
      });
    })
    .catch(handleMutationError)
    .finally(() => (loading.value = false));
}
</script>

<template>
  <Hook0DangerZoneCard
    v-if="canDelete('application')"
    :title="t('remove.deleteApplication')"
    :subtitle="t('remove.deleteApplicationWarning', { name: applicationName })"
    :warning-message="t('remove.irreversibleWarning')"
    :confirm-message="t('remove.confirmDeleteApplication', { name: applicationName })"
    :loading="loading"
    data-test="application-delete-card"
    @confirm="confirmRemove"
  />
</template>
