<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';

import { useTracking } from '@/composables/useTracking';
import { useCelebration } from '@/composables/useCelebration';

import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import WizardStepLayout from '@/pages/tutorial/WizardStepLayout.vue';

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

onMounted(() => {
  celebrate(100);
  trackEvent('tutorial', 'complete');
});
</script>

<template>
  <WizardStepLayout
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
            href="https://documentation.hook0.com/docs/events"
            target="_blank"
          >
            <BookOpen :size="14" aria-hidden="true" />
            {{ t('tutorial.congrats.documentation') }}
          </Hook0Button>
        </template>
      </i18n-t>
    </Hook0Stack>

    <template #footer>
      <Hook0Button variant="primary" type="button" @click="emit('dismiss')">
        {{ t('tutorial.congrats.goToDashboard') }}
        <ArrowRight :size="16" aria-hidden="true" />
      </Hook0Button>
    </template>
  </WizardStepLayout>
</template>
