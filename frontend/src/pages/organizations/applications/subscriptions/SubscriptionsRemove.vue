<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { AlertTriangle, Trash2 } from 'lucide-vue-next';

import { routes } from '@/routes';
import * as SubscriptionsService from './SubscriptionService';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Dialog from '@/components/Hook0Dialog.vue';
import { displayError } from '@/utils/displayError';
import { useTracking } from '@/composables/useTracking';
import { usePermissions } from '@/composables/usePermissions';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();

// Analytics tracking
const { trackEvent } = useTracking();

// Permissions
const { canDelete } = usePermissions();

interface Props {
  applicationId: string;
  subscriptionId: string;
  subscriptionName: string;
}

const props = defineProps<Props>();

const loading = ref(false);
const showDeleteDialog = ref(false);

function remove(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();
  showDeleteDialog.value = true;
}

function confirmRemove() {
  showDeleteDialog.value = false;
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
    }, displayError)
    // finally
    .finally(() => (loading.value = false));
}
</script>

<template>
  <Hook0Card v-if="canDelete('subscription')" data-test="subscription-delete-card">
    <Hook0CardHeader>
      <template #header>
        <Hook0Stack direction="row" align="center" gap="sm">
          <Hook0IconBadge variant="danger">
            <AlertTriangle :size="18" aria-hidden="true" />
          </Hook0IconBadge>
          <span class="sub-remove__title">{{ t('remove.deleteSubscription') }}</span>
        </Hook0Stack>
      </template>
      <template #subtitle>
        {{ t('remove.deleteSubscriptionWarning', { name: subscriptionName }) || '' }}
        <span class="sub-remove__name">{{ subscriptionName }}</span>
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0Alert type="alert">
        <template #description>
          {{
            t('remove.irreversibleWarning') ||
            'This action deletes the subscription and everything it contains. There is no going back.'
          }}
        </template>
      </Hook0Alert>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button
        variant="danger"
        type="button"
        :loading="loading"
        data-test="subscription-delete-button"
        @click="remove($event)"
      >
        <Trash2 :size="16" aria-hidden="true" />
        {{ t('common.delete') }}
      </Hook0Button>
    </Hook0CardFooter>

    <Hook0Dialog
      :open="showDeleteDialog"
      variant="danger"
      :title="t('remove.deleteSubscription')"
      @close="showDeleteDialog = false"
      @confirm="confirmRemove()"
    >
      <p>{{ t('remove.confirmDeleteSubscription', { name: subscriptionName }) }}</p>
    </Hook0Dialog>
  </Hook0Card>
</template>

<style scoped>
.sub-remove__title {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}

.sub-remove__name {
  color: var(--color-text-primary);
  font-weight: 600;
  font-size: 0.875rem;
  line-height: 1.5;
}
</style>
