import {Axios, AxiosError, AxiosRequestConfig, AxiosResponse} from 'axios';
import axios from 'axios';
import iam from './iam';
import featureFlags from './feature-flags';
import {definitions} from "@/types";
import ProblemFactory from "@/utils/problemFactory";
import {identity} from "ramda";

require('bluebird');

function getAxios() {
  return iam.getToken().then((jwt_token) => {
      const client = axios.create({
        baseURL: featureFlags.getOrElse('API_ENDPOINT', process.env.VUE_APP_API_ENDPOINT),
        timeout: 1000,
        headers: {
          Authorization: `Bearer ${jwt_token}`,
        },
        withCredentials: !!process.env.hasOwnProperty('FRONTEND_DEV_MODE'), // false in dev mode, true in staging/production mode
      });

      client.interceptors.response.use(identity, function (error: AxiosError) {
        // Any status codes that falls outside the range of 2xx cause this function to trigger

        // convert timeouts axios's error to Problem
        if (isAxiosError(error) && String(error.message).includes('timeout of')) {
          return Promise.reject(new ProblemFactory(0, "TimeoutExceeded", error.message, error.message));
        }

        return Promise.reject(error);
      });

      return client;
    }
  );
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
};

// Global types
export type UUID = string;

export type Problem = definitions['Problem'];

export function isAxiosError(err: AxiosError | unknown): err is AxiosError {
  const e = err as AxiosError;
  return e !== null && typeof e.isAxiosError === 'boolean' && e.isAxiosError
}
