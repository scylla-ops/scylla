import { selectionStore } from '@shared/presentation/stores/selection.store.ts';
import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';

const EMPTY: string[] = [];

export interface Selection {
  readonly selectedIds: string[];
  select: (id: string) => void;
  selectAll: (ids: string[]) => void;
  clearSelection: () => void;
}

/**
 * A feature's slice of the one keyed selection store — the Svelte counterpart
 * of `use-selection.ts`.
 *
 * Keyed, never per-feature: `createSelection('jobs')` and
 * `createSelection('users')` are independent views over the same store, and
 * `DataTable` and `FeatureHeader` reading the same key is what keeps a list and
 * its toolbar in agreement.
 *
 * No `$state` of its own: the store is the single source of truth and
 * `toRune` makes reading it reactive. Mirroring `selectedIds` into a local rune
 * would be a second copy to keep in sync, which is the mistake the React rules
 * call mirror state.
 */
export const createSelection = (key: string): Selection => {
  const read = toRune(selectionStore);

  return {
    get selectedIds() {
      return read().selectedIds[key] ?? EMPTY;
    },
    select: (id: string) => selectionStore.getState().select(key, id),
    selectAll: (ids: string[]) => selectionStore.getState().selectAll(key, ids),
    clearSelection: () => selectionStore.getState().clearSelection(key),
  };
};
