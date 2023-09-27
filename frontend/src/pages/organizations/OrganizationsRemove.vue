<script setup lang="ts">
import { ref } from 'vue';

import * as OrganizationService from './OrganizationService';
import { isAxiosError, Problem } from '@/http';
import { Alert } from '@/components/Hook0Alert';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';

interface Props {
  organizationId: string;
  organizationName: string;
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

  if (!confirm(`Are you sure to delete "${props.organizationName}" organization?`)) {
    return;
  }

  alert.value.visible = false; // reset alert
  loading.value = true;

  OrganizationService.remove(props.organizationId)
    .then(() => window.location.assign('/'), displayError)
    // finally
    .finally(() => (loading.value = false));
}

function displayError(err: unknown) {
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
      <template #header> Delete this organization </template>
      <template #subtitle>
        This action deletes
        <Hook0Text class="bold">{{ organizationName }}</Hook0Text>
        and everything this organization contains. There is no going back.
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
