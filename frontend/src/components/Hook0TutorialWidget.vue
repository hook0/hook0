<script setup lang="ts">
import { Check } from 'lucide-vue-next';
import type { Step } from '@/pages/tutorial/TutorialService';
import { useI18n } from 'vue-i18n';

import Hook0Button from '@/components/Hook0Button.vue';

const { t } = useI18n();

type Props = {
  steps: Step[];
};

const props = defineProps<Props>();

const isNextStep = (index: number) => {
  if (index === 0) return !props.steps[0].isCompleted;
  for (let i = 0; i < index; i++) {
    if (!props.steps[i].isCompleted) return false;
  }
  return !props.steps[index].isCompleted;
};

const showExplanation = (index: number) => {
  return isNextStep(index) || props.steps[index].isCompleted;
};
</script>

<template>
  <div class="widget">
    <div class="widget__header">
      <span class="widget__title">{{ t('tutorial.widget.howItWorks') }}</span>
      <span class="widget__subtitle">{{ t('tutorial.widget.howItWorksDesc') }}</span>
    </div>

    <ol class="widget__steps">
      <!-- ========== Step 1: Event Type ========== -->
      <li class="widget__step">
        <div class="widget__timeline">
          <div
            class="widget__circle"
            :class="{
              'widget__circle--done': steps[0]?.isCompleted,
              'widget__circle--active': isNextStep(0),
              'widget__circle--future': !steps[0]?.isCompleted && !isNextStep(0),
            }"
          >
            <Check v-if="steps[0]?.isCompleted" :size="14" aria-hidden="true" />
            <span v-else class="widget__circle-num">1</span>
          </div>
          <div
            class="widget__connector"
            :class="steps[0]?.isCompleted ? 'widget__connector--done' : ''"
          />
        </div>
        <div class="widget__content">
          <span
            class="widget__step-title"
            :class="{
              'widget__step-title--done': steps[0]?.isCompleted,
              'widget__step-title--active': isNextStep(0),
              'widget__step-title--future': !steps[0]?.isCompleted && !isNextStep(0),
            }"
          >
            {{ t(steps[0]?.title ?? '') }}
          </span>
          <div v-if="showExplanation(0)" class="widget__split">
            <p class="widget__explanation">
              You tell Hook0 <strong>what kinds of things happen</strong> in your app. <br />
              Event types follow the format
              <code class="widget__code">&lt;service&gt;.&lt;resourceType&gt;.&lt;verb&gt;</code>.
              <br />
              For example:
              <code class="widget__code">billing.invoice.created</code>,
              <code class="widget__code">user.account.signup</code>. <br />
              Your <strong>webhooks use these to filter</strong> which events they receive.
            </p>
            <div class="widget__visual"></div>
          </div>
          <span v-else class="widget__details">{{ t(steps[0]?.details ?? '') }}</span>
          <Hook0Button
            v-if="isNextStep(0) && steps[0]?.route"
            variant="primary"
            size="sm"
            :to="steps[0].route"
            class="widget__cta"
          >
            {{ t(steps[0].title) }} &#8594;
          </Hook0Button>
        </div>
      </li>

      <!-- ========== Step 2: Webhook ========== -->
      <li v-if="steps[1]" class="widget__step">
        <div class="widget__timeline">
          <div
            class="widget__circle"
            :class="{
              'widget__circle--done': steps[1].isCompleted,
              'widget__circle--active': isNextStep(1),
              'widget__circle--future': !steps[1].isCompleted && !isNextStep(1),
            }"
          >
            <Check v-if="steps[1].isCompleted" :size="14" aria-hidden="true" />
            <span v-else class="widget__circle-num">2</span>
          </div>
          <div
            class="widget__connector"
            :class="steps[1].isCompleted ? 'widget__connector--done' : ''"
          />
        </div>
        <div class="widget__content">
          <span
            class="widget__step-title"
            :class="{
              'widget__step-title--done': steps[1].isCompleted,
              'widget__step-title--active': isNextStep(1),
              'widget__step-title--future': !steps[1].isCompleted && !isNextStep(1),
            }"
          >
            {{ t(steps[1].title) }}
          </span>
          <div v-if="showExplanation(1)" class="widget__split">
            <p class="widget__explanation">
              A webhook is a <strong>delivery rule</strong>. <br />
              "When a <code class="widget__code">billing.invoice.created</code> event arrives,
              <strong>send it to</strong>
              <code class="widget__code widget__code--url">https://billing.example.com/hooks</code
              >." <br />
              You pick the <strong>event types</strong> and the <strong>destination URL</strong>.
              <br />
              Hook0 handles delivery and retries automatically.
            </p>
            <div class="widget__visual"></div>
          </div>
          <span v-else class="widget__details">{{ t(steps[1].details) }}</span>
          <Hook0Button
            v-if="isNextStep(1) && steps[1].route"
            variant="primary"
            size="sm"
            :to="steps[1].route"
            class="widget__cta"
          >
            {{ t(steps[1].title) }} &#8594;
          </Hook0Button>
        </div>
      </li>

      <!-- ========== Step 3: Send Event ========== -->
      <li v-if="steps[2]" class="widget__step">
        <div class="widget__timeline">
          <div
            class="widget__circle"
            :class="{
              'widget__circle--done': steps[2].isCompleted,
              'widget__circle--active': isNextStep(2),
              'widget__circle--future': !steps[2].isCompleted && !isNextStep(2),
            }"
          >
            <Check v-if="steps[2].isCompleted" :size="14" aria-hidden="true" />
            <span v-else class="widget__circle-num">3</span>
          </div>
        </div>
        <div class="widget__content">
          <span
            class="widget__step-title"
            :class="{
              'widget__step-title--done': steps[2].isCompleted,
              'widget__step-title--active': isNextStep(2),
              'widget__step-title--future': !steps[2].isCompleted && !isNextStep(2),
            }"
          >
            {{ t(steps[2].title) }}
          </span>
          <div v-if="showExplanation(2)" class="widget__split">
            <p class="widget__explanation">
              When something happens — a customer places an order — your app
              <strong>sends a JSON message</strong> to Hook0. <br /><br />
              Hook0 instantly <strong>delivers it to every matching webhook</strong>.
              <br />
              If the endpoint is down, it <strong>retries automatically</strong>.
            </p>
            <div class="widget__visual"></div>
          </div>
          <span v-else class="widget__details">{{ t(steps[2].details) }}</span>
          <Hook0Button
            v-if="isNextStep(2) && steps[2].route"
            variant="primary"
            size="sm"
            :to="steps[2].route"
            class="widget__cta"
          >
            {{ t(steps[2].title) }} &#8594;
          </Hook0Button>
        </div>
      </li>
    </ol>
  </div>
</template>

<style scoped>
.widget__header {
  display: flex;
  flex-direction: column;
  gap: 0.125rem;
  margin-bottom: 1.25rem;
}

.widget__title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.widget__subtitle {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.widget__steps {
  list-style: none;
  padding: 0;
  margin: 0;
}

.widget__step {
  display: flex;
  gap: 0.875rem;
}

/* Timeline */
.widget__timeline {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex-shrink: 0;
}

.widget__circle {
  width: 2rem;
  height: 2rem;
  border-radius: var(--radius-full);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.widget__circle--done {
  background-color: var(--color-primary);
  color: #fff;
}

.widget__circle--active {
  border: 2px solid var(--color-primary);
  background-color: var(--color-bg-primary);
  color: var(--color-primary);
  box-shadow: 0 0 0 3px var(--color-primary-light);
}

.widget__circle--future {
  border: 2px solid var(--color-border);
  background-color: var(--color-bg-primary);
  color: var(--color-text-muted);
}

.widget__circle-num {
  font-size: 0.75rem;
  font-weight: 700;
}

.widget__connector {
  width: 1.5px;
  flex: 1;
  min-height: 1rem;
  background-color: var(--color-border);
}

.widget__connector--done {
  background-color: var(--color-primary);
}

/* Content */
.widget__content {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding-bottom: 1.5rem;
  min-width: 0;
  flex: 1;
}

.widget__step-title {
  font-size: 0.875rem;
  font-weight: 600;
  line-height: 2rem;
}

.widget__step-title--done {
  color: var(--color-text-primary);
}

.widget__step-title--active {
  color: var(--color-primary);
}

.widget__step-title--future {
  color: var(--color-text-muted);
}

/* Split layout: text left, code right */
.widget__split {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0.75rem;
  margin-top: 0.25rem;
}

@media (min-width: 768px) {
  .widget__split {
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
  }
}

.widget__explanation {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  line-height: 1.7;
  margin: 0;
}

.widget__explanation strong {
  color: var(--color-text-primary);
  font-weight: 600;
}

.widget__details {
  font-size: 0.875rem;
  color: var(--color-text-muted);
  line-height: 1.5;
}

/* Inline code */
.widget__code {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  background-color: var(--color-primary-light);
  color: var(--color-primary);
  padding: 0.0625rem 0.375rem;
  border-radius: 4px;
}

.widget__code--url {
  background-color: var(--color-bg-secondary);
  color: var(--color-text-secondary);
}

.widget__cta {
  margin-top: 0.5rem;
  align-self: flex-start;
}

/* Visual placeholder (right column) */
.widget__visual {
  min-height: 2rem;
}
</style>
