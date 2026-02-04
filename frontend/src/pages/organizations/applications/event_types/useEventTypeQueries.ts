import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as EventTypeService from './EventTypeService';
import type { EventTypePost } from './EventTypeService';
import { eventTypeKeys } from '@/queries/keys';

export function useEventTypeList(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => eventTypeKeys.list(applicationId.value)),
    queryFn: () => EventTypeService.list(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}

export function useEventTypeDetail(id: Ref<string>) {
  return useQuery({
    queryKey: computed(() => eventTypeKeys.detail(id.value)),
    queryFn: () => EventTypeService.get(id.value),
    enabled: computed(() => !!id.value),
  });
}

export function useCreateEventType() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (eventType: EventTypePost) => EventTypeService.create(eventType),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: eventTypeKeys.all });
    },
  });
}

export function useDeactivateEventType() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { applicationId: string; eventTypeName: string }) =>
      EventTypeService.deactivate(params.applicationId, params.eventTypeName),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: eventTypeKeys.all });
    },
  });
}
