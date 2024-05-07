import http, { handleError, Problem } from '@/http.ts';
import { AxiosError, AxiosResponse } from 'axios';

export function deleteUser(): Promise<void> {
  return Promise.reject({
    id: 'ComingSoon',
    title: 'Not implemented yet',
    status: 500,
    detail:
      'This feature is not implemented yet, please contact the support team to delete your account.',
  });
}

export async function changePassword(new_password: string): Promise<void> {
  await http
    .post('/auth/password', {
      new_password,
    })
    .then(
      (res: AxiosResponse<void>) => res.data,
      (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
    );
}

export function verifyEmail(token: string): Promise<void> {
  return http.unauthenticated.post(`/auth/verify-email`, { token }).then(
    (res: AxiosResponse<void>) => res.data,
    (err: AxiosError<AxiosResponse<Problem>>) => Promise.reject(handleError(err))
  );
}
