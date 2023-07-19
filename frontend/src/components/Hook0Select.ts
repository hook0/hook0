export type Hook0SelectSingleOption = {
  value: string;
  label: string;
};

export type Hook0SelectGroupedOption = {
  label?: string;
  options: Array<Hook0SelectSingleOption>;
};
