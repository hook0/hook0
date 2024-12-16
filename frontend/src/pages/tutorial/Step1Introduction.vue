<script setup lang="ts">
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0ProgressBar from '@/components/Hook0ProgressBar.vue';
import { ref } from 'vue';
import FirstStep from '@/pages/tutorial/Step2CreateOrganization.vue';

const isFirstStep = ref(false);
function startTutorial() {
  isFirstStep.value = true;
}
</script>

<template>
  <FirstStep v-if="isFirstStep" />
  <Hook0Card v-else>
    <Hook0CardHeader>
      <template #header>Welcome to Hook0 🎉 - Let’s Get Started!</template>
      <template #subtitle>This tutorial will help you get started quickly with us.</template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0ProgressBar
              actual="1"
              :items="[
                { description: 'Introduction' },
                { description: 'Create Your Organization' },
                { description: 'Create Your Application' },
                { description: 'Create Your Event Type' },
                { description: 'Configure Your Subscription' },
                { description: 'Send Your First Event' },
              ]"
              class="mb-14"
            />
            <img
              src="../../../dist/assets/illustration-tutorial.png"
              alt="Illustration"
              class="w-10/12 h-80 object-cover rounded-lg mx-auto block mb-4"
            />
            <p class="text-center text-gray-600 mb-14">
              Discover Hook0's: easily create and manage organizations, applications, event types,
              and webhooks.
            </p>
            <Hook0Text class="text-lg">
              Hook0 helps you easily manage applications and APIs with our intuitive tools. Prefer
              APIs? Explore our
              <Hook0Button
                target="_blank"
                href="https://documentation.hook0.com/docs/getting-started"
                >comprehensive documentation</Hook0Button
              >
              to configure everything programmatically.
            </Hook0Text>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button class="primary shadow-md" type="button" @click="startTutorial"
        >🚀 Start Step 2: Create Your Organization</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
