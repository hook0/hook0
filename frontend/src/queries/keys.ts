/**
 * TanStack Query key factories for all domains.
 * Each domain exports a key factory object following the convention:
 *   - `all`: base key for the domain (used for broad invalidation)
 *   - `lists()`: key for list queries
 *   - `list(params)`: key for a specific list query with filters
 *   - `details()`: key for detail queries
 *   - `detail(id)`: key for a specific detail query
 */

export const organizationKeys = {
  all: ['organizations'] as const,
  lists: () => [...organizationKeys.all, 'list'] as const,
  details: () => [...organizationKeys.all, 'detail'] as const,
  detail: (id: string) => [...organizationKeys.details(), id] as const,
};

export const memberKeys = {
  all: ['members'] as const,
  lists: () => [...memberKeys.all, 'list'] as const,
  list: (organizationId: string) => [...memberKeys.lists(), organizationId] as const,
};

export const applicationKeys = {
  all: ['applications'] as const,
  lists: () => [...applicationKeys.all, 'list'] as const,
  list: (organizationId: string) => [...applicationKeys.lists(), organizationId] as const,
  details: () => [...applicationKeys.all, 'detail'] as const,
  detail: (id: string) => [...applicationKeys.details(), id] as const,
};

export const eventKeys = {
  all: ['events'] as const,
  lists: () => [...eventKeys.all, 'list'] as const,
  list: (applicationId: string) => [...eventKeys.lists(), applicationId] as const,
  details: () => [...eventKeys.all, 'detail'] as const,
  detail: (id: string, applicationId: string) =>
    [...eventKeys.details(), id, applicationId] as const,
};

export const eventTypeKeys = {
  all: ['eventTypes'] as const,
  lists: () => [...eventTypeKeys.all, 'list'] as const,
  list: (applicationId: string) => [...eventTypeKeys.lists(), applicationId] as const,
  details: () => [...eventTypeKeys.all, 'detail'] as const,
  detail: (id: string) => [...eventTypeKeys.details(), id] as const,
};

export const eventsPerDayKeys = {
  all: ['eventsPerDay'] as const,
  organization: (orgId: string, from: string, to: string) =>
    [...eventsPerDayKeys.all, 'organization', orgId, from, to] as const,
  application: (appId: string, from: string, to: string) =>
    [...eventsPerDayKeys.all, 'application', appId, from, to] as const,
};

export const subscriptionKeys = {
  all: ['subscriptions'] as const,
  lists: () => [...subscriptionKeys.all, 'list'] as const,
  list: (applicationId: string) => [...subscriptionKeys.lists(), applicationId] as const,
  details: () => [...subscriptionKeys.all, 'detail'] as const,
  detail: (id: string) => [...subscriptionKeys.details(), id] as const,
};

export const logKeys = {
  all: ['logs'] as const,
  lists: () => [...logKeys.all, 'list'] as const,
  list: (applicationId: string) => [...logKeys.lists(), applicationId] as const,
  bySubscription: (applicationId: string, subscriptionId: string) =>
    [...logKeys.lists(), applicationId, 'subscription', subscriptionId] as const,
};

export const healthEventKeys = {
  all: ['healthEvents'] as const,
  lists: () => [...healthEventKeys.all, 'list'] as const,
  list: (subscriptionId: string, organizationId: string) =>
    [...healthEventKeys.lists(), subscriptionId, organizationId] as const,
};

export const requestAttemptKeys = {
  all: ['requestAttempts'] as const,
  details: () => [...requestAttemptKeys.all, 'detail'] as const,
  detail: (id: string, applicationId: string) =>
    [...requestAttemptKeys.details(), id, applicationId] as const,
};

export const secretKeys = {
  all: ['secrets'] as const,
  lists: () => [...secretKeys.all, 'list'] as const,
  list: (applicationId: string) => [...secretKeys.lists(), applicationId] as const,
};

export const instanceConfigKeys = {
  all: ['instanceConfig'] as const,
};

export const retryScheduleKeys = {
  all: ['retrySchedules'] as const,
  lists: () => [...retryScheduleKeys.all, 'list'] as const,
  list: (organizationId: string) => [...retryScheduleKeys.lists(), organizationId] as const,
  details: () => [...retryScheduleKeys.all, 'detail'] as const,
  // detail needs organizationId because retry schedules are org-scoped (unlike subscriptions which are app-scoped)
  detail: (id: string, organizationId: string) =>
    [...retryScheduleKeys.details(), id, organizationId] as const,
};

export const responseKeys = {
  all: ['responses'] as const,
  details: () => [...responseKeys.all, 'detail'] as const,
  detail: (id: string, applicationId: string) =>
    [...responseKeys.details(), id, applicationId] as const,
};

export const serviceTokenKeys = {
  all: ['serviceTokens'] as const,
  lists: () => [...serviceTokenKeys.all, 'list'] as const,
  list: (organizationId: string) => [...serviceTokenKeys.lists(), organizationId] as const,
  details: () => [...serviceTokenKeys.all, 'detail'] as const,
  detail: (id: string, organizationId: string) =>
    [...serviceTokenKeys.details(), id, organizationId] as const,
};
