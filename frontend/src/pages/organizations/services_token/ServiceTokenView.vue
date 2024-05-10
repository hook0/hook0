<script setup lang="ts">
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { UUID } from '@/http';
import { getDeserializedBiscuit } from '@/utils/biscuit_auth.ts';
import { list, Application } from '@/pages/organizations/applications/ApplicationService.ts';
import Hook0Button from '@/components/Hook0Button.vue';
import { push } from 'notivue';
import Hook0Error from '@/components/Hook0Error.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Select from '@/components/Hook0Select.vue';

const route = useRoute();

const biscuit_token = ref<null | string>(null);
const organization_id = ref<null | UUID>(null);

const applications$ = ref<null | Promise<Array<Application>>>(null);

function _forceLoad() {
  organization_id.value = route.params.organization_id as UUID;

  let biscuit = getDeserializedBiscuit(route.params.biscuit_token as string);
  if (typeof biscuit === 'object') {
    push.error(biscuit);
    return;
  }

  biscuit_token.value = biscuit;
  applications$.value = list(organization_id.value);
  console.log(applications$.value);
}

function _load() {
  if (organization_id.value !== route.params.organization_id) {
    _forceLoad();
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
    <Hook0Card>
      <Hook0CardHeader>
        <template #header> Service Token </template>
        <template #subtitle>
          <div class="text-sm text-gray-500">
            {{ biscuit_token }}
          </div>
        </template>
      </Hook0CardHeader>
      <Promised :promise="applications$">
        <!-- Use the "pending" slot to display a loading message -->
        <template #pending>
          <Hook0Loader></Hook0Loader>
        </template>
        <!-- The default scoped slot will be used as the result -->
        <template #default="applications">
          <Hook0CardContent>
            <Hook0CardContentLine>
              <template #label> Application </template>
              <template #content>
                <!-- Hook0Select or Hook0Input type checkbox -->
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>
          <Hook0CardFooter>
            <Hook0Button class="primary" type="submit">Generate</Hook0Button>
          </Hook0CardFooter>
        </template>

        <!-- The "rejected" scoped slot will be used if there is an error -->
        <template #rejected="error">
          <Hook0Error :error="error"></Hook0Error>
        </template>
      </Promised>
    </Hook0Card>
  </div>
</template>
