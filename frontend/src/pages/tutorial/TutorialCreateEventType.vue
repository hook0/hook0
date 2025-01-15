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
import EventTypesNew from '@/pages/organizations/applications/event_types/EventTypesNew.vue';
import { push } from 'notivue';
import Hook0ProgressBar from '@/components/Hook0ProgressBar.vue';
import party from 'party-js';
import { progressItems } from '@/pages/tutorial/TutorialService';

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

function goFourthStep() {
  if (organizationId.value && applicationId.value) {
    disabled_button.value = false;
    push.success({
      title: 'Event type created',
      message: 'You can now create your first subscription. 🎉',
      duration: 5000,
    });
    party.confetti(party.Rect.fromScreen(), {
      count: 80,
      spread: 40,
      size: party.variation.range(1.2, 1.6),
    });
    return router.push({
      name: routes.TutorialCreateSubscription,
      params: {
        organization_id: organizationId.value,
        application_id: applicationId.value,
      },
    });
  } else {
    push.error({
      title: 'Organization ID and Application ID are required',
      message: 'Something went wrong. Please try again. If the problem persists, contact support.',
      duration: 5000,
    });
  }
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
      <template #header>Step 3: Create your first event type</template>
      <template #subtitle>Each event sent through a Hook0 must have an event type. </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0ProgressBar actual="3" :items="progressItems" class="mb-14" />
            <EventTypesNew
              v-if="organizationId && applicationId && disabled_button"
              :tutorial-mode="true"
              @tutorial-event-type-created="goFourthStep"
            />
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
    <Hook0CardFooter v-if="organizationId && applicationId && !disabled_button">
      <Hook0Button
        class="primary"
        type="button"
        :disabled="!organizationId || !applicationId || disabled_button"
        @click="goFourthStep"
        >🚀 Continue Step 4: Create Your Subscription</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
