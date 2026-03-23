<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

import Hook0Code from '@/components/Hook0Code.vue';

type BlockInfo = {
  label: string;
  code: string;
};

type Props = {
  blocks: Array<BlockInfo>;
  raw: string;
};

defineProps<Props>();

const { t } = useI18n();

const activeTab = ref<'decoded' | 'raw'>('decoded');
</script>

<template>
  <div class="token-preview">
    <div class="token-preview__tabs" role="tablist" :aria-label="t('serviceTokens.tokenPreview')">
      <button
        type="button"
        role="tab"
        class="token-preview__tab"
        :class="{
          'token-preview__tab--active': activeTab === 'decoded',
        }"
        :aria-selected="activeTab === 'decoded'"
        @click="activeTab = 'decoded'"
      >
        {{ t('serviceTokens.tokenPreviewDecoded') }}
      </button>
      <button
        type="button"
        role="tab"
        class="token-preview__tab"
        :class="{
          'token-preview__tab--active': activeTab === 'raw',
        }"
        :aria-selected="activeTab === 'raw'"
        @click="activeTab = 'raw'"
      >
        {{ t('serviceTokens.tokenPreviewRaw') }}
      </button>
    </div>
    <div v-if="activeTab === 'decoded'" class="token-preview__content">
      <div v-for="blockInfo in blocks" :key="blockInfo.label" class="token-preview__block">
        <span class="token-preview__block-label">
          {{ blockInfo.label }}
        </span>
        <Hook0Code :code="blockInfo.code"></Hook0Code>
      </div>
    </div>
    <div v-else class="token-preview__content">
      <Hook0Code :code="raw"></Hook0Code>
    </div>
  </div>
</template>

<style scoped>
.token-preview {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.token-preview__tabs {
  display: inline-flex;
  border-bottom: 1px solid var(--color-border);
  gap: 0;
}

.token-preview__tab {
  padding: 0.5rem 1rem;
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-secondary);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition:
    color 0.15s ease,
    border-color 0.15s ease;
}

.token-preview__tab:hover {
  color: var(--color-text-primary);
}

.token-preview__tab:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: -2px;
}

.token-preview__tab--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.token-preview__content {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.token-preview__block {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.token-preview__block-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
</style>
