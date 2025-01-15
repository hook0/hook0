<script setup lang="ts">
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { onMounted, ref } from 'vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { routes } from '@/routes.ts';
import { useRouter } from 'vue-router';
import { Problem, UUID } from '@/http.ts';
import { push } from 'notivue';
import { list } from '@/pages/organizations/OrganizationService.ts';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Error from '@/components/Hook0Error.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import Hook0ProgressBar from '@/components/Hook0ProgressBar.vue';
import party from 'party-js';
import { progressItems } from '@/pages/tutorial/TutorialService';

const router = useRouter();

enum Sections {
  CreateOrganization = 'create_organization',
  SelectExistingOrganization = 'select_existing_organization',
}

const organizationId = ref<UUID | null>(null);
const organizations_list = ref<Promise<Array<{ label: string; value: UUID }>>>(Promise.resolve([]));
const selected_organization_id = ref<UUID | null>(null);
const currentSection = ref<Sections | null>(null);

const goSecondStep = (organization_id: UUID) => {
  organizationId.value = organization_id;
  if (selected_organization_id.value) {
    push.success({
      title: 'Organization selected',
      message: 'You can now create your first application. 🎉',
      duration: 5000,
    });
    party.confetti(party.Rect.fromScreen(), {
      count: 80,
      spread: 40,
      size: party.variation.range(1.2, 1.6),
    });
    return router.push({
      name: routes.TutorialCreateApplication,
      params: { organization_id: selected_organization_id.value },
    });
  } else if (organizationId.value) {
    push.success({
      title: 'Organization created',
      message: 'You can now create your first application. 🎉',
      duration: 5000,
    });
    party.confetti(party.Rect.fromScreen(), {
      count: 80,
      spread: 40,
      size: party.variation.range(1.2, 1.6),
    });
    return router.push({
      name: routes.TutorialCreateApplication,
      params: { organization_id: organizationId.value },
    });
  } else {
    push.error({
      title: 'Organization ID is required',
      message: 'Something went wrong. Please try again. If the problem persists, contact support.',
      duration: 5000,
    });
  }
};

function _onLoad() {
  organizations_list.value = list()
    .then((organizations) => [
      { label: '', value: '' },
      ...organizations.map((o) => ({ label: o.name, value: o.organization_id })),
    ])
    .catch((error) => {
      displayError(error as Problem);
      return [];
    });
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

onMounted(() => {
  _onLoad();
});
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>
        <div class="flex items-center justify-between">Step 1: Create your first organization</div>
      </template>
      <template #subtitle>
        Organizations are used to invite your team members and give them access to applications,
        where everything webhook-related happens. If you purchase a plan, it will be applied to an
        organization and not to a user nor an application.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLine type="full-width">
        <template #content>
          <Hook0ProgressBar actual="1" :items="progressItems" class="mb-14" />
          <Hook0Card v-if="!organizationId" class="mb-4">
            <Hook0CardHeader>
              <template #header>First, let's choose an organization!</template>
            </Hook0CardHeader>
            <Hook0CardContent class="p-4 border space-y-4">
              <div class="grid grid-cols-1 sm:grid-cols-2">
                <label
                  class="flex items-center p-4 w-full border-b sm:border-b-0 sm:border-r cursor-pointer"
                >
                  <input
                    v-model="currentSection"
                    type="radio"
                    name="organization_selection"
                    :value="Sections.CreateOrganization"
                    class="h-4 w-4 border-gray-300 rounded focus:ring-indigo-500"
                  />
                  <span class="ml-2 text-sm font-medium text-gray-700">
                    Create a new organization
                  </span>
                </label>
                <label
                  class="flex items-center p-4 w-full border-t sm:border-t-0 sm:border-l cursor-pointer"
                >
                  <input
                    v-model="currentSection"
                    type="radio"
                    name="organization_selection"
                    :value="Sections.SelectExistingOrganization"
                    class="h-4 w-4 border-gray-300 rounded focus:ring-indigo-500"
                  />
                  <span class="ml-2 text-sm font-medium text-gray-700">
                    Select an existing organization already linked to your account
                  </span>
                </label>
              </div>
            </Hook0CardContent>
          </Hook0Card>
          <OrganizationsEdit
            v-if="!organizationId && currentSection === Sections.CreateOrganization"
            :tutorial-mode="true"
            class="mt-12"
            @tutorial-organization-created="goSecondStep($event)"
          />
        </template>
      </Hook0CardContentLine>
      <Promised
        v-if="currentSection === Sections.SelectExistingOrganization"
        :promise="organizations_list"
      >
        <template #pending>
          <Hook0Loader></Hook0Loader>
        </template>
        <template #default="organizations">
          <div class="px-6">
            <Hook0Card>
              <Hook0CardContent>
                <Hook0CardContentLines>
                  <Hook0CardContentLine type="full-width">
                    <template #label>
                      You can also select an existing organization to continue the tutorial.
                    </template>
                    <template #content>
                      <Hook0Select
                        v-model="selected_organization_id"
                        :options="organizations"
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
    <Hook0CardFooter v-if="organizationId || selected_organization_id">
      <Hook0Button class="primary" type="button" @click="goSecondStep">
        🚀 Continue Step 2: Create Your Application
      </Hook0Button>
    </Hook0CardFooter>
  </Hook0Card>
</template>
