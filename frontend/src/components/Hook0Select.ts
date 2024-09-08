export type Hook0SelectSingleOption = {
  value: string;
  label: string;
};

export type Hook0SelectGroupedOption = {
  label?: string;
  options: Array<Hook0SelectSingleOption>;
};

export function isValidOption(
  options: Array<Hook0SelectSingleOption | Hook0SelectGroupedOption>,
  value?: string | null
): boolean {
  return options.some((o) => {
    if ('value' in o) {
      return o.value === value;
    } else {
      return isValidOption(o.options, value);
    }
  });
}

export function firstValue(
  options: Array<Hook0SelectSingleOption | Hook0SelectGroupedOption>
): string {
  const first = options[0];
  if (first) {
    if ('value' in first) {
      return first.value;
    } else {
      return firstValue(first.options);
    }
  } else {
    return '';
  }
}
