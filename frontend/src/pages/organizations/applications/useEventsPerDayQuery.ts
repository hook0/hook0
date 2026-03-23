import { useQuery } from '@tanstack/vue-query';
import { computed, ref, type ComputedRef, type Ref } from 'vue';
import { format, subDays } from 'date-fns';
import * as EventsPerDayService from './EventsPerDayService';
import type { EventsPerDayEntry } from './EventsPerDayService';
import { eventsPerDayKeys } from '@/queries/keys';

/** Entity scope for events-per-day queries. */
type EventsPerDayEntityType = 'organization' | 'application';

/** Return type of the useEventsPerDay composable. */
type UseEventsPerDayReturn = {
  days: Ref<number>;
  from: ComputedRef<string>;
  to: ComputedRef<string>;
  data: Ref<EventsPerDayEntry[] | undefined>;
  isLoading: Ref<boolean>;
  error: Ref<Error | null>;
  refetch: () => void;
};

/**
 * TanStack Query composable for events-per-day chart data.
 * Manages date range state and fetches data reactively when entityId changes.
 */
export function useEventsPerDay(
  entityType: EventsPerDayEntityType,
  entityId: Ref<string>
): UseEventsPerDayReturn {
  const days = ref(7);
  const from = computed(() => format(subDays(new Date(), days.value), 'yyyy-MM-dd'));
  const to = computed(() => format(new Date(), 'yyyy-MM-dd'));

  const queryKey = computed(() =>
    entityType === 'organization'
      ? eventsPerDayKeys.organization(entityId.value, from.value, to.value)
      : eventsPerDayKeys.application(entityId.value, from.value, to.value)
  );

  /** Fetch function dispatching to the correct service endpoint. */
  const queryFn = () =>
    entityType === 'organization'
      ? EventsPerDayService.organization(entityId.value, from.value, to.value)
      : EventsPerDayService.application(entityId.value, from.value, to.value);

  const { data, isLoading, error, refetch } = useQuery({
    queryKey,
    queryFn,
    enabled: computed(() => !!entityId.value),
  });

  /** Trigger a refetch without awaiting the result. */
  const doRefetch = () => {
    void refetch();
  };

  return { days, from, to, data, isLoading, error, refetch: doRefetch };
}
