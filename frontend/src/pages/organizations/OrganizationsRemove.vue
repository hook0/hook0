<script setup lang="ts">
import { ref } from 'vue';
import { toast } from 'vue-sonner';
import { useI18n } from 'vue-i18n';

import { useQueryClient } from '@tanstack/vue-query';

import * as OrganizationService from './OrganizationService';
import { handleMutationError } from '@/utils/handleMutationError';
import { organizationKeys } from '@/queries/keys';
import router from '@/router';
import { routes } from '@/routes';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import Hook0DangerZoneCard from '@/components/Hook0DangerZoneCard.vue';

const { t } = useI18n();
const queryClient = useQueryClient();
const { trackEvent } = useTracking();
const { canDelete } = usePermissions();

type Props = {
  organizationId: string;
  organizationName: string;
};

const props = defineProps<Props>();
const loading = ref(false);

function confirmRemove() {
  loading.value = true;
  OrganizationService.remove(props.organizationId)
    .then(() => {
      trackEvent('organization', 'delete', 'success');
      void queryClient.invalidateQueries({ queryKey: organizationKeys.all });
      toast.success(t('remove.organizationDeleted'), {
        description: t('remove.organizationDeletedMessage', { name: props.organizationName }),
        duration: 5000,
      });
      return router.push({ name: routes.Home });
    })
    .catch(handleMutationError)
    .finally(() => (loading.value = false));
}
</script>

<template>
  <Hook0DangerZoneCard
    v-if="canDelete('organization')"
    :title="t('remove.deleteOrganization')"
    :subtitle="t('remove.deleteOrganizationWarning', { name: organizationName })"
    :warning-message="t('remove.irreversibleWarning')"
    confirm-message="remove.confirmDeleteOrganization"
    :confirm-name="organizationName"
    :loading="loading"
    data-test="organization-delete-card"
    @confirm="confirmRemove"
  />
</template>
