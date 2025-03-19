<script setup lang="ts">
import { onMounted, ref } from 'vue';

import { register } from '@/iam';
import Hook0Input from '@/components/Hook0Input.vue';
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
const acceptTerms = ref<boolean>(false);

const turnstile_site_key = ref<null | string>(null);
const turnstile_token = ref<string>('');

onMounted(async () => {
  const instanceConfig = await getInstanceConfig();
  if (instanceConfig.cloudflare_turnstile_site_key) {
    turnstile_site_key.value = instanceConfig.cloudflare_turnstile_site_key;
  }
});

async function submit() {
  if (!acceptTerms.value) {
    push.warning({ title: 'Warning', message: 'You must accept the terms of service.' });
    return;
  }
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
  <h2 class="text-center lg:text-left lg:text-4xl text-3xl font-extrabold text-[#142850]">
    Set up your Free Webhooks account
  </h2>
  <form class="mt-6" @submit.prevent="submit">
    <Hook0Input
      v-model="firstName"
      type="text"
      required
      label="First Name"
      class="mt-2 sm:mt-1.5"
    />
    <Hook0Input v-model="lastName" type="text" required label="Last Name" class="mt-2 sm:mt-1.5" />
    <Hook0Input v-model="email" type="email" required label="Email" class="mt-2 sm:mt-1.5" />
    <Hook0Input
      v-model="password"
      type="password"
      required
      label="Password"
      class="mt-2 sm:mt-1.5"
    />

    <div class="flex items-center mt-2">
      <Hook0Input v-model="acceptTerms" type="checkbox" class="h-4 w-4" tabindex="9" required />
      <label class="ml-2 text-sm">
        I accept Hook0
        <Hook0Button target="_blank" href="https://hook0.com/terms" class="font-medium"
          >terms of service</Hook0Button
        >.
      </label>
    </div>

    <div v-if="turnstile_site_key" class="mt-4">
      <VueTurnstile
        v-model="turnstile_token"
        :site-key="turnstile_site_key"
        size="flexible"
        action="registration"
      />
    </div>

    <Hook0Button class="primary w-full mt-6 justify-center" submit>Sign Up</Hook0Button>

    <div class="other-links pt-3 text-sm">
      Already have an account?
      <Hook0Button
        class="rounded font-medium focus:ring-2 transition-colors duration-75 focus:outline-none"
        :to="{ name: routes.Login }"
      >
        Sign in
      </Hook0Button>
    </div>
  </form>
</template>
