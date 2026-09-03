<script setup lang="ts">
import { computed, onMounted, toRef } from 'vue';
import { useI18n } from 'vue-i18n';

import { useTracking } from '@/composables/useTracking';
import { useCelebration } from '@/composables/useCelebration';

import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import PlayInspector from '@/components/PlayInspector.vue';
import WizardStepLayout from '@/pages/tutorial/WizardStepLayout.vue';
import { useSubscriptionList } from '@/pages/organizations/applications/subscriptions/useSubscriptionQueries';
import { extractPlayToken } from '@/utils/playEndpoint';

import {
  PartyPopper,
  ArrowRight,
  MessageSquare,
  Github,
  BookOpen,
  Newspaper,
} from 'lucide-vue-next';

type Props = {
  organizationId: string;
  applicationId: string;
};

const props = defineProps<Props>();

const emit = defineEmits<{
  dismiss: [];
}>();

const { t } = useI18n();
const { trackEvent } = useTracking();
const { celebrate } = useCelebration();

// Alert for missing params
const alertVisible = computed(() => !props.organizationId || !props.applicationId);
const alertTitle = computed(() => (alertVisible.value ? t('tutorial.orgAppIdRequired') : ''));
const alertDescription = computed(() =>
  alertVisible.value ? t('tutorial.somethingWentWrong') : ''
);

// Hook0 Play (play.hook0.com) — same default and env override as the subscription step.
const PLAY_BASE = (import.meta.env.VITE_PLAY_ENDPOINT ?? '') || 'https://play.hook0.com';

// Close the loop on the subscription-step drop-off: when the tutorial's endpoint
// is a Play inbox, show the webhook it just received here — the real "aha" moment.
// The token is derived from the persisted subscription (not SPA state), so a reload of
// the success screen still finds it; a user's own endpoint yields null and this screen
// keeps its plain congrats content, since we cannot inspect an inbox we do not own.
const applicationId = toRef(props, 'applicationId');
const subscriptionsEnabled = computed(() => !alertVisible.value && !!props.applicationId);
const { data: subscriptions } = useSubscriptionList(applicationId, subscriptionsEnabled);

const playToken = computed<string | null>(() => {
  const subs = subscriptions.value ?? [];
  if (subs.length === 0) {
    return null;
  }
  // The tutorial just created the most recent subscription; an older one on the same
  // application must not win over it (mirrors the send-event step's selection).
  const newest = subs.reduce((a, b) => (b.created_at > a.created_at ? b : a));
  return extractPlayToken(PLAY_BASE, newest.target?.url ?? '');
});

onMounted(() => {
  celebrate(100);
  trackEvent('tutorial', 'complete');
});
</script>

<template>
  <WizardStepLayout
    data-test="tutorial-success-card"
    :title="t('tutorial.congrats.title')"
    :show-skip="false"
    @skip="emit('dismiss')"
  >
    <template #header-icon>
      <Hook0IconBadge variant="success" size="lg">
        <PartyPopper :size="20" aria-hidden="true" />
      </Hook0IconBadge>
    </template>

    <template v-if="alertVisible">
      <Hook0Alert type="warning" :title="alertTitle" :description="alertDescription" />
      <Hook0Button variant="secondary" type="button" @click="emit('dismiss')">
        {{ t('tutorial.close') }}
      </Hook0Button>
    </template>

    <Hook0Stack v-else direction="column" gap="lg">
      <div v-if="playToken" class="tutorial-success__delivered" data-test="tutorial-success-play">
        <span class="tutorial-success__delivered-title">{{
          t('tutorial.congrats.webhookDelivered')
        }}</span>
        <PlayInspector :base="PLAY_BASE" :token="playToken" />
      </div>

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
            href="https://documentation.hook0.com/concepts/events"
            target="_blank"
          >
            <BookOpen :size="14" aria-hidden="true" />
            {{ t('tutorial.congrats.documentation') }}
          </Hook0Button>
        </template>
      </i18n-t>
    </Hook0Stack>

    <template #footer>
      <Hook0Button
        variant="primary"
        type="button"
        data-test="tutorial-success-dashboard-button"
        @click="emit('dismiss')"
      >
        {{ t('tutorial.congrats.goToDashboard') }}
        <ArrowRight :size="16" aria-hidden="true" />
      </Hook0Button>
    </template>
  </WizardStepLayout>
</template>

<style scoped>
.tutorial-success__delivered {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.tutorial-success__delivered-title {
  font-size: 0.875rem;
  font-weight: 700;
  color: var(--color-text-primary);
}
</style>
