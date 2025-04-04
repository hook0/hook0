<script setup lang="ts">
import { ref } from 'vue';

import { login } from '@/iam';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
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
  <form @submit.prevent="submit">
    <Hook0Card class="shadow-2xl">
      <Hook0CardHeader>
        <template #header> Login </template>
        <template #subtitle>
          <div class="text-sm text-gray-500">Please enter your email and password to login.</div>
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
          v-model="password"
          type="password"
          class="w-11/12 mx-auto"
          placeholder="*********"
          required
          label="Password"
        >
        </Hook0Input>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button class="secondary" :to="{ name: routes.BeginResetPassword }"
          >Forgot password?</Hook0Button
        >
        <Hook0Button class="secondary" :to="{ name: routes.Register }">Sign up</Hook0Button>
        <Hook0Button class="primary" submit>Login</Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>
  </form>
</template>
