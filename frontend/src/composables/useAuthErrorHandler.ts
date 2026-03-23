import type { AxiosError, AxiosResponse } from 'axios';
import { handleError } from '@/http';
import type { Problem } from '@/http';
import { displayError } from '@/utils/displayError';

/**
 * Composable for auth pages that combines handleError + displayError.
 * Returns the Problem object so callers can use it for tracking.
 */
export function useAuthErrorHandler() {
  function handleAuthError(err: unknown): Problem {
    const problem = handleError(err as AxiosError<AxiosResponse<Problem>>);
    displayError(problem);
    return problem;
  }

  return { handleAuthError };
}
