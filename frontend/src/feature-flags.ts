const queryParams: Record<string, string> = [...new URLSearchParams(location.search)].reduce(
  function toObj(o, pair) {
    const [k, v]: string[] = pair;
    // @ts-ignore
    o[k] = v;
    return o;
  },
  {}
);
export default {
  getOrElse(feature: string, fallback: string): string {
    return queryParams.hasOwnProperty(feature) ? queryParams[feature] : fallback;
  },
  getIntegerOrElse(feature: string, fallback: number): number {
    return queryParams.hasOwnProperty(feature) && !Number.isNaN(parseInt(queryParams[feature], 10))
      ? parseInt(queryParams[feature], 10)
      : fallback;
  },
};
