<script setup lang="ts">
import Hook0Input from '@/components/Hook0Input.vue';
import { onMounted, ref } from 'vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { push } from 'notivue';
import { Problem } from '@/http.ts';
import { resetPassword } from '@/pages/user/UserService';
import { routes } from '@/routes.ts';
import router from '@/router.ts';

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
  let options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
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
  <h2 class="text-center lg:text-left lg:text-4xl text-3xl font-extrabold text-[#142850]">
    Reset your account's password
  </h2>
  <form class="mt-6" @submit.prevent="submit">
    <Hook0Input
      v-model="new_password"
      type="password"
      class="mt-2 sm:mt-1.5"
      placeholder="************"
      required
      autofocus
      label="New password"
    >
    </Hook0Input>
    <Hook0Input
      v-model="confirm_new_password"
      type="password"
      class="mt-2 sm:mt-1.5"
      placeholder="************"
      required
      label="Confirm new password"
    >
    </Hook0Input>

    <div class="flex items-center mt-2">
      <Hook0Button :to="{ name: routes.Login }">Back to login</Hook0Button>
    </div>

    <Hook0Button class="primary w-full mt-6 justify-center" submit>Set new password</Hook0Button>
  </form>
</template>
