<script setup lang="ts">
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { ref } from 'vue';
import FirstStep from '@/pages/tutorial/FirstStep.vue';

const isFirstStep = ref(false);
function startTutorial() {
  isFirstStep.value = true;
}
</script>

<template>
  <FirstStep v-if="isFirstStep" />
  <Hook0Card v-else>
    <Hook0CardHeader>
      <template #header>Welcome to Hook0</template>
      <template #subtitle
        >Hook0 is a platform that allows you to create and manage your applications and
        environments.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Text> Some text here ... </Hook0Text>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button class="primary" type="button" @click="startTutorial"
        >Start the tutorial ! 🎉🎉</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
