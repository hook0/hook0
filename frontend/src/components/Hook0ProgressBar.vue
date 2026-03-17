<script setup lang="ts">
import { computed } from 'vue';
import { Check } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

type ProgressBarItem = {
  title: string;
}

type ProgressBarProps = {
  current: number;
  items: ProgressBarItem[];
}

const props = defineProps<ProgressBarProps>();

const { t } = useI18n();

const currentStepTitle = computed(() => {
  const currentIndex = props.current - 1;
  if (currentIndex >= 0 && currentIndex < props.items.length) {
    return props.items[currentIndex].title;
  }
  return '';
});
</script>

<template>
  <div
    class="progress-bar"
    role="navigation"
    :aria-label="t('tutorial.step', { current, total: items.length })"
  >
    <!-- Progress track container -->
    <div class="progress-bar__track-container">
      <!-- Background track -->
      <div class="progress-bar__track" aria-hidden="true"></div>

      <!-- Filled progress -->
      <div
        class="progress-bar__fill"
        :style="{ width: `${((current - 1) / (items.length - 1)) * 100}%` }"
        aria-hidden="true"
      ></div>

      <!-- Step indicators -->
      <div class="progress-bar__steps">
        <template v-for="(_item, index) in items" :key="index">
          <div
            class="progress-bar__step"
            :class="{
              'progress-bar__step--completed': index + 1 < current,
              'progress-bar__step--current': index + 1 === current,
              'progress-bar__step--upcoming': index + 1 > current,
            }"
            :aria-current="index + 1 === current ? 'step' : undefined"
          >
            <div class="progress-bar__step-indicator">
              <Transition name="check" mode="out-in">
                <Check
                  v-if="index + 1 < current"
                  :size="14"
                  aria-hidden="true"
                  class="progress-bar__check-icon"
                />
                <span v-else class="progress-bar__step-number">{{ index + 1 }}</span>
              </Transition>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- Current step label -->
    <div class="progress-bar__label">
      <span class="progress-bar__label-step">{{
        t('tutorial.step', { current, total: items.length })
      }}</span>
      <span class="progress-bar__label-title">{{ currentStepTitle }}</span>
    </div>
  </div>
</template>

<style scoped>
.progress-bar {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  width: 100%;
  padding: 1rem 0;
}

.progress-bar__track-container {
  position: relative;
  display: flex;
  align-items: center;
  height: 2rem;
  padding: 0 0.75rem;
}

.progress-bar__track {
  position: absolute;
  left: 0.75rem;
  right: 0.75rem;
  height: 4px;
  background-color: var(--color-border);
  border-radius: var(--radius-full);
}

.progress-bar__fill {
  position: absolute;
  left: 0.75rem;
  height: 4px;
  background: linear-gradient(90deg, var(--color-primary), var(--color-primary-hover));
  border-radius: var(--radius-full);
  transition: width 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.progress-bar__steps {
  position: relative;
  display: flex;
  justify-content: space-between;
  width: 100%;
  z-index: 1;
}

.progress-bar__step {
  display: flex;
  align-items: center;
  justify-content: center;
}

.progress-bar__step-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.75rem;
  height: 1.75rem;
  border-radius: var(--radius-full);
  font-size: 0.75rem;
  font-weight: 600;
  transition:
    background-color 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    color 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    border-color 0.3s cubic-bezier(0.4, 0, 0.2, 1),
    box-shadow 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* Completed step */
.progress-bar__step--completed .progress-bar__step-indicator {
  background-color: var(--color-primary);
  color: #ffffff;
  box-shadow: 0 0 0 4px var(--color-primary-light);
}

/* Current step */
.progress-bar__step--current .progress-bar__step-indicator {
  background-color: var(--color-bg-primary);
  color: var(--color-primary);
  border: 2px solid var(--color-primary);
  box-shadow: 0 0 0 4px var(--color-primary-light);
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

/* Upcoming step */
.progress-bar__step--upcoming .progress-bar__step-indicator {
  background-color: var(--color-bg-primary);
  color: var(--color-text-tertiary);
  border: 2px solid var(--color-border);
}

.progress-bar__step-number {
  line-height: 1;
}

.progress-bar__check-icon {
  stroke-width: 3;
}

/* Label section */
.progress-bar__label {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background-color: var(--color-bg-secondary);
  border-radius: var(--radius-lg);
  text-align: center;
}

.progress-bar__label-step {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-primary);
  text-transform: uppercase;
  letter-spacing: 0.025em;
}

.progress-bar__label-title {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-primary);
}

/* Pulse animation for current step */
@keyframes pulse {
  0%,
  100% {
    box-shadow: 0 0 0 4px var(--color-primary-light);
  }
  50% {
    box-shadow: 0 0 0 8px color-mix(in srgb, var(--color-primary) 10%, transparent);
  }
}

/* Check icon transition */
.check-enter-active {
  transition:
    opacity 0.3s cubic-bezier(0.34, 1.56, 0.64, 1),
    transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.check-leave-active {
  transition:
    opacity 0.15s ease-out,
    transform 0.15s ease-out;
}

.check-enter-from {
  opacity: 0;
  transform: scale(0) rotate(-180deg);
}

.check-enter-to {
  opacity: 1;
  transform: scale(1) rotate(0);
}

.check-leave-from {
  opacity: 1;
  transform: scale(1);
}

.check-leave-to {
  opacity: 0;
  transform: scale(0.5);
}

/* Responsive design */
@media (max-width: 480px) {
  .progress-bar__track-container {
    padding: 0 0.5rem;
  }

  .progress-bar__track {
    left: 0.5rem;
    right: 0.5rem;
  }

  .progress-bar__fill {
    left: 0.5rem;
  }

  .progress-bar__step-indicator {
    width: 1.5rem;
    height: 1.5rem;
    font-size: 0.6875rem;
  }

  .progress-bar__label {
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.75rem;
  }
}
</style>
