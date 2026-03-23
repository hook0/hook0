import http from '@/http.ts';
import { unwrapResponse } from '@/utils/unwrapResponse';

export async function deleteUser(): Promise<void> {
  return Promise.reject({
    id: 'ComingSoon',
    title: 'Not implemented yet',
    status: 500,
    detail:
      'This feature is not implemented yet, please contact the support team to delete your account.',
  });
}

export async function changePassword(new_password: string): Promise<void> {
  return unwrapResponse(
    http.post<void>('/auth/password', {
      new_password,
    })
  );
}

export async function verifyEmail(token: string): Promise<void> {
  return unwrapResponse(http.unauthenticated.post<void>(`/auth/verify-email`, { token }));
}

export async function beginResetPassword(email: string): Promise<void> {
  return unwrapResponse(http.unauthenticated.post<void>(`/auth/begin-reset-password`, { email }));
}

export async function resetPassword(token: string, new_password: string): Promise<void> {
  return unwrapResponse(
    http.unauthenticated.post<void>(`/auth/reset-password`, { token, new_password })
  );
}
