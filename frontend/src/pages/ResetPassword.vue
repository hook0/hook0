<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import { ref } from 'vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import { push } from 'notivue';
import { AxiosError, AxiosResponse } from 'axios';
import { handleError, Problem } from '@/http.ts';
import { resetPassword } from '@/pages/users/UsersServices.ts';
import { routes } from '@/routes.ts';
import router from '@/router.ts';

const token = ref('');
const new_password = ref('');
const confirm_new_password = ref('');

async function submit() {
  if (new_password.value !== confirm_new_password.value) {
    push.warning({
      title: 'Warning',
      message: 'Passwords do not match.',
      duration: 5000,
    });
    return;
  }
  await resetPassword(token.value, new_password.value)
    .then(() => {
      push.success({
        title: 'Success',
        message: 'Your password has been reset successfully. Please login.',
        duration: 5000,
      });
      return router.push(routes.Login);
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
  <form @submit="submit">
    <Hook0Card>
      <Hook0CardHeader>
        <template #header> Reset my account password </template>
        <template #subtitle>
          <div class="text-sm text-gray-500">
            For security reasons, we need to verify your identity. We've sent a mail to your email
            address. Please click on the link in the email to reset your password.
          </div>
        </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLine>
          <template #label> Email </template>
          <template #content>
            <div class="flex flex-row">
              <Hook0Input v-model="email" type="email" class="w-full" placeholder="Email" required>
              </Hook0Input>
            </div>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button class="primary" type="submit" @click="submit">Submit</Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>
  </form>
</template>
