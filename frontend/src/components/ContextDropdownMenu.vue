<script setup lang="ts" generic="T extends { name: string; [key: string]: unknown }">
/**
 * ContextDropdownMenu - Reusable dropdown menu for context switching (org/app).
 *
 * Generic dropdown that displays a list of items with active state,
 * settings button for the active item, and a "create new" action.
 */
import { ref } from 'vue';
import type { RouteLocationRaw } from 'vue-router';
import { Plus } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import Hook0Button from '@/components/Hook0Button.vue';
import { useMenuKeyboard } from '@/composables/useMenuKeyboard';

const { t } = useI18n();

const props = withDefaults(
  defineProps<{
    items: Array<T>;
    currentId: string | null;
    idKey: string;
    open: boolean;
    createLabel: string;
    /** Optional function returning a route for each item — enables cmd+click to open in new tab. */
    itemTo?: (item: T) => RouteLocationRaw;
  }>(),
  {
    itemTo: undefined,
  }
);

const emit = defineEmits<{
  select: [id: string];
  create: [];
  settings: [id: string];
}>();

const dropdownRef = ref<HTMLElement | null>(null);

function close(): void {
  // Parent controls open state; this is used only for keyboard nav escape
}

const { handleMenuKeydown } = useMenuKeyboard(dropdownRef, close);

function getItemId(item: T): string {
  return String(item[props.idKey]);
}

function getItemName(item: T): string {
  return item.name;
}

function isActive(item: T): boolean {
  return getItemId(item) === props.currentId;
}

function onItemClick(event: MouseEvent, item: T): void {
  if (event.metaKey || event.ctrlKey) return;
  event.preventDefault();
  emit('select', getItemId(item));
}

function onSettingsClick(item: T): void {
  emit('settings', getItemId(item));
}

defineExpose({ dropdownRef });
</script>

<template>
  <Transition name="hook0-dropdown">
    <div
      v-if="open"
      ref="dropdownRef"
      class="hook0-topnav__dropdown"
      role="menu"
      aria-orientation="vertical"
      @keydown="handleMenuKeydown"
    >
      <template v-for="item in items" :key="getItemId(item)">
        <router-link v-if="itemTo" :to="itemTo(item)" custom>
          <template #default="{ href }">
            <a
              :href="href"
              class="hook0-topnav__dropdown-item"
              :class="{ 'hook0-topnav__dropdown-item--active': isActive(item) }"
              role="menuitem"
              @click="onItemClick($event, item)"
            >
              <slot name="icon" :item="item" />
              <div class="hook0-topnav__dropdown-item-content">
                <span class="hook0-topnav__dropdown-item-name">
                  <span class="hook0-topnav__dropdown-item-name-text">{{ getItemName(item) }}</span>
                  <slot name="badge" :item="item" />
                </span>
                <span v-if="isActive(item)" class="hook0-topnav__dropdown-item-meta">
                  {{ t('common.current') }}
                </span>
              </div>
              <Hook0Button
                v-if="isActive(item)"
                variant="secondary"
                size="xs"
                :aria-label="`${t('nav.settings')} ${getItemName(item)}`"
                @click.stop="onSettingsClick(item)"
              >
                {{ t('nav.settings') }}
              </Hook0Button>
            </a>
          </template>
        </router-link>
        <button
          v-else
          class="hook0-topnav__dropdown-item"
          :class="{ 'hook0-topnav__dropdown-item--active': isActive(item) }"
          role="menuitem"
          @click="onItemClick($event, item)"
        >
          <slot name="icon" :item="item" />
          <div class="hook0-topnav__dropdown-item-content">
            <span class="hook0-topnav__dropdown-item-name">
              <span class="hook0-topnav__dropdown-item-name-text">{{ getItemName(item) }}</span>
              <slot name="badge" :item="item" />
            </span>
            <span v-if="isActive(item)" class="hook0-topnav__dropdown-item-meta">
              {{ t('common.current') }}
            </span>
          </div>
          <Hook0Button
            v-if="isActive(item)"
            variant="secondary"
            size="xs"
            :aria-label="`${t('nav.settings')} ${getItemName(item)}`"
            @click.stop="onSettingsClick(item)"
          >
            {{ t('nav.settings') }}
          </Hook0Button>
        </button>
      </template>

      <div class="hook0-topnav__dropdown-separator" />

      <button
        class="hook0-topnav__dropdown-item hook0-topnav__dropdown-item--create"
        role="menuitem"
        @click="emit('create')"
      >
        <Plus :size="16" aria-hidden="true" />
        {{ createLabel }}
      </button>
    </div>
  </Transition>
</template>

<style>
@import './hook0-topnav-dropdown.css';
</style>

<style scoped>
/* Reset anchor styling for dropdown items */
a.hook0-topnav__dropdown-item {
  text-decoration: none;
  color: inherit;
}

button.hook0-topnav__dropdown-item {
  appearance: none;
  background: none;
  border: none;
  width: 100%;
  font: inherit;
  cursor: pointer;
  text-align: left;
}

/* ContextDropdownMenu-specific overrides */
.hook0-topnav__dropdown-item:not(.hook0-topnav__dropdown-item--active):hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

.hook0-topnav__dropdown-item--active {
  background-color: transparent;
  border-radius: 0;
  cursor: default;
}

.hook0-topnav__dropdown-item--create {
  color: var(--color-text-muted);
  font-weight: 400;
  border-bottom: none;
}

.hook0-topnav__dropdown-item--create :deep(svg) {
  border: 1.5px dashed var(--color-border-strong);
  border-radius: var(--radius-sm);
  padding: 1px;
}

.hook0-topnav__dropdown-item--create:hover {
  color: var(--color-text-primary);
}

.hook0-topnav__dropdown-item-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0.0625rem;
}

.hook0-topnav__dropdown-item-name {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  font-weight: 500;
  color: var(--color-text-primary);
  min-width: 0;
  flex: 1;
}

.hook0-topnav__dropdown-item-name-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.hook0-topnav__dropdown-item-meta {
  font-size: 0.6875rem;
  color: var(--color-text-muted);
}

@media (max-width: 767px) {
  .hook0-topnav__dropdown {
    --topnav-height: 3.5rem;

    position: fixed;
    top: var(--topnav-height);
    left: 0.5rem;
    right: 0.5rem;
    min-width: 0;
    max-width: none;
    width: auto;
  }
}
</style>
