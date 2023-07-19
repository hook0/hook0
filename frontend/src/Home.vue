<script setup lang="ts">
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Hello 👋</template>
      <template #subtitle
        >Hook0 API is complete and production ready but the UI is still a work in progress...
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Text
              >Start your journey by selecting your
              <Hook0Text class="bold">organization</Hook0Text>
              on the left selector and then creating your first application.
            </Hook0Text>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
  </Hook0Card>
</template>
