import { RouteLocationRaw } from 'vue-router';

export default interface Hook0DropdownOptions {
  open: () => void;
  close: () => void;
  route: (name: RouteLocationRaw) => void;
  toggle: (e: Event) => void;
}
