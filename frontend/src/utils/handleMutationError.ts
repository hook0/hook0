import { toast } from 'vue-sonner';
import type { Problem } from '@/http';
import { isAxiosError } from '@/http';
import { displayError } from '@/utils/displayError';
import i18n from '@/plugins/i18n';

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

  const t = i18n.global.t;
  toast.error(t('errors.unexpectedError'), {
    description: err instanceof Error ? err.message : t('errors.unknownError'),
    duration: 5000,
  });
}
