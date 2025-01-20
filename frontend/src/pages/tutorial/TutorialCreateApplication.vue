<script setup lang="ts">
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { onMounted, ref } from 'vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { routes } from '@/routes.ts';
import { useRouter, useRoute } from 'vue-router';
import { Problem, UUID } from '@/http.ts';
import { push } from 'notivue';
import { list } from '@/pages/organizations/applications/ApplicationService.ts';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Error from '@/components/Hook0Error.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0ProgressBar from '@/components/Hook0ProgressBar.vue';
import party from 'party-js';
import { progressItems } from '@/pages/tutorial/TutorialService';
import { Alert } from '@/components/Hook0Alert';
import ApplicationsEdit from '../organizations/applications/ApplicationsEdit.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';

const router = useRouter();
const route = useRoute();

const enum Section {
  CreateApplication,
  SelectExistingApplication,
}

const organizationId = ref<UUID | null>(null);
const applicationId = ref<UUID | null>(null);
const applications_list = ref<Promise<Array<{ label: string; value: UUID }>>>(Promise.resolve([]));
const selected_application_id = ref<UUID | null>(null);
const currentSection = ref<Section | null>(null);

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

function _load() {
  organizationId.value = route.params.organization_id as UUID;
  if (!organizationId.value) {
    return displayError({
      id: 'OrganizationIdRequired',
      status: 400,
      title: 'Organization ID is required',
      detail: 'Something went wrong. Please try again. If the problem persists, contact support.',
    });
  }

  applications_list.value = list(organizationId.value)
    .then((applications) => {
      if (applications.length <= 0) {
        currentSection.value = Section.CreateApplication;
      }
      return applications;
    })
    .then((applications) => [
      { label: '', value: '' },
      ...applications.map((a) => ({ label: a.name, value: a.application_id })),
    ])
    .catch((error) => {
      displayError(error as Problem);
      return [];
    });
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

const goThirdStep = (application_id: UUID) => {
  applicationId.value = application_id;
  if (organizationId.value && selected_application_id.value) {
    push.success({
      title: 'Application selected',
      message: 'You can now create your first event type. 🎉',
      duration: 5000,
    });
    party.confetti(party.Rect.fromScreen(), {
      count: 80,
      spread: 40,
      size: party.variation.range(1.2, 1.6),
    });
    return router.push({
      name: routes.TutorialCreateEventType,
      params: {
        organization_id: organizationId.value,
        application_id: selected_application_id.value,
      },
    });
  } else if (organizationId.value && applicationId.value) {
    push.success({
      title: 'Application created',
      message: 'You can now create your first event type. 🎉',
      duration: 5000,
    });
    party.confetti(party.Rect.fromScreen(), {
      count: 80,
      spread: 40,
      size: party.variation.range(1.2, 1.6),
    });
    return router.push({
      name: routes.TutorialCreateEventType,
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
};

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
      <template #header>
        <div class="flex items-center justify-between">Step 2: Create your first application</div>
      </template>
      <template #subtitle>
        An application is an isolated environment in Hook0. It has its own event types, events,
        subscriptions and request attempts. If your plan allows it, you can create multiple
        applications in order to isolate multiple environments or systems inside the same
        organization.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLine type="full-width">
        <template #content>
          <Hook0ProgressBar :current="2" :items="progressItems" class="mb-20" />
          <Promised v-if="organizationId" :promise="applications_list">
            <template #pending>
              <Hook0Loader></Hook0Loader>
            </template>
            <template #default="applications">
              <Hook0Card
                v-if="organizationId && !applicationId && applications.length > 1"
                class="mb-4"
              >
                <Hook0CardHeader>
                  <template #header>Let's choose an application!</template>
                </Hook0CardHeader>
                <Hook0CardContent class="p-4 border space-y-4">
                  <div class="grid grid-cols-1 sm:grid-cols-2">
                    <label
                      class="flex items-center p-4 w-full border-b sm:border-b-0 sm:border-r cursor-pointer"
                    >
                      <input
                        v-model="currentSection"
                        type="radio"
                        name="application_selection"
                        :value="Section.CreateApplication"
                        class="h-4 w-4 border-gray-300 rounded focus:ring-indigo-500"
                      />
                      <span class="ml-2 text-sm font-medium text-gray-700">
                        Create a new application
                      </span>
                    </label>
                    <label
                      class="flex items-center p-4 w-full border-t sm:border-t-0 sm:border-l cursor-pointer"
                    >
                      <input
                        v-model="currentSection"
                        type="radio"
                        name="application_selection"
                        :value="Section.SelectExistingApplication"
                        class="h-4 w-4 border-gray-300 rounded focus:ring-indigo-500"
                      />
                      <span class="ml-2 text-sm font-medium text-gray-700">
                        Select an existing application
                      </span>
                    </label>
                  </div>
                </Hook0CardContent>
              </Hook0Card>
              <ApplicationsEdit
                v-if="organizationId && currentSection === Section.CreateApplication"
                :tutorial-mode="true"
                class="mt-12"
                @tutorial-application-created="goThirdStep($event)"
              /> </template
          ></Promised>
        </template>
      </Hook0CardContentLine>
      <Promised
        v-if="organizationId && currentSection === Section.SelectExistingApplication"
        :promise="applications_list"
      >
        <template #pending>
          <Hook0Loader></Hook0Loader>
        </template>
        <template #default="applications">
          <div class="px-6">
            <Hook0Card>
              <Hook0CardContent>
                <Hook0CardContentLines>
                  <Hook0CardContentLine type="full-width">
                    <template #label>
                      You can also select an existing application to continue the tutorial.
                    </template>
                    <template #content>
                      <Hook0Select
                        v-model="selected_application_id"
                        :options="applications"
                      ></Hook0Select>
                    </template>
                  </Hook0CardContentLine>
                </Hook0CardContentLines>
              </Hook0CardContent>
            </Hook0Card>
          </div>
        </template>
        <template #rejected="error">
          <Hook0Error :error="error"></Hook0Error>
        </template>
      </Promised>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button
        class="secondary"
        type="button"
        @click="
          router.push({
            name: routes.OrganizationsDashboard,
            params: { organization_id: organizationId },
          })
        "
        >Skip</Hook0Button
      >
      <Hook0Button
        v-if="organizationId && (applicationId || selected_application_id)"
        class="primary"
        type="button"
        @click="goThirdStep"
      >
        🚀 Continue Step 3: Create Your First Event Type
      </Hook0Button>
    </Hook0CardFooter>
  </Hook0Card>
</template>
