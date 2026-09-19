/**
 * The catalog's search box — what `use-filter.store.ts` held in Zustand.
 *
 * Module-level `$state` is the rune equivalent of a store: one value, shared by
 * every component that imports it, with no provider. It stays UI-only. **Never
 * put the fetched items here** — the list belongs to TanStack Query, and a copy
 * in a store is a second thing to keep in step.
 */
let filter = $state('');

export const marketplaceFilter = {
  get value(): string {
    return filter;
  },
  set: (next: string) => {
    filter = next;
  },
};

/**
 * Whether an item matches the current search — the rule the list applies,
 * pulled out so it can be tested without rendering anything.
 */
export const matchesFilter = (
  item: { title: string; provider: string },
  search: string,
): boolean => {
  const needle = search.toLowerCase();
  return (
    item.title.toLowerCase().includes(needle) || item.provider.toLowerCase().includes(needle)
  );
};
