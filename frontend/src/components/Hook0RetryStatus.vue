<script setup lang="ts">
import { X } from 'lucide-vue-next';

import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Spinner from '@/components/Hook0Spinner.vue';
import Hook0Badge from '@/components/Hook0Badge.vue';

type Props = {
  isRetrying: boolean;
  message: string;
};

defineProps<Props>();
</script>

<template>
  <Hook0Stack direction="row" align="center" justify="center" gap="sm">
    <Hook0Badge v-if="isRetrying" display="trust" variant="primary">
      <template #icon>
        <Hook0Spinner :size="16" />
      </template>
      {{ message }}
    </Hook0Badge>
    <Hook0Badge v-else display="trust" variant="warning">
      <template #icon>
        <X :size="18" aria-hidden="true" />
      </template>
      {{ message }}
    </Hook0Badge>
  </Hook0Stack>
</template>

<style scoped>
/* Hook0* components handle all styling via variants */
</style>
