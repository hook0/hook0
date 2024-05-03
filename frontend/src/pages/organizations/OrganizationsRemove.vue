<script setup lang="ts">
import { ref } from 'vue';

import * as OrganizationService from './OrganizationService';
import { Problem } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import { push } from 'notivue';

interface Props {
  organizationId: string;
  organizationName: string;
}

const props = defineProps<Props>();

const loading = ref(false);

function remove(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  if (!confirm(`Are you sure to delete "${props.organizationName}" organization?`)) {
    return;
  }

  loading.value = true;

  OrganizationService.remove(props.organizationId)
    .then(() => window.location.assign('/'), displayError)
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
      <template #header> Delete this organization </template>
      <template #subtitle>
        This action deletes
        <Hook0Text class="bold">{{ organizationName }}</Hook0Text>
        and everything this organization contains. There is no going back.
      </template>
    </Hook0CardHeader>
    <Hook0CardFooter>
      <Hook0Button class="danger" type="button" :loading="loading" @click="remove($event)"
        >Delete</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
