export function intersectWith<T, A, C>(
  identity: (a: T) => C,
  mapper: (x: T[]) => A,
  ...lists: T[][]
): A[] {
  const combined = lists.reduce((acc: T[], list: T[]) => acc.concat(list), []);
  const sorted = combined.slice().sort((a: T, b: T) => {
    const idA = identity(a);
    const idB = identity(b);
    if (idA < idB) return -1;
    if (idA > idB) return 1;
    return 0;
  });

  const groups: T[][] = [];
  let currentGroup: T[] = [];

  for (const item of sorted) {
    if (currentGroup.length === 0) {
      currentGroup.push(item);
    } else if (identity(currentGroup[0]) === identity(item)) {
      currentGroup.push(item);
    } else {
      groups.push(currentGroup);
      currentGroup = [item];
    }
  }

  if (currentGroup.length > 0) {
    groups.push(currentGroup);
  }

  return groups.map(mapper);
}
