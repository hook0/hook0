import { z } from 'zod';

export const memberInviteSchema = z.object({
  email: z.string().min(1, 'Email is required').email('Please enter a valid email address'),
  role: z.enum(['admin', 'editor', 'viewer'], { message: 'Please select a valid role' }),
});

export type MemberInviteFormValues = z.infer<typeof memberInviteSchema>;

export const memberEditRoleSchema = z.object({
  role: z.enum(['admin', 'editor', 'viewer'], { message: 'Please select a valid role' }),
});

export type MemberEditRoleFormValues = z.infer<typeof memberEditRoleSchema>;
