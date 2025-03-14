<script setup lang="ts">
import { onMounted, ref } from 'vue';

import { register } from '@/iam';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { AxiosError, AxiosResponse } from 'axios';
import { handleError, Problem } from '@/http.ts';
import { push } from 'notivue';
import { routes } from '@/routes.ts';
import router from '@/router.ts';
import VueTurnstile from 'vue-turnstile';
import { getInstanceConfig } from '@/utils/biscuit_auth';

const email = ref<string>('');
const firstName = ref<string>('');
const lastName = ref<string>('');
const password = ref<string>('');

const turnstile_site_key = ref<null | string>(null);
const turnstile_token = ref<string>('');

onMounted(async () => {
  const instanceConfig = await getInstanceConfig();
  if (instanceConfig.cloudflare_turnstile_site_key) {
    turnstile_site_key.value = instanceConfig.cloudflare_turnstile_site_key;
  }
});

async function submit() {
  await register(
    email.value,
    firstName.value,
    lastName.value,
    password.value,
    turnstile_token.value !== '' ? turnstile_token.value : undefined
  )
    .then(() => {
      push.success({
        title: 'Success',
        message:
          "You're successfully registered. You need to confirm your email address before using Hook0. Check your mailbox!",
        duration: 5000,
      });
      return router.push({ name: routes.Login });
    })
    .catch((err: AxiosError<AxiosResponse<Problem>>) => {
      let problem = handleError(err);
      let options = {
        title: problem.title,
        message: problem.detail,
        duration: 5000,
      };
      problem.status >= 500 ? push.error(options) : push.warning(options);
    });
}
</script>

<template>
  <div>
    <form @submit.prevent="submit">
      <Hook0Card class="shadow-2xl">
        <Hook0CardHeader>
          <template #header> Register </template>
          <template #subtitle>
            <div class="text-sm text-gray-500">
              Welcome to Hook0. Please enter your information to register and start using our
              services.
            </div>
          </template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0Input
            v-model="email"
            type="email"
            class="w-11/12 mx-auto mb-4 mt-4"
            placeholder="johndoe@example.com"
            required
            autofocus
            label="Email"
          >
          </Hook0Input>

          <Hook0Input
            v-model="firstName"
            type="text"
            class="w-11/12 mx-auto mb-4"
            placeholder="John"
            required
            label="First Name"
          >
          </Hook0Input>

          <Hook0Input
            v-model="lastName"
            type="text"
            class="w-11/12 mx-auto mb-4"
            placeholder="Doe"
            required
            label="Last Name"
          >
          </Hook0Input>

          <Hook0Input
            v-model="password"
            type="password"
            class="w-11/12 mx-auto mb-4"
            placeholder="************"
            required
            label="Password"
          >
          </Hook0Input>
        </Hook0CardContent>
        <Hook0CardFooter>
          <div v-if="turnstile_site_key">
            <VueTurnstile
              v-model="turnstile_token"
              :site-key="turnstile_site_key"
              size="flexible"
              action="registration"
            />
          </div>
          <Hook0Button class="secondary" :to="{ name: routes.Login }">Sign in</Hook0Button>
          <Hook0Button class="primary" submit>Register</Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </form>
  </div>
</template>
