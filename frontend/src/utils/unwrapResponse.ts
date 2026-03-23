import type { AxiosError, AxiosResponse } from 'axios';
import { handleError } from '@/http';
import type { Problem } from '@/http';

/**
 * Unwrap an Axios response promise: extract .data on success, handleError on failure.
 */
export function unwrapResponse<T>(promise: Promise<AxiosResponse<T>>): Promise<T> {
  return promise.then(
    (res: AxiosResponse<T>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}
