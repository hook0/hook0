import { push } from 'notivue';
import type { Problem } from '@/http';

/**
 * Shared error display utility.
 * Shows a toast notification for API errors: warning for 4xx, error for 5xx.
 */
export function displayError(err: Problem): void {
  console.error(err);
  const options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}
