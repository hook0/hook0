import { AxiosError, AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import axios from 'axios';

import i18n from '@/plugins/i18n';
import featureFlags from '@/feature-flags';
import type { components } from '@/types';
import { toast } from 'vue-sonner';

type definitions = components['schemas'];

function getAxios(
  authenticated: boolean = true,
  use_refresh_token: boolean = false
): Promise<AxiosInstance> {
  // Dynamic import to avoid circular dependency with auth store
  return import('@/stores/auth').then(({ useAuthStore }) => {
    const authStore = useAuthStore();
    const token = authenticated
      ? use_refresh_token
        ? authStore.refreshToken
        : authStore.accessToken
      : null;

    const headers =
      token !== null
        ? {
            Authorization: `Bearer ${token}`,
          }
        : {};

    const client = axios.create({
      baseURL: featureFlags.getOrElse('API_ENDPOINT', import.meta.env.VITE_API_ENDPOINT ?? ''),
      timeout: featureFlags.getIntegerOrElse(
        'API_TIMEOUT',
        Number.isNaN(parseInt(import.meta.env.VITE_API_TIMEOUT, 10))
          ? 3000
          : parseInt(import.meta.env.VITE_API_TIMEOUT, 10)
      ),
      headers,
    });

    client.interceptors.response.use(
      (response) => response,
      (error: AxiosError) => {
        if (isAxiosError(error)) {
          const problem = handleError(error as AxiosError<AxiosResponse<Problem>>);
          // Don't clear tokens on refresh endpoint — refresh() handles its own errors
          const isRefreshCall = error.config?.url?.includes('/auth/refresh');
          if (problem.id === 'AuthInvalidBiscuit' && !isRefreshCall) {
            toast.error(i18n.global.t('common.error'), {
              description: i18n.global.t('common.sessionExpiredMessage'),
            });
            authStore.clearTokens().catch(console.error);
          }
        }

        return Promise.reject(error);
      }
    );

    return client;
  });
}

export default {
  get<T = unknown, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((client) => client.get(url, config));
  },
  delete<T = unknown, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((client) => client.delete(url, config));
  },
  head<T = unknown, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((client) => client.head(url, config));
  },
  options<T = unknown, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((client) => client.options(url, config));
  },
  post<T = unknown, R = AxiosResponse<T>>(
    url: string,
    data?: unknown,
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then((client) => client.post(url, data, config));
  },
  put<T = unknown, R = AxiosResponse<T>>(
    url: string,
    data?: unknown,
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then((client) => client.put(url, data, config));
  },
  patch<T = unknown, R = AxiosResponse<T>>(
    url: string,
    data?: unknown,
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then((client) => client.patch(url, data, config));
  },

  unauthenticated: {
    get<T = unknown, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
      return getAxios(false).then((client) => client.get(url, config));
    },
    post<T = unknown, R = AxiosResponse<T>>(
      url: string,
      data?: unknown,
      config?: AxiosRequestConfig
    ): Promise<R> {
      return getAxios(false).then((client) => client.post(url, data, config));
    },
  },

  withRefreshToken: {
    post<T = unknown, R = AxiosResponse<T>>(
      url: string,
      data?: unknown,
      config?: AxiosRequestConfig
    ): Promise<R> {
      return getAxios(true, true).then((client) => client.post(url, data, config));
    },
  },
};

// Global types
/** Branded string type for UUID identifiers. */
export type UUID = string;

export type Problem = definitions['Problem'];

export function handleError(err: AxiosError<AxiosResponse<Problem>>): Problem {
  // convert timeouts axios's error to Problem
  if (
    typeof err.code === 'string' &&
    err.code === 'ECONNABORTED' &&
    typeof err.message === 'string' &&
    err.message.includes('timeout of')
  ) {
    return {
      id: 'TimeoutExceeded',
      status: 499,
      title: i18n.global.t('errors.timeoutExceeded'),
      detail: String(err.message).charAt(0).toUpperCase() + String(err.message).slice(1),
    };
  }

  if (err.response?.data && typeof err.response.data === 'object') {
    const data: unknown = err.response.data;
    const problem = data as Problem;
    if (
      typeof problem.detail === 'string' &&
      typeof problem.status === 'number' &&
      typeof problem.id === 'string' &&
      typeof problem.title === 'string'
    ) {
      return problem;
    }
  }

  return {
    id: 'unknown',
    title: i18n.global.t('errors.unknownErrorTitle'),
    status: 500,
    detail: i18n.global.t('errors.unknownErrorDetail', { message: err.message }),
  };
}

export function isAxiosError(err: unknown): err is AxiosError {
  const e = err as AxiosError;
  return e !== null && typeof e.isAxiosError === 'boolean' && e.isAxiosError;
}
