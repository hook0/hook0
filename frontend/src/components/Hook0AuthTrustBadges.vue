<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Shield, CheckCircle } from 'lucide-vue-next';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';

const { t } = useI18n();

interface Props {
  badges?: string[];
}

withDefaults(defineProps<Props>(), {
  badges: () => ['auth.trust.openSource', 'auth.trust.uptime', 'auth.trust.gdpr'],
});
</script>

<template>
  <Hook0Stack align="center" justify="center" gap="lg" wrap responsive>
    <Hook0Badge v-for="badge in badges" :key="badge" display="trust" variant="success">
      <template #icon>
        <Shield v-if="badge === 'auth.trust.openSource'" :size="20" aria-hidden="true" />
        <CheckCircle v-else :size="20" aria-hidden="true" />
      </template>
      {{ t(badge) }}
    </Hook0Badge>
  </Hook0Stack>
</template>
