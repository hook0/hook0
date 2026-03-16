import { useMutation, useQueryClient } from '@tanstack/vue-query';

/**
 * Creates a mutation that invalidates the given query keys on success.
 * For mutations with custom onSuccess logic, use useMutation directly.
 */
export function useInvalidatingMutation<TInput, TResult = unknown>(options: {
  mutationFn: (input: TInput) => Promise<TResult>;
  invalidateKeys: readonly unknown[];
}) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: options.mutationFn,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: options.invalidateKeys });
    },
  });
}
