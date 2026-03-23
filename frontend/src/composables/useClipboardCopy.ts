import { ref, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { toast } from 'vue-sonner';

const COPY_FEEDBACK_MS = 1500;

export function useClipboardCopy() {
  const { t } = useI18n();

  const justCopied = ref(false);
  let timer: ReturnType<typeof setTimeout> | null = null;

  function copy(text: string, description?: string) {
    navigator.clipboard.writeText(text).then(
      () => {
        toast.success(t('common.copied'), {
          description: description ?? t('common.codeCopied'),
          duration: 2000,
        });
        justCopied.value = true;
        if (timer) clearTimeout(timer);
        timer = setTimeout(() => {
          justCopied.value = false;
          timer = null;
        }, COPY_FEEDBACK_MS);
      },
      () => {
        toast.error(t('common.error'), {
          description: t('common.clipboardCopyError'),
        });
      }
    );
  }

  onBeforeUnmount(() => {
    if (timer) clearTimeout(timer);
  });

  return { copy, justCopied };
}
