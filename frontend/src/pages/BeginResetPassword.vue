<script setup lang="ts">
import Hook0Input from '@/components/Hook0Input.vue';
import { ref } from 'vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { push } from 'notivue';
import { AxiosError, AxiosResponse } from 'axios';
import { handleError, Problem } from '@/http.ts';
import { beginResetPassword } from '@/pages/user/UserService';
import { routes } from '@/routes.ts';

const email = ref('');

async function submit() {
  await beginResetPassword(email.value)
    .then(() => {
      push.success({
        title: 'Success',
        message: 'Email sent successfully. Please check your email to reset your password.',
        duration: 5000,
      });
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
    Ask for a password reset
  </h2>
  <form class="mt-6" @submit.prevent="submit">
    <Hook0Input
      v-model="email"
      type="email"
      class="mt-2 sm:mt-1.5"
      placeholder="johndoe@example.com"
      required
      autofocus
      label="Email"
    />

    <div class="flex items-center mt-2">
      <Hook0Button :to="{ name: routes.Login }">Back to login</Hook0Button>
    </div>

    <Hook0Button class="primary w-full mt-6 justify-center" submit
      >Send verification email</Hook0Button
    >
  </form>
</template>
