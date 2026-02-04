import { z } from 'zod';

export const applicationSchema = z.object({
  name: z.string().min(1, 'Application name is required'),
});

export type ApplicationFormValues = z.infer<typeof applicationSchema>;
