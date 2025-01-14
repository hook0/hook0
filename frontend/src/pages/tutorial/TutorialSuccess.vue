<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Button from '@/components/Hook0Button.vue';
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Congratulations on your first steps with Hook0 🎉</template>
      <template #subtitle
        >You have successfully set up your first application, created your first event type, and
        subscribed to your first event in Hook0. Join us on
        <Hook0Button href="https://discord.com/invite/hook0" target="_blank" class="underline"
          >Discord</Hook0Button
        >, and we’re also available on
        <Hook0Button href="https://github.com/hook0/hook0" target="_blank" class="underline"
          >GitHub</Hook0Button
        >.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Text>
              To share your feedback or suggestions, visit our
              <Hook0Button
                href="https://github.com/hook0/hook0/discussions"
                target="_blank"
                class="underline"
                >GitHub Discussions</Hook0Button
              >. Stay updated through our
              <Hook0Button
                href="https://documentation.hook0.com/changelog"
                target="_blank"
                class="underline"
                >Changelog</Hook0Button
              >
              and take the best out of Hook0 by learning more at our
              <Hook0Button
                href="https://documentation.hook0.com/docs/events"
                target="_blank"
                class="underline"
                >documentation</Hook0Button
              >.
            </Hook0Text>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
  </Hook0Card>
</template>
