<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { Problem } from '@/http';
import { routes } from '@/routes';
import * as SubscriptionsService from './SubscriptionService';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import { push } from 'notivue';
import { useTracking } from '@/composables/useTracking';

const router = useRouter();
const route = useRoute();

// Analytics tracking
const { trackEvent } = useTracking();

interface Props {
  applicationId: string;
  subscriptionId: string;
  subscriptionName: string;
}

const props = defineProps<Props>();

const loading = ref(false);

function remove(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  if (!confirm(`Are you sure to delete "${props.subscriptionName}" subscription?`)) {
    return;
  }

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

function displayError(err: Problem) {
  console.error(err);
  let options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}
</script>

<template>
  <Hook0Card data-test="subscription-delete-card">
    <Hook0CardHeader>
      <template #header> Delete this subscription </template>
      <template #subtitle>
        This action deletes
        <Hook0Text class="bold">{{ subscriptionName }}</Hook0Text>
        and everything this subscription contains. There is no going back.
      </template>
    </Hook0CardHeader>
    <Hook0CardFooter>
      <Hook0Button
        class="danger"
        type="button"
        :loading="loading"
        data-test="subscription-delete-button"
        @click="remove($event)"
        >Delete</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
