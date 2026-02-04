<script setup lang="ts">
import { Menu, Search, Copy, Building2, Box, ChevronRight } from 'lucide-vue-next';
import { useContextStore } from '@/stores/context';
import { useUiStore } from '@/stores/ui';
import { push } from 'notivue';
import type { UUID } from '@/http';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const contextStore = useContextStore();
const uiStore = useUiStore();

function copyToClipboard(id: UUID) {
  navigator.clipboard.writeText(id).then(
    () => {
      push.success({
        title: t('common.copied'),
        message: t('common.idCopied'),
      });
    },
    () => {
      push.error({
        title: t('common.error'),
        message: t('common.clipboardCopyError'),
      });
    }
  );
}
</script>

<template>
  <header class="hook0-header">
    <!-- Mobile menu button -->
    <button
      class="hook0-header-mobile-toggle"
      :aria-label="t('header.openSidebar')"
      @click="uiStore.toggleMobileDrawer()"
    >
      <Menu :size="24" aria-hidden="true" />
    </button>

    <!-- Context Badges (Org > App names) -->
    <div class="hook0-header-context">
      <div
        v-if="contextStore.organizationId"
        class="hook0-header-context-badge"
        role="button"
        tabindex="0"
        :title="t('header.clickToCopyOrgId', { id: contextStore.organizationId })"
        :aria-label="t('header.copyOrgId')"
        @click="copyToClipboard(contextStore.organizationId)"
        @keydown.enter="copyToClipboard(contextStore.organizationId)"
        @keydown.space.prevent="copyToClipboard(contextStore.organizationId)"
      >
        <Building2 :size="14" aria-hidden="true" class="hook0-header-context-icon" />
        <span class="hook0-header-context-name">{{
          contextStore.organizationName || t('header.loadingOrg')
        }}</span>
        <Copy :size="12" class="hook0-header-context-copy" aria-hidden="true" />
      </div>

      <ChevronRight
        v-if="contextStore.organizationId && contextStore.applicationId"
        :size="14"
        aria-hidden="true"
        class="hook0-header-context-separator"
      />

      <div
        v-if="contextStore.applicationId"
        class="hook0-header-context-badge"
        role="button"
        tabindex="0"
        :title="t('header.clickToCopyAppId', { id: contextStore.applicationId })"
        :aria-label="t('header.copyAppId')"
        @click="copyToClipboard(contextStore.applicationId)"
        @keydown.enter="copyToClipboard(contextStore.applicationId)"
        @keydown.space.prevent="copyToClipboard(contextStore.applicationId)"
      >
        <Box :size="14" aria-hidden="true" class="hook0-header-context-icon" />
        <span class="hook0-header-context-name">{{
          contextStore.applicationName || t('header.loadingApp')
        }}</span>
        <Copy :size="12" class="hook0-header-context-copy" aria-hidden="true" />
      </div>
    </div>

    <div class="hook0-header-spacer" />

    <!-- Command palette trigger -->
    <button
      class="hook0-header-search"
      aria-label="Open command palette"
      @click="uiStore.openCommandPalette()"
    >
      <Search :size="16" aria-hidden="true" />
      <span class="hook0-header-search-text">{{ t('header.search') }}</span>
      <kbd class="hook0-header-search-kbd">⌘K</kbd>
    </button>
  </header>
</template>

<style scoped>
.hook0-header {
  display: flex;
  align-items: center;
  height: 4rem;
  padding: 0 1rem;
  background-color: var(--color-bg-primary);
  border-bottom: 1px solid var(--color-border);
  gap: 0.75rem;
  flex-shrink: 0;
}

.hook0-header-mobile-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.5rem;
  height: 2.5rem;
  border-radius: var(--radius-md);
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}

@media (min-width: 768px) {
  .hook0-header-mobile-toggle {
    display: none;
  }
}

.hook0-header-mobile-toggle:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-header-context {
  display: none;
  align-items: center;
  gap: 0.375rem;
}

@media (min-width: 640px) {
  .hook0-header-context {
    display: flex;
  }
}

.hook0-header-context-badge {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  padding: 0.375rem 0.625rem;
  border-radius: var(--radius-md);
  background-color: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  cursor: pointer;
  transition: all 0.15s ease;
  font-size: 0.8125rem;
  max-width: 12rem;
}

.hook0-header-context-badge:hover {
  background-color: var(--color-bg-tertiary);
  border-color: var(--color-primary);
}

.hook0-header-context-badge:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.hook0-header-context-icon {
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.hook0-header-context-name {
  font-weight: 500;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.hook0-header-context-copy {
  color: var(--color-text-muted);
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.hook0-header-context-badge:hover .hook0-header-context-copy {
  opacity: 1;
}

.hook0-header-context-separator {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.hook0-header-spacer {
  flex: 1;
}

.hook0-header-search {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.75rem;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background-color: var(--color-bg-secondary);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
  font-size: 0.875rem;
  min-width: 12rem;
}

.hook0-header-search:hover {
  border-color: var(--color-primary);
  color: var(--color-text-secondary);
}

.hook0-header-search-text {
  flex: 1;
  text-align: left;
}

.hook0-header-search-kbd {
  display: none;
  padding: 0.125rem 0.375rem;
  border-radius: var(--radius-sm);
  background-color: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  font-size: 0.7rem;
  font-family: var(--font-mono);
  color: var(--color-text-muted);
}

@media (min-width: 640px) {
  .hook0-header-search-kbd {
    display: inline;
  }
}
</style>
