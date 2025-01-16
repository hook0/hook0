<script setup lang="ts">
import { useRoute, useRouter } from 'vue-router';
import { onMounted, onUpdated, ref, defineProps, defineEmits } from 'vue';

import { Problem, UUID } from '@/http';
import * as ApplicationService from './ApplicationService';
import { Application } from './ApplicationService';
import ApplicationsRemove from '@/pages/organizations/applications/ApplicationsRemove.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import { push } from 'notivue';

const router = useRouter();
const route = useRoute();

const isNew = ref(true);
const application_id = ref<UUID | null>(null);
const application = ref({
  name: '',
});

interface Props {
  tutorialMode?: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits(['tutorial-application-created']);

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

  if (isNew.value) {
    ApplicationService.create({
      name: application.value.name,
      organization_id: route.params.organization_id as string,
    }).then((_resp) => {
      if (props.tutorialMode) {
        emit('tutorial-application-created', _resp.application_id);
      } else {
        cancel();
      }
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
            An application is an isolated environment that contains everything webhook-related.
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

        <Hook0CardFooter>
          <Hook0Button v-if="!tutorialMode" class="secondary" type="button" @click="cancel()"
            >Cancel</Hook0Button
          >
          <Hook0Button
            v-if="!tutorialMode"
            class="primary"
            type="button"
            :disabled="!application.name"
            @click="upsert($event)"
            >{{ isNew ? 'Create' : 'Update' }}
          </Hook0Button>

          <Hook0Button
            v-else
            class="primary"
            type="button"
            :disabled="!application.name"
            tooltip="ℹ️ To continue, you need to add a name for your application."
            @click="upsert($event)"
            >Create Your First Application 🎉
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
