<script setup lang="ts">
import { ref } from 'vue';

import { login, refresh } from '@/iam';

const email = ref<string>('');
const password = ref<string>('');

async function submit() {
  await login(email.value, password.value);
}

async function refr() {
  await refresh();
}
</script>

<template>
  <h1>Login</h1>
  <form @submit.prevent="submit">
    <input v-model="email" type="text" />
    <input v-model="password" type="password" />
    <button type="submit">Sign In</button>
  </form>

  <button @click="refr">Refresh</button>
</template>
