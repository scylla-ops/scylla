import { useCallback, useEffect, useMemo } from 'react';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';

interface UseFeatureSelectionOptions {
  /**
   * Deletes a single item by id. When provided, `headerProps.onDeleteSelection`
   * fans out over the current selection (settling every call) and then clears it.
   */
  deleteItem?: (id: string) => Promise<unknown>;
}

/**
 * Binds the keyed selection store to a concrete list of ids so a feature's
 * `FeatureHeader` and `DataTable` share a single source of truth.
 *
 * Returns row helpers for the table (`select`, `isSelected`) plus a ready-to-spread
 * `headerProps` bundle for `FeatureHeader` (selection count, select-all / clear, and
 * — when `deleteItem` is given — bulk delete). This collapses the boilerplate that
 * every feature header used to repeat (`useSelection` + a hand-rolled delete fan-out).
 */
export const useFeatureSelection = (
  key: string,
  allIds: string[],
  { deleteItem }: UseFeatureSelectionOptions = {},
) => {
  const { selectedIds, select, selectAll, clearSelection } = useSelection(key);

  const selectedCount = selectedIds.length;
  const allSelected = allIds.length > 0 && selectedCount >= allIds.length;

  const isSelected = useCallback((id: string) => selectedIds.includes(id), [selectedIds]);
  const handleSelectAll = useCallback(() => selectAll(allIds), [selectAll, allIds]);

  const handleDeleteSelection = useCallback(async () => {
    if (!deleteItem) return;
    await Promise.allSettled(selectedIds.map(id => deleteItem(id)));
    clearSelection();
  }, [deleteItem, selectedIds, clearSelection]);

  return useMemo(
    () => ({
      selectedIds,
      select,
      isSelected,
      selectedCount,
      allSelected,
      selectAll: handleSelectAll,
      clearSelection,
      /** Spread onto `<FeatureHeader>` to wire its selection toolbar. */
      headerProps: {
        selectedCount,
        allSelected,
        onSelectAll: handleSelectAll,
        onClearSelection: clearSelection,
        ...(deleteItem ? { onDeleteSelection: handleDeleteSelection } : {}),
      },
    }),
    [
      selectedIds,
      select,
      isSelected,
      selectedCount,
      allSelected,
      handleSelectAll,
      clearSelection,
      deleteItem,
      handleDeleteSelection,
    ],
  );
};
