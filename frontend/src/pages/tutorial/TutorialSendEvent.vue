<script setup lang="ts">
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { onMounted, ref } from 'vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { useRoute, useRouter } from 'vue-router';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { Alert } from '@/components/Hook0Alert.ts';
import { Problem, UUID } from '@/http.ts';
import { routes } from '@/routes.ts';
import { push } from 'notivue';
import EventsList from '@/pages/organizations/applications/events/EventsList.vue';
import Hook0ProgressBar from '@/components/Hook0ProgressBar.vue';
import { progressItems } from '@/pages/tutorial/TutorialService';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import { FileText, ArrowRight, X } from 'lucide-vue-next';
import Hook0Badge from '@/components/Hook0Badge.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import { useCelebration } from '@/composables/useCelebration';

const { t } = useI18n();
const router = useRouter();
const route = useRoute();

// Analytics tracking
const { trackEvent } = useTracking();

const disabled_button = ref<boolean>(true);

const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

const organizationId = ref<UUID | null>(null);
const applicationId = ref<UUID | null>(null);

function _load() {
  organizationId.value = route.params.organization_id as UUID;
  applicationId.value = route.params.application_id as UUID;
  if (!organizationId.value || !applicationId.value) {
    displayError({
      id: 'FieldsRequired',
      status: 400,
      title: t('tutorial.orgAppIdRequired'),
      detail: t('tutorial.somethingWentWrong'),
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

const { celebrate } = useCelebration();

function celebrateStep() {
  celebrate();
}

function back_to_application() {
  trackEvent('tutorial', 'step-complete', 'send-event');
  push.success({
    title: t('tutorial.step5.eventSent'),
    message: t('tutorial.step5.eventSentMessage'),
    duration: 5000,
  });
  celebrateStep();
  return router.push({
    name: routes.TutorialSuccess,
    params: {
      organization_id: organizationId.value,
      application_id: applicationId.value,
    },
  });
}

onMounted(() => {
  _load();
});
</script>

<template>
  <Hook0Stack direction="column" gap="none">
    <Hook0CardContent v-if="alert.visible">
      <Hook0Alert
        :type="alert.type"
        :title="alert.title"
        :description="alert.description"
      ></Hook0Alert>
      <Hook0Button variant="secondary" type="button" @click="cancel">{{
        t('tutorial.close')
      }}</Hook0Button>
    </Hook0CardContent>
    <Hook0Card v-else>
      <Hook0CardHeader>
        <template #header>
          <Hook0Stack direction="row" align="center" gap="sm">
            <Hook0Badge display="step" variant="primary">5</Hook0Badge>
            <Hook0Stack direction="row" align="center" gap="none">
              {{ t('tutorial.step5.title') }}
            </Hook0Stack>
          </Hook0Stack>
        </template>
        <template #subtitle>{{ t('tutorial.step5.subtitle') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Stack direction="column" gap="lg">
              <Hook0ProgressBar :current="5" :items="progressItems" />
              <Hook0Stack
                v-if="organizationId && applicationId && disabled_button"
                direction="column"
                gap="md"
              >
                <Hook0Stack direction="row" align="center" gap="sm">
                  <Hook0IconBadge variant="primary">
                    <FileText :size="18" aria-hidden="true" />
                  </Hook0IconBadge>
                  <Hook0Stack direction="row" align="center" gap="none">
                    {{ t('tutorial.step5.title') }}
                  </Hook0Stack>
                </Hook0Stack>
                <EventsList :tutorial-mode="true" @tutorial-event-sent="back_to_application" />
              </Hook0Stack>
            </Hook0Stack>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button
          variant="secondary"
          type="button"
          @click="
            router.push({
              name: routes.ApplicationsDashboard,
              params: { organization_id: organizationId, application_id: applicationId },
            })
          "
        >
          <X :size="16" />
          {{ t('tutorial.step5.skip') }}
        </Hook0Button>
        <Hook0Button
          v-if="!disabled_button"
          variant="primary"
          type="button"
          :disabled="disabled_button"
          @click="back_to_application"
        >
          {{ t('tutorial.step5.backToApplication') }}
          <ArrowRight :size="16" />
        </Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>
  </Hook0Stack>
</template>

<style scoped>
/* No custom styles - using Hook0* components only */
</style>
