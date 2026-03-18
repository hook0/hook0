<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { Rocket, ArrowRight, X } from 'lucide-vue-next';

import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import WizardStepLayout from '@/pages/tutorial/WizardStepLayout.vue';

const { t } = useI18n();

const emit = defineEmits<{
  start: [];
  skip: [];
}>();

type TutorialStepItem = {
  label: string;
  description: string;
};

const tutorialSteps: TutorialStepItem[] = [
  { label: 'tutorial.intro.anOrganization', description: 'tutorial.intro.orgDesc' },
  { label: 'tutorial.intro.anApplication', description: 'tutorial.intro.appDesc' },
  { label: 'tutorial.intro.anEventType', description: 'tutorial.intro.eventTypeDesc' },
  { label: 'tutorial.intro.aSubscription', description: 'tutorial.intro.subDesc' },
  { label: 'tutorial.intro.anEvent', description: 'tutorial.intro.eventDesc' },
];
</script>

<template>
  <WizardStepLayout :title="t('tutorial.intro.title')" @skip="emit('skip')">
    <template #header-icon>
      <Hook0IconBadge variant="primary" size="md">
        <Rocket :size="18" aria-hidden="true" />
      </Hook0IconBadge>
    </template>

    <div class="intro-split" data-test="tutorial-card">
      <!-- Left: text + CTA -->
      <div class="intro-split__left">
        <h2 class="intro-split__heading" data-test="tutorial-header">
          {{ t('tutorial.intro.splitHeading') }}
        </h2>
        <p class="intro-split__desc">{{ t('tutorial.intro.splitDesc') }}</p>
        <p class="intro-split__note">
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
        </p>
        <div class="intro-split__cta">
          <Hook0Button
            variant="primary"
            size="lg"
            type="button"
            data-test="tutorial-start-button"
            @click="emit('start')"
          >
            {{ t('tutorial.intro.startStep1') }}
            <ArrowRight :size="16" aria-hidden="true" />
          </Hook0Button>
        </div>
      </div>

      <!-- Right: vertical timeline -->
      <div class="intro-split__right">
        <ol class="intro-timeline">
          <li v-for="step in tutorialSteps" :key="step.label" class="intro-timeline__item">
            <span class="intro-timeline__dot" />
            <div class="intro-timeline__content">
              <span class="intro-timeline__label">{{ t(step.label) }}</span>
              <span class="intro-timeline__desc">{{ t(step.description) }}</span>
            </div>
          </li>
        </ol>
      </div>
    </div>

    <template #footer>
      <Hook0Button
        variant="secondary"
        type="button"
        data-test="tutorial-skip-button"
        @click="emit('skip')"
      >
        <X :size="16" aria-hidden="true" />
        {{ t('tutorial.intro.skipButton') }}
      </Hook0Button>
    </template>
  </WizardStepLayout>
</template>

<style scoped>
.intro-split {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2rem;
  align-items: start;
}

.intro-split__left {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.intro-split__heading {
  font-size: 1.375rem;
  font-weight: 700;
  line-height: 1.3;
  letter-spacing: -0.02em;
  color: var(--color-text-primary);
}

.intro-split__desc {
  font-size: 0.9375rem;
  color: var(--color-text-secondary);
  line-height: 1.6;
}

.intro-split__note {
  font-size: 0.8125rem;
  color: var(--color-text-muted);
}

.intro-split__cta {
  margin-top: 0.5rem;
}

/* Timeline */
.intro-split__right {
  padding-left: 1rem;
}

.intro-timeline {
  list-style: none;
  padding: 0;
  margin: 0;
  position: relative;
}

.intro-timeline::before {
  content: '';
  position: absolute;
  left: 0.75rem;
  top: 0;
  bottom: 0;
  width: 2px;
  background-color: var(--color-border);
}

.intro-timeline__item {
  position: relative;
  padding: 0 0 1.25rem 2.5rem;
}

.intro-timeline__item:last-child {
  padding-bottom: 0;
}

.intro-timeline__dot {
  position: absolute;
  left: 0.125rem;
  top: 0.125rem;
  width: 1.25rem;
  height: 1.25rem;
  border-radius: var(--radius-full);
  background-color: var(--color-primary-light);
  border: 2px solid var(--color-primary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
  z-index: 1;
}

.intro-timeline__content {
  display: flex;
  flex-direction: column;
}

.intro-timeline__label {
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.intro-timeline__desc {
  font-size: 0.75rem;
  color: var(--color-text-muted);
  margin-top: 0.125rem;
}

@media (max-width: 640px) {
  .intro-split {
    grid-template-columns: 1fr;
  }

  .intro-split__right {
    padding-left: 0;
  }
}
</style>
