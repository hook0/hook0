<script setup lang="ts">
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { useRouter } from 'vue-router';
import { routes } from '@/routes.ts';
import { useTracking } from '@/composables/useTracking';

const router = useRouter();

// Analytics tracking
const { trackEvent } = useTracking();

function startTutorial() {
  trackEvent('Tutorial', 'Start');
  return router.push({
    name: routes.TutorialCreateOrganization,
  });
}

function skipTutorial() {
  trackEvent('Tutorial', 'Skip');
  return router.push({ name: routes.Home });
}
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Welcome to Hook0 🎉 - Let’s Get Started!</template>
      <template #subtitle>This tutorial will help you get started quickly with Hook0.</template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <img
              src="/illustration-tutorial.png"
              alt="Illustration"
              class="w-10/12 h-80 object-cover rounded-lg mx-auto block mb-4"
            />
            <p>
              <Hook0Text class="text-lg"> In this tutorial, you will be creating: </Hook0Text>
            </p>
            <ul class="list-disc">
              <li class="ml-4">an organization</li>
              <li class="ml-4">an application</li>
              <li class="ml-4">an event type</li>
              <li class="ml-4">a subscription</li>
              <li class="ml-4">an event</li>
            </ul>
            <p class="mt-5">
              <Hook0Text class="text-lg">
                Note that you can skip this, and do it by yourself
                <Hook0Button
                  target="_blank"
                  href="https://documentation.hook0.com/docs/getting-started"
                  >programmatically</Hook0Button
                >
                or using our UI.
              </Hook0Text>
            </p>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button class="secondary" type="button" @click="skipTutorial">Skip</Hook0Button>
      <Hook0Button class="primary shadow-md" type="button" @click="startTutorial"
        >🚀 Start Step 1: Create Your Organization</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
