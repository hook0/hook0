import type { TypedSchema, TypedSchemaError } from 'vee-validate';
import type { z } from 'zod';

/**
 * Adapter to bridge Zod v4 schemas with VeeValidate's TypedSchema interface.
 * Required because @vee-validate/zod only supports Zod v3.
 */
export function toTypedSchema<TInput, TOutput>(
  zodSchema: z.ZodType<TOutput, TInput>
): TypedSchema<TInput, TOutput> {
  return {
    __type: 'VVTypedSchema',
    parse(values: TInput) {
      const result = zodSchema.safeParse(values);
      if (result.success) {
        return Promise.resolve({
          value: result.data,
          errors: [] as TypedSchemaError[],
        });
      }
      const errors: TypedSchemaError[] = result.error.issues.map((issue) => ({
        path: issue.path.join('.'),
        errors: [issue.message],
      }));
      return Promise.resolve({ errors });
    },
  };
}
