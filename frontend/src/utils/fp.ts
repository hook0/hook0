import { concat, groupWith, map, pipe, sortBy } from 'ramda';

export function intersectWith<T, A, C>(
  identity: (a: T) => C,
  mapper: (x: T[]) => A,
  ...lists: T[][]
): A[] {
  return pipe(
    // @ts-ignore
    sortBy(identity),
    groupWith((a, b) => identity(a as T) === identity(b as T)),
    // @ts-ignore
    map(mapper)
    // @ts-ignore
  )(concat(...lists)) as A[];
}
