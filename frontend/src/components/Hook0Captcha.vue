<script setup lang="ts">
import { onMounted, ref } from 'vue';
import VueTurnstile from 'vue-turnstile';
import { getInstanceConfig } from '@/utils/instance-config';

type CaptchaTheme = 'auto' | 'light' | 'dark';
type CaptchaSize = 'normal' | 'flexible' | 'compact';

type Props = {
  action?: string;
  theme?: CaptchaTheme;
  size?: CaptchaSize;
};

withDefaults(defineProps<Props>(), {
  action: 'form',
  theme: 'auto',
  size: 'flexible',
});

const token = defineModel<string>({ default: '' });

const siteKey = ref<string | null>(null);

onMounted(() => {
  getInstanceConfig()
    .then((instanceConfig) => {
      if (instanceConfig.cloudflare_turnstile_site_key) {
        siteKey.value = instanceConfig.cloudflare_turnstile_site_key;
      }
    })
    .catch(console.error);
});
</script>

<template>
  <div v-if="siteKey" class="hook0-captcha">
    <VueTurnstile
      v-model="token"
      :site-key="siteKey"
      :size="size"
      :action="action"
      :theme="theme"
    />
  </div>
</template>

<style scoped>
.hook0-captcha {
  display: flex;
  justify-content: center;
}
</style>
