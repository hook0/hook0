<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0SkeletonGroup from '@/components/Hook0SkeletonGroup.vue';
import Hook0Skeleton from '@/components/Hook0Skeleton.vue';

type Props = {
  lines?: number;
  showHeader?: boolean;
}

withDefaults(defineProps<Props>(), {
  lines: 3,
  showHeader: true,
});
</script>

<template>
  <Hook0Card aria-hidden="true">
    <Hook0CardHeader v-if="showHeader">
      <template #header>
        <Hook0Skeleton size="card-header" />
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0SkeletonGroup :count="lines" />
    </Hook0CardContent>
  </Hook0Card>
</template>
