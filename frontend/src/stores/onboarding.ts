import { defineStore } from 'pinia';
import { ref } from 'vue';

import type { UseCaseId } from '@/utils/usecasePreset';

// Holds the use-case chosen at the tutorial intro so the later wizard steps
// (event type + subscription) can pre-fill a domain-relevant example. Lives for
// the SPA session; a page reload resets it to the neutral default and the steps
// still to come fall back to the generic examples.
//
// Nothing that decides whether the tutorial's webhook is delivered may be read
// from here: the send-event step seeds itself from the subscription that was
// actually persisted, precisely so a reload cannot desynchronize the two.
export const useOnboardingStore = defineStore('onboarding', () => {
  const useCase = ref<UseCaseId>('other');

  function setUseCase(value: UseCaseId): void {
    useCase.value = value;
  }

  return {
    useCase,
    setUseCase,
  };
});
