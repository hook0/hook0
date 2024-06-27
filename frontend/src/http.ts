import { AxiosError, AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import axios from 'axios';
import { identity } from 'ramda';

import featureFlags from '@/feature-flags';
import type { components } from '@/types';
import ProblemFactory from '@/utils/problemFactory';
import { getAccessToken, getRefreshToken } from '@/iam';

type definitions = components['schemas'];

// eslint-disable-next-line @typescript-eslint/require-await
async function getAxios(
  authenticated: boolean = true,
  use_refresh_token: boolean = false
): Promise<AxiosInstance> {
  const token = authenticated
    ? use_refresh_token
      ? getRefreshToken().value
      : getAccessToken().value
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
    withCredentials: !!import.meta.env.hasOwnProperty('FRONTEND_DEV_MODE'), // false in dev mode, true in staging/production mode
  });

  client.interceptors.response.use(identity, function (error: AxiosError) {
    // Any status codes that falls outside the range of 2xx cause this function to trigger

    // convert timeouts axios's error to Problem
    if (isAxiosError(error) && String(error.message).includes('timeout of')) {
      return Promise.reject(new ProblemFactory(0, 'TimeoutExceeded', error.message, error.message));
    }

    return Promise.reject(error);
  });

  return client;
}

export default {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  get<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((axios) => axios.get(url, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  delete<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((axios) => axios.delete(url, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  head<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((axios) => axios.head(url, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  options<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then((axios) => axios.options(url, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  post<T = any, R = AxiosResponse<T>>(
    url: string,
    data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then((axios) => axios.post(url, data, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  put<T = any, R = AxiosResponse<T>>(
    url: string,
    data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then((axios) => axios.put(url, data, config));
  },
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  patch<T = any, R = AxiosResponse<T>>(
    url: string,
    data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then((axios) => axios.patch(url, data, config));
  },

  unauthenticated: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    get<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
      return getAxios(false).then((axios) => axios.get(url, config));
    },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    post<T = any, R = AxiosResponse<T>>(
      url: string,
      data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
      config?: AxiosRequestConfig
    ): Promise<R> {
      return getAxios(false).then((axios) => axios.post(url, data, config));
    },
  },

  withRefreshToken: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    post<T = any, R = AxiosResponse<T>>(
      url: string,
      data?: any, // eslint-disable-line @typescript-eslint/no-explicit-any
      config?: AxiosRequestConfig
    ): Promise<R> {
      return getAxios(true, true).then((axios) => axios.post(url, data, config));
    },
  },
};

// Global types
export type UUID = string;

export type Problem = definitions['Problem'];

export function handleError(err: AxiosError<AxiosResponse<Problem>>): Problem {
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
    title: 'Unknown Error',
    status: 500,
    detail: `An unknown error occurred: ${err.message}`,
  };
}

export function isAxiosError(err: unknown): err is AxiosError {
  const e = err as AxiosError;
  return e !== null && typeof e.isAxiosError === 'boolean' && e.isAxiosError;
}
