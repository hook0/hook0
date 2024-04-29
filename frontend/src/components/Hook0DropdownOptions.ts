export default interface Hook0DropdownOptions {
  open: () => void;
  close: () => void;
  route: (name: string) => void;
  toggle: (e: Event) => void;
}
