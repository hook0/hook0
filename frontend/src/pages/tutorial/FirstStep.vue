<script setup lang="ts">
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { onMounted, onUpdated, ref } from 'vue';
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

const router = useRouter();

const organizationId = ref<UUID | null>(null);
const organizations_list = ref<Promise<Array<{ label: string; value: UUID }>>>(Promise.resolve([]));
const selected_organization_id = ref<UUID | null>(null);

const goSecondStep = (organization_id: UUID) => {
  organizationId.value = organization_id;
  if (selected_organization_id.value) {
    return router.push({
      name: routes.TutorialStep2,
      params: { organization_id: selected_organization_id.value },
    });
  } else if (organizationId.value) {
    return router.push({
      name: routes.TutorialStep2,
      params: { organization_id: organizationId.value },
    });
  } else {
    push.error({
      title: 'Organization id is required',
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

onUpdated(() => {
  _onLoad();
});
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Step 1: Create your first organization</template>
      <template #subtitle
        >Organizations are used to group your applications and environments. You can create multiple
        organizations to separate your projects. Like production, staging, and development.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine v-if="!selected_organization_id" type="full-width">
          <template v-if="!organizationId" #content>
            <OrganizationsEdit
              :tutorial-mode="true"
              @tutorial-organization-created="goSecondStep($event)"
            />
          </template>
        </Hook0CardContentLine>
        <Promised v-if="!organizationId" :promise="organizations_list">
          <!-- Use the "pending" slot to display a loading message -->
          <template #pending>
            <Hook0Loader></Hook0Loader>
          </template>

          <!-- The default scoped slot will be used as the result -->
          <template #default="organizations">
            <Hook0Card>
              <Hook0CardContent>
                <Hook0CardContentLine>
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
              </Hook0CardContent>
            </Hook0Card>
          </template>

          <!-- The "rejected" scoped slot will be used if there is an error -->
          <template #rejected="error">
            <Hook0Error :error="error"></Hook0Error>
          </template>
        </Promised>
      </Hook0CardContentLines>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button
        class="primary"
        type="button"
        :disabled="!organizationId && !selected_organization_id"
        @click="goSecondStep"
        >Next</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
