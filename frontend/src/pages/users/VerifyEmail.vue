<script setup lang="ts">
import * as UserService from './UsersServices.ts';
import { Problem } from '@/http.ts';
import { onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import router from '@/router.ts';
import { routes } from '@/routes.ts';
import { push } from 'notivue';
import { Alert } from '@/components/Hook0Alert.ts';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';

const route = useRoute();

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;

  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}

function displaySuccess() {
  push.success({
    title: 'Email verified',
    message: 'You are successfully verified.',
    duration: 5000,
  });
}

async function _onLoad() {
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
  await UserService.verifyEmail(token)
    .then(() => {
      displaySuccess();
      return router.push(routes.Login);
    })
    .catch(displayError);
}

onMounted(() => {
  _onLoad().catch(console.error);
});
</script>

<template>
  <Hook0CardContent v-if="alert.visible">
    <Hook0Alert
      :type="alert.type"
      :title="alert.title"
      :description="alert.description"
    ></Hook0Alert>
  </Hook0CardContent>
</template>
