<script setup lang="ts">
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Text from '@/components/Hook0Text.vue';
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Verify Your Email</template>
      <template #subtitle
        >We've sent a verification email to your address. Please check your inbox and confirm your
        email.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <div class="flex flex-col justify-center items-center">
        <Hook0Button class="primary my-4" type="button" href="mailto:support@hook0.com">
          Contact support
        </Hook0Button>
        <Hook0Text class="text-sm text-gray-600">
          If you didn’t receive the email, click the button above to contact support.
        </Hook0Text>
      </div>
    </Hook0CardContent>
  </Hook0Card>
</template>
