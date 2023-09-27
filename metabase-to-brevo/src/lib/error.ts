import {AxiosRequestConfig} from 'axios';
export class ConnectorError extends Error {
  name = 'ConnectorError';

  constructor(message: any, previousError: any = {}, public status: number = 500) {
    super(message);
    Error.captureStackTrace(this, this.constructor);
    this.message = `${message}${previousError.message ? `: ${previousError.message}` : JSON.stringify(previousError)}`;
  }
}

export function onError(message: string, status?: number) {
  return function catchError(error: any) {
    throw new ConnectorError(message, error, status || error.status);
  };
}

export function onAxiosError(message: string, axiosConfig: AxiosRequestConfig) {
  return function catchError(error: any) {
    throw new ConnectorError(
      `${message}: ${JSON.stringify(error.response?.data)}, sent: ${JSON.stringify(axiosConfig)}`,
      error,
      error.response?.status
    );
  };
}
