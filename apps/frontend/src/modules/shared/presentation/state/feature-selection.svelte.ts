import { createSelection } from './selection.svelte.ts';

export interface FeatureSelectionOptions {
  /** When set, deleting the selection calls it for every selected id, then clears the selection. */
  deleteItem?: (id: string) => Promise<unknown>;
}

export interface FeatureSelectionHeaderProps {
  selectedCount: number;
  allSelected: boolean;
  onSelectAll: () => void;
  onClearSelection: () => void;
  onDeleteSelection?: () => Promise<void>;
}

export interface FeatureSelection {
  readonly selectedIds: string[];
  readonly selectedCount: number;
  readonly allSelected: boolean;
  select: (id: string) => void;
  isSelected: (id: string) => boolean;
  selectAll: () => void;
  clearSelection: () => void;
  /** Spread onto `<FeatureHeader>`. */
  readonly headerProps: FeatureSelectionHeaderProps;
}

/**
 * Binds the keyed selection to a list of ids, shared by `FeatureHeader` and `DataTable`.
 * `allIds` is a getter: the list arrives and changes after the first render.
 */
export const createFeatureSelection = (
  key: string,
  allIds: () => string[],
  { deleteItem }: FeatureSelectionOptions = {},
): FeatureSelection => {
  const selection = createSelection(key);

  const selectAll = () => selection.selectAll(allIds());

  const deleteSelection = async (): Promise<void> => {
    if (!deleteItem) return;
    // `allSettled`: one failed delete must not stop the others.
    await Promise.allSettled(selection.selectedIds.map(id => deleteItem(id)));
    selection.clearSelection();
  };

  return {
    get selectedIds() {
      return selection.selectedIds;
    },
    get selectedCount() {
      return selection.selectedIds.length;
    },
    get allSelected() {
      const ids = allIds();
      return ids.length > 0 && selection.selectedIds.length >= ids.length;
    },
    select: selection.select,
    isSelected: (id: string) => selection.selectedIds.includes(id),
    selectAll,
    clearSelection: selection.clearSelection,

    get headerProps() {
      return {
        selectedCount: this.selectedCount,
        allSelected: this.allSelected,
        onSelectAll: selectAll,
        onClearSelection: selection.clearSelection,
        ...(deleteItem ? { onDeleteSelection: deleteSelection } : {}),
      };
    },
  };
};
