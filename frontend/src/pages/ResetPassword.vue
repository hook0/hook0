<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import { onMounted, ref } from 'vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import { push } from 'notivue';
import { Problem } from '@/http.ts';
import { resetPassword } from '@/pages/user/UserService';
import { routes } from '@/routes.ts';
import router from '@/router.ts';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

let token: string = '';
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
  await resetPassword(token, new_password.value)
    .then(() => {
      push.success({
        title: 'Success',
        message: 'Your password has been reset successfully. Please login.',
        duration: 5000,
      });
      return router.push({ name: routes.Login });
    })
    .catch((err: Problem) => {
      displayError(err);
    });
}

function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;

  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}

function _onLoad() {
  token = router.currentRoute.value.query.token as string;
  if (!token) {
    displayError({
      id: 'InvalidToken',
      status: 400,
      title: 'Invalid token',
      detail: 'Token is required to reset password',
    });
    return;
  }
}

onMounted(() => {
  _onLoad();
});
</script>

<template>
  <Hook0Card v-if="alert.visible">
    <Hook0CardContent>
      <Hook0Alert
        :type="alert.type"
        :title="alert.title"
        :description="alert.description"
      ></Hook0Alert>
    </Hook0CardContent>
  </Hook0Card>
  <form v-else @submit.prevent="submit">
    <Hook0Card>
      <Hook0CardHeader>
        <template #header> Set new account's password </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0Input
          v-model="new_password"
          type="password"
          class="w-11/12 mx-auto mb-4 mt-4"
          placeholder="************"
          required
          autofocus
          label="New password"
        >
        </Hook0Input>
        <Hook0Input
          v-model="confirm_new_password"
          type="password"
          class="w-11/12 mx-auto mb-4"
          placeholder="************"
          required
          label="Confirm new password"
        >
        </Hook0Input>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button class="primary" submit>Set new password</Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>
  </form>
</template>
