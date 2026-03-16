<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import {
  Rocket,
  Building2,
  AppWindow,
  FolderTree,
  Link,
  FileText,
  ArrowRight,
  X,
} from 'lucide-vue-next';

import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Illustration from '@/components/Hook0Illustration.vue';
import Hook0Button from '@/components/Hook0Button.vue';

const { t } = useI18n();

defineEmits<{
  start: [];
  skip: [];
}>();

type TutorialStepItem = {
  icon: typeof Building2;
  label: string;
};

const tutorialSteps: TutorialStepItem[] = [
  { icon: Building2, label: 'tutorial.intro.anOrganization' },
  { icon: AppWindow, label: 'tutorial.intro.anApplication' },
  { icon: FolderTree, label: 'tutorial.intro.anEventType' },
  { icon: Link, label: 'tutorial.intro.aSubscription' },
  { icon: FileText, label: 'tutorial.intro.anEvent' },
];
</script>

<template>
  <div class="wizard-modal__header">
    <Hook0Stack direction="row" align="center" gap="sm">
      <Hook0IconBadge variant="primary" size="md">
        <Rocket :size="18" aria-hidden="true" />
      </Hook0IconBadge>
      <span id="wizard-step-title" class="wizard-modal__title">{{
        t('tutorial.intro.title')
      }}</span>
    </Hook0Stack>
    <button
      class="wizard-modal__close"
      type="button"
      :aria-label="t('tutorial.intro.skipButton')"
      @click="$emit('skip')"
    >
      <X :size="18" aria-hidden="true" />
    </button>
  </div>

  <div class="wizard-modal__content" data-test="tutorial-card">
    <Hook0Stack direction="column" gap="lg">
      <span class="wizard-modal__subtitle">{{ t('tutorial.intro.subtitle') }}</span>

      <Hook0Illustration
        variant="tutorial"
        size="md"
        :alt="t('tutorial.intro.illustrationAlt')"
      />

      <Hook0Stack direction="column" gap="md" data-test="tutorial-header">
        {{ t('tutorial.intro.inThisTutorial') }}
      </Hook0Stack>

      <ul class="tutorial-steps">
        <li
          v-for="step in tutorialSteps"
          :key="step.label"
          class="tutorial-steps__item"
        >
          <span class="tutorial-steps__icon">
            <component :is="step.icon" :size="16" aria-hidden="true" />
          </span>
          <span class="tutorial-steps__label">{{ t(step.label) }}</span>
        </li>
      </ul>

      <Hook0Stack direction="column" gap="none">
        <i18n-t keypath="tutorial.intro.skipNote" tag="span">
          <template #link>
            <Hook0Button
              variant="link"
              target="_blank"
              href="https://documentation.hook0.com/docs/getting-started"
              >{{ t('tutorial.intro.programmatically') }}</Hook0Button
            >
          </template>
        </i18n-t>
      </Hook0Stack>
    </Hook0Stack>
  </div>

  <div class="wizard-modal__footer">
    <Hook0Button
      variant="secondary"
      type="button"
      data-test="tutorial-skip-button"
      @click="$emit('skip')"
    >
      <X :size="16" aria-hidden="true" />
      {{ t('tutorial.intro.skipButton') }}
    </Hook0Button>
    <Hook0Button
      variant="primary"
      type="button"
      data-test="tutorial-start-button"
      @click="$emit('start')"
    >
      {{ t('tutorial.intro.startStep1') }}
      <ArrowRight :size="16" aria-hidden="true" />
    </Hook0Button>
  </div>
</template>

<style scoped>
.tutorial-steps {
  list-style: none;
  padding: 0;
  margin: 0;
}

.tutorial-steps__item {
  display: flex;
  align-items: center;
  padding: 0.5rem 0.75rem;
  font-size: 0.875rem;
  border-radius: var(--radius-md);
  transition: background-color 0.15s ease;
}

.tutorial-steps__icon {
  flex-shrink: 0;
  width: 1.75rem;
  height: 1.75rem;
  margin-right: 0.625rem;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
}

.tutorial-steps__label {
  color: var(--color-text-primary);
}

@media (prefers-reduced-motion: reduce) {
  .tutorial-steps__item {
    transition: none;
  }
}
</style>
