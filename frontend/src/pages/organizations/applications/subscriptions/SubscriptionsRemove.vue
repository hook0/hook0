<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';

import * as SubscriptionsService from './SubscriptionService';
import { routes } from '@/routes';
import { displayError } from '@/utils/displayError';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import Hook0DangerZoneCard from '@/components/Hook0DangerZoneCard.vue';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();
const { trackEvent } = useTracking();
const { canDelete } = usePermissions();

interface Props {
  applicationId: string;
  subscriptionId: string;
  subscriptionName: string;
}

const props = defineProps<Props>();
const loading = ref(false);

function confirmRemove() {
  loading.value = true;
  SubscriptionsService.remove(props.applicationId, props.subscriptionId)
    .then(() => {
      trackEvent('subscription', 'delete', 'success');
      return router.push({
        name: routes.SubscriptionsList,
        params: {
          organization_id: route.params.organization_id,
          application_id: route.params.application_id,
        },
      });
    })
    .catch(displayError)
    .finally(() => (loading.value = false));
}
</script>

<template>
  <Hook0DangerZoneCard
    v-if="canDelete('subscription')"
    :title="t('remove.deleteSubscription')"
    :subtitle="t('remove.deleteSubscriptionWarning', { name: subscriptionName })"
    :warning-message="t('remove.irreversibleWarning')"
    :confirm-message="t('remove.confirmDeleteSubscription', { name: subscriptionName })"
    :loading="loading"
    data-test="subscription-delete-card"
    @confirm="confirmRemove"
  />
</template>
