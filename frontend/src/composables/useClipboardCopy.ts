import { useI18n } from 'vue-i18n';
import { toast } from 'vue-sonner';

export function useClipboardCopy() {
  const { t } = useI18n();

  return function copyToClipboard(text: string, description?: string) {
    navigator.clipboard.writeText(text).then(
      () => {
        toast.success(t('common.copied'), {
          description: description ?? t('common.codeCopied'),
          duration: 2000,
        });
      },
      () => {
        toast.error(t('common.error'), {
          description: t('common.clipboardCopyError'),
        });
      }
    );
  };
}
