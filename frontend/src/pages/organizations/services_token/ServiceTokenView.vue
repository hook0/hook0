<script setup lang="ts">
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { Problem, UUID } from '@/http';
import { attenuateBiscuit, getDeserializedBiscuit } from '@/utils/biscuit_auth.ts';
import { list } from '@/pages/organizations/applications/ApplicationService.ts';
import Hook0Button from '@/components/Hook0Button.vue';
import { push } from 'notivue';
import Hook0Error from '@/components/Hook0Error.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Select from '@/components/Hook0Select.vue';
import { Biscuit } from '@biscuit-auth/biscuit-wasm';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0Code from '@/components/Hook0Code.vue';

const route = useRoute();

const biscuit_token = ref<null | Biscuit>(null);
const organization_id = ref<null | UUID>(null);
const applications$ = ref<Promise<Array<{ label: string; value: UUID }>>>(Promise.resolve([]));

const selected_application_id = ref<null | UUID>(null);
const is_date_expiration_attenuation = ref(false);
const date_attenuation = ref<null | Date>(null);

const attenuated_biscuit = ref<null | Biscuit>(null);

function _forceLoad() {
  organization_id.value = route.params.organization_id as UUID;

  try {
    biscuit_token.value = getDeserializedBiscuit(route.params.biscuit_token as string);
  } catch (e) {
    console.log(e);
    push.error({
      title: 'Invalid biscuit token',
      message: 'The biscuit token is invalid',
      duration: 5000,
    });
    return;
  }

  applications$.value = list(organization_id.value)
    .then((applications) => [
      { label: '', value: '' },
      ...applications.map((a) => ({ label: a.name, value: a.application_id })),
    ])
    .catch((error) => {
      displayError(error as Problem);
      return [];
    });
}

function _load() {
  if (organization_id.value !== route.params.organization_id) {
    _forceLoad();
  }
}

function submit() {
  if (!biscuit_token.value) {
    push.error({
      title: 'Invalid biscuit token',
      message: 'The biscuit token is invalid',
      duration: 5000,
    });
    return;
  }

  if (!selected_application_id.value && !date_attenuation.value) {
    push.error({
      title: 'Invalid form',
      message: 'You must select an application or an expiration date',
      duration: 5000,
    });
    return;
  }

  attenuated_biscuit.value = attenuateBiscuit(
    biscuit_token.value as Biscuit,
    selected_application_id.value,
    date_attenuation.value
  );
  push.success({
    title: 'Service token generated',
    message: 'The service token has been generated',
    duration: 5000,
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
          <form @submit="submit">
            <Hook0CardContent>
              <Hook0CardContentLine>
                <template #label> Application </template>
                <template #content>
                  <Hook0Select
                    v-model="selected_application_id"
                    :options="applications"
                  ></Hook0Select>
                </template>
              </Hook0CardContentLine>
              <Hook0CardContentLine>
                <template #label> Do you want attenuate the token expiration date? </template>
                <template #content>
                  <input type="checkbox" v-model="is_date_expiration_attenuation" />
                </template>
              </Hook0CardContentLine>
              <Hook0CardContentLine v-if="is_date_expiration_attenuation">
                <template #label> Expiration date </template>
                <template #content>
                  <input type="date" v-model="date_attenuation" />
                </template>
              </Hook0CardContentLine>
            </Hook0CardContent>
            <Hook0CardFooter>
              <Hook0Button class="primary" type="submit" @click="submit">Generate</Hook0Button>
            </Hook0CardFooter>
          </form>
        </template>

        <!-- The "rejected" scoped slot will be used if there is an error -->
        <template #rejected="error">
          <Hook0Error :error="error"></Hook0Error>
        </template>
      </Promised>
    </Hook0Card>
    <Hook0Card v-if="attenuated_biscuit">
      <Hook0CardHeader>
        <template #header> Attenuated Token </template>
        <template #subtitle>
          <div class="text-sm text-gray-500">
            This is your attenuated token with your params. Don't share with anyone.
          </div>
        </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLines>
          <Hook0Code :code="attenuated_biscuit.toBase64()"></Hook0Code>
        </Hook0CardContentLines>
      </Hook0CardContent>
    </Hook0Card>
  </div>
</template>
