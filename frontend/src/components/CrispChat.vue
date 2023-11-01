<script setup lang="ts">
import { onMounted } from 'vue';
import { Crisp } from 'crisp-sdk-web';

interface Props {
  email: string;
  name: string;
}
const props = defineProps<Props>();
const websiteId = import.meta.env.VITE_CRISP_WEBSITE_ID;

onMounted(() => {
  if (websiteId) {
    Crisp.configure(websiteId);
    Crisp.user.setEmail(props.email);
    Crisp.user.setNickname(props.name);
  }
});
</script>

<template><!-- eslint-disable-line vue/valid-template-root --></template>
