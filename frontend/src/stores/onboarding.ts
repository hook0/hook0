import { defineStore } from 'pinia';
import { ref } from 'vue';

import type { UseCaseId } from '@/utils/usecasePreset';

// Holds the use-case chosen at the tutorial intro so the later wizard steps
// (event type + send event) can pre-fill a domain-relevant example. Lives for the
// SPA session; a page reload resets it to the neutral default, which simply falls
// back to the generic examples.
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
