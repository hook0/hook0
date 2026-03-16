import { z } from 'zod';

export const subscriptionSchema = z.object({
  description: z.string().min(1, 'Subscription description is required'),
  target_method: z.string().min(1, 'HTTP method is required'),
  target_url: z.string().min(1, 'URL is required').url('A valid URL is required'),
});

export type SubscriptionFormValues = z.infer<typeof subscriptionSchema>;
