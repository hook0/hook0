import { push } from 'notivue';
import type { Problem } from '@/http';
import { isAxiosError } from '@/http';
import { displayError } from '@/utils/displayError';

function isProblem(value: unknown): value is Problem {
  if (value === null || typeof value !== 'object') return false;
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.id === 'string' &&
    typeof obj.status === 'number' &&
    typeof obj.title === 'string' &&
    typeof obj.detail === 'string'
  );
}

export function handleMutationError(err: unknown): void {
  console.error(err);

  if (isProblem(err)) {
    displayError(err);
    return;
  }

  if (isAxiosError(err)) {
    const data: unknown = err.response?.data;
    if (isProblem(data)) {
      displayError(data);
      return;
    }
  }

  push.error({
    title: 'Unexpected error',
    message: err instanceof Error ? err.message : 'An unknown error occurred',
    duration: 5000,
  });
}
