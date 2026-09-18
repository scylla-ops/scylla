import { createSelection } from './selection.svelte.ts';

export interface FeatureSelectionOptions {
  /**
   * Deletes a single item by id. When provided, `headerProps.onDeleteSelection`
   * fans out over the current selection (settling every call) and then clears it.
   */
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
  /** Spread onto `<FeatureHeader>` to wire its selection toolbar. */
  readonly headerProps: FeatureSelectionHeaderProps;
}

/**
 * Binds the keyed selection store to a concrete list of ids so a feature's
 * `FeatureHeader` and `DataTable` share a single source of truth.
 *
 * `allIds` is a getter, not an array: the list is server data that arrives and
 * changes, and taking the value once would freeze "select all" on whatever the
 * first render happened to hold. React re-ran the hook on every render and got
 * this for free; in Svelte the caller passes `() => rows.map(r => r.id)`.
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
    // `allSettled`: one failing delete must not strand the others, and the
    // selection is cleared either way so the toolbar cannot keep offering to
    // delete rows that are already gone.
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
