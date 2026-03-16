<script setup lang="ts">
import { ref } from 'vue';
import { Search } from 'lucide-vue-next';
import { useUiStore } from '@/stores/ui';
import { useCommandPalette } from '@/composables/useCommandPalette';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const uiStore = useUiStore();

const inputRef = ref<HTMLInputElement | null>(null);
const { query, selectedIndex, filteredCommands, groupedCommands, close, onKeydown } =
  useCommandPalette(inputRef);
</script>

<template>
  <Teleport to="body">
    <Transition name="command-palette">
      <div
        v-if="uiStore.commandPaletteOpen"
        class="hook0-command-palette-overlay"
        @click.self="close"
      >
        <div
          class="hook0-command-palette"
          role="dialog"
          aria-label="Command palette"
          @keydown="onKeydown"
        >
          <div class="hook0-command-palette-input-wrapper">
            <Search :size="20" class="hook0-command-palette-search-icon" aria-hidden="true" />
            <input
              ref="inputRef"
              v-model="query"
              class="hook0-command-palette-input"
              :placeholder="t('commandPalette.placeholder')"
              type="text"
              role="combobox"
              aria-expanded="true"
              aria-controls="command-list"
              aria-autocomplete="list"
            />
          </div>

          <div id="command-list" class="hook0-command-palette-list" role="listbox">
            <div v-if="filteredCommands.length === 0" class="hook0-command-palette-empty">
              {{ t('commandPalette.noResults') }}
            </div>

            <div v-for="(items, category) in groupedCommands" :key="category">
              <div class="hook0-command-palette-group-label">{{ category }}</div>
              <button
                v-for="item in items"
                :key="item.id"
                class="hook0-command-palette-item"
                :class="{ selected: filteredCommands.indexOf(item) === selectedIndex }"
                role="option"
                :aria-selected="filteredCommands.indexOf(item) === selectedIndex"
                @click="item.action()"
                @mouseenter="selectedIndex = filteredCommands.indexOf(item)"
              >
                <component :is="item.icon" :size="18" aria-hidden="true" />
                <span>{{ item.label }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.hook0-command-palette-overlay {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 20vh;
  background-color: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
}

.hook0-command-palette {
  width: 100%;
  max-width: 36rem;
  margin: 0 1rem;
  background-color: var(--color-bg-primary);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-xl);
  overflow: hidden;
  border: 1px solid var(--color-border);
}

.hook0-command-palette-input-wrapper {
  display: flex;
  align-items: center;
  padding: 0 1rem;
  border-bottom: 1px solid var(--color-border);
}

.hook0-command-palette-search-icon {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.hook0-command-palette-input {
  flex: 1;
  padding: 0.875rem 0.75rem;
  font-size: 1rem;
  border: none;
  background: transparent;
  color: var(--color-text-primary);
  outline: none;
}

.hook0-command-palette-input::placeholder {
  color: var(--color-text-muted, #9ca3af) !important;
  opacity: 1;
}

.hook0-command-palette-list {
  max-height: 20rem;
  overflow-y: auto;
  padding: 0.5rem;
}

.hook0-command-palette-empty {
  padding: 2rem 1rem;
  text-align: center;
  font-size: 0.875rem;
  color: var(--color-text-muted);
}

.hook0-command-palette-group-label {
  padding: 0.5rem 0.75rem 0.25rem;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--color-text-muted);
}

.hook0-command-palette-item {
  display: flex;
  align-items: center;
  flex-wrap: nowrap;
  white-space: nowrap;
  gap: 0.75rem;
  width: 100%;
  padding: 0.625rem 0.75rem;
  border-radius: var(--radius-md);
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.1s ease;
  text-align: left;
}

.hook0-command-palette-item :deep(svg) {
  flex-shrink: 0;
}

.hook0-command-palette-item:hover,
.hook0-command-palette-item.selected {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}
</style>
