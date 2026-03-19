import { useQuery } from '@tanstack/vue-query';
import { computed, ref, watch, type Ref } from 'vue';
import { format, subDays } from 'date-fns';
import * as EventsPerDayService from './EventsPerDayService';
import type { EventsPerDayEntry } from './EventsPerDayService';
import { eventsPerDayKeys } from '@/queries/keys';

/** Entity scope for events-per-day queries. */
type EventsPerDayEntityType = 'organization' | 'application';

/** Return type of the useEventsPerDay composable. */
type UseEventsPerDayReturn = {
  days: Ref<number>;
  from: Ref<string>;
  to: Ref<string>;
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
  const from = ref(format(subDays(new Date(), 7), 'yyyy-MM-dd'));
  const to = ref(format(new Date(), 'yyyy-MM-dd'));

  /** Update date range when days preset changes. */
  watch(days, (d) => {
    from.value = format(subDays(new Date(), d), 'yyyy-MM-dd');
    to.value = format(new Date(), 'yyyy-MM-dd');
  });

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
