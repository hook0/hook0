<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { useRouter, useRoute } from 'vue-router';
import { routes } from '@/routes.ts';
import { ref, onMounted } from 'vue';
import { Problem, UUID } from '@/http';
import { Alert } from '@/components/Hook0Alert';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { useTracking } from '@/composables/useTracking';

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

const router = useRouter();
const route = useRoute();

// Analytics tracking
const { trackEvent } = useTracking();

const organizationId = ref<UUID | null>(null);
const applicationId = ref<UUID | null>(null);

function _load() {
  organizationId.value = route.params.organization_id as UUID;
  applicationId.value = route.params.application_id as UUID;
  if (!organizationId.value || !applicationId.value) {
    displayError({
      id: 'FieldsRequired',
      status: 400,
      title: 'Organization ID and Application ID are required',
      detail: 'Something went wrong. Please try again. If the problem persists, contact support.',
    });
  }
}
function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;

  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}

function cancel() {
  router.back();
}

function goToApplicationDashboard() {
  trackEvent('Tutorial', 'Complete');
  return router.push({
    name: routes.ApplicationsDashboard,
    params: { organization_id: organizationId.value as string },
  });
}

onMounted(() => {
  _load();
});
</script>

<template>
  <Hook0Card v-if="alert.visible">
    <Hook0CardContent>
      <Hook0Alert
        :type="alert.type"
        :title="alert.title"
        :description="alert.description"
      ></Hook0Alert>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button class="secondary" type="button" @click="cancel">Close</Hook0Button>
    </Hook0CardFooter>
  </Hook0Card>
  <Hook0Card v-else>
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
                href="https://documentation.hook0.com/discuss"
                target="_blank"
                class="underline"
                >discussions</Hook0Button
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
    <Hook0CardFooter>
      <Hook0Button class="primary" type="button" @click="goToApplicationDashboard"
        >🚀 Go To Your Application Dashboard</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
