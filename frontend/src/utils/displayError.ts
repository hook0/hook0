import { toast } from 'vue-sonner';
import type { Problem } from '@/http';

/**
 * Shared error display utility.
 * Shows a toast notification for API errors: warning for 4xx, error for 5xx.
 */
export function displayError(err: Problem): void {
  console.error(err);
  const options = { description: err.detail, duration: 5000 };
  err.status >= 500 ? toast.error(err.title, options) : toast.warning(err.title, options);
}
