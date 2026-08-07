import http from '@/http.ts';
import type { components } from '@/types';
import { unwrapResponse } from '@/utils/unwrapResponse';

type LoginResponse = components['schemas']['LoginResponse'];

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

export async function verifyEmail(token: string): Promise<LoginResponse> {
  return unwrapResponse(http.unauthenticated.post<LoginResponse>(`/auth/verify-email`, { token }));
}

/**
 * Stop the onboarding reactivation reminders for the account the token belongs
 * to. Unauthenticated on purpose: the link comes from an email and must work
 * without a session.
 */
export async function unsubscribeReactivation(token: string): Promise<void> {
  return unwrapResponse(
    http.unauthenticated.post<void>(`/email-preferences/unsubscribe-reactivation`, { token })
  );
}

export async function resendVerificationEmail(email: string): Promise<void> {
  return unwrapResponse(
    http.unauthenticated.post<void>(`/auth/resend-verification-email`, { email })
  );
}

export async function beginResetPassword(email: string): Promise<void> {
  return unwrapResponse(http.unauthenticated.post<void>(`/auth/begin-reset-password`, { email }));
}

export async function resetPassword(token: string, new_password: string): Promise<void> {
  return unwrapResponse(
    http.unauthenticated.post<void>(`/auth/reset-password`, { token, new_password })
  );
}
