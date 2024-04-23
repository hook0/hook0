<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { Problem } from '@/http';
import { Alert } from '@/components/Hook0Alert';
import { routes } from '@/routes';
import * as SubscriptionsService from './SubscriptionService';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';

const router = useRouter();
const route = useRoute();

interface Props {
  applicationId: string;
  subscriptionId: string;
  subscriptionName: string;
}

const props = defineProps<Props>();

const loading = ref(false);
const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

function remove(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  if (!confirm(`Are you sure to delete "${props.subscriptionName}" subscription?`)) {
    return;
  }

  alert.value.visible = false; // reset alert
  loading.value = true;

  SubscriptionsService.remove(props.applicationId, props.subscriptionId)
    .then(
      () =>
        router.push({
          name: routes.SubscriptionsList,
          params: {
            organization_id: route.params.organization_id,
            application_id: route.params.application_id,
          },
        }),
      displayError
    )
    // finally
    .finally(() => (loading.value = false));
}

function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;

  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header> Delete this subscription </template>
      <template #subtitle>
        This action deletes
        <Hook0Text class="bold">{{ subscriptionName }}</Hook0Text>
        and everything this subscription contains. There is no going back.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent v-if="alert.visible">
      <Hook0Alert
        :type="alert.type"
        :title="alert.title"
        :description="alert.description"
      ></Hook0Alert>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button class="danger" type="button" :loading="loading" @click="remove($event)"
        >Delete</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
