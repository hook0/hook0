import { useQuery } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as EventTypeService from './EventTypeService';
import type { EventTypePost } from './EventTypeService';
import { eventTypeKeys } from '@/queries/keys';
import { useInvalidatingMutation } from '@/composables/queryHelpers';
import { useCursorInfiniteQuery } from '@/composables/useCursorInfiniteQuery';

export function useEventTypeList(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => eventTypeKeys.list(applicationId.value)),
    queryFn: () => EventTypeService.list(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}

/**
 * Cursor-paginated infinite query for the event types list page.
 * Use {@link useEventTypeList} when you need a flat (full) array — typically
 * inside dropdowns where users pick a few items, not browse a large catalog.
 */
export function useEventTypeListInfinite(applicationId: Ref<string>) {
  return useCursorInfiniteQuery(
    () => [...eventTypeKeys.list(applicationId.value), 'infinite'],
    (cursor) => EventTypeService.listPage(applicationId.value, cursor),
    { enabled: () => !!applicationId.value }
  );
}

export function useEventTypeDetail(id: Ref<string>) {
  return useQuery({
    queryKey: computed(() => eventTypeKeys.detail(id.value)),
    queryFn: () => EventTypeService.get(id.value),
    enabled: computed(() => !!id.value),
  });
}

export function useCreateEventType() {
  return useInvalidatingMutation({
    mutationFn: (eventType: EventTypePost) => EventTypeService.create(eventType),
    invalidateKeys: eventTypeKeys.all,
  });
}

export function useDeactivateEventType() {
  return useInvalidatingMutation({
    mutationFn: (params: { applicationId: string; eventTypeName: string }) =>
      EventTypeService.deactivate(params.applicationId, params.eventTypeName),
    invalidateKeys: eventTypeKeys.all,
  });
}
