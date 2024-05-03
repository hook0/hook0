<script setup lang="ts">
import * as VerifyEmailService from './VerifyEmailService';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import { Problem } from '@/http.ts';
import { onMounted } from 'vue';
import { useRoute } from 'vue-router';
import router from '@/router.ts';
import { routes } from '@/routes.ts';
import { push } from 'notivue';

const route = useRoute();

function displayError(err: Problem) {
  console.error(err);
  let options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}

function displaySuccess() {
  push.success({
    title: 'Email verified',
    message: 'You will be redirected to login page in 3 seconds',
    duration: 3000,
  });
}

function _onLoad() {
  const token = route.query.token as string;
  if (!token) {
    displayError({
      id: 'InvalidToken',
      status: 400,
      title: 'Invalid token',
      detail: 'Token is required to verify email',
    });
    return;
  }
  VerifyEmailService.verifyEmail(token)
    .then(() => {
      setInterval(() => {
        displaySuccess();
        setTimeout(() => {
          void router.push(routes.Login);
        }, 5000);
      }, 1000);
    })
    .catch(displayError);
}

onMounted(() => {
  _onLoad();
});
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Verify email</template>
      <template #subtitle>If request failed, retry in a few minutes or contact support</template>
    </Hook0CardHeader>
  </Hook0Card>
</template>
