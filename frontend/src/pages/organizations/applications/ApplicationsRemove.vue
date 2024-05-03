<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import * as ApplicationsService from './ApplicationService';
import { Problem } from '@/http';
import { routes } from '@/routes';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { push } from 'notivue';

const router = useRouter();
const route = useRoute();

interface Props {
  applicationId: string;
  applicationName: string;
}

const props = defineProps<Props>();

const loading = ref(false);

function remove(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  if (!confirm(`Are you sure to delete "${props.applicationName}" application?`)) {
    return;
  }

  loading.value = true;

  ApplicationsService.remove(props.applicationId)
    .then(
      () =>
        router.push({
          name: routes.OrganizationsDashboard,
          params: {
            organization_id: route.params.organization_id,
          },
        }),
      displayError
    )
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
  <Hook0Card>
    <Hook0CardHeader>
      <template #header> Delete this application </template>
      <template #subtitle>
        This action deletes
        <Hook0Text class="bold">{{ applicationName }}</Hook0Text>
        and everything this application contains. There is no going back.
      </template>
    </Hook0CardHeader>
    <Hook0CardFooter>
      <Hook0Button class="danger" type="button" :loading="loading" @click="remove($event)"
        >Delete</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
