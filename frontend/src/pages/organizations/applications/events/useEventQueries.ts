import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query';
import { computed, type Ref } from 'vue';
import * as EventsService from './EventsService';
import { eventKeys, eventTypeKeys } from '@/queries/keys';

export function useEventList(applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => eventKeys.list(applicationId.value)),
    queryFn: () => EventsService.list(applicationId.value),
    enabled: computed(() => !!applicationId.value),
  });
}

export function useEventDetail(eventId: Ref<string>, applicationId: Ref<string>) {
  return useQuery({
    queryKey: computed(() => eventKeys.detail(eventId.value, applicationId.value)),
    queryFn: () => EventsService.get(eventId.value, applicationId.value),
    enabled: computed(() => !!eventId.value && !!applicationId.value),
  });
}

export function useSendEvent() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: {
      applicationId: string;
      eventId: string;
      eventType: string;
      labels: Record<string, string>;
      occurredAt: Date;
      payload: string;
    }) =>
      EventsService.send_json_event(
        params.applicationId,
        params.eventId,
        params.eventType,
        params.labels,
        params.occurredAt,
        params.payload
      ),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: eventKeys.all });
      void queryClient.invalidateQueries({ queryKey: eventTypeKeys.all });
    },
  });
}

export function useReplayEvent() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { eventId: string; applicationId: string }) =>
      EventsService.replay(params.eventId, params.applicationId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: eventKeys.all });
    },
  });
}
