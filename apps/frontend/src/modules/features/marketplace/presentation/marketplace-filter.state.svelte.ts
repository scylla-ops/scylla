/** Module-level `$state`, shared by the components. UI only: never store the fetched items here. */
let filter = $state('');

export const marketplaceFilter = {
  get value(): string {
    return filter;
  },
  set: (next: string) => {
    filter = next;
  },
};

export const matchesFilter = (
  item: { title: string; provider: string },
  search: string,
): boolean => {
  const needle = search.toLowerCase();
  return (
    item.title.toLowerCase().includes(needle) || item.provider.toLowerCase().includes(needle)
  );
};
