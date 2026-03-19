<script setup lang="ts">
/**
 * Hook0UserMenu - User avatar button + dropdown menu.
 *
 * Displays the user's initial in a gradient avatar. On click, opens a dropdown
 * with Settings link, theme toggle, and logout action.
 *
 * @example
 * <Hook0UserMenu ref="userMenuRef" @close-dropdowns="closeAll" />
 */
import { ref } from 'vue';
import { Settings, LogOut, Sun, Moon } from 'lucide-vue-next';
import { routes } from '@/routes';
import { useAuthStore } from '@/stores/auth';
import { useUiStore } from '@/stores/ui';
import { useI18n } from 'vue-i18n';
import Hook0Button from '@/components/Hook0Button.vue';

const { t } = useI18n();
const authStore = useAuthStore();
const uiStore = useUiStore();

const emit = defineEmits<{
  'close-dropdowns': [];
}>();

const isOpen = ref(false);

/** Close the user dropdown. */
function closeDropdowns(): void {
  isOpen.value = false;
}

/** Toggle the user dropdown. */
function toggleDropdown(): void {
  isOpen.value = !isOpen.value;
  emit('close-dropdowns');
}

const triggerRef = ref<HTMLButtonElement | null>(null);

/**
 * Focus the trigger button.
 * Called by the parent when Escape is pressed.
 */
function focusTrigger(): void {
  triggerRef.value?.focus();
}

/** Check whether the user dropdown is open. */
function hasOpenDropdown(): boolean {
  return isOpen.value;
}

defineExpose({ closeDropdowns, focusTrigger, hasOpenDropdown });
</script>

<template>
  <div class="hook0-topnav__dropdown-anchor">
    <button
      ref="triggerRef"
      class="hook0-topnav__user-trigger"
      :aria-expanded="isOpen"
      aria-haspopup="true"
      :aria-label="t('nav.userMenu')"
      @click.stop="toggleDropdown()"
    >
      <div class="hook0-topnav__user-avatar">
        {{ authStore.userInfo?.email?.charAt(0)?.toUpperCase() ?? '\u00A0\u00A0' }}
      </div>
    </button>

    <Transition name="dropdown">
      <div
        v-if="isOpen"
        class="hook0-topnav__dropdown hook0-topnav__user-dropdown"
        role="menu"
        aria-orientation="vertical"
      >
        <div class="hook0-topnav__dropdown-user-info">
          <div class="hook0-topnav__dropdown-user-email">
            {{ authStore.userInfo?.email }}
          </div>
        </div>
        <div class="hook0-topnav__dropdown-separator" />
        <router-link
          :to="{ name: routes.UserSettings }"
          class="hook0-topnav__dropdown-item"
          role="menuitem"
        >
          <Settings :size="16" aria-hidden="true" />
          {{ t('nav.settings') }}
        </router-link>
        <Hook0Button
          variant="ghost"
          class="hook0-topnav__dropdown-item"
          role="menuitem"
          @click="uiStore.toggleColorMode()"
        >
          <Sun v-if="uiStore.effectiveColorMode === 'dark'" :size="16" aria-hidden="true" />
          <Moon v-else :size="16" aria-hidden="true" />
          {{ uiStore.effectiveColorMode === 'dark' ? t('nav.lightMode') : t('nav.darkMode') }}
        </Hook0Button>
        <div class="hook0-topnav__dropdown-separator" />
        <Hook0Button
          variant="ghost"
          class="hook0-topnav__dropdown-item hook0-topnav__dropdown-item--danger"
          role="menuitem"
          @click="void authStore.logout()"
        >
          <LogOut :size="16" aria-hidden="true" />
          {{ t('nav.logout') }}
        </Hook0Button>
      </div>
    </Transition>
  </div>
</template>

<style>
@import './hook0-topnav-dropdown.css';
</style>

<style scoped>
/* User avatar trigger */
.hook0-topnav__user-trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  cursor: pointer;
  padding: 0;
}

.hook0-topnav__user-trigger:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
  border-radius: var(--radius-full);
}

.hook0-topnav__user-avatar {
  width: 1.75rem;
  height: 1.75rem;
  border-radius: var(--radius-full);
  background: linear-gradient(
    135deg,
    var(--color-primary),
    color-mix(in srgb, var(--color-primary) 70%, var(--color-text-primary))
  );
  color: var(--color-primary-text, #fff);
  font-size: 0.6875rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: box-shadow 0.15s ease;
}

.hook0-topnav__user-trigger:hover .hook0-topnav__user-avatar {
  box-shadow:
    0 0 0 2px var(--color-bg-primary),
    0 0 0 4px var(--color-primary);
}

/* UserMenu-specific dropdown overrides */
.hook0-topnav__user-dropdown {
  left: auto;
  right: 0;
  min-width: 12rem;
}

.hook0-topnav__dropdown-item--danger {
  color: var(--color-error);
}

.hook0-topnav__dropdown-item--danger:hover {
  background-color: var(--color-error-light);
  color: var(--color-error);
}

.hook0-topnav__dropdown-user-info {
  padding: 0.625rem 0.75rem;
}

.hook0-topnav__dropdown-user-email {
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

@media (prefers-reduced-motion: reduce) {
  .hook0-topnav__user-avatar {
    transition: none;
  }
}
</style>
