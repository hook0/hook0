<script setup lang="ts">
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { onMounted, ref } from 'vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { useRoute, useRouter } from 'vue-router';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';
import { Problem, UUID } from '@/http.ts';
import { routes } from '@/routes.ts';
import { push } from 'notivue';
import EventsList from '@/pages/organizations/applications/events/EventsList.vue';

const router = useRouter();
const route = useRoute();

const disabled_button = ref<boolean>(true);

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

const organizationId = ref<UUID | null>(null);
const applicationId = ref<UUID | null>(null);

function _load() {
  organizationId.value = route.params.organization_id as UUID;
  applicationId.value = route.params.application_id as UUID;
  if (!organizationId.value || !applicationId.value) {
    displayError({
      id: 'FieldsRequired',
      status: 400,
      title: 'Organization ID and Application ID are required',
      detail: 'Organization ID and Application ID are required to create an event type',
    });
  }
}

function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;

  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}

function cancel() {
  router.back();
}

function backToHome() {
  push.success({
    title: 'Event sent',
    message: 'Wow ! You just sent an event to your webhook ! 🎉🎉',
    duration: 5000,
  });
  return router.push({ name: routes.Home });
}

onMounted(() => {
  _load();
});
</script>

<template>
  <Hook0CardContent v-if="alert.visible">
    <Hook0Alert
      :type="alert.type"
      :title="alert.title"
      :description="alert.description"
    ></Hook0Alert>
    <Hook0Button class="secondary" type="button" @click="cancel">Close</Hook0Button>
  </Hook0CardContent>
  <Hook0Card v-else>
    <Hook0CardHeader>
      <template #header>Step 5: Send an event</template>
      <template #subtitle>
        In this step, you will send a test event. You should make it match your subscription's event
        type and label so that you receive a webhook!
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <EventsList
              v-if="organizationId && applicationId && disabled_button"
              :tutorial-mode="true"
              @tutorial-event-send="backToHome"
            />
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button class="primary" type="button" :disabled="disabled_button" @click="backToHome"
        >Back to home</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
