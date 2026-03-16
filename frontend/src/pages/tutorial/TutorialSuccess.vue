<script setup lang="ts">
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { useRouter, useRoute } from 'vue-router';
import { routes } from '@/routes.ts';
import { ref, onMounted } from 'vue';
import { Problem, UUID } from '@/http';
import { Alert } from '@/components/Hook0Alert';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import {
  PartyPopper,
  ArrowRight,
  MessageSquare,
  Github,
  BookOpen,
  Newspaper,
} from 'lucide-vue-next';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import { useCelebration } from '@/composables/useCelebration';

const { t } = useI18n();

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

function goToApplicationDashboard() {
  trackEvent('tutorial', 'complete');
  return router.push({
    name: routes.ApplicationsDashboard,
    params: { organization_id: organizationId.value as string },
  });
}

const { celebrate } = useCelebration();

function celebrateSuccess() {
  celebrate(100);
}

onMounted(() => {
  _load();
  celebrateSuccess();
});
</script>

<template>
  <Hook0Stack direction="column" gap="none">
    <Hook0Card v-if="alert.visible">
      <Hook0CardContent>
        <Hook0Alert
          :type="alert.type"
          :title="alert.title"
          :description="alert.description"
        ></Hook0Alert>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button variant="secondary" type="button" @click="cancel">{{
          t('tutorial.close')
        }}</Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>
    <Hook0Card v-else>
      <Hook0CardHeader>
        <template #header>
          <Hook0Stack direction="row" align="center" gap="sm">
            <Hook0IconBadge variant="success" size="lg">
              <PartyPopper :size="20" aria-hidden="true" />
            </Hook0IconBadge>
            <Hook0Stack direction="row" align="center" gap="none">
              {{ t('tutorial.congrats.title') }}
            </Hook0Stack>
          </Hook0Stack>
        </template>
        <template #subtitle>
          <i18n-t keypath="tutorial.congrats.subtitle" tag="span">
            <template #discord>
              <Hook0Button variant="link" href="https://discord.com/invite/hook0" target="_blank">
                <MessageSquare :size="14" aria-hidden="true" />
                Discord
              </Hook0Button>
            </template>
            <template #github>
              <Hook0Button variant="link" href="https://github.com/hook0/hook0" target="_blank">
                <Github :size="14" aria-hidden="true" />
                GitHub
              </Hook0Button>
            </template>
          </i18n-t>
        </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Stack direction="column" gap="md">
              <i18n-t keypath="tutorial.congrats.feedback" tag="span">
                <template #discussions>
                  <Hook0Button
                    variant="link"
                    href="https://documentation.hook0.com/discuss"
                    target="_blank"
                  >
                    <BookOpen :size="14" aria-hidden="true" />
                    {{ t('tutorial.congrats.discussions') }}
                  </Hook0Button>
                </template>
                <template #changelog>
                  <Hook0Button
                    variant="link"
                    href="https://documentation.hook0.com/changelog"
                    target="_blank"
                  >
                    <Newspaper :size="14" aria-hidden="true" />
                    {{ t('tutorial.congrats.changelog') }}
                  </Hook0Button>
                </template>
                <template #documentation>
                  <Hook0Button
                    variant="link"
                    href="https://documentation.hook0.com/docs/events"
                    target="_blank"
                  >
                    <BookOpen :size="14" aria-hidden="true" />
                    {{ t('tutorial.congrats.documentation') }}
                  </Hook0Button>
                </template>
              </i18n-t>
            </Hook0Stack>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button variant="primary" type="button" @click="goToApplicationDashboard">
          {{ t('tutorial.congrats.goToDashboard') }}
          <ArrowRight :size="16" />
        </Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>
  </Hook0Stack>
</template>

<style scoped>
/* No custom styles - using Hook0* components only */
</style>
