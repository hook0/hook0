import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { differenceInMilliseconds, subMinutes } from 'date-fns';
import http from '@/http';
import router from '@/router';
import { routes } from '@/routes';
import type { components } from '@/types';
import formbricks from '@formbricks/js';
import { initializeFormbricks, trackFormbricksRoute } from '@/composables/useFormbricksInit';

type LoginResponse = components['schemas']['LoginResponse'];

type AuthState = {
  accessToken: string;
  accessTokenExpiration: Date;
  refreshToken: string;
  refreshTokenExpiration: Date;
  userId: string;
  email: string;
  firstName: string;
  lastName: string;
};

const LOCAL_STORAGE_KEY = 'auth';
const REFRESH_MARGIN_MINUTES = 2;

export const useAuthStore = defineStore('auth', () => {
  const state = ref<AuthState | null>(null);
  let refreshTimerId: number | null = null;
  let refreshInProgress: Promise<void> | null = null;

  // Computed
  const isAuthenticated = computed(() => state.value !== null);
  const accessToken = computed(() => state.value?.accessToken ?? null);
  const refreshToken = computed(() => state.value?.refreshToken ?? null);
  const userInfo = computed(() => {
    if (!state.value) return null;
    return {
      email: state.value.email,
      firstName: state.value.firstName,
      lastName: state.value.lastName,
      name: `${state.value.firstName} ${state.value.lastName}`,
    };
  });

  // Storage
  function readFromStorage(): AuthState | null {
    const data = window.localStorage.getItem(LOCAL_STORAGE_KEY);
    if (!data) {
      return null;
    }

    let parsed: {
      accessToken: string;
      accessTokenExpiration: string;
      refreshToken: string;
      refreshTokenExpiration: string;
      userId: string;
      email: string;
      firstName: string;
      lastName: string;
    } | null;

    try {
      parsed = JSON.parse(data) as typeof parsed;
    } catch {
      removeFromStorage();
      return null;
    }

    if (
      !parsed ||
      typeof parsed.accessToken !== 'string' ||
      typeof parsed.refreshToken !== 'string' ||
      typeof parsed.userId !== 'string' ||
      typeof parsed.email !== 'string'
    ) {
      removeFromStorage();
      return null;
    }

    const refreshTokenExpirationDate = new Date(parsed.refreshTokenExpiration);
    if (refreshTokenExpirationDate <= new Date()) {
      return null;
    }

    return {
      ...parsed,
      accessTokenExpiration: new Date(parsed.accessTokenExpiration),
      refreshTokenExpiration: refreshTokenExpirationDate,
    };
  }

  function writeToStorage(authState: AuthState): void {
    window.localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(authState));
  }

  function removeFromStorage(): void {
    window.localStorage.removeItem(LOCAL_STORAGE_KEY);
  }

  // Token refresh
  function scheduleAutoRefresh(): Promise<void> {
    if (refreshTimerId !== null) {
      clearTimeout(refreshTimerId);
    }

    if (!state.value) {
      return Promise.resolve();
    }

    if (state.value.refreshTokenExpiration <= new Date()) {
      state.value = null;
      removeFromStorage();
      return Promise.resolve();
    }

    if (state.value.accessTokenExpiration <= new Date()) {
      return refresh().catch(() => {
        state.value = null;
        removeFromStorage();
      });
    }

    // Refresh before expiration for extra margin (especially during HMR reloads)
    const refreshInMs = Math.max(
      0,
      differenceInMilliseconds(
        subMinutes(state.value.accessTokenExpiration, REFRESH_MARGIN_MINUTES),
        new Date()
      )
    );

    refreshTimerId = window.setTimeout(() => {
      refresh().catch(() => {
        // Will be handled on next API call
      });
    }, refreshInMs);

    return Promise.resolve();
  }

  function setAuthState(data: LoginResponse): void {
    state.value = {
      accessToken: data.access_token,
      accessTokenExpiration: new Date(data.access_token_expiration),
      refreshToken: data.refresh_token,
      refreshTokenExpiration: new Date(data.refresh_token_expiration),
      userId: data.user_id,
      email: data.email,
      firstName: data.first_name,
      lastName: data.last_name,
    };
    if (state.value) {
      writeToStorage(state.value);
    }
  }

  // Actions
  function login(email: string, password: string): Promise<void> {
    return http.unauthenticated
      .post<LoginResponse>('/auth/login', { email, password })
      .then((res) => {
        setAuthState(res.data);
        return scheduleAutoRefresh();
      })
      .then(() => {
        if (state.value) {
          return formbricks.setUserId(state.value.userId).catch((e) => {
            console.warn(`Formbricks setUserId failed: ${e}`);
          });
        }
      });
  }

  function register(
    email: string,
    firstName: string,
    lastName: string,
    password: string,
    turnstile_token?: string,
    gclid?: string
  ): Promise<void> {
    return http.unauthenticated.post('/register', {
      email,
      first_name: firstName,
      last_name: lastName,
      password,
      turnstile_token,
      gclid,
    });
  }

  function refresh(): Promise<void> {
    if (refreshInProgress) {
      return refreshInProgress;
    }

    if (!state.value) {
      return Promise.resolve();
    }

    refreshInProgress = http.withRefreshToken
      .post<LoginResponse>('/auth/refresh')
      .then((res) => {
        setAuthState(res.data);
        return scheduleAutoRefresh();
      })
      .catch(() => {
        // Before clearing, check if another tab already refreshed successfully
        const freshState = readFromStorage();
        if (freshState && state.value && freshState.refreshToken !== state.value.refreshToken) {
          state.value = freshState;
          return scheduleAutoRefresh();
        }
        void clearTokens();
      })
      .finally(() => {
        refreshInProgress = null;
      });

    return refreshInProgress;
  }

  function logout(): Promise<void> {
    if (!state.value) return Promise.resolve();

    return http
      .post('/auth/logout')
      .catch((e) => {
        console.error(`Logout failed: ${JSON.stringify(e as Error)}`);
      })
      .then(() => clearTokens());
  }

  function clearTokens(): Promise<void> {
    if (refreshTimerId !== null) {
      clearTimeout(refreshTimerId);
    }
    state.value = null;
    removeFromStorage();

    const formbricksLogout = window.formbricks
      ? formbricks.logout().catch(console.warn)
      : Promise.resolve();

    return formbricksLogout.then(() => {
      return router.push({ name: routes.Login }).then(() => {});
    });
  }

  /**
   * Initialize auth state from localStorage and set up cross-tab sync.
   * Must be called once at app startup. Registers singleton listeners for:
   * - `storage` event: syncs auth state when another tab writes/clears tokens
   * - `visibilitychange` event: re-checks token freshness when tab becomes visible
   */
  function initialize(): void {
    initializeFormbricks().catch(console.warn);

    const storedState = readFromStorage();
    if (storedState) {
      state.value = storedState;
      scheduleAutoRefresh().catch(console.error);
    } else {
      removeFromStorage();
      if (window.formbricks) {
        formbricks.logout().catch(console.warn);
      }
    }

    // Sync auth state across tabs via localStorage events
    window.addEventListener('storage', (e) => {
      if (e.key !== LOCAL_STORAGE_KEY) return;
      if (e.newValue === null) {
        if (refreshTimerId !== null) clearTimeout(refreshTimerId);
        state.value = null;
      } else {
        const freshState = readFromStorage();
        if (freshState) {
          state.value = freshState;
          scheduleAutoRefresh().catch(console.error);
        }
      }
    });

    // Re-check token freshness when tab becomes visible again
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'visible' && state.value) {
        scheduleAutoRefresh().catch(console.error);
      }
    });
  }

  // Router guard setup — split into analytics and auth guards
  function setupRouterGuard(): void {
    // Analytics guard: Formbricks route tracking
    router.beforeEach((to) => {
      if (
        window.formbricks &&
        (to.meta?.requiresAuth ?? true) &&
        state.value !== null &&
        !(to.meta?.tutorial ?? false)
      ) {
        trackFormbricksRoute();
      }
      return true;
    });

    // Auth guard: redirect unauthenticated users to login
    router.beforeEach((to) => {
      if ((to.meta?.requiresAuth ?? true) && state.value === null) {
        const storedState = readFromStorage();
        if (storedState) {
          state.value = storedState;
          scheduleAutoRefresh().catch(console.error);
          return; // Allow navigation, session restored
        }
        return { name: routes.Login, query: { redirect_to: to.fullPath } };
      } else if (
        !(to.meta?.requiresAuth ?? true) &&
        (to.meta?.redirectIfLoggedIn ?? true) &&
        state.value !== null
      ) {
        return { name: routes.Home };
      }

      return true;
    });
  }

  return {
    // State
    state,
    isAuthenticated,
    accessToken,
    refreshToken,
    userInfo,
    // Actions
    login,
    register,
    refresh,
    logout,
    clearTokens,
    initialize,
    setupRouterGuard,
  };
});
