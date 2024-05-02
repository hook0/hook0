import http, { handleError, Problem } from '@/http';
import { AxiosError, AxiosResponse } from 'axios';

export function verifyEmail(token: string): Promise<void> {
  return http.post(`/auth/verify-email`, { token }).then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}
