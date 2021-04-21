import { AxiosRequestConfig, AxiosResponse } from 'axios';
import axios from 'axios';
import iam from './iam';

require('bluebird');

function getAxios() {
  return iam.getToken().then(jwt_token =>
    axios.create({
      baseURL: process.env.VUE_APP_API_ENDPOINT,
      timeout: 1000,
      headers: {
        Authorization: `Bearer ${jwt_token}`,
      },
      withCredentials: true,
    })
  );
}

export default {
  get<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then(axios => axios.get(url, config));
  },
  delete<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then(axios => axios.delete(url, config));
  },
  head<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then(axios => axios.head(url, config));
  },
  options<T = any, R = AxiosResponse<T>>(url: string, config?: AxiosRequestConfig): Promise<R> {
    return getAxios().then(axios => axios.options(url, config));
  },
  post<T = any, R = AxiosResponse<T>>(
    url: string,
    data?: any,
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then(axios => axios.post(url, data, config));
  },
  put<T = any, R = AxiosResponse<T>>(
    url: string,
    data?: any,
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then(axios => axios.put(url, data, config));
  },
  patch<T = any, R = AxiosResponse<T>>(
    url: string,
    data?: any,
    config?: AxiosRequestConfig
  ): Promise<R> {
    return getAxios().then(axios => axios.patch(url, data, config));
  },
};

// Global types
export type UUID = string;
