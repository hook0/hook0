<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import { ref } from 'vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
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
  <form @submit.prevent="submit">
    <Hook0Card>
      <Hook0CardHeader>
        <template #header>Reset your account's password</template>
        <template #subtitle>
          <div class="text-sm text-gray-500">
            You will receive an email with a verification link that will allow you to set a new
            pasword.
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
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button class="secondary" :to="{ name: routes.Login }">Back to login</Hook0Button>
        <Hook0Button class="primary" submit>Send verification email</Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>
  </form>
</template>
