export type Hook0KeyValueKeyValuePair = {
  key: string;
  value: string;
};

export function kvPairsToRecord(pairs: Hook0KeyValueKeyValuePair[]): Record<string, string> {
  return pairs.reduce<Record<string, string>>((m, { key, value }) => {
    m[key] = value;
    return m;
  }, {});
}

export function recordToKvPairs(record: Record<string, unknown>): Hook0KeyValueKeyValuePair[] {
  return Object.entries(record).map(([key, value]) => ({
    key,
    value: String(value),
  }));
}
