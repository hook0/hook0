<script setup lang="ts">
import { ref } from 'vue';

import { login } from '@/iam';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { handleError, Problem } from '@/http.ts';
import { AxiosError, AxiosResponse } from 'axios';
import { useRouter } from 'vue-router';
import { routes } from '@/routes.ts';
import { push } from 'notivue';
import * as OrganizationService from './organizations/OrganizationService';
import * as ApplicationService from './organizations/applications/ApplicationService';

const router = useRouter();

const email = ref<string>('');
const password = ref<string>('');

async function submit() {
  await login(email.value, password.value)
    .then(async () => {
      push.success({
        title: 'Success',
        message: 'You have successfully logged in.',
        duration: 5000,
      });

      const organizations = await OrganizationService.list();

      if (organizations.length < 1) {
        return router.push({ name: routes.Tutorial });
      } else if (organizations.length === 1) {
        let applications = await ApplicationService.list(organizations[0].organization_id);

        if (applications.length < 1) {
          return router.push({ name: routes.Tutorial });
        } else {
          return router.push({ name: routes.Home });
        }
      } else {
        return router.push({ name: routes.Home });
      }
    })
    .catch((err: AxiosError<AxiosResponse<Problem>>) => {
      let problem = handleError(err);
      displayError(problem);
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
</script>

<template>
  <h2 class="text-center lg:text-left lg:text-4xl text-3xl font-extrabold text-[#142850]">
    Welcome Back to Hook0!
  </h2>
  <form class="mt-6" @submit.prevent="submit">
    <Hook0Input
      v-model="email"
      type="email"
      required
      label="Email"
      class="mt-2 sm:mt-1.5"
      placeholder="johndoe@example.com"
      autofocus
    />
    <Hook0Input
      id="password"
      v-model="password"
      type="password"
      required
      label="Password"
      class="mt-2 sm:mt-1.5"
      placeholder="*********"
    />

    <div class="flex items-center mt-2">
      <Hook0Button :to="{ name: routes.BeginResetPassword }"> Forgot password? </Hook0Button>
    </div>

    <Hook0Button class="primary w-full mt-6 justify-center" submit>Login</Hook0Button>

    <div class="other-links pt-3 text-sm">
      Don't have an account?
      <Hook0Button
        class="rounded font-medium focus:ring-2 transition-colors duration-75 focus:outline-none"
        :to="{ name: routes.Register }"
      >
        Sign up
      </Hook0Button>
    </div>
  </form>
</template>
