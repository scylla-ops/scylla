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
 * One feature's view of the keyed selection store. `DataTable` and `FeatureHeader`
 * read the same key, so a list and its toolbar agree.
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
