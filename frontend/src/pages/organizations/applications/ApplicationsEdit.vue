<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { onMounted, onUpdated, ref } from 'vue';

import { isAxiosError, Problem, UUID } from '@/http';
import * as ApplicationService from './ApplicationService';
import { Application } from './ApplicationService';
import { Alert } from '@/components/Hook0Alert';
import Hook0Alert from '@/components/Hook0Alert.vue';
import ApplicationsRemove from '@/pages/organizations/applications/ApplicationsRemove.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Input from '@/components/Hook0Input.vue';

const router = useRouter();
const route = useRoute();

const isNew = ref(true);
const application_id = ref<UUID | null>(null);
const application = ref({
  name: '',
});
const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

function _load() {
  if (application_id.value !== route.params.application_id) {
    application_id.value = route.params.application_id as UUID;
    isNew.value = !application_id.value;

    if (!isNew.value) {
      ApplicationService.get(application_id.value)
        .then((app: Application) => {
          application.value.name = app.name;
        })
        .catch(displayError);
    }
  }
}

function cancel() {
  router.back();
}

function upsert(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  alert.value.visible = false; // reset alert

  if (isNew.value) {
    ApplicationService.create({
      name: application.value.name,
      organization_id: route.params.organization_id as string,
    }).then((_resp) => {
      cancel();
    }, displayError);
    return;
  }

  ApplicationService.update(application_id.value as UUID, {
    name: application.value.name,
    organization_id: route.params.organization_id as string,
  }).then((_resp) => {
    cancel();
  }, displayError);
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

onMounted(() => {
  _load();
});

onUpdated(() => {
  _load();
});
</script>

<template>
  <div>
    <form @submit="upsert">
      <Hook0Card>
        <Hook0CardHeader>
          <template v-if="isNew" #header> Create new application </template>
          <template v-else #header> Edit application </template>
          <template #subtitle>
            An application emit events that are consumed by customers through webhooks
          </template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label> Application Name </template>
            <template #content>
              <Hook0Input
                v-model="application.name"
                type="text"
                placeholder="my awesome api - production"
                required
              >
                <template #helpText
                  >Name of your company's product or API. Don't forget also to specify the
                  environment, for example: "facebook-production"
                </template>
              </Hook0Input>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>

        <Hook0CardContent v-if="alert.visible">
          <Hook0Alert
            :type="alert.type"
            :title="alert.title"
            :description="alert.description"
          ></Hook0Alert>
        </Hook0CardContent>
        <Hook0CardFooter>
          <Hook0Button class="secondary" type="button" @click="cancel()">Cancel</Hook0Button>
          <Hook0Button class="primary" type="button" @click="upsert($event)"
            >{{ isNew ? 'Create' : 'Update' }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </form>

    <ApplicationsRemove
      v-if="!isNew && application_id"
      :application-id="application_id"
      :application-name="application.name"
    ></ApplicationsRemove>
  </div>
</template>
