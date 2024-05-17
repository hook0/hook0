<script setup lang="ts">
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { handleError, Problem, UUID } from '@/http';
import { attenuateBiscuit, getInstanceConfig, InstanceConfig } from '@/utils/biscuit_auth.ts';
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
import { ServiceToken, get } from '@/pages/organizations/services_token/ServicesTokenService.ts';
import { AxiosError, AxiosResponse } from 'axios';
import { isBefore } from 'date-fns';
import Hook0Text from '@/components/Hook0Text.vue';
const route = useRoute();

// Params references
const organization_id = ref<null | UUID>(null);
const service_token_id = ref<null | UUID>(null);

// Load references
const biscuit_public_key$ = ref<null | string>(null);
const service_token$ = ref<null | ServiceToken>(null);
const applications$ = ref<Promise<Array<{ label: string; value: UUID }>>>(Promise.resolve([]));

// Form references
const selected_application_id = ref<null | UUID>(null);
const is_date_expiration_attenuation = ref(false);
const date_attenuation = ref<null | string>(null);

// Attenuated token
const attenuated_biscuit = ref<null | Biscuit>(null);

function _forceLoad() {
  organization_id.value = route.params.organization_id as UUID;
  service_token_id.value = route.params.service_token_id as UUID;

  getInstanceConfig()
    .then((config: InstanceConfig) => {
      biscuit_public_key$.value = config.biscuit_public_key;
    })
    .catch((err: AxiosError<AxiosResponse<Problem>>) => {
      let problem = handleError(err);
      displayError(problem);
    });

  get(service_token_id.value, organization_id.value)
    .then((service_token) => {
      service_token$.value = service_token;
    })
    .catch((err: AxiosError<AxiosResponse<Problem>>) => {
      let problem = handleError(err);
      displayError(problem);
    });

  // Get organization applications and put them into applications$ references and add an empty option
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
  if (!biscuit_public_key$.value) {
    push.error({
      title: 'Something went wrong',
      message:
        'An error occurred while getting the biscuit public key from Hook0. If the problem persists, please contact support.',
      duration: 5000,
    });
    return;
  }

  if (!service_token$.value) {
    push.error({
      title: 'Invalid service token',
      message: 'The service token is invalid',
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

  if (date_attenuation.value && isBefore(date_attenuation.value, new Date())) {
    push.error({
      title: 'Invalid expiration date',
      message: 'The expiration date must be in the future',
      duration: 5000,
    });
    return;
  }

  try {
    attenuated_biscuit.value = attenuateBiscuit(
      service_token$.value?.biscuit,
      selected_application_id.value,
      date_attenuation.value ? new Date(date_attenuation.value) : null,
      biscuit_public_key$.value
    );
    push.success({
      title: 'Service token generated',
      message: 'The service token has been generated',
      duration: 5000,
    });
  } catch (e) {
    if (e instanceof Error) {
      push.error({
        title: 'Something went wrong',
        message: e.message,
        duration: 5000,
      });
    } else {
      push.error({
        title: 'Something went wrong',
        message: 'An error occurred while generating the service token',
        duration: 5000,
      });
    }
  }
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
            This is a service token link to your organization. Actually he got an unlimited
            expiration date and can be used to authenticated all your applications. If you want to
            restrict it to a specific application or to a specific expiration date, you can generate
            a new token. By attenuating the token, you will create a new token with the same rights
            but with a specific expiration date or linked to a specific application.
          </div>
        </template>
      </Hook0CardHeader>
      <Hook0CardContent v-if="service_token$">
        <Hook0CardContentLine>
          <template #label>
            Service Token

            <Hook0Text class="helpText mt-2 block">
              This is your actual service token, this token get every access in your organization.
              <strong>Don't share with anyone.</strong>
            </Hook0Text>

            <Hook0Text class="helpText mt-2 block"> </Hook0Text>
          </template>
          <template #content>
            <Hook0Code :code="service_token$.biscuit"></Hook0Code>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
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
                  <input v-model="is_date_expiration_attenuation" type="checkbox" />
                </template>
              </Hook0CardContentLine>
              <Hook0CardContentLine v-if="is_date_expiration_attenuation">
                <template #label> Expiration date </template>
                <template #content>
                  <input v-model="date_attenuation" type="datetime-local" />
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
