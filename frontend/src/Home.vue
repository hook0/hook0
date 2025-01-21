<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import OrganizationSelector from '@/pages/OrganizationAndApplicationSelector.vue';
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Welcome to Hook0 👋</template>
      <template #subtitle
        >If your are new, take a look at our
        <a
          href="https://documentation.hook0.com/docs/getting-started"
          target="_blank"
          class="underline"
          >Getting Started</a
        >
        guide.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <OrganizationSelector :display-as-cards="true" />
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
  </Hook0Card>
</template>
