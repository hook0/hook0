import { AxiosError, AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import axios from 'axios';

import i18n from '@/plugins/i18n';
import featureFlags from '@/feature-flags';
import type { components } from '@/types';
import { push } from 'notivue';

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
          if (problem.id === 'AuthInvalidBiscuit') {
            push.error({
              title: i18n.global.t('common.error'),
              message: i18n.global.t('common.sessionExpiredMessage'),
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
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  get<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((client) => client.get(url, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  delete<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((client) => client.delete(url, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  head<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((client) => client.head(url, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  options<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((client) => client.options(url, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  post<T = any, R = AxiosResponse<T>>(
    url: string,
    data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then((client) => client.post(url, data, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  put<T = any, R = AxiosResponse<T>>(
    url: string,
    data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then((client) => client.put(url, data, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  patch<T = any, R = AxiosResponse<T>>(
    url: string,
    data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then((client) => client.patch(url, data, config));
  },

  unauthenticated: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    get<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
      return getAxios(false).then((client) => client.get(url, config));
    },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    post<T = any, R = AxiosResponse<T>>(
      url: string,
      data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
      config?: AxiosRequestConfig
    ): Promise<R> {
      return getAxios(false).then((client) => client.post(url, data, config));
    },
  },

  withRefreshToken: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    post<T = any, R = AxiosResponse<T>>(
      url: string,
      data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
      config?: AxiosRequestConfig
    ): Promise<R> {
      return getAxios(true, true).then((client) => client.post(url, data, config));
    },
  },
};

// Global types
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
    const problem = err.response.data as unknown as Problem;
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
