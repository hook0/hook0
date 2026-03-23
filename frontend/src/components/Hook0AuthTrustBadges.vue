<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Shield, CheckCircle } from 'lucide-vue-next';
import Hook0Badge from '@/components/Hook0Badge.vue';

const { t } = useI18n();

type Props = {
  badges?: string[];
};

withDefaults(defineProps<Props>(), {
  badges: () => ['auth.trust.openSource', 'auth.trust.uptime', 'auth.trust.gdpr'],
});
</script>

<template>
  <div class="trust-badges">
    <Hook0Badge v-for="badge in badges" :key="badge" display="trust" variant="success">
      <template #icon>
        <Shield v-if="badge === 'auth.trust.openSource'" :size="20" aria-hidden="true" />
        <CheckCircle v-else :size="20" aria-hidden="true" />
      </template>
      {{ t(badge) }}
    </Hook0Badge>
  </div>
</template>

<style scoped>
.trust-badges {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  gap: 1rem;
}

@media (max-width: 640px) {
  .trust-badges {
    flex-direction: column;
    align-items: flex-start;
    width: fit-content;
    margin: 0 auto;
  }
}
</style>
