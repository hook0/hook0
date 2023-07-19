<script setup lang="ts">
import { AxiosError } from 'axios';
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import * as ApplicationsService from './ApplicationService';
import { isAxiosError, Problem } from '@/http';
import { Alert } from '@/components/Hook0Alert';
import { routes } from '@/routes';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Button from '@/components/Hook0Button.vue';

const router = useRouter();
const route = useRoute();

interface Props {
  applicationId: string;
  applicationName: string;
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

  if (!confirm(`Are you sure to delete "${props.applicationName}" application?`)) {
    return;
  }

  alert.value.visible = false; // reset alert
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

function displayError(err: AxiosError | unknown) {
  console.error(err);
  alert.value.visible = true;

  if (isAxiosError(err) && err.response) {
    const problem: Problem = err.response.data as Problem;
    alert.value.type = problem.status >= 500 ? 'alert' : 'warning';
    alert.value.title = problem.title;
    alert.value.description = problem.detail;
  } else {
    alert.value.type = 'alert';
    alert.value.title = 'An error occurred';
    alert.value.description = String(err);
  }
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
