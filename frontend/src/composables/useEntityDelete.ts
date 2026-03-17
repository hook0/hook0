import { ref, type Ref } from 'vue';
import { push } from 'notivue';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';

type UseEntityDeleteOptions<T> = {
  deleteFn: (entity: T) => Promise<unknown>;
  successTitle: string;
  successMessage: string | ((entity: T) => string);
  onSuccess?: () => void;
}

export function useEntityDelete<T>(options: UseEntityDeleteOptions<T>) {
  const showDeleteDialog = ref(false);
  const entityToDelete = ref<T | null>(null) as Ref<T | null>;
  const deleteLoading = ref(false);

  function requestDelete(entity: T) {
    entityToDelete.value = entity;
    showDeleteDialog.value = true;
  }

  function confirmDelete() {
    const entity = entityToDelete.value;
    if (!entity) return;

    showDeleteDialog.value = false;
    deleteLoading.value = true;

    options
      .deleteFn(entity)
      .then(() => {
        const message =
          typeof options.successMessage === 'function'
            ? options.successMessage(entity)
            : options.successMessage;
        push.success({ title: options.successTitle, message, duration: 5000 });
        options.onSuccess?.();
      })
      .catch((err: Problem) => displayError(err))
      .finally(() => {
        deleteLoading.value = false;
        entityToDelete.value = null;
      });
  }

  return {
    showDeleteDialog,
    entityToDelete,
    deleteLoading,
    requestDelete,
    confirmDelete,
  };
}
