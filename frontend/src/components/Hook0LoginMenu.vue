<script setup lang="ts">
import { useAuthStore } from '@/stores/auth';
import { routes } from '@/routes';
import { User, LogOut, Settings } from 'lucide-vue-next';
import { ref, onMounted, onBeforeUnmount } from 'vue';
import CrispChat from '@/components/CrispChat.vue';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const authStore = useAuthStore();
const isOpen = ref(false);
const dropdownRef = ref<HTMLElement | null>(null);

function logout() {
  void authStore.logout();
}

function toggle() {
  isOpen.value = !isOpen.value;
}

function close() {
  isOpen.value = false;
}

function handleClickOutside(event: MouseEvent) {
  if (dropdownRef.value && !dropdownRef.value.contains(event.target as Node)) {
    close();
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <div ref="dropdownRef" class="login-menu">
    <button v-if="authStore.userInfo" class="login-menu__trigger" @click="toggle">
      <User :size="20" aria-hidden="true" />
    </button>

    <Transition name="dropdown">
      <div v-if="isOpen && authStore.userInfo" class="login-menu__dropdown">
        <div class="login-menu__user-info">
          <span class="login-menu__greeting">{{ t('loginMenu.hi') }}</span>
          <span class="login-menu__email">{{ authStore.userInfo.email }}</span>
        </div>
        <div class="login-menu__divider"></div>
        <router-link :to="{ name: routes.UserSettings }" class="login-menu__item" @click="close">
          <Settings :size="16" aria-hidden="true" />
          {{ t('loginMenu.settings') }}
        </router-link>
        <button
          class="login-menu__item"
          data-e2e="logout"
          @click="
            logout();
            close();
          "
        >
          <LogOut :size="16" aria-hidden="true" />
          {{ t('loginMenu.logout') }}
        </button>
      </div>
    </Transition>
  </div>

  <CrispChat
    v-if="authStore.userInfo"
    :email="authStore.userInfo.email"
    :name="authStore.userInfo.name"
  />
</template>

<style scoped>
.login-menu {
  position: relative;
}

.login-menu__trigger {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2.25rem;
  height: 2.25rem;
  border-radius: 9999px;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
  cursor: pointer;
  transition: all 0.15s ease;
}

.login-menu__trigger:hover {
  background-color: var(--color-bg-secondary);
  color: var(--color-text-primary);
  border-color: var(--color-border-strong);
}

.login-menu__trigger:focus-visible {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}

.login-menu__dropdown {
  position: absolute;
  right: 0;
  top: calc(100% + 0.5rem);
  width: 14rem;
  padding: 0.5rem;
  border-radius: var(--radius-lg);
  background-color: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  box-shadow: var(--shadow-xl);
  z-index: 50;
}

.login-menu__user-info {
  display: flex;
  flex-direction: column;
  padding: 0.5rem 0.75rem;
  gap: 0.125rem;
}

.login-menu__greeting {
  font-size: 0.75rem;
  color: var(--color-text-tertiary);
}

.login-menu__email {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.login-menu__divider {
  height: 1px;
  margin: 0.25rem 0;
  background-color: var(--color-border);
}

.login-menu__item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-md, 0.375rem);
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  background: none;
  border: none;
  cursor: pointer;
  text-decoration: none;
  transition: all 0.15s ease;
}

.login-menu__item:hover {
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
}

/* Dropdown transition */
.dropdown-enter-active {
  transition: all 0.15s ease-out;
}

.dropdown-leave-active {
  transition: all 0.1s ease-in;
}

.dropdown-enter-from {
  opacity: 0;
  transform: translateY(-4px) scale(0.95);
}

.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.95);
}
</style>
