import http, { handleError, Problem } from '@/http.ts';
import { AxiosError, AxiosResponse } from 'axios';

export interface AccountDeletionStatus {
  deletion_requested: boolean;
}

export function getAccountDeletionStatus(): Promise<AccountDeletionStatus> {
  return http.get('/api/v1/account/deletion-status').then(
    (res: AxiosResponse<AccountDeletionStatus>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function requestAccountDeletion(): Promise<void> {
  return http.delete('/api/v1/account').then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export function cancelAccountDeletion(): Promise<void> {
  return http.post('/api/v1/account/cancel-deletion').then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export async function changePassword(new_password: string): Promise<void> {
  return http
    .post('/auth/password', {
      new_password,
    })
    .then(
      (res: AxiosResponse<void>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export async function verifyEmail(token: string): Promise<void> {
  return http.unauthenticated.post(`/auth/verify-email`, { token }).then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export async function beginResetPassword(email: string): Promise<void> {
  return http.unauthenticated.post(`/auth/begin-reset-password`, { email }).then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}

export async function resetPassword(token: string, new_password: string): Promise<void> {
  return http.unauthenticated.post(`/auth/reset-password`, { token, new_password }).then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}
