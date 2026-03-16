import { watch, type ComputedRef } from 'vue';
import { useForm } from 'vee-validate';
import { toTypedSchema } from '@/utils/zod-adapter';
import type { ZodType } from 'zod';
import { push } from 'notivue';
import { displayError } from '@/utils/displayError';
import type { Problem } from '@/http';

interface UseEntityFormOptions<
  TFormValues extends Record<string, unknown>,
  TCreateResult = unknown,
  TUpdateResult = unknown,
> {
  schema: ZodType;
  isNew: ComputedRef<boolean>;
  existingValues: ComputedRef<TFormValues | undefined>;
  createFn: (values: TFormValues) => Promise<TCreateResult>;
  updateFn: (values: TFormValues) => Promise<TUpdateResult>;
  onCreated?: (result: TCreateResult, values: TFormValues) => void;
  onUpdated?: (result: TUpdateResult, values: TFormValues) => void;
  /** Skip success toast (e.g. in tutorial mode). Can be a ref-like getter. */
  skipToast?: boolean | (() => boolean);
  successCreateTitle: string;
  successCreateMessage: string | ((values: TFormValues) => string);
  successUpdateTitle: string;
  successUpdateMessage: string | ((values: TFormValues) => string);
}

export function useEntityForm<
  TFormValues extends Record<string, unknown>,
  TCreateResult = unknown,
  TUpdateResult = unknown,
>(options: UseEntityFormOptions<TFormValues, TCreateResult, TUpdateResult>) {
  const { errors, defineField, handleSubmit, resetForm, isSubmitting } = useForm<TFormValues>({
    validationSchema: toTypedSchema(options.schema),
  });

  // Sync form when existing data loads.
  // Cast through unknown: a complete TFormValues always satisfies _PartialDeep<TFormValues>,
  // but VeeValidate's internal type is not publicly exported so TS cannot prove it.
  watch(
    () => options.existingValues.value,
    (values) => {
      if (values) {
        (resetForm as (opts: { values: TFormValues }) => void)({ values });
      }
    },
    { immediate: true }
  );

  const onSubmit = handleSubmit((values: TFormValues) => {
    const isCreate = options.isNew.value;
    const fn = isCreate ? options.createFn : options.updateFn;
    const title = isCreate ? options.successCreateTitle : options.successUpdateTitle;
    const msg = isCreate ? options.successCreateMessage : options.successUpdateMessage;

    return (fn as (values: TFormValues) => Promise<TCreateResult | TUpdateResult>)(values)
      .then((result) => {
        const skip =
          typeof options.skipToast === 'function' ? options.skipToast() : options.skipToast;
        if (!skip) {
          const message = typeof msg === 'function' ? msg(values) : msg;
          push.success({ title, message, duration: 5000 });
        }
        if (isCreate) {
          options.onCreated?.(result as TCreateResult, values);
        } else {
          options.onUpdated?.(result as TUpdateResult, values);
        }
      })
      .catch((err: Problem) => displayError(err));
  });

  return {
    errors,
    defineField,
    isSubmitting,
    onSubmit,
    resetForm,
  };
}
