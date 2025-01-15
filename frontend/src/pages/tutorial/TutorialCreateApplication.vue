<script setup lang="ts">
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { onMounted, ref } from 'vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import ApplicationsEdit from '@/pages/organizations/applications/ApplicationsEdit.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { useRoute, useRouter } from 'vue-router';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';
import { Problem, UUID } from '@/http.ts';
import { routes } from '@/routes.ts';
import { push } from 'notivue';
import Hook0ProgressBar from '@/components/Hook0ProgressBar.vue';
import party from 'party-js';
import { progressItems } from '@/pages/tutorial/TutorialService';

const router = useRouter();
const route = useRoute();

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

const organization_id = ref<UUID | null>(null);
const application_id = ref<UUID | null>(null);

function _load() {
  organization_id.value = route.params.organization_id as UUID;
  if (!organization_id.value) {
    displayError({
      id: 'OrganizationIdRequired',
      status: 400,
      title: 'Organization ID is required',
      detail: 'Organization ID is required to create an application',
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

function goThirdStep(applicationId: UUID) {
  application_id.value = applicationId;
  if (organization_id.value && application_id.value) {
    party.confetti(party.Rect.fromScreen(), {
      count: 80,
      spread: 40,
      size: party.variation.range(1.2, 1.6),
    });
    return router.push({
      name: routes.TutorialCreateEventType,
      params: {
        organization_id: organization_id.value,
        application_id: application_id.value,
      },
    });
  } else {
    push.error({
      title: 'Organization ID and Application ID are required',
      message: 'Organization ID and Application ID are required to create an event type',
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
      <template #header>Step 2: Create your first application</template>
      <template #subtitle
        >An application is an isolated environment in Hook0. It has its own event types, events,
        subscriptions. You can create multiple applications to group your services based on your
        needs.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0ProgressBar actual="2" :items="progressItems" class="mb-14" />
            <ApplicationsEdit
              v-if="organization_id && !application_id"
              :tutorial-mode="true"
              @tutorial-application-created="goThirdStep($event)"
            />
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
    <Hook0CardFooter v-if="organization_id && application_id">
      <Hook0Button
        class="primary"
        type="button"
        :disabled="!organization_id || !application_id"
        @click="goThirdStep"
        >🚀 Continue Step 3: Create Your First Event Type</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
