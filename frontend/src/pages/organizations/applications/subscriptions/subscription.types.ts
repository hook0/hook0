import type { EventType } from '../event_types/EventTypeService';

export type SelectableEventType = EventType & {
  selected: boolean;
};
